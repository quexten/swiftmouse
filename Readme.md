## Swiftmouse

Swiftmouse is a tool to quickly navigate your screen using only a keyboard. It is inspired by the shortcat app for mac and works by utilizing pipewire, and xdg desktop portal apis / x11 as fallback so it works on most wayland and x11 environments.
[Screencast from 2024-07-20 01-27-11.webm](https://github.com/user-attachments/assets/ffca1ea5-e0e5-46cb-8b3c-920c5d0642b9)

It is optimized for navigation speed, since this is important to ensure an interactive experience that does not feel
like there is a layer between you, and the action you intend to do (click a link). For this, low-latency screencapture (pipewire) and
custom, optimized image processing (fully multithreaded) are used. For a 4k screen, on a Tiger-lake i7 mobile cpu, this takes ~100ms for a 1080p screen
and 300-400 ms on a 4k screen.

### Usage

Build the daemon and gui bin's, then:
```
./daemon
```

Bind the following to your DE's custom shortcut facilities:
```
dbus-send --print-reply --dest=com.quexten.swiftmouse  /com/quexten/swiftmouse com.quexten.swiftmouse.Run
```

Then, when the gui is active after you pressed the shortcut:
```
a - image
o - lines
e - boxes
u - links
esc - exit
enter - left click & exit
```
