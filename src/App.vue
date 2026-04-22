<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import GalleryScreen from './components/GalleryScreen.vue'
import LiveView from './components/LiveView.vue'
import SettingsModal from './components/SettingsModal.vue'
import ThumbnailGallery from './components/ThumbnailGallery.vue'
import OBSConnect from './components/OBSConnect.vue'
import CameraConnect from './components/CameraConnect.vue'
import { initPusher, disconnectPusher } from './lib/pusherClient'
import { remoteSite, uploadCaptureFormField, uploadCaptureSecret } from './config/remoteSite.js'

// ── DigiCamControl (camera settings) ──
const connected = ref(false)
const cameraInfo = ref(null)

async function onConnected(info) {
  cameraInfo.value = info
  connected.value = true
}

async function disconnect() {
  connected.value = false
  cameraInfo.value = null
}

// ── OBS WebSocket (live feed) ──
const obsConnected = ref(false)
const obsInfo = ref(null)

function onOBSConnected(info) {
  obsInfo.value = info
  obsConnected.value = true
}

function onOBSDisconnected() {
  obsConnected.value = false
  obsInfo.value = null
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
  console.log('pushGalleryItem', item)
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

// ── Thumbnail refresh ─────────────────────────────────────────────
const thumbnailRefreshTrigger = ref(0)

function refreshThumbnails() {
  thumbnailRefreshTrigger.value++
  // Also update imageFolder ref for ThumbnailGallery
  imageFolder.value = localStorage.getItem('setting_image_path') || ''
}

// ── Gallery screen ────────────────────────────────────────────────
const showGalleryScreen = ref(false)
const galleryScreenRef = ref(null)
const autoPrint = ref(localStorage.getItem('setting_auto_print') === 'true')
const imageFolder = ref(localStorage.getItem('setting_image_path') || '')
const videoFolder = ref(localStorage.getItem('setting_video_path') || '')

// Ref for calling LiveView methods
const liveViewRef = ref(null)
let recordStopTimer = null

// Pusher connection state
const pusherConnected = ref(false)

// Keep folder refs in sync when the gallery screen opens
function openGalleryScreen() {
  imageFolder.value = localStorage.getItem('setting_image_path') || ''
  videoFolder.value = localStorage.getItem('setting_video_path') || ''
  showGalleryScreen.value = true
}

async function onCaptureSuccess(captureStartMs = Date.now()) {
  const folder = localStorage.getItem('setting_image_path') || ''
  addToast('📷 Photo captured!')

  if (!folder) return

  const { invoke, convertFileSrc } = await import('@tauri-apps/api/core')
  const EXTS = ['jpg', 'jpeg', 'png', 'cr2', 'cr3', 'nef', 'arw', 'tif', 'tiff']

  console.log('[capture] waiting for file modified after', new Date(captureStartMs).toISOString())

  // Add a placeholder gallery entry while we wait
  pushGalleryItem({ type: 'photo', folder, path: '', ts: captureStartMs })
  refreshThumbnails()

  // Poll every 500 ms (up to 20 s) for any file whose mtime >= captureStartMs
  // Using since_ms covers both new files AND overwritten files with same name
  const MAX_ATTEMPTS = 40
  for (let i = 0; i < MAX_ATTEMPTS; i++) {
    await new Promise(r => setTimeout(r, 500))
    try {
      const files = await invoke('list_folder_files', { folder, extensions: EXTS, sinceMs: captureStartMs })
      const newest = files[0] ?? null
      console.log('[capture] poll', i, 'files since capture:', files.length, '| newest:', newest)
      if (!newest) continue

      // New file detected — replace the placeholder in the gallery
      const idx = gallery.value.findIndex(g => g.type === 'photo' && g.folder === folder && !g.path)
      const item = { type: 'photo', folder, path: newest, ts: Date.now() }
      if (idx >= 0) gallery.value.splice(idx, 1, item)
      else gallery.value.unshift(item)
      localStorage.setItem(GALLERY_KEY, JSON.stringify(gallery.value))
      refreshThumbnails()

      // Upload
      console.log('[capture] uploading', newest, '->', remoteSite.uploadCapture, '| field:', uploadCaptureFormField, '| secret:', uploadCaptureSecret ? 'set' : 'none')
      try {
        await invoke('upload_capture_file', {
          filePath: newest,
          url: remoteSite.uploadCapture,
          fieldName: uploadCaptureFormField,
          sharedSecret: uploadCaptureSecret,
        })
        console.log('[capture] upload ok')
        addToast('☁️ Uploaded to server')
      } catch (uploadErr) {
        console.error('[capture] upload_capture_file failed:', uploadErr)
        addToast('Upload to server failed', 'error')
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
}

function onRecordSaved(path) {
  const name = path.split(/[\\\/]/).pop()
  addToast(`🎬 Saved: ${name}`)
  pushGalleryItem({ type: 'video', path, folder: '', ts: Date.now() })
  // Upload via chunked multipart — avoids timeouts on large video files
  import('@tauri-apps/api/core').then(({ invoke }) => {
    addToast('⏫ Uploading video (chunked)…')
    invoke('upload_video_chunked', {
      filePath: path,
      urlChunk: remoteSite.uploadVideoChunk,
      urlAssemble: remoteSite.uploadVideoAssemble,
      sharedSecret: uploadCaptureSecret || null,
    })
      .then(() => addToast('☁️ Video uploaded!'))
      .catch((e) => {
        console.error('[video upload] failed:', e)
        addToast('Video upload failed', 'error')
      })
  })
}

onMounted(loadGallery)
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
        const mode = (data && typeof data.mode === 'string') ? data.mode.toLowerCase() : 'photo'
        const durationSec = Math.max(3, Math.min(30, Number(data?.durationSec) || 10))

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
        if (liveViewRef.value?.captureFrame) {
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
</script>

<template>
  <main class="app-shell">

    <!-- ── Header ──────────────────────────────────────────────────────────── -->
    <header class="app-header">
      <div class="brand">
        <span class="brand-icon">◉</span>
        <span class="brand-name">Canon Control</span>
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
        <button class="btn btn-ghost btn-xs settings-btn" @click="showSettingsModal = true">⚙ Settings</button>
        <button class="btn btn-ghost btn-xs gallery-btn" @click="openGalleryScreen">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"
            style="vertical-align:middle;margin-right:6px">
            <path d="M21 19V5H3V19H21Z" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"
              stroke-linejoin="round" />
            <path d="M8 11L10.5 14L13 11L16 16H8L8 11Z" stroke="currentColor" stroke-width="1" />
          </svg>
          Gallery
        </button>
      </div>
    </header>

    <!-- ── Body ───────────────────────────────────────────────────────────── -->
    <div class="app-body">

      <!-- Left: Thumbnail gallery -->
      <ThumbnailGallery :image-folder="imageFolder" :video-folder="videoFolder"
        :refresh-trigger="thumbnailRefreshTrigger" @open-gallery="openGalleryScreen" />

      <!-- Keep OBS connect mounted (hidden) so app stays connected to OBS even when settings modal is closed -->
      <div style="display:none">
        <OBSConnect @connected="onOBSConnected" @disconnected="onOBSDisconnected" />
      </div>
      <!-- Keep Camera connect mounted (hidden) so app can ping & detect DigiCamControl on startup -->
      <div style="display:none">
        <CameraConnect @connected="onConnected" />
      </div>

      <!-- Main area: live view only -->
      <section class="main-area">
        <LiveView ref="liveViewRef" :obs-connected="obsConnected" :connected="connected"
          :obs-instance="obsInfo?.obs ?? null" @capture-success="onCaptureSuccess" @record-saved="onRecordSaved" />
      </section>

    </div>
  </main>


  <!-- ── Toast notifications ─────────────────────────────────────────── -->
  <div class="toast-wrap">
    <transition-group name="toast">
      <div v-for="t in toasts" :key="t.id" class="toast">
        {{ t.message }}
      </div>
    </transition-group>
  </div>

  <!-- ── Settings Modal ───────────────────────────────────────── -->
  <SettingsModal :show="showSettingsModal" :obs-connected="obsConnected" :obs-info="obsInfo" :connected="connected"
    :gallery-items="gallery" :obs-instance="obsInfo?.obs ?? null" @close="showSettingsModal = false"
    @obs-connected="onOBSConnected" @obs-disconnected="onOBSDisconnected" @camera-connected="onConnected"
    @clear-gallery="clearGallery" />

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
