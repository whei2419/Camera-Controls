<script setup>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({ connected: Boolean })

const state = ref('idle') // idle | halfway | capturing | done | error
const errorMsg = ref('')

async function takePicture() {
  if (!props.connected || state.value === 'capturing') return
  state.value = 'capturing'
  errorMsg.value = ''
  try {
    await invoke('take_picture')
    state.value = 'done'
    setTimeout(() => { state.value = 'idle' }, 1500)
  } catch (e) {
    errorMsg.value = String(e)
    state.value = 'error'
    setTimeout(() => { state.value = 'idle' }, 2500)
  }
}

// Press-and-hold mode
async function halfwayDown() {
  if (!props.connected) return
  try { await invoke('press_shutter_halfway') } catch {}
  state.value = 'halfway'
}

async function halfwayUp() {
  if (!props.connected) return
  try { await invoke('release_shutter') } catch {}
  if (state.value === 'halfway') state.value = 'idle'
}
</script>

<template>
  <div class="panel shutter-panel">
    <div class="panel-header">
      <h2>Shutter</h2>
    </div>

    <div class="shutter-center">
      <button
        class="shutter-btn"
        :class="state"
        :disabled="!connected"
        @mousedown="halfwayDown"
        @mouseup="halfwayUp"
        @click="takePicture"
        title="Click to take photo"
      >
        <span class="shutter-ring">
          <span class="shutter-dot"></span>
        </span>
        <span class="shutter-label">
          <template v-if="state === 'idle'">SHOOT</template>
          <template v-else-if="state === 'halfway'">AF…</template>
          <template v-else-if="state === 'capturing'">…</template>
          <template v-else-if="state === 'done'">✓</template>
          <template v-else-if="state === 'error'">!</template>
        </span>
      </button>
    </div>

    <p v-if="errorMsg" class="error-msg">{{ errorMsg }}</p>
    <p v-if="!connected" class="hint">Connect a camera to enable the shutter.</p>
  </div>
</template>

<style scoped>
.shutter-panel { text-align: center; }
.panel-header h2 {
  margin: 0 0 1rem;
  font-size: 1rem;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--c-text-muted);
}
.shutter-center { display: flex; justify-content: center; padding: 1rem 0; }

.shutter-btn {
  --sz: 120px;
  width: var(--sz);
  height: var(--sz);
  border-radius: 50%;
  border: none;
  background: transparent;
  cursor: pointer;
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  transition: transform 0.1s;
}
.shutter-btn:disabled { cursor: not-allowed; opacity: 0.35; }
.shutter-btn:not(:disabled):active { transform: scale(0.94); }

.shutter-ring {
  --ring: 96px;
  width: var(--ring);
  height: var(--ring);
  border-radius: 50%;
  border: 4px solid var(--c-border);
  display: flex;
  align-items: center;
  justify-content: center;
  position: absolute;
  transition: border-color 0.2s;
}
.shutter-btn:not(:disabled):hover .shutter-ring { border-color: var(--c-accent); }
.shutter-btn.halfway .shutter-ring { border-color: #facc15; }
.shutter-btn.capturing .shutter-ring { border-color: var(--c-accent); animation: pulse 0.4s infinite alternate; }
.shutter-btn.done .shutter-ring { border-color: #4ade80; }
.shutter-btn.error .shutter-ring { border-color: var(--c-error); }

@keyframes pulse { from { opacity: 0.5; } to { opacity: 1; } }

.shutter-dot {
  --dot: 60px;
  width: var(--dot);
  height: var(--dot);
  border-radius: 50%;
  background: var(--c-surface-2);
  border: 2px solid var(--c-border);
  transition: background 0.2s;
}
.shutter-btn:not(:disabled):hover .shutter-dot { background: var(--c-accent); }
.shutter-btn.capturing .shutter-dot { background: var(--c-accent); }
.shutter-btn.done .shutter-dot { background: #4ade80; }
.shutter-btn.error .shutter-dot { background: var(--c-error); }

.shutter-label {
  position: absolute;
  font-size: 0.62rem;
  font-weight: 700;
  letter-spacing: 0.12em;
  color: var(--c-text-muted);
  pointer-events: none;
}
.error-msg { color: var(--c-error); font-size: 0.82rem; margin-top: 0.5rem; }
.hint { color: var(--c-text-muted); font-size: 0.78rem; margin-top: 0.5rem; }
</style>
