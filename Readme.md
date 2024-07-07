## Swiftmouse

Swiftmouse is a tool to quickly navigate your screen using only a keyboard. It is inspired by the shortcat app for mac and works by utilizing xdg desktop portal apis, so it works on wayland.

https://github.com/quexten/swiftmouse/assets/11866552/4fe3c589-a4e5-4766-8f69-4831705292e5

### Usage

```
swiftmouse {left/right/middle/double}
```

will take a screenshot, use image processing to detect ui elements, and show a ui in fullscreen. You can either enter 2 letters from a label to click on that position, or hit escape to cancel.
If you want to bind swiftmouse to a shortcut please use whatever facilities your desktop environment provides. Dual screen system are not supported at them moment.
