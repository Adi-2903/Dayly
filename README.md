# Dayly v0.3 (Tauri)

## First: uninstall the old "DayWidget" (Settings > Apps > Installed apps), then:
## Update your existing GitHub repo (the one that built v0.2)
1. Quit the old widget (tray icon near the clock -> Quit, or the Quit button in its old settings; or Task Manager -> end "day-widget").
2. Copy everything from this folder over your repo folder (replace files; keep the hidden `.github` folder).
3. In a terminal inside the repo:
     git add .
     git commit -m "v0.3"
     git push
4. GitHub -> Actions -> "Build Windows installer" (it starts by itself on push; otherwise "Run workflow").
5. Download the artifact, unzip, run the Setup .exe (installs over the old version).

## Using it
- Tap = flip (days <-> clock). Drag = move. Mouse wheel = next event. Ctrl+wheel = resize.
- Drag the little corner handle (bottom-right, appears on hover) to resize like a normal window.
- Right-click (or hover + gear) = settings window next to the widget. Click anywhere else and it closes; every change is saved instantly.
- Tray icon (bottom-right of the taskbar, maybe under the ^ arrow): Settings / Stick to desktop on-off / Reset position / Quit.

## "Stick to desktop" (new, experimental)
Attaches the widget to the desktop layer: above your desktop icons, below every app, and it stays when you press Win+D.
If the widget ever looks black, empty or invisible: tray icon -> "Stick to desktop: on / off", then
tray -> "Reset position". Always-on-top has been removed.

Preview in any browser: open preview.html (same widget; settings appear beside it).
