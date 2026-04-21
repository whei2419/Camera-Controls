<script setup>
import { ref, watch, onMounted, onUnmounted } from 'vue'

const DC = 'http://localhost:5513'

const props = defineProps({ obsConnected: Boolean, connected: Boolean, obsInstance: Object })
const emit = defineEmits(['capture-success', 'record-saved'])

const active = ref(false)
const error = ref('')
const videoRef = ref(null)
const videoDevices = ref([])
const selectedDevice = ref('')
let stream = null

// Enumerate without opening any stream — avoids "Device in use" from a temp stream.
// Labels may be empty on first run (no prior permission); they populate after startFeed
// acquires the real stream and we re-enumerate.
async function loadDevices() {
  try {
    const devices = await navigator.mediaDevices.enumerateDevices()
    applyDeviceList(devices)
  } catch { /* no permission yet — list stays empty until startFeed */ }
}

function applyDeviceList(devices) {
  videoDevices.value = devices.filter(d => d.kind === 'videoinput')
  const obs = videoDevices.value.find(d =>
    d.label.toLowerCase().includes('obs') ||
    d.label.toLowerCase().includes('virtual')
  )
  // Keep current selection if still valid, otherwise pick OBS / first device
  const ids = videoDevices.value.map(d => d.deviceId)
  if (!selectedDevice.value || !ids.includes(selectedDevice.value)) {
    selectedDevice.value = obs
      ? obs.deviceId
      : (videoDevices.value[0]?.deviceId ?? '')
  }
}

async function startFeed() {
  error.value = ''
  try {
    stream = await navigator.mediaDevices.getUserMedia({
      video: selectedDevice.value
        ? { deviceId: { exact: selectedDevice.value } }
        : true,
      audio: false
    })
    if (videoRef.value) {
      videoRef.value.srcObject = stream
      await videoRef.value.play()
    }
    active.value = true

    // Re-enumerate now that we have permission — labels will be populated
    const devices = await navigator.mediaDevices.enumerateDevices()
    applyDeviceList(devices)
  } catch (e) {
    console.error('startFeed error', e)
    error.value = String(e)
  }
}

function stopFeed() {
  if (stream) {
    stream.getTracks().forEach(t => t.stop())
    stream = null
  }
  if (videoRef.value) {
    videoRef.value.srcObject = null
  }
  active.value = false
}

async function toggle() {
  if (active.value) stopFeed()
  else await startFeed()
}

// ── Capture (Canon camera — full quality) ────────────────────────────────────
const capturing = ref(false)
const captureMsg = ref('')

async function captureFrame() {
  if (!props.connected || capturing.value) return
  capturing.value = true
  captureMsg.value = ''
  try {
    // Snapshot existing files BEFORE the capture so App.vue can detect the new one
    let snapshotSet = new Set()
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const folder = localStorage.getItem('setting_image_path') || ''
      if (folder) {
        const before = await invoke('list_folder_files', {
          folder,
          extensions: ['jpg', 'jpeg', 'png', 'cr2', 'cr3', 'nef', 'arw', 'tif', 'tiff']
        })
        snapshotSet = new Set(before)
      }
    } catch { }

    await fetch(`${DC}/?CMD=Capture&_=${Date.now()}`, { mode: 'no-cors' })
    captureMsg.value = 'Saved!'
    setTimeout(() => { captureMsg.value = '' }, 1500)
    emit('capture-success', snapshotSet)
  } catch (e) {
    captureMsg.value = 'Failed'
    setTimeout(() => { captureMsg.value = '' }, 2000)
  } finally {
    capturing.value = false
  }
}

// ── Record (OBS — full quality) ───────────────────────────────────────────────
const recording = ref(false)
const recordingTime = ref(0)
let recordTimer = null

function onRecordStateChanged({ outputActive, outputPath }) {
  recording.value = outputActive
  if (outputActive) {
    recordingTime.value = 0
    recordTimer = setInterval(() => { recordingTime.value++ }, 1000)
  } else {
    clearInterval(recordTimer)
    if (outputPath) emit('record-saved', outputPath)
  }
}

// Attach / detach the RecordStateChanged listener as the OBS instance changes
watch(() => props.obsInstance, (obs, oldObs) => {
  if (oldObs) oldObs.off('RecordStateChanged', onRecordStateChanged)
  if (obs) obs.on('RecordStateChanged', onRecordStateChanged)
})

async function startRecording() {
  if (!props.obsInstance) return
  error.value = ''
  try { await props.obsInstance.call('StartRecord') } catch (e) { error.value = String(e) }
}

async function stopRecording() {
  if (!props.obsInstance) return
  try { await props.obsInstance.call('StopRecord') } catch (e) { error.value = String(e) }
}

function formatTime(s) {
  const m = Math.floor(s / 60).toString().padStart(2, '0')
  const sec = (s % 60).toString().padStart(2, '0')
  return `${m}:${sec}`
}

// Auto-start feed when OBS connects; stop when it disconnects
watch(() => props.obsConnected, async (val) => {
  if (val && !active.value) await startFeed()
  else if (!val && active.value) stopFeed()
})

// Debug helper — logs devices and attempts a quick getUserMedia test
async function debugDevices() {
  try {
    const devices = await navigator.mediaDevices.enumerateDevices()
    console.log('Video devices', devices.filter(d => d.kind === 'videoinput'))
    // Try to open a temporary stream to detect permission / hardware errors
    const testStream = await navigator.mediaDevices.getUserMedia({ video: true })
    console.log('getUserMedia test succeeded')
    testStream.getTracks().forEach(t => t.stop())
  } catch (err) {
    console.error('getUserMedia test failed', err)
    error.value = String(err)
  }
}

onMounted(loadDevices)
onUnmounted(() => {
  if (active.value) stopFeed()
  clearInterval(recordTimer)
  if (props.obsInstance) props.obsInstance.off('RecordStateChanged', onRecordStateChanged)
})

// Expose control methods to parent (used by Pusher triggers)
defineExpose({
  captureFrame,
  startRecording,
  stopRecording,
  toggle
})
</script>

<template>
  <div class="lv-panel">

    <!-- ── Toolbar ── -->
    <div class="lv-toolbar">

      <select
        v-if="videoDevices.length > 0"
        v-model="selectedDevice"
        class="device-select"
        :disabled="active"
        title="Select video source"
      >
        <option v-for="d in videoDevices" :key="d.deviceId" :value="d.deviceId">
          {{ d.label || 'Camera ' + (videoDevices.indexOf(d) + 1) }}
        </option>
      </select>
      <span v-else class="no-device">No devices</span>

      <div class="toolbar-sep"></div>

      <div class="toolbar-sep"></div>

      <!-- Debug: list devices & test permission -->
      <button class="tbtn" title="Debug devices" @click="debugDevices" style="opacity:0.8">
        🔍
      </button>

      <!-- Shoot -->
      <button
        class="tbtn tbtn-shoot"
        :class="{ ok: captureMsg === 'Saved!', err: captureMsg === 'Failed' }"
        :disabled="!connected || capturing"
        title="Capture photo (Canon)"
        @click="captureFrame"
      >
        <span class="tbtn-icon">📷</span>
        <span class="tbtn-label">{{ capturing ? '…' : captureMsg || 'Shoot' }}</span>
      </button>

      <!-- Record -->
      <button
        class="tbtn tbtn-rec"
        :class="{ active: recording }"
        :disabled="!obsInstance"
        :title="obsInstance ? (recording ? 'Stop OBS recording' : 'Record via OBS') : 'Connect OBS to record'"
        @click="recording ? stopRecording() : startRecording()"
      >
        <span class="rec-dot"></span>
        <span class="tbtn-label">{{ recording ? formatTime(recordingTime) : 'Record' }}</span>
      </button>

      <div class="toolbar-sep"></div>

      <!-- Feed toggle -->
      <button
        class="tbtn tbtn-feed"
        :class="active ? 'feed-on' : 'feed-off'"
        :disabled="videoDevices.length === 0"
        :title="active ? 'Disconnect feed' : (videoDevices.length ? 'Connect feed' : 'No video device')"
        @click="toggle"
      >
        <span class="feed-dot" :class="{ live: active }"></span>
        <span class="tbtn-label">{{ active ? 'Live' : 'Offline' }}</span>
      </button>

    </div>

    <!-- Viewport -->
    <div class="lv-viewport-wrap">
      <div class="lv-viewport" :class="{ active }">
        <video
          ref="videoRef"
          v-show="active"
          class="lv-video"
          autoplay
          playsinline
          muted
        />
        <div v-if="!active" class="lv-placeholder">
          <span v-if="videoDevices.length === 0">No video devices found</span>
          <span v-else>Connect a device using the selector and press Live</span>
        </div>
      </div>
    </div>

    <p v-if="error" class="error-msg">{{ error }}</p>
  </div>
</template>

<style scoped>
.lv-panel {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
}

/* ── Toolbar ─────────────────────────────────────────────────────────────── */
.lv-toolbar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border-bottom: 1px solid var(--c-border);
  background: var(--c-surface);
  flex-shrink: 0;
}

.toolbar-sep {
  width: 1px;
  height: 20px;
  background: var(--c-border);
  margin: 0 2px;
  flex-shrink: 0;
}

.device-select {
  background: var(--c-surface-2);
  border: 1px solid var(--c-border);
  border-radius: 6px;
  color: var(--c-text);
  font-size: 0.75rem;
  padding: 0.28rem 0.5rem;
  outline: none;
  max-width: 160px;
  cursor: pointer;
  height: 30px;
}
.device-select:disabled { opacity: 0.45; cursor: not-allowed; }
.device-select:focus { border-color: var(--c-accent); }

.no-device {
  font-size: 0.72rem;
  color: var(--c-text-muted);
  font-style: italic;
}

/* ── Toolbar buttons ─────────────────────────────────────────────────────── */
.tbtn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  height: 30px;
  padding: 0 12px;
  border-radius: 6px;
  border: 1px solid transparent;
  font-family: inherit;
  font-size: 0.78rem;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.13s, border-color 0.13s, opacity 0.13s;
  white-space: nowrap;
  flex-shrink: 0;
}
.tbtn:disabled { opacity: 0.35; cursor: not-allowed; }
.tbtn:not(:disabled):active { opacity: 0.75; }

.tbtn-icon { font-size: 0.9rem; line-height: 1; }
.tbtn-label { font-size: 0.76rem; }

/* Shoot */
.tbtn-shoot {
  background: var(--c-surface-2);
  color: var(--c-text);
  border-color: var(--c-border);
}
.tbtn-shoot:not(:disabled):hover { background: var(--c-border); }
.tbtn-shoot.ok  { color: #4ade80; border-color: #4ade8055; background: rgba(74,222,128,0.08); }
.tbtn-shoot.err { color: var(--c-error); border-color: var(--c-error); }

/* Record */
.tbtn-rec {
  background: rgba(127,29,29,0.35);
  color: #fca5a5;
  border-color: #7f1d1d;
  display: flex;
  align-items: center;
  gap: 6px;
}
.tbtn-rec:not(:disabled):hover { background: rgba(153,27,27,0.5); }
.tbtn-rec.active {
  background: rgba(220,38,38,0.55);
  color: #fff;
  border-color: #ef4444;
  animation: rec-pulse 1.4s infinite;
}

.rec-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  flex-shrink: 0;
}

@keyframes rec-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.55; }
}

/* Feed toggle */
.tbtn-feed {
  background: var(--c-surface-2);
  color: var(--c-text-muted);
  border-color: var(--c-border);
  display: flex;
  align-items: center;
  gap: 6px;
}
.tbtn-feed.feed-on {
  background: rgba(59,130,246,0.12);
  color: var(--c-accent);
  border-color: var(--c-accent);
}
.tbtn-feed.feed-off:not(:disabled):hover { background: var(--c-surface-2); color: var(--c-text); }

.feed-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--c-text-muted);
  flex-shrink: 0;
  transition: background 0.2s;
}
.feed-dot.live {
  background: #4ade80;
  box-shadow: 0 0 5px #4ade80aa;
  animation: live-blink 2s infinite;
}

@keyframes live-blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

/* ── Viewport ────────────────────────────────────────────────────────────── */
.lv-viewport-wrap {
  flex: 1;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #0a0a0a;
  padding: 0.75rem;
}

.lv-viewport {
  aspect-ratio: 9 / 16;
  height: 100%;
  max-height: 100%;
  border: 2px solid var(--c-border);
  border-radius: 10px;
  overflow: hidden;
  background: #0a0a0a;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  transition: border-color 0.25s;
}

.lv-viewport.active { border-color: var(--c-accent); }

.lv-video {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.lv-placeholder {
  color: var(--c-text-muted);
  font-size: 0.82rem;
  user-select: none;
}

.error-msg { margin: 0.4rem 1rem; color: var(--c-error); font-size: 0.8rem; }
</style>
