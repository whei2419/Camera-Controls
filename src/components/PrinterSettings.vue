<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const PRINTER_KEY   = 'setting_printer_name'
const PAPER_KEY     = 'setting_paper_size'
const ORIENT_KEY    = 'setting_orientation'
const COPIES_KEY    = 'setting_copies'
const FIT_KEY       = 'setting_print_fit'

const printers     = ref([])
const loadingPrint = ref(false)

const selectedPrinter = ref(localStorage.getItem(PRINTER_KEY) || '')
const paperSize       = ref(localStorage.getItem(PAPER_KEY)   || '4x6')
const orientation     = ref(localStorage.getItem(ORIENT_KEY)  || 'portrait')
const copies          = ref(Number(localStorage.getItem(COPIES_KEY)) || 1)
const fitMode         = ref(localStorage.getItem(FIT_KEY)     || 'contain')

const PAPER_SIZES = [
  { value: '4x6',    label: '4×6 in  (10×15 cm)' },
  { value: '5x7',    label: '5×7 in  (13×18 cm)' },
  { value: '4x4',    label: '4×4 in  (10×10 cm) Square' },
  { value: 'letter', label: 'Letter (8.5×11 in)' },
  { value: 'a4',     label: 'A4 (210×297 mm)' },
  { value: 'a5',     label: 'A5 (148×210 mm)' },
]

function save() {
  localStorage.setItem(PRINTER_KEY, selectedPrinter.value)
  localStorage.setItem(PAPER_KEY,   paperSize.value)
  localStorage.setItem(ORIENT_KEY,  orientation.value)
  localStorage.setItem(COPIES_KEY,  String(copies.value))
  localStorage.setItem(FIT_KEY,     fitMode.value)
}

async function loadPrinters() {
  loadingPrint.value = true
  try {
    printers.value = await invoke('list_printers')
    // If saved printer is still in list, keep it; otherwise default to first
    if (selectedPrinter.value && !printers.value.includes(selectedPrinter.value)) {
      selectedPrinter.value = printers.value[0] ?? ''
      save()
    }
    if (!selectedPrinter.value && printers.value.length > 0) {
      selectedPrinter.value = printers.value[0]
      save()
    }
  } catch { /* desktop only */ }
  loadingPrint.value = false
}

onMounted(loadPrinters)
</script>

<template>
  <div class="panel pset-panel">
    <div class="panel-header">
      <h2>Printer</h2>
      <button class="btn-icon" :disabled="loadingPrint" title="Refresh printers" @click="loadPrinters">
        <span :class="{ spin: loadingPrint }">⟳</span>
      </button>
    </div>

    <div class="pset-form">

      <!-- Printer selector -->
      <div class="pset-group">
        <label class="pset-label">Printer</label>
        <select
          v-model="selectedPrinter"
          class="pset-select"
          @change="save"
        >
          <option value="" disabled>— Select printer —</option>
          <option v-for="p in printers" :key="p" :value="p">{{ p }}</option>
        </select>
        <span v-if="printers.length === 0 && !loadingPrint" class="pset-hint-warn">
          No printers found. Make sure a printer is installed.
        </span>
      </div>

      <!-- Paper size -->
      <div class="pset-group">
        <label class="pset-label">Paper size</label>
        <select v-model="paperSize" class="pset-select" @change="save">
          <option v-for="s in PAPER_SIZES" :key="s.value" :value="s.value">{{ s.label }}</option>
        </select>
      </div>

      <!-- Orientation -->
      <div class="pset-group pset-row">
        <label class="pset-label">Orientation</label>
        <div class="seg-ctrl">
          <button :class="{ active: orientation === 'portrait' }"  @click="orientation = 'portrait';  save()">Portrait</button>
          <button :class="{ active: orientation === 'landscape' }" @click="orientation = 'landscape'; save()">Landscape</button>
        </div>
      </div>

      <!-- Fit mode -->
      <div class="pset-group pset-row">
        <label class="pset-label">Image fit</label>
        <div class="seg-ctrl">
          <button :class="{ active: fitMode === 'contain' }" @click="fitMode = 'contain'; save()">Fit</button>
          <button :class="{ active: fitMode === 'cover' }"   @click="fitMode = 'cover';   save()">Fill</button>
        </div>
      </div>

      <!-- Copies -->
      <div class="pset-group pset-row">
        <label class="pset-label">Copies</label>
        <div class="copies-ctrl">
          <button @click="copies = Math.max(1, copies - 1); save()">−</button>
          <span>{{ copies }}</span>
          <button @click="copies = Math.min(10, copies + 1); save()">+</button>
        </div>
      </div>

    </div>
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

.btn-icon {
  background: none;
  border: 1px solid var(--c-border);
  color: var(--c-text-muted);
  border-radius: 6px;
  width: 28px;
  height: 28px;
  font-size: 1rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.13s;
}
.btn-icon:hover { background: var(--c-surface-2); color: var(--c-text); }
.btn-icon:disabled { opacity: 0.4; cursor: not-allowed; }
.spin { display: inline-block; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }

.pset-form {
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
}

.pset-group {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.pset-row {
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
}

.pset-label {
  font-size: 0.72rem;
  font-weight: 600;
  color: var(--c-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.pset-select {
  width: 100%;
  background: var(--c-surface-2);
  border: 1px solid var(--c-border);
  border-radius: 6px;
  color: var(--c-text);
  font-size: 0.82rem;
  padding: 0.38rem 0.55rem;
  font-family: inherit;
  outline: none;
  cursor: pointer;
}
.pset-select:focus { border-color: var(--c-accent); }

.pset-hint-warn {
  font-size: 0.72rem;
  color: #fbbf24;
}

/* Segmented control */
.seg-ctrl {
  display: flex;
  border: 1px solid var(--c-border);
  border-radius: 6px;
  overflow: hidden;
}
.seg-ctrl button {
  flex: 1;
  background: none;
  border: none;
  color: var(--c-text-muted);
  font-size: 0.75rem;
  font-weight: 600;
  padding: 0.3rem 0.6rem;
  cursor: pointer;
  font-family: inherit;
  transition: background 0.13s, color 0.13s;
}
.seg-ctrl button + button { border-left: 1px solid var(--c-border); }
.seg-ctrl button:hover { background: var(--c-surface-2); color: var(--c-text); }
.seg-ctrl button.active { background: var(--c-accent); color: #fff; }

/* Copies stepper */
.copies-ctrl {
  display: flex;
  align-items: center;
  gap: 0;
  border: 1px solid var(--c-border);
  border-radius: 6px;
  overflow: hidden;
}
.copies-ctrl button {
  background: none;
  border: none;
  color: var(--c-text);
  font-size: 1rem;
  width: 28px;
  height: 28px;
  cursor: pointer;
  font-family: inherit;
  transition: background 0.13s;
}
.copies-ctrl button:hover { background: var(--c-surface-2); }
.copies-ctrl span {
  min-width: 28px;
  text-align: center;
  font-size: 0.82rem;
  font-weight: 600;
  border-left: 1px solid var(--c-border);
  border-right: 1px solid var(--c-border);
  padding: 0 4px;
  line-height: 28px;
}
</style>
