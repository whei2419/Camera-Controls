<script setup>
import { ref, onMounted } from 'vue'
import OBSConnect from './OBSConnect.vue'
import RecordingSettings from './RecordingSettings.vue'
import PrinterSettings from './PrinterSettings.vue'
import CameraConnect from './CameraConnect.vue'
import { appBaseUrl } from '../config/remoteSite.js'

const props = defineProps({
  show: Boolean,
  obsConnected: Boolean,
  obsInfo: Object,
  connected: Boolean,
  obsInstance: Object,
})

const emit = defineEmits(['close', 'obs-connected', 'obs-disconnected', 'camera-connected'])

const tab = ref('obs') // 'obs' | 'files' | 'printer' | 'camera' | 'server'

const RECORD_MEDIA_SOURCE_KEY = 'setting_record_media_source'
const recordMediaSource = ref(localStorage.getItem(RECORD_MEDIA_SOURCE_KEY) || '')

const serverUrl = ref('')
const serverUrlMsg = ref('')
const RECORDING_DURATION_KEY = 'setting_recording_duration_sec'
const recordingDurationSec = ref(20)
const recordingSaving = ref(false)
const recordingMsg = ref('')

function normalizeRecordingDuration(value) {
  const n = Number(value)
  if (!Number.isFinite(n)) return 20
  return Math.max(3, Math.min(600, Math.round(n)))
}

onMounted(() => {
  serverUrl.value = localStorage.getItem('setting_base_url') || appBaseUrl
  recordingDurationSec.value = normalizeRecordingDuration(localStorage.getItem(RECORDING_DURATION_KEY))
  loadRecordingConfig()
})

async function loadRecordingConfig() {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const config = await invoke('get_app_config')
    const secs = normalizeRecordingDuration(config?.recordingDurationSec)
    recordingDurationSec.value = secs
    localStorage.setItem(RECORDING_DURATION_KEY, String(secs))
  } catch {
    // Keep local value if backend config can't be fetched.
  }
}

function saveServerUrl() {
  serverUrlMsg.value = ''
  let url = serverUrl.value.trim().replace(/\/+$/, '')

  // Empty field → reset to default
  if (!url) {
    localStorage.removeItem('setting_base_url')
    serverUrl.value = appBaseUrl
    serverUrlMsg.value = `Reset to default: ${appBaseUrl}`
    setTimeout(() => { window.location.reload() }, 800)
    return
  }

  // Basic URL validation — must start with http:// or https://
  if (!/^https?:\/\/.+/i.test(url)) {
    serverUrlMsg.value = 'Invalid URL — must start with http:// or https://'
    setTimeout(() => { serverUrlMsg.value = '' }, 3000)
    return
  }

  localStorage.setItem('setting_base_url', url)
  window.location.reload()
}

async function saveRecordingDuration() {
  recordingSaving.value = true
  recordingMsg.value = ''
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const secs = normalizeRecordingDuration(recordingDurationSec.value)
    const config = await invoke('set_recording_duration_sec', { seconds: secs })
    const persisted = normalizeRecordingDuration(config?.recordingDurationSec)
    recordingDurationSec.value = persisted
    localStorage.setItem(RECORDING_DURATION_KEY, String(persisted))
    recordingMsg.value = 'Saved'
  } catch {
    recordingMsg.value = 'Save failed'
  } finally {
    recordingSaving.value = false
    setTimeout(() => { recordingMsg.value = '' }, 2000)
  }
}

function onOBSConnected(info) {
  emit('obs-connected', info)
}

function onOBSDisconnected() {
  emit('obs-disconnected')
}

function onCameraConnected(info) {
  emit('camera-connected', info)
}

const recordMediaSourceMsg = ref('')

function applyMediaSource() {
  localStorage.setItem(RECORD_MEDIA_SOURCE_KEY, recordMediaSource.value.trim())
  recordMediaSourceMsg.value = 'Saved ✓'
  setTimeout(() => { recordMediaSourceMsg.value = '' }, 2000)
}
</script>

<template>
  <teleport to="body">
    <transition name="sm-fade">
      <div v-if="show" class="sm-overlay" @click.self="$emit('close')">
        <div class="sm-modal">

          <!-- Header -->
          <div class="sm-header">
            <h1>⚙ Settings</h1>
            <button class="sm-close" @click="$emit('close')">✕</button>
          </div>

          <!-- Tabs -->
          <div class="sm-tabs">
            <button :class="{ active: tab === 'obs' }" @click="tab = 'obs'">
              <span class="tab-icon">📡</span> OBS
            </button>
            <button :class="{ active: tab === 'files' }" @click="tab = 'files'">
              <span class="tab-icon">📁</span> Files
            </button>
            <button :class="{ active: tab === 'printer' }" @click="tab = 'printer'">
              <span class="tab-icon">🖨</span> Printer
            </button>
            <button :class="{ active: tab === 'camera' }" @click="tab = 'camera'">
              <span class="tab-icon">📷</span> Camera
            </button>
            <button :class="{ active: tab === 'server' }" @click="tab = 'server'">
              <span class="tab-icon">🌐</span> Server
            </button>
          </div>

          <!-- Body -->
          <div class="sm-body">

            <!-- OBS Connection -->
            <div v-if="tab === 'obs'" class="sm-section">
              <OBSConnect @connected="onOBSConnected" @disconnected="onOBSDisconnected" />
              <div class="obs-extra">
                <label class="field-label">Play media source on record start</label>
                <p class="field-hint">Exact name of an OBS media/audio source to play when recording begins. Leave blank
                  to disable.</p>
                <div class="duration-row">
                  <input v-model="recordMediaSource" class="field-input" placeholder="e.g. Intro Music"
                    spellcheck="false" @keydown.enter="applyMediaSource" />
                  <button class="save-btn" @click="applyMediaSource">Apply</button>
                </div>
                <span v-if="recordMediaSourceMsg" class="save-msg">{{ recordMediaSourceMsg }}</span>
              </div>
            </div>

            <!-- File Paths -->
            <div v-if="tab === 'files'" class="sm-section">
              <RecordingSettings :obs-instance="obsInstance" />
            </div>

            <!-- Printer -->
            <div v-if="tab === 'printer'" class="sm-section">
              <PrinterSettings />
            </div>

            <!-- Camera -->
            <div v-if="tab === 'camera'" class="sm-section">
              <CameraConnect v-if="!connected" @connected="onCameraConnected" />
              <div v-else class="camera-connected">
                <p>✓ Camera connected</p>
                <p class="hint">Close settings to access camera controls</p>
              </div>
            </div>

            <!-- Server -->
            <div v-if="tab === 'server'" class="sm-section">
              <div class="server-settings">
                <label class="field-label">Server Base URL</label>
                <p class="field-hint">Base URL of your Laravel server (e.g. <code>http://Wowsome-micorsite.test</code>).
                  Saved to local storage — app reloads on save.</p>
                <input v-model="serverUrl" type="url" class="field-input"
                  :class="{ 'field-input-error': serverUrlMsg && !serverUrlMsg.startsWith('Reset') }"
                  placeholder="http://Wowsome-micorsite.test" />
                <button class="save-btn" @click="saveServerUrl">Save &amp; Reload</button>
                <span v-if="serverUrlMsg" class="save-msg"
                  :class="{ 'save-msg-error': !serverUrlMsg.startsWith('Reset') }">{{ serverUrlMsg }}</span>

                <div class="field-divider"></div>

                <label class="field-label">Default Video Recording Duration (seconds)</label>
                <p class="field-hint">Used when remote capture requests do not send a duration. Stored in app backend
                  config and local app settings.</p>
                <div class="duration-row">
                  <input v-model.number="recordingDurationSec" type="number" min="3" max="600" step="1"
                    class="field-input" />
                  <button class="save-btn" :disabled="recordingSaving" @click="saveRecordingDuration">
                    {{ recordingSaving ? 'Saving…' : 'Save Duration' }}
                  </button>
                </div>
                <span v-if="recordingMsg" class="save-msg">{{ recordingMsg }}</span>
              </div>
            </div>

          </div>

        </div>
      </div>
    </transition>
  </teleport>
</template>

<style scoped>
/* Overlay */
.sm-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.85);
  z-index: 999;
  display: flex;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(6px);
  padding: 20px;
}

/* Modal */
.sm-modal {
  background: var(--c-surface);
  border: 1px solid var(--c-border);
  border-radius: 12px;
  width: min(700px, 100%);
  height: min(85vh, 760px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.8);
}

/* Header */
.sm-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 22px;
  border-bottom: 1px solid var(--c-border);
  flex-shrink: 0;
}

.sm-header h1 {
  margin: 0;
  font-size: 1.15rem;
  font-weight: 700;
  color: var(--c-text);
}

.sm-close {
  background: none;
  border: 1px solid var(--c-border);
  color: var(--c-text-muted);
  border-radius: 6px;
  width: 32px;
  height: 32px;
  font-size: 1rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.13s, color 0.13s;
}

.sm-close:hover {
  background: var(--c-surface-2);
  color: var(--c-error);
  border-color: var(--c-error);
}

/* Tabs */
.sm-tabs {
  display: flex;
  gap: 0;
  padding: 0 12px;
  border-bottom: 1px solid var(--c-border);
  flex-shrink: 0;
  overflow-x: auto;
}

.sm-tabs button {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 16px;
  border: none;
  background: none;
  color: var(--c-text-muted);
  font-family: inherit;
  font-size: 0.82rem;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  border-bottom: 2px solid transparent;
  transition: color 0.15s, border-color 0.15s;
}

.sm-tabs button:hover {
  color: var(--c-text);
}

.sm-tabs button.active {
  color: var(--c-accent);
  border-bottom-color: var(--c-accent);
}

.tab-icon {
  font-size: 1rem;
}

/* Body */
.sm-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.sm-section {
  max-width: 520px;
  margin: 0 auto;
}

/* Camera connected state */
.camera-connected {
  text-align: center;
  padding: 3rem 1rem;
}

.camera-connected p {
  margin: 0.5rem 0;
  color: var(--c-text);
  font-size: 0.95rem;
}

.camera-connected .hint {
  color: var(--c-text-muted);
  font-size: 0.8rem;
}

/* Transitions */
.sm-fade-enter-active,
.sm-fade-leave-active {
  transition: opacity 0.2s;
}

.sm-fade-enter-from,
.sm-fade-leave-to {
  opacity: 0;
}

/* Remove panel borders in modal context */
.sm-section :deep(.panel) {
  border: none;
  border-bottom: none;
  padding: 0;
}

/* Server tab */
.server-settings {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.obs-extra {
  max-width: 480px;
  margin: 1.25rem auto 0;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.field-label {
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--c-text);
}

.field-hint {
  margin: 0;
  font-size: 0.78rem;
  color: var(--c-text-muted);
  line-height: 1.5;
}

.field-hint code {
  font-size: 0.78rem;
  background: var(--c-surface-2);
  padding: 1px 4px;
  border-radius: 3px;
}

.field-input {
  width: 100%;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid var(--c-border);
  background: var(--c-surface-2);
  color: var(--c-text);
  font-size: 0.88rem;
  font-family: inherit;
  box-sizing: border-box;
}

.field-input:focus {
  outline: none;
  border-color: var(--c-accent);
}

.save-btn {
  align-self: flex-start;
  padding: 7px 18px;
  border-radius: 6px;
  border: none;
  background: var(--c-accent);
  color: #fff;
  font-size: 0.82rem;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.15s;
}

.save-btn:hover {
  opacity: 0.82;
}

.field-divider {
  height: 1px;
  background: var(--c-border);
  margin: 4px 0 8px;
}

.duration-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.save-msg {
  font-size: 0.78rem;
  color: #4ade80;
}

.save-msg-error {
  color: var(--c-error);
}

.field-input-error {
  border-color: var(--c-error) !important;
  outline-color: var(--c-error);
}
</style>
