# Video Upload — Architecture & Reference

**Last updated:** April 28, 2026  
**Target server:** glassbooth.wowbynow.com.my (Apache 2.4 + PHP 8.1 + Laravel)

---

## Problem History

### Attempt 1 — Rust reqwest multipart
Rust sent video chunks as `multipart/form-data` with binary `chunk_data` file part.  
**Failed:** Apache's `mod_reqtimeout` dropped the binary body mid-transfer → PHP received `UPLOAD_ERR_PARTIAL` (error 3) → `$request->hasFile('chunk_data')` returned false → Laravel returned 422 `no_chunk_data_provided`.

### Attempt 2 — Rust reqwest JSON + base64
Rust sent chunks as `application/json` with `chunk_data` as a base64 string.  
**Failed:** For a 1 MB chunk, the base64 string is ~1.33 MB. Apache/mod_reqtimeout still killed the large body before PHP could read it. Laravel saw the request but all `$request->input()` fields came back empty — `upload_id: (empty)`.

### Solution — JS fetch + Rust file reader (current)
Split the responsibility:
- **Rust** reads local file bytes and returns base64 (no HTTP)  
- **Browser fetch()** sends JSON to Apache/Laravel  

Browser HTTP is not affected by `mod_reqtimeout` the same way reqwest is. Apache handles browser WebView connections without the aggressive timeout that kills slow `Content-Length`-declared bodies from desktop HTTP clients.

---

## Current Architecture

```
OBS records video
      │
      ▼
wait_for_file_stable (Rust)
      │  returns fileSize when file stops growing
      ▼
uploadVideoChunked (JS — src/lib/videoChunkUpload.js)
      │
      ├─ loop: invoke('read_file_chunk', offset, length)  ← Rust reads disk
      │         └─ returns base64 string
      │
      ├─ fetch(urlChunk, { body: JSON })                  ← Browser sends HTTP
      │         └─ POST /api/upload-video/chunk
      │
      └─ fetch(urlAssemble, { body: JSON })               ← Browser sends HTTP
                └─ POST /api/upload-video/assemble
```

---

## Files Involved

### Tauri / Rust

| File | What it does |
|---|---|
| `src-tauri/src/commands.rs` | `read_file_chunk` — reads N bytes from offset, returns base64 |
| `src-tauri/src/commands.rs` | `wait_for_file_stable` — polls file until size stops changing, returns size |
| `src-tauri/src/lib.rs` | Registers `read_file_chunk` in the invoke handler |

### Vue / JS

| File | What it does |
|---|---|
| `src/lib/videoChunkUpload.js` | `uploadVideoChunked()` — the full chunk loop using browser fetch |
| `src/App.vue` | `onRecordSaved()` — triggers wait → upload, shows toasts |
| `src/config/remoteSite.js` | `remoteSite.uploadVideoChunk` and `uploadVideoAssemble` URLs |

### Laravel (Herd: `C:\Users\whei\Herd\Wowsome-micorsite`)

| File | What it does |
|---|---|
| `app/Http/Controllers/VideoChunkController.php` | `chunk()` — decodes base64 `$request->input('chunk_data')`, writes to `storage/app/chunks/{upload_id}/{chunk_index}` |
| `app/Http/Controllers/VideoChunkController.php` | `assemble()` — concatenates chunks 0..N-1 into `storage/app/public/videos/`, fires `VideoUploaded` event |
| `routes/api.php` | `POST /api/upload-video/chunk` and `POST /api/upload-video/assemble` |

---

## Chunk Request Format

### POST `/api/upload-video/chunk`
```json
{
  "upload_id":    "vid_1745842890123_a3f2k1",
  "chunk_index":  0,
  "total_chunks": 12,
  "filename":     "2026-04-28 18-24-30.mp4",
  "chunk_data":   "<base64-encoded 1MB block>"
}
```
**Response:** `{ "ok": true, "chunk": 0 }`

### POST `/api/upload-video/assemble`
```json
{
  "upload_id":    "vid_1745842890123_a3f2k1",
  "total_chunks": 12,
  "filename":     "2026-04-28 18-24-30.mp4"
}
```
**Response:** `{ "ok": true, "path": "videos/vid_xxx.mp4", "url": "https://..." }`

---

## Key Decisions

| Decision | Reason |
|---|---|
| 1 MB chunk size | Base64 overhead → ~1.33 MB JSON body, well under Apache's 512 MB limit |
| Browser fetch, not Rust reqwest | Browser WebView HTTP bypasses mod_reqtimeout issues that kill reqwest bodies |
| Rust reads file, JS sends HTTP | Rust has filesystem access; browser has reliable HTTP to Apache |
| Base64 JSON (not multipart) | `$request->input()` is always reliable; `$request->hasFile()` is not on this server |
| Sequential chunks (not parallel) | Prevents out-of-order assembly; keeps memory use flat |

---

## Server Specs Reference

See `SERVER_SPECS_UPLOAD.md` in the Laravel project for full Apache/PHP configuration.  
Key limits: upload_max_filesize 512MB, post_max_size 512MB, LimitRequestBody 512MB, RequestReadTimeout body=20-600 minrate=10.

---

## Debugging

**Console (browser devtools):**
```
[video upload] JS chunked upload for: C:/Users/.../video.mp4 | size: 45678900
[video upload] chunk 1/44
[video upload] chunk 2/44
...
```

**Laravel log** (`storage/logs/laravel.log` on server):
```
VideoChunk: request received {...}
VideoChunk: decoding from base64 {"chunk_size": 1048576}
VideoChunk: chunk stored successfully {"chunk_index": 0, "file_size": 1048576}
...
VideoAssemble: all chunks assembled {"total_size_mb": 43.5}
VideoAssemble: success {"url": "https://glassbooth.wowbynow.com.my/storage/videos/..."}
```
