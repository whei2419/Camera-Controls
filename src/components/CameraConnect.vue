<script setup>
import { ref, onMounted } from 'vue'

const DC = 'http://localhost:5513'

const connecting = ref(false)
const error = ref('')
const PHOTO_CAPTURE_SOURCE_KEY = 'setting_photo_capture_source'

const emit = defineEmits(['connected'])

async function connect() {
  connecting.value = true
  error.value = ''
  try {
    await fetch(`${DC}/session.json?_=${Date.now()}`, {
      mode: 'no-cors',
      signal: AbortSignal.timeout(4000)
    })
    // Any response (even an error page) means DigiCamControl's WebServer is up
    emit('connected', { name: 'DigiCamControl', port: '5513' })
  } catch (e) {
    error.value = `Cannot reach DigiCamControl at ${DC}. Make sure it is running with the WebServer plugin enabled (port 5513).`
  } finally {
    connecting.value = false
  }
}

function shouldAutoConnectDigicam() {
  const source = (localStorage.getItem(PHOTO_CAPTURE_SOURCE_KEY) || 'digicamcontrol').toLowerCase()
  return source !== 'obs'
}

// Auto-connect on mount (try to ping the local DigiCamControl server)
onMounted(() => {
  if (shouldAutoConnectDigicam()) {
    connect().catch(() => { })
  }
})
</script>

<template>
  <div class="panel connect-panel">
    <div class="panel-header">
      <h2>Camera</h2>
    </div>
    <p class="hint">Make sure DigiCamControl is running with the WebServer plugin enabled on port 5513.</p>
    <button class="btn btn-primary" style="width:100%;margin-top:0.75rem" :disabled="connecting" @click="connect">
      {{ connecting ? 'Connecting…' : 'Connect to DigiCamControl' }}
    </button>
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

.btn-icon:hover {
  background: var(--c-surface-2);
}

.btn-icon:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.spin {
  display: inline-block;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.empty-state {
  text-align: center;
  padding: 2rem 1rem;
  color: var(--c-text-muted);
}

.empty-state p {
  margin: 0.3rem 0;
}

.hint {
  font-size: 0.8rem;
}

.camera-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.camera-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--c-surface-2);
  border: 1px solid var(--c-border);
  border-radius: 8px;
  padding: 0.75rem 1rem;
}

.camera-meta {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.camera-name {
  font-weight: 600;
  font-size: 0.95rem;
}

.camera-port {
  font-size: 0.75rem;
  color: var(--c-text-muted);
  font-family: monospace;
}

.error-msg {
  margin-top: 0.75rem;
  color: var(--c-error);
  font-size: 0.85rem;
}
</style>
