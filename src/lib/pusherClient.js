import Pusher from 'pusher-js'
import { remoteSite } from '../config/remoteSite'

let pusher = null
let channel = null

export function initPusher({ key, cluster, channelName = 'camera-control', onCapture, onRecordStart, onRecordStop, onFeedToggle, onReload, onRequestState, onConnected, onDisconnected }) {
  if (pusher) return pusher
  pusher = new Pusher(key, { cluster, forceTLS: true })
  channel = pusher.subscribe(channelName)

  if (onCapture) channel.bind('capture', (data) => onCapture(data))
  if (onRecordStart) channel.bind('record:start', (data) => onRecordStart(data))
  if (onRecordStop) channel.bind('record:stop', (data) => onRecordStop(data))
  if (onFeedToggle) channel.bind('feed:toggle', (data) => onFeedToggle(data))
  if (onReload) channel.bind('reload', () => onReload())
  if (onRequestState) channel.bind('request_state', () => onRequestState())

  // connection lifecycle
  pusher.connection.bind('connected', () => {
    console.log('Pusher connection established')
    if (onConnected) onConnected()
  })
  pusher.connection.bind('disconnected', () => {
    console.log('Pusher connection closed')
    if (onDisconnected) onDisconnected()
  })

  console.log('Pusher initialized, listening on', channelName)
  return pusher
}

export function disconnectPusher() {
  if (!pusher) return
  if (channel) pusher.unsubscribe(channel.name)
  pusher.disconnect()
  pusher = null
  channel = null
}

/**
 * Broadcast a status event to the monitoring page by posting to the remote
 * Laravel API, which re-publishes it on the camera-control Pusher channel.
 * Fire-and-forget — errors are logged but never thrown.
 *
 * @param {string} event  e.g. 'obs_connected', 'recording_started'
 * @param {object} [data] optional payload
 */
export async function broadcastEvent(event, data = {}) {
  try {
    await fetch(`${remoteSite.baseUrl}/api/monitor/broadcast`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ event, data }),
    })
  } catch (e) {
    console.warn('[broadcastEvent]', event, e)
  }
}
