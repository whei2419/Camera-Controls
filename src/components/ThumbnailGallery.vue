<script setup>
import { ref, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'

const props = defineProps({
    imageFolder: { type: String, default: '' },
    videoFolder: { type: String, default: '' },
    refreshTrigger: { type: Number, default: 0 }, // Increment to force refresh
})

const emit = defineEmits(['open-gallery'])

const IMG_EXTS = ['jpg', 'jpeg', 'png', 'cr2', 'cr3', 'nef', 'arw', 'tif', 'tiff']

const images = ref([])
const videos = ref([])
const loading = ref(false)

async function refresh() {
    if (!props.imageFolder && !props.videoFolder) {
        images.value = []
        videos.value = []
        return
    }

    loading.value = true
    try {
        const [imgFiles, vidFiles] = await Promise.all([
            props.imageFolder ? invoke('list_folder_files', { folder: props.imageFolder, extensions: IMG_EXTS }) : Promise.resolve([]),
            props.videoFolder ? invoke('list_folder_files', { folder: props.videoFolder, extensions: ['mp4', 'mov', 'mkv', 'avi', 'wmv'] }) : Promise.resolve([]),
        ])
        images.value = (imgFiles || []).slice(0, 40).map(p => ({ path: p, src: convertFileSrc(p) }))
        videos.value = (vidFiles || []).slice(0, 20).map(p => ({ path: p, src: convertFileSrc(p) }))
    } catch (e) {
        images.value = []
        videos.value = []
    }
    loading.value = false
}

watch(() => props.imageFolder, refresh, { immediate: true })
watch(() => props.videoFolder, refresh)
watch(() => props.refreshTrigger, refresh)

onMounted(refresh)
</script>

<template>
    <div class="tg-panel">

        <!-- Header -->
        <div class="tg-header">
            <div class="tg-title">
                <span class="tg-icon" aria-hidden>
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                        <path
                            d="M4 7H6L7.5 4H16.5L18 7H20C21.1 7 22 7.9 22 9V19C22 20.1 21.1 21 20 21H4C2.9 21 2 20.1 2 19V9C2 7.9 2.9 7 4 7Z"
                            stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
                        <circle cx="12" cy="14" r="3" stroke="currentColor" stroke-width="1.2" />
                    </svg>
                </span>
                <span>Recent Captures</span>
            </div>

            <button class="tg-icon-btn" title="Refresh" @click="refresh">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"
                    :class="{ spin: loading }">
                    <path d="M21 12A9 9 0 1 0 11.1 4.6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"
                        stroke-linejoin="round" />
                    <path d="M21 3V10H14" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"
                        stroke-linejoin="round" />
                </svg>
            </button>
        </div>

        <!-- Body -->
        <div class="tg-body">

            <!-- Empty state -->
            <div v-if="images.length === 0 && videos.length === 0 && !loading" class="tg-empty">
                <p class="empty-ico" aria-hidden>
                    <svg width="48" height="48" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                        <path
                            d="M4 7H6L7.5 4H16.5L18 7H20C21.1 7 22 7.9 22 9V19C22 20.1 21.1 21 20 21H4C2.9 21 2 20.1 2 19V9C2 7.9 2.9 7 4 7Z"
                            stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
                        <circle cx="12" cy="14" r="3" stroke="currentColor" stroke-width="1.2" />
                    </svg>
                </p>
                <p>No captures yet</p>
                <p class="hint">Photos and videos will appear here after capture</p>
            </div>

            <!-- Loading -->
            <div v-if="loading && images.length === 0 && videos.length === 0" class="tg-loading">Loading…</div>

            <!-- Rows: videos then images -->
            <div v-else class="tg-rows">
                <div class="tg-row tg-row-videos" v-if="videos.length">
                    <div class="row-title"><svg width="14" height="14" viewBox="0 0 24 24" fill="none"
                            xmlns="http://www.w3.org/2000/svg">
                            <path d="M23 7L16 12L23 17V7Z" fill="currentColor" />
                            <path d="M1 5H15V19H1V5Z" stroke="currentColor" stroke-width="1" />
                        </svg><span>Videos</span></div>
                    <div class="row-grid">
                        <div v-for="v in videos" :key="v.path" class="tg-thumb tg-thumb-video small"
                            :title="v.path.split(/[\\/]/).pop()" @click="$emit('open-gallery')">
                            <svg class="video-play-icon" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                                <rect width="24" height="24" rx="4" fill="currentColor" opacity="0.12"/>
                                <polygon points="9,7 19,12 9,17" fill="currentColor"/>
                            </svg>
                            <span class="video-name">{{ v.path.split(/[\\/]/).pop() }}</span>
                        </div>
                    </div>
                </div>

                <div class="tg-row tg-row-images" v-if="images.length">
                    <div class="row-title"><svg width="14" height="14" viewBox="0 0 24 24" fill="none"
                            xmlns="http://www.w3.org/2000/svg">
                            <path
                                d="M4 7H6L7.5 4H16.5L18 7H20C21.1 7 22 7.9 22 9V19C22 20.1 21.1 21 20 21H4C2.9 21 2 20.1 2 19V9C2 7.9 2.9 7 4 7Z"
                                stroke="currentColor" stroke-width="1.2" stroke-linecap="round"
                                stroke-linejoin="round" />
                            <circle cx="12" cy="14" r="3" stroke="currentColor" stroke-width="1.2" />
                        </svg><span>Photos</span></div>
                    <div class="row-grid">
                        <div v-for="img in images" :key="img.path" class="tg-thumb small"
                            :title="img.path.split(/[\\/]/).pop()" @click="$emit('open-gallery')">
                            <img :src="img.src" loading="lazy" />
                        </div>
                    </div>
                </div>
            </div>

        </div>

        <!-- Footer -->
        <div class="tg-footer">
            <button class="tg-view-btn" @click="$emit('open-gallery')">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"
                    style="vertical-align:middle;margin-right:6px">
                    <path d="M21 19V5H3V19H21Z" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"
                        stroke-linejoin="round" />
                    <path d="M8 11L10.5 14L13 11L16 16H8L8 11Z" stroke="currentColor" stroke-width="1" />
                </svg>
                View All
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
    to {
        transform: rotate(360deg);
    }
}

/* Body */
.tg-body {
    flex: 1;
    overflow-y: auto;
    padding: 10px;
}

.tg-empty,
.tg-loading {
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
.tg-rows {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.tg-row {
    display: flex;
    flex-direction: column;
    gap: 6px;
}

.row-title {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--c-text-muted);
    font-size: 0.75rem;
    font-weight: 700;
}

.row-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(56px, 1fr));
    gap: 6px;
}

.tg-thumb {
    aspect-ratio: 1;
    background: var(--c-surface-2);
    border: 1px solid var(--c-border);
    border-radius: 6px;
    overflow: hidden;
    cursor: pointer;
    transition: border-color 0.12s, transform 0.09s;
}

.tg-thumb.small {
    width: 100%;
    height: auto;
}

.tg-thumb:hover {
    border-color: var(--c-accent);
    transform: scale(1.02);
}

.tg-thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
}

.tg-thumb-video {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 4px;
    color: var(--c-text-muted);
}

.video-play-icon {
    width: 32px;
    height: 32px;
    flex-shrink: 0;
}

.video-name {
    font-size: 0.6rem;
    line-height: 1.2;
    text-align: center;
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    word-break: break-all;
    color: var(--c-text-muted);
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
