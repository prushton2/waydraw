# Waydraw
Turn a laptop into a drawing tablet connected to another computer. Built using H.264 for video encoding, Enigo for mouse control, Iced for UI, Pinray for screen capture, and Iroh for peer-to-peer connections.

## Support

OS | Supported | Tested
--|--|--
Linux (Wayland) | ✅ | ✅
Linux (X11)     | ✅ | ❌
Windows         | ✅ | ✅
MacOS           | ✅ | ✅

## Troubleshooting

* Timeout Error
  * click 'Allow Connections' on the server to generate a new pin, and retry connecting with the new pin
* TLS Error
  * Some networks may try to intercept the TLS of Iroh, the peer-to-peer library. Neither I nor Iroh will allow connecting with a bad TLS cert, please try a different network or a VPN.
* Random freeze
  * If the client stops receiving frames or sending mouse movements, force close the app and relaunch, and click disconnect on the host.

## Known Issues
* MacOS
  * I dont have a mac, so all my macos support is based on what claude says. Feel free to make patches for compatibility
* Fractional scaling on Wayland
  * This is an issue in the display_info and pinray library, which I use for getting display info. This issue will persist until either library resolves the issue