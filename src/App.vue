<script setup>
import { ref, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import CameraConnect from './components/CameraConnect.vue'
import CameraControls from './components/CameraControls.vue'
import ShutterButton from './components/ShutterButton.vue'
import LiveView from './components/LiveView.vue'

const connected = ref(false)
const cameraInfo = ref(null)

async function onConnected(info) {
  cameraInfo.value = info
  connected.value = true
}

async function disconnect() {
  try { await invoke('disconnect_camera') } catch { }
  connected.value = false
  cameraInfo.value = null
}

// Disconnect camera when app closes
onUnmounted(async () => {
  try { await invoke('disconnect_camera') } catch { }
})
</script>

<template>
  <main class="app-shell">

    <!-- ── Header ──────────────────────────────────────────────────────────── -->
    <header class="app-header">
      <div class="brand">
        <span class="brand-icon">◉</span>
        <span class="brand-name">Canon Control</span>
      </div>

      <div v-if="connected && cameraInfo" class="connection-badge">
        <span class="dot dot-live"></span>
        {{ cameraInfo.name }}
        <button class="btn btn-ghost btn-xs" @click="disconnect">Disconnect</button>
      </div>
    </header>

    <!-- ── Body ───────────────────────────────────────────────────────────── -->
    <div class="app-body">

      <!-- Left sidebar: connect + shutter -->
      <aside class="sidebar">
        <CameraConnect v-if="!connected" @connected="onConnected" />
        <ShutterButton :connected="connected" />
      </aside>

      <!-- Main area: controls + live view -->
      <section class="main-area">
        <CameraControls :connected="connected" />
        <LiveView :connected="connected" />
      </section>

    </div>
  </main>
</template>

<style>
/* ── Design tokens ──────────────────────────────────────────────────────────── */
:root {
  --c-bg: #0d0d0d;
  --c-surface: #161616;
  --c-surface-2: #1e1e1e;
  --c-border: #2e2e2e;
  --c-text: #e8e8e8;
  --c-text-muted: #888;
  --c-accent: #3b82f6;
  --c-error: #f87171;

  font-family: 'SF Pro Display', 'Inter', Helvetica, Arial, sans-serif;
  font-size: 14px;
  line-height: 1.5;
  color: var(--c-text);
  background: var(--c-bg);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

*,
*::before,
*::after {
  box-sizing: border-box;
}

body {
  margin: 0;
}

/* ── Layout ──────────────────────────────────────────────────────────────────── */
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 1.25rem;
  height: 48px;
  border-bottom: 1px solid var(--c-border);
  background: var(--c-surface);
  flex-shrink: 0;
}

.brand {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-weight: 700;
  font-size: 0.9rem;
  letter-spacing: 0.03em;
}

.brand-icon {
  color: var(--c-accent);
  font-size: 1.1rem;
}

.connection-badge {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.8rem;
  color: var(--c-text-muted);
}

.dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  display: inline-block;
}

.dot-live {
  background: #4ade80;
  box-shadow: 0 0 6px #4ade80;
  animation: blink 2s infinite;
}

@keyframes blink {

  0%,
  100% {
    opacity: 1;
  }

  50% {
    opacity: 0.4;
  }
}

.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.sidebar {
  width: 280px;
  flex-shrink: 0;
  border-right: 1px solid var(--c-border);
  display: flex;
  flex-direction: column;
  gap: 0;
  overflow-y: auto;
  background: var(--c-surface);
}

.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  gap: 0;
}

/* ── Shared panel shell ───────────────────────────────────────────────────────── */
.panel {
  padding: 1.25rem;
  border-bottom: 1px solid var(--c-border);
}

/* ── Buttons ─────────────────────────────────────────────────────────────────── */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  padding: 0.45rem 1rem;
  border-radius: 6px;
  font-size: 0.82rem;
  font-weight: 600;
  border: 1px solid transparent;
  cursor: pointer;
  transition: background 0.15s, opacity 0.15s;
  letter-spacing: 0.02em;
  font-family: inherit;
}

.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--c-accent);
  color: #fff;
  border-color: var(--c-accent);
}

.btn-primary:not(:disabled):hover {
  background: #2563eb;
}

.btn-danger {
  background: #dc2626;
  color: #fff;
  border-color: #dc2626;
}

.btn-danger:not(:disabled):hover {
  background: #b91c1c;
}

.btn-ghost {
  background: transparent;
  color: var(--c-text-muted);
  border-color: var(--c-border);
}

.btn-ghost:not(:disabled):hover {
  background: var(--c-surface-2);
  color: var(--c-text);
}

.btn-xs {
  padding: 0.2rem 0.6rem;
  font-size: 0.72rem;
}
</style>
