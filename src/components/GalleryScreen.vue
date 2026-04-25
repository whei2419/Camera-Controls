<script setup>
import { ref, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'
import { openPath } from '@tauri-apps/plugin-opener'

const props = defineProps({
  show: Boolean,
  imageFolder: { type: String, default: '' },
  videoFolder: { type: String, default: '' },
})

const emit = defineEmits(['close', 'update:autoPrint'])

// ── Auto-print ──────────────────────────────────────────────────────────────
const AUTO_PRINT_KEY = 'setting_auto_print'
const autoPrint = ref(localStorage.getItem(AUTO_PRINT_KEY) === 'true')
watch(autoPrint, v => {
  localStorage.setItem(AUTO_PRINT_KEY, String(v))
  emit('update:autoPrint', v)
})

// ── Gallery files ───────────────────────────────────────────────────────────
const IMG_EXTS = ['jpg', 'jpeg', 'png', 'cr2', 'cr3', 'nef', 'arw', 'tif', 'tiff']
const VID_EXTS = ['mp4', 'mkv', 'mov', 'avi', 'wmv', 'webm', 'mts', 'm2ts']

const images = ref([])
const videos = ref([])
const loading = ref(false)
const selected = ref(null)   // { src, path, type }
const tab = ref('photos')    // 'photos' | 'videos'

async function refresh() {
  loading.value = true
  images.value = []
  videos.value = []
  try {
    if (props.imageFolder) {
      const files = await invoke('list_folder_files', {
        folder: props.imageFolder,
        extensions: IMG_EXTS,
      })
      images.value = files.map(p => ({ path: p, src: convertFileSrc(p) }))
    }
    if (props.videoFolder) {
      const files = await invoke('list_folder_files', {
        folder: props.videoFolder,
        extensions: VID_EXTS,
      })
      videos.value = files.map(p => ({ path: p, src: convertFileSrc(p) }))
    }
  } catch { /* folders may not exist yet */ }
  loading.value = false
}

watch(() => props.show, v => { if (v) refresh() })

function fileName(p) { return p.split(/[\\/]/).pop() }

function selectImage(item) {
  selected.value = { ...item, type: 'photo' }
}

function selectVideo(item) {
  selected.value = { ...item, type: 'video' }
}

async function openFile(path) {
  try { await openPath(path) } catch { await navigator.clipboard.writeText(path).catch(() => { }) }
}

// ── Print ───────────────────────────────────────────────────────────────────
const PAPER_CSS = {
  '4x6': '6in 4in',
  '5x7': '7in 5in',
  '4x4': '4in 4in',
  'letter': '8.5in 11in',
  'a4': '210mm 297mm',
  'a5': '148mm 210mm',
}

async function printImage(src, filePath = '') {
  const printer = localStorage.getItem('setting_printer_name') || ''
  const paper = localStorage.getItem('setting_paper_size') || '4x6'
  const orient = localStorage.getItem('setting_orientation') || 'portrait'
  const copies = Number(localStorage.getItem('setting_copies')) || 1
  const fit = localStorage.getItem('setting_print_fit') || 'contain'

  // If a printer is configured and we have a real file path → print directly (no dialog)
  if (printer && filePath) {
    try {
      for (let i = 0; i < copies; i++) {
        await invoke('print_file', { path: filePath, printer })
        if (i < copies - 1) await new Promise(r => setTimeout(r, 800))
      }
      return
    } catch { /* fall through to browser print */ }
  }

  // Browser print fallback (shows print dialog)
  const [pw, ph] = (PAPER_CSS[paper] || '6in 4in').split(' ')
  const pageSize = orient === 'landscape' ? `${ph} ${pw}` : `${pw} ${ph}`
  const copiesAttr = copies > 1 ? `<style>@page{size:${pageSize};margin:0;-webkit-print-copies:${copies}}</style>` : `<style>@page{size:${pageSize};margin:0}</style>`

  for (let i = 0; i < copies; i++) {
    const iframe = document.createElement('iframe')
    iframe.style.cssText = 'position:fixed;top:-9999px;left:-9999px;width:1px;height:1px;border:0'
    iframe.srcdoc = `<!DOCTYPE html><html><head>${copiesAttr}<style>
      *{margin:0;padding:0;box-sizing:border-box}
      html,body{width:${pw};height:${ph}}
      body{display:flex;align-items:center;justify-content:center;background:#fff}
      img{width:100%;height:100%;object-fit:${fit}}
      @media print{html,body{width:${pw};height:${ph}}}
    </style></head><body><img src="${src}" onload="window.print();setTimeout(()=>window.frameElement&&window.frameElement.remove(),800)"/></body></html>`
    document.body.appendChild(iframe)
    setTimeout(() => { try { document.body.removeChild(iframe) } catch { } }, 6000)
    if (i < copies - 1) await new Promise(r => setTimeout(r, 1000))
  }
}

// Exposed so parent can call this after a capture
defineExpose({ printImage, autoPrint })

// Items shown in grid based on active tab
const gridItems = computed(() => tab.value === 'photos' ? images.value : videos.value)
</script>

<template>
  <teleport to="body">
    <transition name="gs-fade">
      <div v-if="show" class="gs-overlay" @click.self="$emit('close')">
        <div class="gs-modal">

          <!-- Header -->
          <div class="gs-header">
            <div class="gs-tabs">
              <button :class="['gs-tab', { active: tab === 'photos' }]" @click="tab = 'photos'">
                📷 Photos <span class="gs-count">{{ images.length }}</span>
              </button>
              <button :class="['gs-tab', { active: tab === 'videos' }]" @click="tab = 'videos'">
                🎬 Videos <span class="gs-count">{{ videos.length }}</span>
              </button>
            </div>

            <div class="gs-header-actions">
              <!-- Auto-print toggle -->
              <label class="auto-print-toggle" title="Automatically print each photo after capture">
                <input type="checkbox" v-model="autoPrint" />
                <span class="toggle-track"><span class="toggle-thumb"></span></span>
                <span class="toggle-label">Auto-print</span>
              </label>

              <button class="gs-icon-btn" title="Refresh" @click="refresh">⟳</button>
              <button class="gs-icon-btn gs-close" @click="$emit('close')">✕</button>
            </div>
          </div>

          <!-- Body -->
          <div class="gs-body">

            <!-- Loading -->
            <div v-if="loading" class="gs-loading">Loading files…</div>

            <!-- Empty -->
            <div v-else-if="gridItems.length === 0" class="gs-empty">
              <p v-if="tab === 'photos'">
                {{ imageFolder ? 'No photos found in: ' + imageFolder : 'Set an image save path in File Paths settings.'
                }}
              </p>
              <p v-else>
                {{ videoFolder ? 'No videos found in: ' + videoFolder : 'Set a video save path in File Paths settings.'
                }}
              </p>
            </div>

            <!-- Photo grid -->
            <div v-else-if="tab === 'photos'" class="gs-grid">
              <div v-for="img in images" :key="img.path" class="gs-thumb"
                :class="{ selected: selected?.path === img.path }" @click="selectImage(img)">
                <img :src="img.src" :alt="fileName(img.path)" loading="lazy" />
                <div class="gs-thumb-actions">
                  <button title="Print" @click.stop="printImage(img.src, img.path)">🖨</button>
                  <button title="Open" @click.stop="openFile(img.path)">↗</button>
                </div>
                <span class="gs-thumb-name">{{ fileName(img.path) }}</span>
              </div>
            </div>

            <!-- Video list -->
            <div v-else class="gs-video-list">
              <div v-for="vid in videos" :key="vid.path" class="gs-video-item">
                <span class="gs-video-icon">🎬</span>
                <span class="gs-video-name" :title="vid.path">{{ fileName(vid.path) }}</span>
                <button class="gs-action-btn" @click="selectVideo(vid)">▶ Preview</button>
                <button class="gs-action-btn" @click="openFile(vid.path)">▶ Open</button>
              </div>
            </div>
          </div>

          <!-- Lightbox -->
          <transition name="gs-fade">
            <div v-if="selected" class="gs-lightbox" @click.self="selected = null">
              <img v-if="selected.type === 'photo'" :src="selected.src" class="gs-lightbox-img" />
              <video v-else :src="selected.src" class="gs-lightbox-video" controls autoplay playsinline
                preload="metadata"></video>
              <div class="gs-lightbox-bar">
                <span class="gs-lightbox-name">{{ fileName(selected.path) }}</span>
                <button v-if="selected.type === 'photo'" @click="printImage(selected.src, selected.path)">🖨
                  Print</button>
                <button @click="openFile(selected.path)">↗ Open</button>
                <button @click="selected = null">✕ Close</button>
              </div>
            </div>
          </transition>

        </div>
      </div>
    </transition>
  </teleport>
</template>

<style scoped>
/* ── Overlay / modal shell ── */
.gs-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.82);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(4px);
}

.gs-modal {
  background: var(--c-surface);
  border: 1px solid var(--c-border);
  border-radius: 12px;
  width: min(95vw, 1100px);
  height: min(88vh, 800px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.7);
}

/* ── Header ── */
.gs-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-bottom: 1px solid var(--c-border);
  flex-shrink: 0;
  gap: 1rem;
}

.gs-tabs {
  display: flex;
  gap: 4px;
}

.gs-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border-radius: 6px;
  border: 1px solid transparent;
  background: none;
  color: var(--c-text-muted);
  font-family: inherit;
  font-size: 0.82rem;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.13s, color 0.13s;
}

.gs-tab:hover {
  background: var(--c-surface-2);
  color: var(--c-text);
}

.gs-tab.active {
  background: var(--c-surface-2);
  color: var(--c-text);
  border-color: var(--c-border);
}

.gs-count {
  background: var(--c-border);
  border-radius: 999px;
  padding: 1px 7px;
  font-size: 0.7rem;
  color: var(--c-text-muted);
}

.gs-header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.gs-icon-btn {
  background: none;
  border: 1px solid var(--c-border);
  color: var(--c-text-muted);
  border-radius: 6px;
  width: 30px;
  height: 30px;
  font-size: 0.9rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.13s;
}

.gs-icon-btn:hover {
  background: var(--c-surface-2);
  color: var(--c-text);
}

.gs-close:hover {
  color: var(--c-error);
  border-color: var(--c-error);
}

/* ── Auto-print toggle ── */
.auto-print-toggle {
  display: flex;
  align-items: center;
  gap: 7px;
  cursor: pointer;
  user-select: none;
}

.auto-print-toggle input {
  display: none;
}

.toggle-track {
  width: 34px;
  height: 18px;
  background: var(--c-border);
  border-radius: 999px;
  position: relative;
  transition: background 0.2s;
  flex-shrink: 0;
}

.auto-print-toggle input:checked~.toggle-track {
  background: var(--c-accent);
}

.toggle-thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 14px;
  height: 14px;
  background: #fff;
  border-radius: 50%;
  transition: transform 0.2s;
}

.auto-print-toggle input:checked~.toggle-track .toggle-thumb {
  transform: translateX(16px);
}

.toggle-label {
  font-size: 0.78rem;
  color: var(--c-text-muted);
  white-space: nowrap;
}

/* ── Body ── */
.gs-body {
  flex: 1;
  overflow-y: auto;
  padding: 14px;
}

.gs-loading,
.gs-empty {
  text-align: center;
  padding: 3rem 1rem;
  color: var(--c-text-muted);
  font-size: 0.85rem;
  line-height: 1.6;
}

/* ── Photo grid ── */
.gs-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 10px;
}

.gs-thumb {
  position: relative;
  aspect-ratio: 1;
  background: var(--c-surface-2);
  border: 2px solid var(--c-border);
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  transition: border-color 0.15s;
}

.gs-thumb:hover {
  border-color: var(--c-accent);
}

.gs-thumb.selected {
  border-color: var(--c-accent);
}

.gs-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.gs-thumb-name {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.75));
  color: #fff;
  font-size: 0.6rem;
  padding: 14px 4px 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  opacity: 0;
  transition: opacity 0.15s;
}

.gs-thumb:hover .gs-thumb-name {
  opacity: 1;
}

.gs-thumb-actions {
  position: absolute;
  top: 4px;
  right: 4px;
  display: flex;
  gap: 3px;
  opacity: 0;
  transition: opacity 0.15s;
}

.gs-thumb:hover .gs-thumb-actions {
  opacity: 1;
}

.gs-thumb-actions button {
  background: rgba(0, 0, 0, 0.65);
  border: none;
  border-radius: 4px;
  color: #fff;
  font-size: 0.75rem;
  width: 22px;
  height: 22px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.gs-thumb-actions button:hover {
  background: rgba(0, 0, 0, 0.9);
}

/* ── Video list ── */
.gs-video-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.gs-video-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: var(--c-surface-2);
  border: 1px solid var(--c-border);
  border-radius: 8px;
}

.gs-video-icon {
  font-size: 1.1rem;
  flex-shrink: 0;
}

.gs-video-name {
  flex: 1;
  font-size: 0.82rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.gs-action-btn {
  flex-shrink: 0;
  background: var(--c-surface);
  border: 1px solid var(--c-border);
  color: var(--c-text);
  border-radius: 6px;
  padding: 4px 10px;
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
  font-family: inherit;
}

.gs-action-btn:hover {
  background: var(--c-border);
}

/* ── Lightbox ── */
.gs-lightbox {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.92);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  z-index: 10;
  gap: 12px;
}

.gs-lightbox-img {
  max-width: calc(100% - 40px);
  max-height: calc(100% - 80px);
  object-fit: contain;
  border-radius: 6px;
}

.gs-lightbox-video {
  max-width: calc(100% - 40px);
  max-height: calc(100% - 80px);
  width: min(960px, calc(100% - 40px));
  background: #000;
  border-radius: 6px;
}

.gs-lightbox-bar {
  display: flex;
  align-items: center;
  gap: 10px;
}

.gs-lightbox-name {
  font-size: 0.78rem;
  color: var(--c-text-muted);
  font-family: monospace;
}

.gs-lightbox-bar button {
  background: var(--c-surface-2);
  border: 1px solid var(--c-border);
  color: var(--c-text);
  border-radius: 6px;
  padding: 5px 12px;
  font-size: 0.78rem;
  font-weight: 600;
  cursor: pointer;
  font-family: inherit;
}

.gs-lightbox-bar button:hover {
  background: var(--c-border);
}

/* ── Transitions ── */
.gs-fade-enter-active,
.gs-fade-leave-active {
  transition: opacity 0.2s;
}

.gs-fade-enter-from,
.gs-fade-leave-to {
  opacity: 0;
}
</style>
