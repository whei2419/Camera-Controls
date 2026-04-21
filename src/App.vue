<script setup>
import { ref, onMounted } from 'vue'
import GalleryScreen from './components/GalleryScreen.vue'
import LiveView from './components/LiveView.vue'
import SettingsModal from './components/SettingsModal.vue'
import ThumbnailGallery from './components/ThumbnailGallery.vue'

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

// Keep folder refs in sync when the gallery screen opens
function openGalleryScreen() {
  imageFolder.value = localStorage.getItem('setting_image_path') || ''
  videoFolder.value = localStorage.getItem('setting_video_path') || ''
  showGalleryScreen.value = true
}

function onCaptureSuccess() {
  const folder = localStorage.getItem('setting_image_path') || ''
  addToast('📷 Photo captured!')
  pushGalleryItem({ type: 'photo', folder, path: folder, ts: Date.now() })
  // Refresh thumbnails
  refreshThumbnails()
  // Auto-print: ask the gallery screen to print the latest image
  if (autoPrint.value && folder && galleryScreenRef.value) {
    // Give DigiCamControl a moment to save the file, then refresh + print newest
    setTimeout(async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        const { convertFileSrc } = await import('@tauri-apps/api/core')
        const files = await invoke('list_folder_files', {
          folder,
          extensions: ['jpg', 'jpeg', 'png', 'cr2', 'cr3', 'nef', 'arw', 'tif', 'tiff']
        })
        if (files.length > 0) {
          galleryScreenRef.value.printImage(convertFileSrc(files[0]), files[0])
        }
      } catch { }
    }, 2500)
  }
}

function onRecordSaved(path) {
  const name = path.split(/[\\\/]/).pop()
  addToast(`🎬 Saved: ${name}`)
  pushGalleryItem({ type: 'video', path, folder: '', ts: Date.now() })
}

onMounted(loadGallery)
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
        <div v-if="connected && cameraInfo" class="connection-badge">
          <span class="dot dot-live"></span>
          {{ cameraInfo.name }}
          <button class="btn btn-ghost btn-xs" @click="disconnect">Disconnect</button>
        </div>
        <button class="btn btn-ghost btn-xs settings-btn" @click="showSettingsModal = true">⚙ Settings</button>
        <button class="btn btn-ghost btn-xs gallery-btn" @click="openGalleryScreen">🖼 Gallery</button>
      </div>
    </header>

    <!-- ── Body ───────────────────────────────────────────────────────────── -->
    <div class="app-body">

      <!-- Left: Thumbnail gallery -->
      <ThumbnailGallery
        :image-folder="imageFolder"
        :refresh-trigger="thumbnailRefreshTrigger"
        @open-gallery="openGalleryScreen"
      />

      <!-- Main area: live view only -->
      <section class="main-area">
        <LiveView
          :obs-connected="obsConnected"
          :connected="connected"
          :obs-instance="obsInfo?.obs ?? null"
          @capture-success="onCaptureSuccess"
          @record-saved="onRecordSaved"
        />
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
  <SettingsModal
    :show="showSettingsModal"
    :obs-connected="obsConnected"
    :obs-info="obsInfo"
    :connected="connected"
    :gallery-items="gallery"
    :obs-instance="obsInfo?.obs ?? null"
    @close="showSettingsModal = false"
    @obs-connected="onOBSConnected"
    @obs-disconnected="onOBSDisconnected"
    @camera-connected="onConnected"
    @clear-gallery="clearGallery"
  />

  <!-- ── Settings Modal ───────────────────────────────────────── -->
  <SettingsModal
    :show="showSettingsModal"
    :obs-connected="obsConnected"
    :obs-info="obsInfo"
    :connected="connected"
    :gallery-items="gallery"
    :obs-instance="obsInfo?.obs ?? null"
    @close="showSettingsModal = false"
    @obs-connected="onOBSConnected"
    @obs-disconnected="onOBSDisconnected"
    @camera-connected="onConnected"
    @clear-gallery="clearGallery"
  />

  <!-- ── Gallery Screen ───────────────────────────────────────── -->
  <GalleryScreen
    ref="galleryScreenRef"
    :show="showGalleryScreen"
    :image-folder="imageFolder"
    :video-folder="videoFolder"
    @close="showGalleryScreen = false"
    @update:auto-print="v => { autoPrint = v }"
  />
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
  box-shadow: 0 4px 24px rgba(0,0,0,0.5);
  min-width: 200px;
  backdrop-filter: blur(8px);
}

.toast-enter-active, .toast-leave-active { transition: opacity 0.3s, transform 0.3s; }
.toast-enter-from { opacity: 0; transform: translateY(12px); }
.toast-leave-to   { opacity: 0; transform: translateY(12px); }
</style>
