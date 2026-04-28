/**
 * Remote Laravel site (Laravel Herd) — capture API endpoints.
 * Keep in sync with APP_URL on the server so Storage::url() and uploads match.
 *
 * Set VITE_APP_URL in `.env` at the project root (e.g. https://Wowsome-micorsite.test if Herd uses HTTPS).
 * Herd hostname follows the site folder name (Wowsome-micorsite); renaming the folder changes the .test host.
 */

function normalizeBaseUrl(url) {
  return String(url ?? '').trim().replace(/\/+$/, '')
}

/** Default matches Herd mapping for folder `Wowsome-micorsite` */
const DEFAULT_APP_URL = 'http://Wowsome-micorsite.test'

const storedBaseUrl = (() => {
  try { return localStorage.getItem('setting_base_url') || '' } catch { return '' }
})()

export const appBaseUrl = normalizeBaseUrl(
  storedBaseUrl || import.meta.env.VITE_APP_URL || DEFAULT_APP_URL
)

/** Resolved URLs for the microsite (paths match the Laravel routes) */
export const remoteSite = Object.freeze({
  baseUrl: appBaseUrl,
  site: appBaseUrl,
  player: `${appBaseUrl}/player`,
  uploadCapture: `${appBaseUrl}/api/upload-capture`,
  latestCapture: `${appBaseUrl}/api/captures/latest`,
  triggerCapture: `${appBaseUrl}/api/trigger-capture`,
  uploadVideo: `${appBaseUrl}/api/upload-video`,
  latestVideo: `${appBaseUrl}/api/videos/latest`,
  uploadVideoChunk: `${appBaseUrl}/api/upload-video/chunk`,
  uploadVideoAssemble: `${appBaseUrl}/api/upload-video/assemble`,
})

/** Multipart field name for `upload-capture` — must match Laravel `$request->file(...)` key. */
export const uploadCaptureFormField =
  (import.meta.env.VITE_UPLOAD_CAPTURE_FIELD &&
    String(import.meta.env.VITE_UPLOAD_CAPTURE_FIELD).trim()) ||
  'image'

