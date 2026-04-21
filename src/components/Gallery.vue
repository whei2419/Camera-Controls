<script setup>
import { openPath } from '@tauri-apps/plugin-opener'

defineProps({ items: { type: Array, default: () => [] } })
defineEmits(['clear'])

function fileName(path) {
  return path ? path.split(/[\\/]/).pop() : 'Unknown'
}

function formatTime(ts) {
  return new Date(ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
}

async function openItem(item) {
  // Videos: open the file directly. Photos: open the save folder.
  const target = item.type === 'video' ? item.path : (item.folder || item.path)
  if (!target) return
  try {
    await openPath(target)
  } catch {
    // Fallback: copy path to clipboard
    await navigator.clipboard.writeText(target).catch(() => {})
  }
}
</script>

<template>
  <div class="panel gallery-panel">
    <div class="panel-header">
      <h2>Recent Files</h2>
      <button v-if="items.length > 0" class="btn btn-ghost btn-xs" @click="$emit('clear')">Clear</button>
    </div>

    <div v-if="items.length === 0" class="gallery-empty">
      No files yet. Captures and recordings will appear here.
    </div>

    <ul v-else class="gallery-list">
      <li v-for="item in items" :key="item.ts" class="gallery-item">
        <span class="gallery-icon">{{ item.type === 'photo' ? '📷' : '🎬' }}</span>
        <div class="gallery-meta">
          <span class="gallery-name" :title="item.path">
            {{ item.type === 'photo' ? 'Photo' : fileName(item.path) }}
          </span>
          <span class="gallery-time">{{ formatTime(item.ts) }}</span>
          <span v-if="item.folder || item.path" class="gallery-path" :title="item.folder || item.path">
            {{ item.folder || item.path }}
          </span>
        </div>
        <button
          class="open-btn"
          :title="item.type === 'video' ? 'Open video file' : 'Open save folder'"
          @click="openItem(item)"
        >
          {{ item.type === 'video' ? '▶' : '📂' }}
        </button>
      </li>
    </ul>
  </div>
</template>

<style scoped>
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

.gallery-empty {
  text-align: center;
  padding: 1rem 0;
  color: var(--c-text-muted);
  font-size: 0.8rem;
  line-height: 1.5;
}

.gallery-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  max-height: 260px;
  overflow-y: auto;
}

.gallery-item {
  display: flex;
  align-items: flex-start;
  gap: 0.55rem;
  padding: 0.45rem 0.6rem;
  border-radius: 6px;
  background: var(--c-surface-2);
  border: 1px solid var(--c-border);
  cursor: default;
}

.open-btn {
  margin-left: auto;
  flex-shrink: 0;
  background: none;
  border: 1px solid var(--c-border);
  color: var(--c-text-muted);
  border-radius: 5px;
  width: 26px;
  height: 26px;
  font-size: 0.75rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.13s, color 0.13s;
  align-self: center;
}
.open-btn:hover {
  background: var(--c-border);
  color: var(--c-text);
}

.gallery-icon {
  font-size: 0.95rem;
  flex-shrink: 0;
  line-height: 1.5;
}

.gallery-meta {
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
  min-width: 0;
}

.gallery-name {
  font-size: 0.82rem;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.gallery-time {
  font-size: 0.7rem;
  color: var(--c-text-muted);
}

.gallery-path {
  font-size: 0.65rem;
  color: var(--c-text-muted);
  font-family: monospace;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 230px;
}
</style>
