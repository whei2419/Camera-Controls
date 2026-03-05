# Canon Camera Controls

A desktop app built with **Tauri + Vue 3** to remotely control Canon cameras over a wired USB connection using the **Canon EOS Digital SDK (EDSDK)**.

## Features

- **Camera Discovery** – scans for connected Canon cameras
- **Aperture (Av)** – full f-stop selector
- **Shutter Speed (Tv)** – full speed selector
- **ISO** – ISO 100 → 102400 + Auto
- **White Balance** – all WB modes
- **Battery indicator**
- **Shutter trigger** – full press & halfway (AF) press
- **Live View** – real-time JPEG preview streamed from the camera

## Prerequisites

1. **Rust & Cargo** — https://rustup.rs
2. **Node.js** — https://nodejs.org
3. **Canon EDSDK** — Register at https://developercommunity.usa.canon.com/ and download the EOS Digital SDK.
   Place `EDSDK.framework` (macOS) in `src-tauri/libs/` — see [src-tauri/libs/README.md](src-tauri/libs/README.md).

## Setup & Run

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

## Project Structure

```
src/
  components/
    CameraConnect.vue    # Scan & connect to camera
    CameraControls.vue   # Av / Tv / ISO / WB controls
    ShutterButton.vue    # Take photo
    LiveView.vue         # Live preview
  App.vue

src-tauri/
  src/
    edsdk/
      ffi.rs             # Raw C FFI bindings to EDSDK
      mod.rs             # Safe Rust wrapper
      properties.rs      # Value → label lookup tables
    commands.rs          # Tauri commands (invoke targets)
    lib.rs               # App entry-point & command registration
  libs/                  # ← place EDSDK.framework here
  build.rs               # Links EDSDK at compile time
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
