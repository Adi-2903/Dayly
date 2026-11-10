# Dayly v0.4

> A lightweight, fully customisable desktop countdown widget for Windows — built with Tauri + Rust.  
> Lives on your desktop, never in your taskbar. Flip it to see the clock. Zero idle RAM.

---

## ✨ Features at a glance

| | |
|---|---|
| 📅 **Multiple events** | Track as many countdowns as you want; scroll the widget to switch |
| 🎨 **20 themes** | Dark, light, and everything in between — all CSS-gradient based, no images |
| 🔲 **6 shapes** | Square, Wide, Tall, Circle, Arch, Pill — each clips the window to its exact outline |
| 🗓 **12 day styles** | Ring, Minimal, Split, Ticket, Stamp, Poster, Flip cards, Dots, Bar, D·H·M·S, Sentence, 5-days-to-go |
| 🕐 **5 clock styles** | Digital, Analog, Flip, Stacked, Rings |
| 📌 **Stick to desktop** | Attaches below every app window; survives Win+D |
| 🚀 **Start with Windows** | Optional autostart at login |
| ⚡ **Low power** | Redraws once per minute (or once per second for live styles); pauses when hidden |

---

## 🖼 Shapes

| Shape | Size (logical px) | Corner |
|-------|------------------|--------|
| **Square** | 280 × 280 | Rounded (34 px) |
| **Wide** | 420 × 200 | Rounded (32 px) |
| **Tall** | 200 × 300 | Rounded (36 px) |
| **Circle** | 270 × 270 | Full circle |
| **Arch** | 240 × 300 | Arch top, square-ish bottom |
| **Pill** | 320 × 120 | Fully rounded |

> **v0.4 new:** When pinned to the desktop the window is clipped to the card's exact shape (no white box or shadow leaking outside the outline).

---

## 🗓 Day Styles (front face)

| Style | What it shows |
|-------|--------------|
| **Ring** | Progress ring + big day number |
| **5 days to go** | Bold number + pill label ("5 days to go / today / 3 days ago") |
| **Split** | Giant number left, label right |
| **Ticket** | Boarding-pass layout with dashed tear line |
| **Stamp** | Rotated rubber-stamp block |
| **Poster** | Ghost outline number behind solid text |
| **Flip cards** | Mechanical flip-clock digit tiles |
| **Minimal** | Just the number and unit, extra large |
| **Dots** | Grid of dots, filled = elapsed days (up to 60) |
| **Bar** | Segmented progress bar + live hours/minutes/seconds |
| **D·H·M·S** | Days · Hours · Minutes · Seconds tiles (live, 1 s refresh) |
| **Sentence** | Typographic sentence: *"in **42** days until Exam"* |

---

## 🕐 Clock Styles (back face — tap the widget to flip)

| Style | Description |
|-------|-------------|
| **Digital** | HH:MM with AM/PM or 24 h |
| **Analog** | Traditional SVG clock face with tick marks |
| **Flip** | Mechanical flip-clock tiles for H and M |
| **Stacked** | Hours stacked above minutes, large bold type |
| **Rings** | Three concentric SVG arcs (hours / minutes / seconds) |

---

## 🎨 Themes (20 total)

### Dark themes
| Theme | Vibe |
|-------|------|
| **Midnight Dots** | Deep navy with dot-grid overlay — the default |
| **Aurora** | Northern-lights radial gradients |
| **Aero Glass** | Windows 7 frosted-glass look |
| **Blueprint** | Engineering grid on dark blue |
| **Sunset Stripes** | Diagonal stripes over warm purple-to-orange |
| **Neon Waves** | Cyan/magenta repeating radials on near-black |
| **Synthwave Grid** | Hot-pink perspective grid, retro-80s |
| **Lavender Mesh** | Soft purple/pink radial mesh |
| **Ember** | Warm orange glow on dark brown |
| **Ocean Ripples** | Cyan ripples over deep blue |
| **Topo Forest** | Topographic contour lines on dark green |
| **Terminal** | Green-on-black scanline CRT |
| **Carbon** | Diagonal weave on pure black |

### Light themes
| Theme | Vibe |
|-------|------|
| **Paper** | Dot-grid on cream — serif font |
| **Cream Serif** | Plain warm cream — serif font |
| **Sakura** | Pink cherry-blossom petals |
| **Mint Lines** | Diagonal lines on soft mint green |
| **Checker** | Subtle checkerboard on off-white |
| **Halftone Ink** | Halftone dot pattern on white |

---

## 🖱 Controls

| Gesture | Action |
|---------|--------|
| **Click / tap** | Flip between days and clock |
| **Drag** | Move the widget anywhere on screen |
| **Mouse wheel** | Cycle through your events (when you have more than one) |
| **Ctrl + scroll** | Resize (35 % – 160 %) |
| **Drag corner handle** | Resize live (bottom-right grip, appears on hover) |
| **Right-click** | Open settings panel next to the widget |
| **Hover → ⚙ icon** | Open settings panel |
| **Escape** | Close settings panel |

### Tray icon (near the clock, may be under `^`)
- **Settings** — open the settings panel
- **Stick to desktop: on / off** — toggle desktop pinning
- **Reset position** — snap back to top-right corner
- **Quit Dayly**

---

## ⚙️ Settings panel

Opened by right-clicking the widget or the tray icon → Settings. Click anywhere outside to close — **every change is saved instantly** to `localStorage`.

| Section | Options |
|---------|---------|
| **Events** | Add / rename / delete / reorder. Quick presets: Exam, Birthday, Trip, Deadline, Festival, Launch, Result, Wedding |
| **Size** | Slider 35 % – 160 % (also Ctrl+scroll or corner drag) |
| **Shape** | Square · Wide · Tall · Circle · Arch · Pill |
| **Days style** | 12 visual styles for the front face |
| **Clock style** | 5 visual styles for the back face |
| **Theme** | 20 colour-swatch grid |
| **Toggles** | Stick to desktop · Start with Windows · 24-hour clock |

---

## 📌 Stick to desktop

Attaches the widget to the Windows desktop layer:
- **Above** desktop icons
- **Below** every application window
- **Stays visible** when you press Win+D ("Show desktop")

> If the widget ever looks black or invisible after toggling:  
> Tray → **Stick to desktop: on / off**, then Tray → **Reset position**.

---

## 🏗 Architecture

```
dayly/
├── src/
│   └── index.html          # Entire frontend: UI, logic, themes — single self-contained file
├── src-tauri/
│   ├── src/
│   │   └── main.rs         # Rust/Tauri backend: window management, drag, pin, shape clip
│   ├── Cargo.toml          # Rust dependencies
│   ├── tauri.conf.json     # App config, bundle settings
│   ├── build.rs
│   ├── icons/              # App icons (PNG + ICO)
│   └── capabilities/       # Tauri permission scopes
├── .github/
│   └── workflows/
│       └── build.yml       # GitHub Actions: builds NSIS installer on every push to main
├── package.json
└── preview.html            # Browser preview (no Tauri needed)
```

### Tech stack
- **Frontend:** Vanilla HTML/CSS/JS — no framework, no bundler, no dependencies
- **Backend:** Rust + [Tauri v2](https://tauri.app)
- **Plugins:** `tauri-plugin-autostart`, `tauri-plugin-single-instance`
- **Windows API:** `windows-sys` — `SetWindowRgn`, `SetParent`, `ScreenToClient` for desktop pinning and shape clipping
- **Build:** GitHub Actions → NSIS installer (single-user, no admin required)

---

## 🚀 Building from source (via GitHub Actions)

1. Push to `main` — the **"Build Windows installer"** workflow starts automatically.
2. Go to **Actions** → click the latest run → scroll to **Artifacts** → download **`Dayly-installer`**.
3. Unzip → run the **Setup `.exe`** (installs for the current user, no admin prompt).

To trigger manually: **Actions → Build Windows installer → Run workflow**.

### Local build (requires Rust + Node 20)
```sh
npm install
npm run tauri build
# installer → src-tauri/target/release/bundle/nsis/*.exe
```

---

## 🔄 Updating

1. Quit Dayly (tray → **Quit**).
2. Run the new Setup `.exe` — it installs over the previous version. Settings are preserved.

---

## 🌐 Browser preview

Open `preview.html` in any browser — same widget, settings panel appears inline beside it. No Tauri or Rust required.

---

## 📋 Changelog

| Version | Highlights |
|---------|-----------|
| **v0.4** | Shape-clipped window region when pinned to desktop — no white box around the card |
| **v0.3** | Tauri v2 rewrite; stick-to-desktop (Win+D); tray menu; corner-grip resize; 20 themes; 12 day styles; 5 clock styles |

---

*Built by [Aditya Jain](https://github.com/Adi-2903)*
