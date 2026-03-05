# libs/

This folder is intentionally empty when using DigiCamControl.

No SDK files or libraries are required. DigiCamControl communicates with the
camera over USB and exposes a local HTTP REST API on port 5513 that this app
talks to.

## Setup

1. Download and install **DigiCamControl** (free, open-source, Windows only):
   https://digicamcontrol.com/

2. In DigiCamControl go to **Extra → Plugins** and enable **WebServer**.

3. Restart DigiCamControl. The WebServer will start automatically on
   `http://localhost:5513`.

4. Connect your camera via USB and power it on.

5. Launch this Tauri app. It will detect cameras through DigiCamControl.

## Supported cameras

DigiCamControl supports 700+ Canon and Nikon DSLR/mirrorless bodies, plus many
other brands via WIA/Windows drivers. See the full list at:
https://digicamcontrol.com/cameras


## Getting the SDK

Register as a developer at: https://developercommunity.usa.canon.com/
Then download the **EOS Digital SDK (EDSDK)**.
