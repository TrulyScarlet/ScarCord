# ScarCord

> A blazing fast, lightweight Discord desktop wrapper built with Tauri v2, Rust, and Windows WebView2.

![ScarCord Preview](https://raw.githubusercontent.com/TrulyScarlet/ScarCord/main/src-tauri/icons/128x128.png)

## Why ScarCord?

Official Discord runs on Electron, regularly consuming **300 MB to 700 MB+** of RAM and bundling an entire Chromium distribution.

**ScarCord** uses Windows's built-in **WebView2 (Chromium Evergreen)** runtime with a custom Rust window manager:
- **Tiny footprint:** Single ~5.7 MB standalone executable.
- **Resource light:** Typically runs at **~40–70 MB RAM** on idle.
- **Native 1:1 UI:** Zero artificial grey title bar. Seamless Discord-style controls aligned cleanly with the top-right header.
- **Background persistence:** Closing the window (`X`) hides it directly to the system tray so voice calls, stream audio, and online status never drop.
- **External link routing:** Web links open immediately in your default browser instead of navigating the Discord webview.
- **Safe & ToS-compliant:** Loads the official web client (`discord.com/app`) without injecting user-token bots or unauthorized client modifications.

---

## Features

- **Frameless 1:1 Aesthetic:** Full borderless window with natural top drag zone and double-click to maximize/restore.
- **System Tray Integration:** Left-click toggles hide/show. Right-click brings up Quick Show, Reload, and Quit actions.
- **Hardware & Media Permissions:** Built with media-stream flags so microphones, headsets, and audio feeds work out of the box without prompt loops.
- **External Link Interceptor:** Outbound YouTube, Twitter/X, Reddit, and website links route directly to your default browser.

---

## Building from Source

### Prerequisites
- [Rust](https://www.rust-lang.org/) (stable)
- [Node.js](https://nodejs.org/) (optional, for Tauri CLI tooling)
- Windows 10/11 (WebView2 is pre-installed)

### Build
```powershell
cd src-tauri
cargo build --release
```
The optimized binary will be created at:
```
src-tauri/target/release/scarcord.exe
```

---

## Author
Created by **TrulyScarlet**.
