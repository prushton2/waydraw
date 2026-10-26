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
  * You must allow the in the `Accessbility` and `Screen & System Audio Recording` settings for the app to be able to be a host
  * On first launch, use `right click -> open`
  * Macos will not remember the accessibility settings between versions, since I dont have an Apple developer ID
* Windows
  * The app is flagged as a virus by default. Microsoft defender remains a mystery to me, so I just ended up disabling it for waydraw to work.
* Fractional scaling on Wayland
  * This is an issue in the display_info and pinray library, which I use for getting display info. This issue will persist until either library resolves the issue