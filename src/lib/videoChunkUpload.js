/**
 * Upload a local video file to the Laravel chunked upload API using the
 * browser's native fetch(). Metadata is sent as JSON fields alongside the
 * base64-encoded chunk, which Laravel reads cleanly via $request->input().
 *
 * File bytes are read from the local filesystem via the Tauri `read_file_chunk`
 * command (Rust), then sent over HTTP by the browser — avoiding the Apache/
 * mod_reqtimeout issues that affect Rust's reqwest for large bodies.
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
  const CHUNK_SIZE = 1024 * 1024 // 1 MB

  const filename = filePath.replace(/\\/g, '/').split('/').pop() ?? 'video.mp4'
  const totalChunks = Math.ceil(fileSize / CHUNK_SIZE)
  const uploadId = `vid_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`

  for (let chunkIndex = 0; chunkIndex < totalChunks; chunkIndex++) {
    const offset = chunkIndex * CHUNK_SIZE
    const length = Math.min(CHUNK_SIZE, fileSize - offset)

    // Read chunk bytes from local file via Rust (returns base64 string)
    const chunkB64 = await invoke('read_file_chunk', {
      filePath,
      offset,
      length,
    })

    const res = await fetch(urlChunk, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Accept: 'application/json',
      },
      body: JSON.stringify({
        upload_id: uploadId,
        chunk_index: chunkIndex,
        total_chunks: totalChunks,
        filename,
        chunk_data: chunkB64,
      }),
    })

    if (!res.ok) {
      const text = await res.text().catch(() => '')
      throw new Error(`Chunk ${chunkIndex} failed: ${res.status} — ${text}`)
    }

    onProgress?.(chunkIndex + 1, totalChunks)
  }

  // All chunks uploaded — trigger server-side assembly
  const assembleRes = await fetch(urlAssemble, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Accept: 'application/json',
    },
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
