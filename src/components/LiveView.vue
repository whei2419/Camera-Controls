<script setup>
import { ref, watch, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({ connected: Boolean })

const active = ref(false)
const frameData = ref('') // base64 JPEG
const fps = ref(0)
const error = ref('')

let rafId = null
let frameCount = 0
let fpsTimer = null

async function toggle() {
  if (active.value) {
    await stopLV()
  } else {
    await startLV()
  }
}

async function startLV() {
  error.value = ''
  try {
    await invoke('start_live_view')
    active.value = true
    frameCount = 0
    fpsTimer = setInterval(() => {
      fps.value = frameCount
      frameCount = 0
    }, 1000)
    scheduleFrame()
  } catch (e) {
    error.value = String(e)
  }
}

async function stopLV() {
  active.value = false
  if (rafId) cancelAnimationFrame(rafId)
  clearInterval(fpsTimer)
  fps.value = 0
  try { await invoke('stop_live_view') } catch {}
  frameData.value = ''
}

function scheduleFrame() {
  if (!active.value) return
  rafId = requestAnimationFrame(fetchFrame)
}

async function fetchFrame() {
  if (!active.value) return
  try {
    const b64 = await invoke('get_live_view_frame')
    frameData.value = b64
    frameCount++
  } catch (e) {
    // camera might report OBJECT_NOTREADY during rapid polling; retry
    if (!String(e).includes('0x00000881')) {
      error.value = String(e)
    }
  }
  scheduleFrame()
}

watch(() => props.connected, (val) => {
  if (!val && active.value) stopLV()
})

onUnmounted(() => {
  if (active.value) stopLV()
})
</script>

<template>
  <div class="panel lv-panel">
    <div class="panel-header">
      <h2>Live View</h2>
      <span v-if="active" class="fps-badge">{{ fps }} fps</span>
      <button
        class="btn btn-sm"
        :class="active ? 'btn-danger' : 'btn-primary'"
        :disabled="!connected"
        @click="toggle"
      >
        {{ active ? 'Stop' : 'Start' }}
      </button>
    </div>

    <div class="lv-viewport" :class="{ active }">
      <img
        v-if="active && frameData"
        :src="`data:image/jpeg;base64,${frameData}`"
        alt="Live view"
        class="lv-img"
      />
      <div v-else class="lv-placeholder">
        <span v-if="!connected">No camera</span>
        <span v-else-if="!active">Live view off</span>
        <span v-else>Waiting for frame…</span>
      </div>
    </div>

    <p v-if="error" class="error-msg">{{ error }}</p>
  </div>
</template>

<style scoped>
.lv-panel { display: flex; flex-direction: column; }
.panel-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-bottom: 0.75rem;
}
.panel-header h2 {
  margin: 0;
  flex: 1;
  font-size: 1rem;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--c-text-muted);
}
.fps-badge {
  font-size: 0.72rem;
  font-weight: 700;
  color: var(--c-accent);
  font-family: monospace;
  background: #0e2d1e;
  padding: 2px 8px;
  border-radius: 999px;
}

.lv-viewport {
  flex: 1;
  border: 2px solid var(--c-border);
  border-radius: 10px;
  overflow: hidden;
  background: #0a0a0a;
  min-height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  transition: border-color 0.25s;
}
.lv-viewport.active { border-color: var(--c-accent); }

.lv-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  display: block;
}

.lv-placeholder {
  color: var(--c-text-muted);
  font-size: 0.82rem;
  user-select: none;
}

.btn-sm { font-size: 0.78rem; padding: 0.3rem 0.75rem; }
.error-msg { margin-top: 0.5rem; color: var(--c-error); font-size: 0.82rem; }
</style>
