<script setup>
import { ref, watch, onUnmounted, onMounted } from 'vue'
import OBSWebSocket from 'obs-websocket-js'

const STORAGE_KEY = 'obs_creds'

function loadSaved() {
    try {
        const raw = localStorage.getItem(STORAGE_KEY)
        return raw ? JSON.parse(raw) : null
    } catch { return null }
}

function saveCreds() {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
        host: host.value,
        port: port.value,
        password: password.value
    }))
}

const saved = loadSaved()
const host = ref(saved?.host ?? 'localhost')
const port = ref(saved?.port ?? 4455)
const password = ref(saved?.password ?? '')
const status = ref('disconnected') // disconnected | connecting | connected | error
const error = ref('')
const obsInfo = ref(null)

const emit = defineEmits(['connected', 'disconnected'])

let obs = null

// Persist whenever any field changes
watch([host, port, password], saveCreds)

async function connect() {
    status.value = 'connecting'
    error.value = ''
    saveCreds()
    const attempt = new OBSWebSocket()
    obs = attempt
    try {
        attempt.on('ConnectionClosed', () => {
            if (obs !== attempt) return
            status.value = 'disconnected'
            obsInfo.value = null
            emit('disconnected')
        })

        attempt.on('ConnectionError', () => {
            if (obs !== attempt) return
            status.value = 'error'
            obsInfo.value = null
            emit('disconnected')
        })

        const { obsWebSocketVersion } = await attempt.connect(
            `ws://${host.value}:${port.value}`,
            password.value || undefined
        )

        // Cancelled while connecting — drop the newly-established connection
        if (obs !== attempt) {
            attempt.disconnect()
            return
        }

        const { currentProgramSceneName } = await attempt.call('GetCurrentProgramScene')
        obsInfo.value = { version: obsWebSocketVersion, scene: currentProgramSceneName, obs: attempt }
        status.value = 'connected'
        emit('connected', obsInfo.value)
    } catch (e) {
        if (obs !== attempt) return
        error.value = String(e)
        status.value = 'error'
        obs = null
    }
}

async function disconnect() {
    if (obs) {
        obs.disconnect()
        obs = null
    }
    status.value = 'disconnected'
    obsInfo.value = null
    emit('disconnected')
}

function cancelConnect() {
    if (obs) {
        obs.off('ConnectionClosed')
        obs.off('ConnectionError')
        obs.disconnect()
        obs = null
    }
    status.value = 'disconnected'
    error.value = ''
}

onUnmounted(() => { if (obs) obs.disconnect() })

// Auto-connect if we have saved credentials
onMounted(() => {
    if (saved && saved.host) {
        // try to connect automatically
        connect().catch(() => { /* swallow - UI will show error */ })
    }
})
</script>

<template>
    <div class="panel obs-connect-panel">
        <div class="panel-header">
            <h2>OBS Connection</h2>
            <span class="status-dot" :class="status" :title="status"></span>
        </div>

        <div v-if="status === 'connected' && obsInfo" class="obs-info">
            <div class="info-row">
                <span class="info-label">Scene</span>
                <span class="info-value">{{ obsInfo.scene }}</span>
            </div>
            <div class="info-row">
                <span class="info-label">WebSocket</span>
                <span class="info-value">v{{ obsInfo.version }}</span>
            </div>
            <button class="btn btn-ghost btn-sm mt" @click="disconnect">Disconnect OBS</button>
        </div>

        <div v-else class="obs-form">
            <div class="field-row">
                <label>Host</label>
                <input v-model="host" class="field-input" placeholder="localhost" />
            </div>
            <div class="field-row">
                <label>Port</label>
                <input v-model.number="port" class="field-input" type="number" placeholder="4455" />
            </div>
            <div class="field-row">
                <label>Password</label>
                <input v-model="password" class="field-input" type="password" placeholder="(optional)" />
            </div>
            <button v-if="status === 'connecting'" class="btn btn-ghost btn-full mt" @click="cancelConnect">
                Cancel
            </button>
            <button v-else class="btn btn-primary btn-full mt" @click="connect">
                Connect to OBS
            </button>
        </div>

        <p v-if="error" class="error-msg">{{ error }}</p>
    </div>
</template>

<style scoped>
.obs-connect-panel {
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

.status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--c-border);
    display: inline-block;
}

.status-dot.connected {
    background: #22c55e;
    box-shadow: 0 0 6px #22c55e88;
}

.status-dot.connecting {
    background: #eab308;
    animation: blink 0.8s infinite;
}

.status-dot.error {
    background: var(--c-error);
}

@keyframes blink {

    0%,
    100% {
        opacity: 1;
    }

    50% {
        opacity: 0.3;
    }
}

.obs-form {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
}

.field-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.field-row label {
    width: 70px;
    font-size: 0.8rem;
    color: var(--c-text-muted);
    flex-shrink: 0;
}

.field-input {
    flex: 1;
    background: var(--c-surface-2);
    border: 1px solid var(--c-border);
    border-radius: 6px;
    color: var(--c-text);
    padding: 0.35rem 0.6rem;
    font-size: 0.85rem;
    outline: none;
}

.field-input:focus {
    border-color: var(--c-accent);
}

.field-input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.obs-info {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
}

.info-row {
    display: flex;
    justify-content: space-between;
    font-size: 0.85rem;
}

.info-label {
    color: var(--c-text-muted);
}

.info-value {
    color: var(--c-text);
    font-weight: 500;
}

.btn-full {
    width: 100%;
}

.mt {
    margin-top: 0.5rem;
}

.error-msg {
    color: var(--c-error);
    font-size: 0.8rem;
    margin: 0.5rem 0 0;
}
</style>
