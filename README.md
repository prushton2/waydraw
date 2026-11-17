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
* TLS Error
  * Some networks may try to intercept the TLS of Iroh, the peer-to-peer library. Neither I nor Iroh will allow connecting with a bad TLS cert, please try a different network or a VPN.

## Using
* Linux
  * Make sure the app has execute permissions with `chmod +x ~/Downloads/waydraw-linux`
* MacOS
  * You must allow waydraw in the `Accessbility` and `Screen & System Audio Recording` accessibility settings for the app to be able to be a host
  * On first launch, use `right click -> open`
  * Macos will not remember the accessibility settings between versions, since I dont have an Apple developer ID.
* Windows
  * The app is flagged as a virus by default. Microsoft defender remains a mystery to me, so I just ended up disabling it for waydraw to work.
