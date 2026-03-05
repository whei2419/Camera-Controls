<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const cameras = ref([])
const scanning = ref(false)
const connecting = ref(false)
const error = ref('')

const emit = defineEmits(['connected'])

async function scan() {
  scanning.value = true
  error.value = ''
  try {
    cameras.value = await invoke('list_cameras')
    if (cameras.value.length === 0) {
      error.value = 'No cameras found. Make sure the camera is on and connected via USB.'
    }
  } catch (e) {
    error.value = String(e)
  } finally {
    scanning.value = false
  }
}

async function connect(index) {
  connecting.value = true
  error.value = ''
  try {
    const info = await invoke('connect_camera', { index })
    emit('connected', info)
  } catch (e) {
    error.value = String(e)
  } finally {
    connecting.value = false
  }
}

onMounted(scan)
</script>

<template>
  <div class="panel connect-panel">
    <div class="panel-header">
      <h2>Camera Connection</h2>
      <button class="btn-icon" :disabled="scanning" @click="scan" title="Scan for cameras">
        <span :class="{ spin: scanning }">⟳</span>
      </button>
    </div>

    <div v-if="cameras.length === 0 && !scanning" class="empty-state">
      <p>No cameras detected.</p>
      <p class="hint">Connect your Canon camera via USB and make sure it is powered on.</p>
    </div>

    <ul v-else class="camera-list">
      <li v-for="cam in cameras" :key="cam.index" class="camera-item">
        <div class="camera-meta">
          <span class="camera-name">{{ cam.name }}</span>
          <span class="camera-port">{{ cam.port }}</span>
        </div>
        <button class="btn btn-primary" :disabled="connecting" @click="connect(cam.index)">
          {{ connecting ? 'Connecting…' : 'Connect' }}
        </button>
      </li>
    </ul>

    <p v-if="error" class="error-msg">{{ error }}</p>
  </div>
</template>

<style scoped>
.connect-panel {
  max-width: 480px;
  margin: 0 auto;
}
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem;
}
.panel-header h2 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--c-text-muted);
}
.btn-icon {
  background: none;
  border: 1px solid var(--c-border);
  color: var(--c-text);
  border-radius: 6px;
  width: 32px;
  height: 32px;
  font-size: 1.2rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s;
}
.btn-icon:hover { background: var(--c-surface-2); }
.btn-icon:disabled { opacity: 0.4; cursor: not-allowed; }
.spin { display: inline-block; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

.empty-state { text-align: center; padding: 2rem 1rem; color: var(--c-text-muted); }
.empty-state p { margin: 0.3rem 0; }
.hint { font-size: 0.8rem; }

.camera-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.5rem; }
.camera-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--c-surface-2);
  border: 1px solid var(--c-border);
  border-radius: 8px;
  padding: 0.75rem 1rem;
}
.camera-meta { display: flex; flex-direction: column; gap: 0.15rem; }
.camera-name { font-weight: 600; font-size: 0.95rem; }
.camera-port { font-size: 0.75rem; color: var(--c-text-muted); font-family: monospace; }
.error-msg { margin-top: 0.75rem; color: var(--c-error); font-size: 0.85rem; }
</style>
