<script setup>
import { ref, watch } from 'vue'

const props = defineProps({ obsInstance: Object }) // OBSWebSocket instance or null

const IMAGE_PATH_KEY = 'setting_image_path'
const VIDEO_PATH_KEY = 'setting_video_path'
const PHOTO_CAPTURE_SOURCE_KEY = 'setting_photo_capture_source'

const imagePath = ref(localStorage.getItem(IMAGE_PATH_KEY) || '')
const videoPath = ref(localStorage.getItem(VIDEO_PATH_KEY) || '')
const photoCaptureSource = ref(localStorage.getItem(PHOTO_CAPTURE_SOURCE_KEY) || 'digicamcontrol')
const videoMsg = ref('')
const videoSaving = ref(false)

// Persist image path on every change
watch(imagePath, val => localStorage.setItem(IMAGE_PATH_KEY, val))
watch(photoCaptureSource, val => localStorage.setItem(PHOTO_CAPTURE_SOURCE_KEY, val))

// When OBS connects, pull the current recording directory from OBS
watch(() => props.obsInstance, async (obs) => {
  if (!obs) return
  try {
    const { recordDirectory } = await obs.call('GetRecordDirectory')
    if (recordDirectory) {
      videoPath.value = recordDirectory
      localStorage.setItem(VIDEO_PATH_KEY, recordDirectory)
    }
  } catch { /* OBS may not support this version */ }
}, { immediate: true })

async function applyVideoPath() {
  if (!videoPath.value) return
  localStorage.setItem(VIDEO_PATH_KEY, videoPath.value)

  if (!props.obsInstance) {
    videoMsg.value = 'Saved locally'
    setTimeout(() => { videoMsg.value = '' }, 1500)
    return
  }

  videoSaving.value = true
  try {
    await props.obsInstance.call('SetRecordDirectory', { recordDirectory: videoPath.value })
    videoMsg.value = 'Applied to OBS ✓'
  } catch (e) {
    videoMsg.value = 'OBS error'
  } finally {
    videoSaving.value = false
    setTimeout(() => { videoMsg.value = '' }, 2000)
  }
}
</script>

<template>
  <div class="panel rset-panel">
    <div class="panel-header">
      <h2>File Paths</h2>
    </div>

    <div class="rset-form">

      <!-- Image path -->
      <div class="rset-group">
        <label class="rset-label">Image output</label>
        <p class="rset-hint">Where DigiCamControl saves captures (configure matching path there too)</p>
        <input
          v-model="imagePath"
          class="path-input"
          placeholder="C:\captures\images"
          spellcheck="false"
        />
      </div>

      <div class="rset-group">
        <label class="rset-label">Photo Capture Source</label>
        <p class="rset-hint">Choose where photo capture comes from when you press Shoot or receive a remote photo trigger.</p>
        <select v-model="photoCaptureSource" class="source-select">
          <option value="digicamcontrol">DigiCamControl (USB camera)</option>
          <option value="obs">OBS (current program scene screenshot)</option>
        </select>
      </div>

      <!-- Video path -->
      <div class="rset-group">
        <label class="rset-label">Video output</label>
        <p class="rset-hint">OBS recording directory</p>
        <div class="path-row">
          <input
            v-model="videoPath"
            class="path-input"
            placeholder="C:\captures\videos"
            spellcheck="false"
            @keydown.enter="applyVideoPath"
          />
          <button
            class="btn btn-sm btn-primary"
            :disabled="videoSaving || !videoPath"
            @click="applyVideoPath"
          >
            {{ videoSaving ? '…' : 'Apply' }}
          </button>
        </div>
        <span v-if="videoMsg" class="rset-msg">{{ videoMsg }}</span>
        <span v-if="!obsInstance" class="rset-hint-warn">Connect OBS to sync path automatically</span>
      </div>

    </div>
  </div>
</template>

<style scoped>
.rset-panel { }

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

.rset-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.rset-group {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.rset-label {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--c-text);
}

.rset-hint {
  font-size: 0.72rem;
  color: var(--c-text-muted);
  margin: 0;
  line-height: 1.4;
}

.rset-hint-warn {
  font-size: 0.72rem;
  color: #eab308;
}

.rset-msg {
  font-size: 0.75rem;
  color: #4ade80;
}

.path-row {
  display: flex;
  gap: 0.4rem;
  align-items: center;
}

.path-input {
  flex: 1;
  background: var(--c-surface-2);
  border: 1px solid var(--c-border);
  border-radius: 6px;
  color: var(--c-text);
  padding: 0.35rem 0.6rem;
  font-size: 0.8rem;
  font-family: monospace;
  outline: none;
  min-width: 0;
}

.path-input:focus {
  border-color: var(--c-accent);
}

.source-select {
  background: var(--c-surface-2);
  border: 1px solid var(--c-border);
  border-radius: 6px;
  color: var(--c-text);
  padding: 0.4rem 0.55rem;
  font-size: 0.8rem;
  outline: none;
}

.source-select:focus {
  border-color: var(--c-accent);
}
</style>
