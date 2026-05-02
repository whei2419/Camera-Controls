<script setup>
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { Icon } from '@iconify/vue'
import GalleryScreen from './components/GalleryScreen.vue'
import LiveView from './components/LiveView.vue'
import SettingsModal from './components/SettingsModal.vue'
import ThumbnailGallery from './components/ThumbnailGallery.vue'
import OBSConnect from './components/OBSConnect.vue'
import CameraConnect from './components/CameraConnect.vue'
import { initPusher, disconnectPusher, broadcastEvent } from './lib/pusherClient'
import { remoteSite, uploadCaptureFormField } from './config/remoteSite.js'
import { uploadVideoChunked } from './lib/videoChunkUpload.js'

// ── DigiCamControl (camera settings) ──
const connected = ref(false)
const cameraInfo = ref(null)

function getCaptureSource() {
  return (localStorage.getItem('setting_photo_capture_source') || 'digicamcontrol').toLowerCase()
}

async function onConnected(info) {
  cameraInfo.value = info
  connected.value = true
  broadcastEvent('camera_connected', { name: info?.name ?? '', source: getCaptureSource() })
}

async function disconnect() {
  connected.value = false
  cameraInfo.value = null
  broadcastEvent('camera_disconnected', { source: getCaptureSource() })
}

// ── OBS WebSocket (live feed) ──
const obsConnected = ref(false)
const obsInfo = ref(null)

function onOBSConnected(info) {
  obsInfo.value = info
  obsConnected.value = true
  if (getCaptureSource() === 'obs') {
    broadcastEvent('camera_connected', { name: info?.scene ?? 'OBS Virtual Camera', source: 'obs' })
  }
}

function onOBSDisconnected() {
  obsConnected.value = false
  obsInfo.value = null
  if (getCaptureSource() === 'obs') {
    broadcastEvent('camera_disconnected', { source: 'obs' })
  }
}

// No cleanup needed — DigiCamControl manages the camera connection

// ── Toast notifications ────────────────────────────────────────────
const toasts = ref([])
let _toastId = 0
function addToast(message, type = 'success') {
  const id = ++_toastId
  toasts.value.push({ id, message, type })
  setTimeout(() => { toasts.value = toasts.value.filter(t => t.id !== id) }, 3500)
}

// ── Gallery (recent files) ─────────────────────────────────────────
const GALLERY_KEY = 'gallery_items'
const gallery = ref([])

function loadGallery() {
  try { gallery.value = JSON.parse(localStorage.getItem(GALLERY_KEY) || '[]') } catch { gallery.value = [] }
}

function pushGalleryItem(item) {
  gallery.value.unshift(item)
  if (gallery.value.length > 30) gallery.value = gallery.value.slice(0, 30)
  localStorage.setItem(GALLERY_KEY, JSON.stringify(gallery.value))
}

function clearGallery() {
  gallery.value = []
  localStorage.removeItem(GALLERY_KEY)
}

// ── Settings modal ────────────────────────────────────────────────
const showSettingsModal = ref(false)
const obsConnectRef = ref(null)
const cameraConnectRef = ref(null)

function openSettings() {
  showSettingsModal.value = true
}

// ── Startup connection check modal ────────────────────────────────
const showStartupCheck = ref(false)
const startupStatus = ref('checking') // 'checking' | 'done'
const startupObs = ref('pending')     // 'pending' | 'ok' | 'fail'
const startupCam = ref('pending')     // 'pending' | 'ok' | 'fail'
const startupFeed = ref('pending')    // 'pending' | 'ok' | 'fail'

async function runStartupCheck() {
  // Read saved settings to know what to check
  let obsCreds = null
  try { obsCreds = JSON.parse(localStorage.getItem('obs_creds') || 'null') } catch { }
  const obsConfigured = !!(obsCreds?.host)

  const captureSource = (localStorage.getItem('setting_photo_capture_source') || 'digicamcontrol').toLowerCase()
  const camConfigured = captureSource !== 'obs'

  // Nothing configured at all — skip the modal entirely
  if (!obsConfigured && !camConfigured) return

  // Post-OBS-reload: OBS will auto-connect on its own, no need for the startup modal again.
  // We use a timestamp in localStorage — only skip if the reload happened within the last 10s.
  const reloadTs = parseInt(localStorage.getItem('_obs_reload_ts') || '0', 10)
  localStorage.removeItem('_obs_reload_ts') // always clear immediately
  if (Date.now() - reloadTs < 10000) return

  showStartupCheck.value = true
  startupStatus.value = 'checking'
  startupObs.value = obsConfigured ? 'pending' : 'skip'
  startupCam.value = camConfigured ? 'pending' : 'skip'
  startupFeed.value = 'pending'

  const checks = []

  if (obsConfigured) {
    checks.push((async () => {
      try {
        await Promise.race([
          new Promise((resolve) => {
            const stop = watch(obsConnected, val => {
              if (val) { stop(); resolve() }
            }, { immediate: true })
            if (!obsConnected.value && obsConnectRef.value) {
              obsConnectRef.value.reconnect().catch(() => { })
            }
          }),
          new Promise((_, reject) => setTimeout(() => reject(new Error('timeout')), 5000))
        ])
        startupObs.value = 'ok'
        // Reload so the browser re-enumerates devices with labels now that OBS is connected.
        // This guarantees the OBS Virtual Camera is selected correctly on first run.
        localStorage.setItem('_obs_reload_ts', String(Date.now()))
        window.location.reload()
      } catch {
        startupObs.value = 'fail'
      }
    })())
  }

  if (camConfigured) {
    checks.push((async () => {
      try {
        await Promise.race([
          new Promise((resolve) => {
            const stop = watch(connected, val => {
              if (val) { stop(); resolve() }
            }, { immediate: true })
            if (!connected.value && cameraConnectRef.value) {
              cameraConnectRef.value.reconnect().catch(() => { })
            }
          }),
          new Promise((_, reject) => setTimeout(() => reject(new Error('timeout')), 5000))
        ])
        startupCam.value = 'ok'
      } catch {
        startupCam.value = 'fail'
      }
    })())
  }

  // Feed check runs in parallel — LiveView auto-starts when OBS connects,
  // so we just watch for active with a 10s window to cover OBS connect time + feed start time.
  checks.push((async () => {
    try {
      await Promise.race([
        new Promise((resolve) => {
          const stop = watch(() => liveViewRef.value?.active, val => {
            if (val) { stop(); resolve() }
          }, { immediate: true })
          // Only call startFeed directly if OBS is not configured (no virtual cam expected)
          if (!obsConfigured && !liveViewRef.value?.active && liveViewRef.value?.startFeed) {
            liveViewRef.value.startFeed().catch(() => { })
          }
        }),
        new Promise((_, reject) => setTimeout(() => reject(new Error('timeout')), 10000))
      ])
      startupFeed.value = 'ok'
    } catch {
      startupFeed.value = 'fail'
    }
  })())

  await Promise.all(checks)
  startupStatus.value = 'done'
}

// ── Thumbnail refresh ─────────────────────────────────────────────
const thumbnailRefreshTrigger = ref(0)

function refreshThumbnails() {
  thumbnailRefreshTrigger.value++
  imageFolder.value = localStorage.getItem('setting_image_path') || ''
  videoFolder.value = localStorage.getItem('setting_video_path') || ''
}

// ── Gallery screen ────────────────────────────────────────────────
const showGalleryScreen = ref(false)
const galleryScreenRef = ref(null)
const autoPrint = ref(localStorage.getItem('setting_auto_print') === 'true')
const imageFolder = ref(localStorage.getItem('setting_image_path') || '')
const videoFolder = ref(localStorage.getItem('setting_video_path') || '')
const RECORDING_DURATION_KEY = 'setting_recording_duration_sec'
const recordingDurationSec = ref(normalizeRecordingDuration(localStorage.getItem(RECORDING_DURATION_KEY)))

// Ref for calling LiveView methods
const liveViewRef = ref(null)
let recordStopTimer = null

// Pusher connection state
const pusherConnected = ref(false)

function normalizeRecordingDuration(value) {
  const n = Number(value)
  if (!Number.isFinite(n)) return 20
  return Math.max(3, Math.min(600, Math.round(n)))
}

async function loadRecordingDurationConfig() {
  // Tauri startup sync: load backend config and mirror to localStorage.
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const config = await invoke('get_app_config')
    const secs = normalizeRecordingDuration(config?.recordingDurationSec)
    recordingDurationSec.value = secs
    localStorage.setItem(RECORDING_DURATION_KEY, String(secs))
  } catch {
    // Keep local fallback when backend config is unavailable.
  }
}

// Keep folder refs in sync when the gallery screen opens
function openGalleryScreen() {
  imageFolder.value = localStorage.getItem('setting_image_path') || ''
  videoFolder.value = localStorage.getItem('setting_video_path') || ''
  showGalleryScreen.value = true
}

async function onCaptureSuccess(captureStartMs = Date.now()) {
  console.log('[capture] onCaptureSuccess called, captureStartMs:', captureStartMs)
  const folder = localStorage.getItem('setting_image_path') || ''
  addToast('📷 Photo captured!')

  if (!folder) {
    console.warn('[capture] no image folder set — aborting upload')
    return
  }

  const { invoke, convertFileSrc } = await import('@tauri-apps/api/core')
  const EXTS = ['jpg', 'jpeg', 'png', 'cr2', 'cr3', 'nef', 'arw', 'tif', 'tiff']

  // Add a placeholder gallery entry while we wait
  pushGalleryItem({ type: 'photo', folder, path: '', ts: captureStartMs })
  refreshThumbnails()

  // Poll every 500 ms (up to 20 s) for any file whose mtime >= captureStartMs
  // Using since_ms covers both new files AND overwritten files with same name
  const MAX_ATTEMPTS = 40
  let fileFound = false
  for (let i = 0; i < MAX_ATTEMPTS; i++) {
    await new Promise(r => setTimeout(r, 500))
    try {
      const files = await invoke('list_folder_files', { folder, extensions: EXTS, sinceMs: captureStartMs })
      console.log(`[capture] poll ${i + 1}/${MAX_ATTEMPTS} — found ${files.length} file(s):`, files)
      const newest = files[0] ?? null
      if (!newest) continue

      fileFound = true
      // New file detected — replace the placeholder in the gallery
      const idx = gallery.value.findIndex(g => g.type === 'photo' && g.folder === folder && !g.path)
      const item = { type: 'photo', folder, path: newest, ts: Date.now() }
      if (idx >= 0) gallery.value.splice(idx, 1, item)
      else gallery.value.unshift(item)
      localStorage.setItem(GALLERY_KEY, JSON.stringify(gallery.value))
      refreshThumbnails()

      // Upload
      console.log('[upload] filePath:', newest, '| url:', remoteSite.uploadCapture, '| field:', uploadCaptureFormField)
      try {
        await invoke('upload_capture_file', {
          filePath: newest,
          url: remoteSite.uploadCapture,
          fieldName: uploadCaptureFormField,
        })
        addToast('☁️ Uploaded to server')
      } catch (uploadErr) {
        console.error('[capture] upload_capture_file failed:', uploadErr)
        addToast(`Upload failed: ${uploadErr}`, 'error')
      }

      // Auto-print
      if (autoPrint.value && galleryScreenRef.value) {
        galleryScreenRef.value.printImage(convertFileSrc(newest), newest)
      }

      break
    } catch (loopErr) {
      console.error('[capture] poll loop error:', loopErr)
    }
  }

  if (!fileFound) {
    // Remove the placeholder that was added at the start
    const idx = gallery.value.findIndex(g => g.type === 'photo' && g.folder === folder && !g.path)
    if (idx >= 0) gallery.value.splice(idx, 1)
    localStorage.setItem(GALLERY_KEY, JSON.stringify(gallery.value))
    refreshThumbnails()
    addToast('No file found in output folder after capture', 'error')
  }
}

function onRecordSaved(path) {
  const name = path.split(/[\\/]/).pop()
  addToast(`🎬 Saved: ${name}`)
  pushGalleryItem({ type: 'video', path, folder: '', ts: Date.now() })
  refreshThumbnails()

  // Wait for OBS to finish flushing the file before uploading
  import('@tauri-apps/api/core').then(({ invoke }) => {
    addToast('⏳ Waiting for file to finish…')
    invoke('wait_for_file_stable', { filePath: path, timeoutSecs: 30, stableChecks: 2 })
      .then((fileSize) => {
        addToast(`⏫ Uploading: ${name}`)
        console.log('[video upload] JS chunked upload for:', path, '| size:', fileSize)
        return uploadVideoChunked({
          invoke,
          filePath: path,
          fileSize,
          urlChunk: remoteSite.uploadVideoChunk,
          urlAssemble: remoteSite.uploadVideoAssemble,
          onProgress: (done, total) => {
            console.log(`[video upload] chunk ${done}/${total}`)
          },
        })
      })
      .then(() => addToast('☁️ Video uploaded!'))
      .catch((e) => {
        console.error('[video upload] failed:', e)
        addToast(`Video upload failed: ${e}`, 'error')
      })
  })
}

onMounted(loadGallery)
onMounted(loadRecordingDurationConfig)
onMounted(runStartupCheck)
onMounted(() => {
  // Broadcast current camera/OBS state so monitoring page gets live status on load
  const src = getCaptureSource()
  if (src === 'obs') {
    if (obsConnected.value) broadcastEvent('camera_connected', { name: obsInfo.value?.scene ?? 'OBS Virtual Camera', source: 'obs' })
  } else {
    if (connected.value) broadcastEvent('camera_connected', { name: cameraInfo.value?.name ?? '', source: src })
  }
  if (obsConnected.value) broadcastEvent('obs_connected', { scene: obsInfo.value?.scene ?? '' })
})
// Initialize Pusher client (listen for remote triggers)
onMounted(() => {
  try {
    // Provided credentials (do NOT use secret on client-side serverside only)
    initPusher({
      key: 'a1e22a2a180cb0de8d72',
      cluster: 'ap1',
      channelName: 'camera-control',
      onConnected: () => { pusherConnected.value = true },
      onDisconnected: () => { pusherConnected.value = false },
      onCapture: async (data) => {
        console.log('[pusher] onCapture received', data)
        const payload = data && typeof data === 'object' ? data : {}
        const nested = payload?.data && typeof payload.data === 'object' ? payload.data : {}

        const rawMode =
          (typeof payload.mode === 'string' && payload.mode) ||
          (typeof nested.mode === 'string' && nested.mode) ||
          ''

        const payloadDuration = Number(payload?.durationSec ?? nested?.durationSec)
        const mode = rawMode
          ? rawMode.toLowerCase()
          : (Number.isFinite(payloadDuration) && payloadDuration > 0 ? 'video' : 'photo')

        console.log('[pusher] mode:', mode, '| durationSec from payload:', payloadDuration)
        const durationSec = normalizeRecordingDuration(
          Number.isFinite(payloadDuration) && payloadDuration > 0
            ? payloadDuration
            : recordingDurationSec.value
        )

        if (mode === 'video') {
          if (recordStopTimer) {
            clearTimeout(recordStopTimer)
            recordStopTimer = null
          }
          if (liveViewRef.value?.startRecording) {
            liveViewRef.value.startRecording()
            addToast(`🎬 Recording ${durationSec}s`)
            recordStopTimer = setTimeout(() => {
              if (liveViewRef.value?.stopRecording) {
                liveViewRef.value.stopRecording()
              }
              recordStopTimer = null
            }, durationSec * 1000)
          }
          return
        }

        // Photo flow
        console.log('[pusher] triggering photo capture, liveViewRef:', !!liveViewRef.value)
        if (liveViewRef.value?.captureFrame) {
          console.log('[pusher] calling captureFrame()')
          liveViewRef.value.captureFrame()
        } else {
          // Fallback: call DigiCamControl capture directly
          try { await fetch('http://localhost:5513/?CMD=Capture&_=' + Date.now(), { mode: 'no-cors' }) } catch { }
        }
      },
      onRecordStart: async () => {
        if (liveViewRef.value?.startRecording) liveViewRef.value.startRecording()
      },
      onRecordStop: async () => {
        if (liveViewRef.value?.stopRecording) liveViewRef.value.stopRecording()
      },
      onFeedToggle: async (payload) => {
        // payload { action: 'on'|'off' } optional
        if (liveViewRef.value?.toggle) liveViewRef.value.toggle()
      },
      onReload: () => reloadApp(),
      onRequestState: () => {
        const src = getCaptureSource()
        if (src === 'obs') {
          if (obsConnected.value) broadcastEvent('camera_connected', { name: obsInfo.value?.scene ?? 'OBS Virtual Camera', source: 'obs' })
          else broadcastEvent('camera_disconnected', { source: 'obs' })
        } else {
          if (connected.value) broadcastEvent('camera_connected', { name: cameraInfo.value?.name ?? '', source: src })
          else broadcastEvent('camera_disconnected', { source: src })
        }
        if (obsConnected.value) broadcastEvent('obs_connected', { scene: obsInfo.value?.scene ?? '' })
        else broadcastEvent('obs_disconnected')
        if (liveViewRef.value?.active) broadcastEvent('feed_started', { device: '' })
        else broadcastEvent('feed_stopped')
        // Audio source config
        const mediaSource = localStorage.getItem('setting_record_media_source')?.trim() || ''
        broadcastEvent('audio_configured', { source: mediaSource })
      }
    })
  } catch (e) { console.warn('Pusher init failed', e) }
})

onUnmounted(() => {
  if (recordStopTimer) {
    clearTimeout(recordStopTimer)
    recordStopTimer = null
  }
  disconnectPusher()
})

function reloadApp() {
  window.location.reload()
}
</script>

<template>
  <main class="app-shell">

    <!-- ── Header ──────────────────────────────────────────────────────────── -->
    <header class="app-header">
      <div class="brand">
        <span class="brand-icon">◉</span>
      </div>

      <div class="header-badges">
        <div v-if="obsConnected && obsInfo" class="connection-badge">
          <span class="dot dot-obs"></span>
          OBS · {{ obsInfo.scene }}
        </div>
        <div v-if="pusherConnected" class="connection-badge">
          <span class="dot dot-push"></span>
          PUSH
        </div>
        <div v-if="connected && cameraInfo" class="connection-badge">
          <span class="dot dot-live"></span>
          {{ cameraInfo.name }}
          <button class="btn btn-ghost btn-xs" @click="disconnect">Disconnect</button>
        </div>
        <button class="btn btn-ghost btn-xs settings-btn" @click="openSettings">
          <Icon icon="heroicons:cog-6-tooth" width="14" height="14" style="vertical-align:middle;margin-right:4px" />
          Settings
        </button>
        <button class="btn btn-ghost btn-xs gallery-btn" @click="openGalleryScreen">
          <Icon icon="heroicons:photo" width="14" height="14" style="vertical-align:middle;margin-right:6px" />
          Gallery
        </button>
        <button class="btn btn-ghost btn-xs" title="Reload app" @click="reloadApp">
          <Icon icon="heroicons:arrow-path" width="14" height="14" style="vertical-align:middle" />
        </button>
      </div>
    </header>

    <!-- ── Body ───────────────────────────────────────────────────────────── -->
    <div class="app-body">

      <!-- Left: Thumbnail gallery -->
      <ThumbnailGallery :image-folder="imageFolder" :video-folder="videoFolder"
        :refresh-trigger="thumbnailRefreshTrigger" @open-gallery="openGalleryScreen" @refresh="refreshThumbnails" />

      <!-- Keep OBS connect mounted (hidden) so app stays connected to OBS even when settings modal is closed -->
      <div style="display:none">
        <OBSConnect ref="obsConnectRef" @connected="onOBSConnected" @disconnected="onOBSDisconnected" />
      </div>
      <!-- Keep Camera connect mounted (hidden) so app can ping & detect DigiCamControl on startup -->
      <div style="display:none">
        <CameraConnect ref="cameraConnectRef" @connected="onConnected" />
      </div>

      <!-- Main area: live view only -->
      <section class="main-area">
        <LiveView ref="liveViewRef" :obs-connected="obsConnected" :connected="connected"
          :obs-instance="obsInfo?.obs ?? null" @capture-success="onCaptureSuccess" @record-saved="onRecordSaved"
          @capture-error="msg => addToast('📷 ' + msg, 'error')" />
      </section>

    </div>
  </main>


  <!-- ── Toast notifications ─────────────────────────────────────────── -->
  <div class="toast-wrap">
    <transition-group name="toast">
      <div v-for="t in toasts" :key="t.id" class="toast" :class="{ 'toast-error': t.type === 'error' }">
        {{ t.message }}
      </div>
    </transition-group>
  </div>
  <!-- ── Startup Connection Check Modal ───────────────────────── -->
  <teleport to="body">
    <transition name="sc-fade">
      <div v-if="showStartupCheck" class="sc-overlay">
        <div class="sc-modal">
          <h2 class="sc-title">Starting up…</h2>
          <p class="sc-subtitle">Checking connections</p>

          <div class="sc-checks">
            <div v-if="startupObs !== 'skip'" class="sc-row">
              <span class="sc-label">OBS WebSocket</span>
              <span class="sc-badge" :class="startupObs">
                <span v-if="startupObs === 'pending'">connecting…</span>
                <span v-else-if="startupObs === 'ok'">connected ✓</span>
                <span v-else>not found</span>
              </span>
            </div>
            <div v-if="startupCam !== 'skip'" class="sc-row">
              <span class="sc-label">DigiCamControl</span>
              <span class="sc-badge" :class="startupCam">
                <span v-if="startupCam === 'pending'">connecting…</span>
                <span v-else-if="startupCam === 'ok'">connected ✓</span>
                <span v-else>not found</span>
              </span>
            </div>
            <div class="sc-row">
              <span class="sc-label">Live feed</span>
              <span class="sc-badge" :class="startupFeed">
                <span v-if="startupFeed === 'pending'">starting…</span>
                <span v-else-if="startupFeed === 'ok'">active ✓</span>
                <span v-else>no camera found</span>
              </span>
            </div>
          </div>

          <div v-if="startupStatus === 'done'" class="sc-actions">
            <p v-if="startupObs === 'fail' || startupCam === 'fail' || startupFeed === 'fail'" class="sc-hint">
              Some connections failed. Open Settings to fix credentials and reconnect.
            </p>
            <div class="sc-buttons">
              <button v-if="startupObs === 'fail' || startupCam === 'fail' || startupFeed === 'fail'"
                class="btn btn-primary btn-sm" @click="showStartupCheck = false; openSettings()">
                Open Settings
              </button>
              <button class="btn btn-ghost btn-sm" @click="showStartupCheck = false">Continue</button>
            </div>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
  <!-- ── Settings Modal ───────────────────────────────────────── -->
  <SettingsModal :show="showSettingsModal" :obs-connected="obsConnected" :obs-info="obsInfo" :connected="connected"
    :obs-instance="obsInfo?.obs ?? null" @close="showSettingsModal = false; refreshThumbnails()"
    @obs-connected="onOBSConnected" @obs-disconnected="onOBSDisconnected" @camera-connected="onConnected" />

  <!-- ── Gallery Screen ───────────────────────────────────────── -->
  <GalleryScreen ref="galleryScreenRef" :show="showGalleryScreen" :image-folder="imageFolder"
    :video-folder="videoFolder" @close="showGalleryScreen = false" @update:auto-print="v => { autoPrint = v }" />
</template>

<style>
/* ── Design tokens ──────────────────────────────────────────────────────────── */
:root {
  --c-bg: #0d0d0d;
  --c-surface: #161616;
  --c-surface-2: #1e1e1e;
  --c-border: #2e2e2e;
  --c-text: #e8e8e8;
  --c-text-muted: #888;
  --c-accent: #3b82f6;
  --c-error: #f87171;

  font-family: 'SF Pro Display', 'Inter', Helvetica, Arial, sans-serif;
  font-size: 14px;
  line-height: 1.5;
  color: var(--c-text);
  background: var(--c-bg);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

*,
*::before,
*::after {
  box-sizing: border-box;
}

body {
  margin: 0;
}

/* ── Scrollbar ───────────────────────────────────────────────────────────────── */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--c-border);
  border-radius: 99px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--c-text-muted);
}

::-webkit-scrollbar-corner {
  background: transparent;
}

* {
  scrollbar-width: thin;
  scrollbar-color: var(--c-border) transparent;
}

/* ── Layout ──────────────────────────────────────────────────────────────────── */
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 1.25rem;
  height: 48px;
  border-bottom: 1px solid var(--c-border);
  background: var(--c-surface);
  flex-shrink: 0;
}

.brand {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-weight: 700;
  font-size: 0.9rem;
  letter-spacing: 0.03em;
}

.brand-icon {
  color: var(--c-accent);
  font-size: 1.1rem;
}

.header-badges {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.connection-badge {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.8rem;
  color: var(--c-text-muted);
}

.dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  display: inline-block;
}

.dot-live {
  background: #4ade80;
  box-shadow: 0 0 6px #4ade80;
  animation: blink 2s infinite;
}

.dot-obs {
  background: #a78bfa;
  box-shadow: 0 0 6px #a78bfa88;
}

.dot-checking {
  background: #eab308;
  box-shadow: 0 0 6px #eab30888;
  animation: blink 0.8s infinite;
}

.dot-push {
  background: #60a5fa;
  box-shadow: 0 0 6px #60a5fa88;
  animation: blink 2s infinite;
}

@keyframes blink {

  0%,
  100% {
    opacity: 1;
  }

  50% {
    opacity: 0.4;
  }
}

.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  gap: 0;
}

/* ── Shared panel shell ───────────────────────────────────────────────────────── */
.panel {
  padding: 1.25rem;
  border-bottom: 1px solid var(--c-border);
}

/* ── Buttons ─────────────────────────────────────────────────────────────────── */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  padding: 0.45rem 1rem;
  border-radius: 6px;
  font-size: 0.82rem;
  font-weight: 600;
  border: 1px solid transparent;
  cursor: pointer;
  transition: background 0.15s, opacity 0.15s;
  letter-spacing: 0.02em;
  font-family: inherit;
}

.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--c-accent);
  color: #fff;
  border-color: var(--c-accent);
}

.btn-primary:not(:disabled):hover {
  background: #2563eb;
}

.btn-danger {
  background: #dc2626;
  color: #fff;
  border-color: #dc2626;
}

.btn-danger:not(:disabled):hover {
  background: #b91c1c;
}

.btn-ghost {
  background: transparent;
  color: var(--c-text-muted);
  border-color: var(--c-border);
}

.btn-ghost:not(:disabled):hover {
  background: var(--c-surface-2);
  color: var(--c-text);
}

.btn-xs {
  padding: 0.2rem 0.6rem;
  font-size: 0.72rem;
}

.gallery-btn {
  border-color: var(--c-border);
  color: var(--c-text);
  padding: 0.25rem 0.75rem;
}

.settings-btn {
  border-color: var(--c-border);
  color: var(--c-text);
  padding: 0.25rem 0.75rem;
}

/* ── Startup check modal ────────────────────────────────────────────────── */
.sc-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9000;
}

.sc-modal {
  background: var(--c-surface);
  border: 1px solid var(--c-border);
  border-radius: 12px;
  padding: 2rem 2.5rem;
  width: 360px;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.sc-title {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--c-text);
}

.sc-subtitle {
  margin: -0.75rem 0 0;
  font-size: 0.82rem;
  color: var(--c-text-muted);
}

.sc-checks {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.sc-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 0.85rem;
}

.sc-label {
  color: var(--c-text-muted);
}

.sc-badge {
  font-size: 0.78rem;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 20px;
  background: var(--c-surface-2);
  color: var(--c-text-muted);
}

.sc-badge.pending {
  color: #eab308;
  animation: sc-blink 0.8s infinite;
}

.sc-badge.ok {
  background: #14532d55;
  color: #4ade80;
}

.sc-badge.fail {
  background: #7f1d1d55;
  color: var(--c-error);
}

.sc-hint {
  margin: 0;
  font-size: 0.78rem;
  color: var(--c-text-muted);
  line-height: 1.5;
}

.sc-actions {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.sc-buttons {
  display: flex;
  gap: 0.5rem;
  justify-content: flex-end;
}

.sc-fade-enter-active,
.sc-fade-leave-active {
  transition: opacity 0.25s;
}

.sc-fade-enter-from,
.sc-fade-leave-to {
  opacity: 0;
}

@keyframes sc-blink {

  0%,
  100% {
    opacity: 1;
  }

  50% {
    opacity: 0.4;
  }
}

/* ── Toast ──────────────────────────────────────────────────────────────── */
.toast-wrap {
  position: fixed;
  bottom: 24px;
  right: 20px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
}



.toast {
  background: var(--c-surface-2);
  border: 1px solid #4ade8055;
  border-radius: 8px;
  padding: 10px 16px;
  font-size: 0.85rem;
  font-weight: 600;
  color: #4ade80;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.5);
  min-width: 200px;
  backdrop-filter: blur(8px);
}

.toast-error {
  border-color: #f8717155;
  color: var(--c-error);
}

.toast-enter-active,
.toast-leave-active {
  transition: opacity 0.3s, transform 0.3s;
}

.toast-enter-from {
  opacity: 0;
  transform: translateY(12px);
}

.toast-leave-to {
  opacity: 0;
  transform: translateY(12px);
}
</style>
