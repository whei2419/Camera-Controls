<script setup>
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps({ connected: Boolean })

const settings = ref(null)
const options = ref({ iso: [], aperture: [], shutter_speed: [], white_balance: [] })
const error = ref('')
const successMsg = ref('')

let refreshTimer = null

async function loadOptions() {
    try {
        options.value = await invoke('get_setting_options')
    } catch { }
}

async function refreshSettings() {
    if (!props.connected) return
    try {
        settings.value = await invoke('get_settings')
    } catch { }
}

async function setSetting(command, value) {
    error.value = ''
    try {
        await invoke(command, { value })
        successMsg.value = 'Saved'
        setTimeout(() => { successMsg.value = '' }, 1200)
        await refreshSettings()
    } catch (e) {
        error.value = String(e)
    }
}

// Parse a battery string like "75%" or "100%" into a number for the bar
function batteryPct(str) {
    const n = parseInt(str)
    return isNaN(n) ? 0 : Math.min(100, Math.max(0, n))
}
function batteryClass(str) {
    const n = batteryPct(str)
    if (n <= 20) return 'low'
    if (n <= 50) return 'mid'
    return 'ok'
}

watch(() => props.connected, async (val) => {
    if (val) {
        await loadOptions()
        await refreshSettings()
        refreshTimer = setInterval(refreshSettings, 3000)
    } else {
        settings.value = null
        clearInterval(refreshTimer)
    }
}, { immediate: true })
</script>

<template>
    <div class="panel controls-panel">
        <div class="panel-header">
            <h2>Camera Controls</h2>
            <span v-if="successMsg" class="badge badge-ok">{{ successMsg }}</span>
            <span v-else-if="!connected" class="badge badge-off">Disconnected</span>
        </div>

        <div v-if="!connected" class="empty-state">
            <p>Connect a camera to access controls.</p>
        </div>

        <div v-else-if="!settings" class="empty-state">
            <p>Loading settings…</p>
        </div>

        <div v-else class="settings-grid">

            <!-- Aperture -->
            <div class="setting-row">
                <label>Aperture (Av)</label>
                <div class="setting-control">
                    <select :value="settings.aperture" @change="setSetting('set_aperture', $event.target.value)">
                        <option v-for="opt in options.aperture" :key="opt" :value="opt">{{ opt }}</option>
                    </select>
                    <span class="current-label">{{ settings.aperture }}</span>
                </div>
            </div>

            <!-- Shutter Speed -->
            <div class="setting-row">
                <label>Shutter (Tv)</label>
                <div class="setting-control">
                    <select :value="settings.shutter_speed"
                        @change="setSetting('set_shutter_speed', $event.target.value)">
                        <option v-for="opt in options.shutter_speed" :key="opt" :value="opt">{{ opt }}</option>
                    </select>
                    <span class="current-label">{{ settings.shutter_speed }}</span>
                </div>
            </div>

            <!-- ISO -->
            <div class="setting-row">
                <label>ISO</label>
                <div class="setting-control">
                    <select :value="settings.iso" @change="setSetting('set_iso', $event.target.value)">
                        <option v-for="opt in options.iso" :key="opt" :value="opt">{{ opt }}</option>
                    </select>
                    <span class="current-label">{{ settings.iso }}</span>
                </div>
            </div>

            <!-- White Balance -->
            <div class="setting-row">
                <label>White Balance</label>
                <div class="setting-control">
                    <select :value="settings.white_balance"
                        @change="setSetting('set_white_balance', $event.target.value)">
                        <option v-for="opt in options.white_balance" :key="opt" :value="opt">{{ opt }}</option>
                    </select>
                    <span class="current-label">{{ settings.white_balance }}</span>
                </div>
            </div>

            <!-- Battery -->
            <div class="setting-row battery-row">
                <label>Battery</label>
                <div class="battery-bar-wrap">
                    <div class="battery-bar" :style="{ width: batteryPct(settings.battery) + '%' }"
                        :class="batteryClass(settings.battery)">
                    </div>
                </div>
                <span class="battery-pct">{{ settings.battery }}</span>
            </div>

        </div>

        <p v-if="error" class="error-msg">{{ error }}</p>
    </div>
</template>

<style scoped>
.controls-panel {
    flex: 1;
}

.panel-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1.25rem;
}

.panel-header h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--c-text-muted);
    flex: 1;
}

.badge {
    font-size: 0.7rem;
    font-weight: 700;
    padding: 2px 8px;
    border-radius: 999px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
}

.badge-ok {
    background: #1a3d2a;
    color: #4ade80;
}

.badge-off {
    background: #2d1f1f;
    color: #f87171;
}

.empty-state {
    text-align: center;
    padding: 2rem 1rem;
    color: var(--c-text-muted);
}

.settings-grid {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
}

.setting-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
}

.setting-row label {
    width: 120px;
    flex-shrink: 0;
    font-size: 0.8rem;
    color: var(--c-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.setting-control {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

select {
    flex: 1;
    background: var(--c-surface-2);
    border: 1px solid var(--c-border);
    color: var(--c-text);
    border-radius: 6px;
    padding: 0.4rem 0.6rem;
    font-size: 0.85rem;
    cursor: pointer;
    appearance: none;
    -webkit-appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6'%3E%3Cpath d='M0 0l5 6 5-6z' fill='%23888'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 0.6rem center;
    padding-right: 1.8rem;
}

select:focus {
    outline: 2px solid var(--c-accent);
    outline-offset: 1px;
}

.current-label {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--c-accent);
    min-width: 70px;
}

/* Battery */
.battery-row {
    align-items: center;
}

.battery-bar-wrap {
    flex: 1;
    height: 8px;
    background: var(--c-surface-2);
    border: 1px solid var(--c-border);
    border-radius: 4px;
    overflow: hidden;
}

.battery-bar {
    height: 100%;
    border-radius: 4px;
    transition: width 0.4s;
}

.battery-bar.ok {
    background: #4ade80;
}

.battery-bar.mid {
    background: #facc15;
}

.battery-bar.low {
    background: #f87171;
}

.battery-pct {
    font-size: 0.8rem;
    min-width: 36px;
    text-align: right;
}

.error-msg {
    margin-top: 0.75rem;
    color: var(--c-error);
    font-size: 0.85rem;
}
</style>
