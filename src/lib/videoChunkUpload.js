/**
 * Upload a local video file to the Laravel chunked upload API.
 *
 * Architecture: Rust reads file bytes (IPC → base64), JS decodes them and
 * sends each chunk via browser fetch() as application/octet-stream.
 * Using browser fetch (not Rust reqwest) is intentional: the Chromium/WebView2
 * HTTP stack satisfies Apache mod_reqtimeout's minrate requirement. Rust reqwest
 * on Windows sends body data in a way that Apache drops mid-transfer, causing
 * partial writes and corrupted assembly.
 *
 * Metadata travels in URL query params — parsed by Apache before the body,
 * so upload_id/filename are always reliable regardless of body size or timeout.
 *
 * @param {object}   options
 * @param {Function} options.invoke       - Tauri invoke (from '@tauri-apps/api/core')
 * @param {string}   options.filePath    - Absolute local path to the video file
 * @param {number}   options.fileSize    - Total file size in bytes
 * @param {string}   options.urlChunk    - POST /api/upload-video/chunk
 * @param {string}   options.urlAssemble - POST /api/upload-video/assemble
 * @param {Function} [options.onProgress] - Called with (done, total) per chunk
 * @returns {Promise<object>} Parsed JSON from the assemble endpoint
 */
export async function uploadVideoChunked({
  invoke,
  filePath,
  fileSize,
  urlChunk,
  urlAssemble,
  onProgress,
}) {
  const CHUNK_SIZE = 4 * 1024 * 1024 // 4 MB — balanced for slow connections
  const CONCURRENCY = 2 // conservative: avoids saturating Apache's connection pool
  const MAX_RETRIES = 3 // retry transient network errors (ERR_CONNECTION_TIMED_OUT etc.)

  const filename = filePath.replace(/\\/g, '/').split('/').pop() ?? 'video.mp4'
  const totalChunks = Math.ceil(fileSize / CHUNK_SIZE)
  const uploadId = `vid_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`

  let chunksDone = 0

  async function uploadChunk(chunkIndex, attempt = 0) {
    const offset = chunkIndex * CHUNK_SIZE
    const length = Math.min(CHUNK_SIZE, fileSize - offset)

    // Read from local disk via Rust IPC — returns base64 string
    const chunkB64 = await invoke('read_file_chunk', { filePath, offset, length })

    // Decode base64 → raw bytes — no base64 on the wire, only in IPC
    const binaryStr = atob(chunkB64)
    const bytes = new Uint8Array(binaryStr.length)
    for (let i = 0; i < binaryStr.length; i++) {
      bytes[i] = binaryStr.charCodeAt(i)
    }

    const params = new URLSearchParams({
      upload_id: uploadId,
      chunk_index: chunkIndex,
      total_chunks: totalChunks,
      filename,
      expected_size: bytes.length, // PHP validates this to catch partial writes
    })

    let res
    try {
      res = await fetch(`${urlChunk}?${params}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/octet-stream', Accept: 'application/json' },
        body: bytes,
      })
    } catch (networkErr) {
      // ERR_CONNECTION_TIMED_OUT, ERR_NETWORK_CHANGED, etc. — retry with backoff
      if (attempt < MAX_RETRIES) {
        await new Promise((r) => setTimeout(r, 1500 * (attempt + 1)))
        return uploadChunk(chunkIndex, attempt + 1)
      }
      throw new Error(
        `Chunk ${chunkIndex} network error after ${attempt + 1} attempts: ${networkErr.message}`,
      )
    }

    if (!res.ok) {
      const text = await res.text().catch(() => '')
      // Retry server errors (5xx) but not client errors (4xx — bad request, won't change)
      if (attempt < MAX_RETRIES && res.status >= 500) {
        await new Promise((r) => setTimeout(r, 1500 * (attempt + 1)))
        return uploadChunk(chunkIndex, attempt + 1)
      }
      throw new Error(`Chunk ${chunkIndex} failed: ${res.status} — ${text}`)
    }

    chunksDone++
    onProgress?.(chunksDone, totalChunks)
  }

  const queue = Array.from({ length: totalChunks }, (_, i) => i)

  async function worker() {
    while (queue.length > 0) {
      await uploadChunk(queue.shift())
    }
  }

  await Promise.all(Array.from({ length: CONCURRENCY }, worker))

  // All chunks stored — trigger server-side assembly
  const assembleRes = await fetch(urlAssemble, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
    body: JSON.stringify({ upload_id: uploadId, total_chunks: totalChunks, filename }),
  })

  if (!assembleRes.ok) {
    const text = await assembleRes.text().catch(() => '')
    throw new Error(`Assemble failed: ${assembleRes.status} — ${text}`)
  }

  return assembleRes.json()
}
