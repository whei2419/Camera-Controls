// Example: Using the new resumable upload in Vue
import { invoke } from '@tauri-apps/api/core'

async function uploadVideoResumable(filePath) {
  try {
    const result = await invoke('upload_video_resumable', {
      filePath: filePath,
      url: 'https://glassbooth.wowbynow.com.my/api/upload-video/chunked'
    })
    
    console.log('Upload complete:', result)
    return result
  } catch (error) {
    console.error('Upload failed:', error)
    throw error
  }
}

// Usage in your component:
// await uploadVideoResumable('C:\\path\\to\\video.mp4')
