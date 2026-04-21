<script setup>
import { ref, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'

const props = defineProps({
  imageFolder: { type: String, default: '' },
  refreshTrigger: { type: Number, default: 0 }, // Increment to force refresh
})

const emit = defineEmits(['open-gallery'])

const IMG_EXTS = ['jpg', 'jpeg', 'png', 'cr2', 'cr3', 'nef', 'arw', 'tif', 'tiff']

const images = ref([])
const loading = ref(false)

async function refresh() {
  if (!props.imageFolder) {
    images.value = []
    return
  }
  loading.value = true
  try {
    const files = await invoke('list_folder_files', {
      folder: props.imageFolder,
      extensions: IMG_EXTS,
    })
    images.value = files.slice(0, 50).map(p => ({ path: p, src: convertFileSrc(p) }))
  } catch {
    images.value = []
  }
  loading.value = false
}

watch(() => props.imageFolder, refresh, { immediate: true })
watch(() => props.refreshTrigger, refresh)

onMounted(refresh)
</script>

<template>
  <div class="tg-panel">

    <!-- Header -->
    <div class="tg-header">
      <div class="tg-title">
        <span class="tg-icon">📸</span>
        <span>Recent Captures</span>
      </div>
      <button class="tg-icon-btn" title="Refresh" @click="refresh">
        <span :class="{ spin: loading }">⟳</span>
      </button>
    </div>

    <!-- Body -->
    <div class="tg-body">

      <!-- Empty state -->
      <div v-if="images.length === 0 && !loading" class="tg-empty">
        <p>📷</p>
        <p>No captures yet</p>
        <p class="hint">Photos will appear here after capture</p>
      </div>

      <!-- Loading -->
      <div v-if="loading && images.length === 0" class="tg-loading">Loading…</div>

      <!-- Thumbnail grid -->
      <div v-else class="tg-grid">
        <div
          v-for="img in images"
          :key="img.path"
          class="tg-thumb"
          :title="img.path.split(/[\\\\/]/).pop()"
          @click="$emit('open-gallery')"
        >
          <img :src="img.src" loading="lazy" />
        </div>
      </div>

    </div>

    <!-- Footer -->
    <div class="tg-footer">
      <button class="tg-view-btn" @click="$emit('open-gallery')">
        🖼 View All
      </button>
    </div>

  </div>
</template>

<style scoped>
.tg-panel {
  width: 280px;
  flex-shrink: 0;
  border-right: 1px solid var(--c-border);
  display: flex;
  flex-direction: column;
  background: var(--c-surface);
  overflow: hidden;
}

/* Header */
.tg-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  border-bottom: 1px solid var(--c-border);
  flex-shrink: 0;
}

.tg-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.9rem;
  font-weight: 700;
  color: var(--c-text);
}

.tg-icon {
  font-size: 1.1rem;
}

.tg-icon-btn {
  background: none;
  border: 1px solid var(--c-border);
  color: var(--c-text-muted);
  border-radius: 6px;
  width: 28px;
  height: 28px;
  font-size: 0.95rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.13s;
}
.tg-icon-btn:hover {
  background: var(--c-surface-2);
  color: var(--c-text);
}
.spin {
  display: inline-block;
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Body */
.tg-body {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.tg-empty, .tg-loading {
  text-align: center;
  padding: 3rem 1rem;
  color: var(--c-text-muted);
}

.tg-empty p:first-child {
  font-size: 2.5rem;
  margin: 0 0 0.5rem;
  opacity: 0.3;
}

.tg-empty p {
  margin: 0.3rem 0;
  font-size: 0.85rem;
}

.tg-empty .hint {
  font-size: 0.75rem;
  color: var(--c-text-muted);
  opacity: 0.7;
}

/* Grid */
.tg-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 8px;
}

.tg-thumb {
  aspect-ratio: 1;
  background: var(--c-surface-2);
  border: 2px solid var(--c-border);
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  transition: border-color 0.15s, transform 0.13s;
}

.tg-thumb:hover {
  border-color: var(--c-accent);
  transform: scale(1.03);
}

.tg-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

/* Footer */
.tg-footer {
  padding: 10px;
  border-top: 1px solid var(--c-border);
  flex-shrink: 0;
}

.tg-view-btn {
  width: 100%;
  background: var(--c-surface-2);
  border: 1px solid var(--c-border);
  color: var(--c-text);
  border-radius: 6px;
  padding: 8px 12px;
  font-size: 0.82rem;
  font-weight: 600;
  cursor: pointer;
  font-family: inherit;
  transition: background 0.13s, border-color 0.13s;
}

.tg-view-btn:hover {
  background: var(--c-border);
  border-color: var(--c-accent);
  color: var(--c-accent);
}
</style>
