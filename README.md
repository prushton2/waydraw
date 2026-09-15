# Waydraw
Turn a laptop into a drawing tablet connected to another computer. Built using H.264 for video encoding, Enigo for mouse control, Iced for UI, Pinray for screen capture, and Iroh for peer-to-peer connections.

## Support

OS | Support | Client Build | Server Build
--|--|--|--
Linux (Wayland) | Tested      | ✅ | ✅
Linux (X11)     | Untested    | ✅ | ✅
Windows         | Tested      | ✅ | ✅
MacOS           | Unsupported | ❌ | ❌

## Usage

On the host machine, download the server app. On the client machine you want to use as the drawing tablet, download the client app.<br />
After launching, select a display on the host machine and click 'Allow Connections'. On the client, either enter the pin or the key and click connect.

## Troubleshooting

* Timeout Error
  * click 'Allow Connections' on the server to generate a new pin, and retry connecting with the new pin
* TLS Error
  * Some networks cause issues with Iroh, the peer-to-peer library. These issues prevent Iroh from ensuring a private TLS connection. If you encounter this, try another network or use a VPN.
* Random freeze
  * If the client stops receiving frames or sending mouse movements, force close the app and relaunch, and click disconnect on the host.

## Known Issues
* Fractional scaling on Wayland
  * This is an issue in the display_info and pinray library, which I use for getting display info. This issue will persist until either library resolves the issue