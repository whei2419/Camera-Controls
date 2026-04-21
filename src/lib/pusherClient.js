import Pusher from 'pusher-js'

let pusher = null
let channel = null

export function initPusher({ key, cluster, channelName = 'camera-control', onCapture, onRecordStart, onRecordStop, onFeedToggle, onConnected, onDisconnected }) {
  if (pusher) return pusher
  pusher = new Pusher(key, { cluster, forceTLS: true })
  channel = pusher.subscribe(channelName)

  if (onCapture) channel.bind('capture', (data) => onCapture(data))
  if (onRecordStart) channel.bind('record:start', (data) => onRecordStart(data))
  if (onRecordStop) channel.bind('record:stop', (data) => onRecordStop(data))
  if (onFeedToggle) channel.bind('feed:toggle', (data) => onFeedToggle(data))

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
