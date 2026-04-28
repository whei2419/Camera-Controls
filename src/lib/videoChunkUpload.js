/**
 * Upload a local video file to the Laravel chunked upload API using the
 * browser's native fetch(). Metadata is sent as JSON fields alongside the
 * base64-encoded chunk, which Laravel reads cleanly via $request->input().
 *
 * File bytes are read from the local filesystem via the Tauri `read_file_chunk`
 * command (Rust), then sent over HTTP by the browser — avoiding the Apache/
 * mod_reqtimeout issues that affect Rust's reqwest for large bodies.
 *
 * Performance tuning:
 *   - CHUNK_SIZE: 8 MB — fewer round trips, within Apache 512 MB limit
 *   - CONCURRENCY: 5 parallel in-flight chunks
 *   - Binary body (not base64 JSON) eliminates 33% wire overhead and
 *     removes server-side base64_decode cost
 *   - Metadata in URL query params — Apache parses query strings before
 *     touching the body, so upload_id/filename are always reliable
 *
 * @param {object} options
 * @param {import('@tauri-apps/api/core').invoke} options.invoke  - Tauri invoke fn
 * @param {string}  options.filePath    - Absolute local path to the video file
 * @param {number}  options.fileSize    - Total file size in bytes
 * @param {string}  options.urlChunk    - POST /api/upload-video/chunk
 * @param {string}  options.urlAssemble - POST /api/upload-video/assemble
 * @param {function} [options.onProgress] - Called with (chunksDone, totalChunks)
 * @returns {Promise<object>} Parsed JSON response from the assemble endpoint
 */
export async function uploadVideoChunked({
  invoke,
  filePath,
  fileSize,
  urlChunk,
  urlAssemble,
  onProgress,
}) {
  const CHUNK_SIZE = 8 * 1024 * 1024 // 8 MB — fewer round trips
  const CONCURRENCY = 5 // parallel in-flight uploads

  const filename = filePath.replace(/\\/g, '/').split('/').pop() ?? 'video.mp4'
  const totalChunks = Math.ceil(fileSize / CHUNK_SIZE)
  const uploadId = `vid_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`

  let chunksDone = 0

  /**
   * Read one chunk from disk via Rust and POST it to the server as raw binary.
   * Metadata goes in query params — Apache parses those before reading the body.
   */
  async function uploadChunk(chunkIndex) {
    const offset = chunkIndex * CHUNK_SIZE
    const length = Math.min(CHUNK_SIZE, fileSize - offset)

    // Read chunk from local disk via Rust IPC (returns base64 string)
    const chunkB64 = await invoke('read_file_chunk', { filePath, offset, length })

    // Decode base64 → raw bytes to avoid 33% wire overhead and server-side decode
    const binaryStr = atob(chunkB64)
    const bytes = new Uint8Array(binaryStr.length)
    for (let i = 0; i < binaryStr.length; i++) {
      bytes[i] = binaryStr.charCodeAt(i)
    }

    // Metadata in query params — always reliable regardless of body size
    const params = new URLSearchParams({
      upload_id: uploadId,
      chunk_index: chunkIndex,
      total_chunks: totalChunks,
      filename,
    })

    const res = await fetch(`${urlChunk}?${params}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/octet-stream', Accept: 'application/json' },
      body: bytes,
    })

    if (!res.ok) {
      const text = await res.text().catch(() => '')
      throw new Error(`Chunk ${chunkIndex} failed: ${res.status} — ${text}`)
    }

    chunksDone++
    onProgress?.(chunksDone, totalChunks)
  }

  // Upload all chunks with bounded concurrency
  const indices = Array.from({ length: totalChunks }, (_, i) => i)
  const queue = [...indices]

  async function worker() {
    while (queue.length > 0) {
      const idx = queue.shift()
      await uploadChunk(idx)
    }
  }

  await Promise.all(Array.from({ length: CONCURRENCY }, worker))

  // All chunks uploaded — trigger server-side assembly
  const assembleRes = await fetch(urlAssemble, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', Accept: 'application/json' },
    body: JSON.stringify({
      upload_id: uploadId,
      total_chunks: totalChunks,
      filename,
    }),
  })

  if (!assembleRes.ok) {
    const text = await assembleRes.text().catch(() => '')
    throw new Error(`Assemble failed: ${assembleRes.status} — ${text}`)
  }

  return assembleRes.json()
}
