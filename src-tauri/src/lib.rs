use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::WebviewWindowBuilder,
    Manager, WebviewUrl, WindowEvent,
};
use tauri_plugin_opener::OpenerExt;

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show", "Show Discord", true, None::<&str>)?;
    let restart_item = MenuItem::with_id(app, "reload", "Reload Discord", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit ScarCord", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &restart_item, &quit_item])?;

    let icon = app.default_window_icon().cloned().ok_or("No default window icon")?;

    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("ScarCord - Lightweight Discord")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.unminimize();
                    let _ = win.set_focus();
                }
            }
            "reload" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.eval("window.location.href = 'https://discord.com/app';");
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(win) = app.get_webview_window("main") {
                    if win.is_visible().unwrap_or(false) {
                        let _ = win.hide();
                    } else {
                        let _ = win.show();
                        let _ = win.unminimize();
                        let _ = win.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Client injection: 1:1 Discord native top-bar integration with proper icon clearance
            let init_script = r#"
                (function() {
                    function winCmd(action) {
                        try {
                            if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
                                window.__TAURI_INTERNALS__.invoke('plugin:window|' + action, { label: 'main' }).catch(function(e){ console.error('Tauri window cmd error:', e); });
                            } else if (window.__TAURI__ && window.__TAURI__.window && window.__TAURI__.window.getCurrentWindow) {
                                var w = window.__TAURI__.window.getCurrentWindow();
                                if (action === 'minimize') w.minimize();
                                else if (action === 'toggle_maximize') w.toggleMaximize();
                                else if (action === 'hide') w.hide();
                                else if (action === 'start_dragging') w.startDragging();
                                else if (action === 'internal_toggle_maximize') w.toggleMaximize();
                            }
                        } catch (e) {
                            console.error('Tauri invoke error:', e);
                        }
                    }

                    // 1. Intercept popup links to default external browser
                    window.open = function(url) {
                        if (url) {
                            try {
                                const parsed = new URL(url, window.location.href);
                                if (parsed.host === window.location.host || parsed.host.endsWith('.discord.com') || parsed.host.endsWith('.discordapp.com')) {
                                    window.location.href = url;
                                } else {
                                    const a = document.createElement('a');
                                    a.href = url;
                                    a.target = '_blank';
                                    a.rel = 'noreferrer noopener';
                                    document.body.appendChild(a);
                                    a.click();
                                    a.remove();
                                }
                            } catch(e) {
                                window.location.href = url;
                            }
                        }
                        return null;
                    };

                    // 2. Global external link routing
                    document.addEventListener('click', (e) => {
                        const anchor = e.target.closest('a');
                        if (!anchor || !anchor.href) return;

                        try {
                            const parsed = new URL(anchor.href, window.location.href);
                            const isDiscord = parsed.host === 'discord.com' || parsed.host.endsWith('.discord.com') || parsed.host === 'discordapp.com' || parsed.host.endsWith('.discordapp.com');
                            if (!isDiscord && (parsed.protocol === 'http:' || parsed.protocol === 'https:')) {
                                e.preventDefault();
                                e.stopPropagation();
                                if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
                                    window.__TAURI_INTERNALS__.invoke('plugin:opener|open_url', { path: anchor.href }).catch(function() {
                                        window.open(anchor.href);
                                    });
                                } else {
                                    window.open(anchor.href);
                                }
                            }
                        } catch (err) {}
                    }, true);

                    // 3. Inject native 1:1 window control buttons into Discord top-right
                    function injectNativeControls() {
                        if (document.getElementById('scarcord-window-controls')) return;

                        const controls = document.createElement('div');
                        controls.id = 'scarcord-window-controls';
                        controls.innerHTML = `
                            <button class="scarcord-win-btn" id="scarcord-min" title="Minimize">
                                <svg width="11" height="1" viewBox="0 0 11 1"><rect fill="currentColor" width="11" height="1"/></svg>
                            </button>
                            <button class="scarcord-win-btn" id="scarcord-max" title="Maximize">
                                <svg width="10" height="10" viewBox="0 0 10 10"><rect fill="none" stroke="currentColor" stroke-width="1" x="0.5" y="0.5" width="9" height="9"/></svg>
                            </button>
                            <button class="scarcord-win-btn scarcord-win-close" id="scarcord-close" title="Close to Tray">
                                <svg width="10" height="10" viewBox="0 0 10 10"><polygon fill="currentColor" points="10 0.7 9.3 0 5 4.3 0.7 0 0 0.7 4.3 5 0 9.3 0.7 10 5 5.7 9.3 10 10 9.3 5.7 5"/></svg>
                            </button>
                        `;

                        const style = document.createElement('style');
                        style.id = 'scarcord-native-style';
                        style.textContent = `
                            /* Native 1:1 window control buttons at top-right - 28px titlebar, hover stays inside */
                            #scarcord-window-controls {
                                position: fixed;
                                top: 0;
                                right: 0;
                                height: 28px;
                                display: flex;
                                align-items: stretch;
                                z-index: 99999999;
                                -webkit-user-select: none;
                                user-select: none;
                                pointer-events: auto;
                                background: transparent;
                                overflow: hidden;
                            }
                            .scarcord-win-btn {
                                display: flex;
                                align-items: center;
                                justify-content: center;
                                width: 46px;
                                height: 28px;
                                line-height: 28px;
                                background: transparent;
                                border: none;
                                outline: none;
                                color: #949ba4;
                                cursor: pointer;
                                padding: 0;
                                margin: 0;
                                box-sizing: border-box;
                                transition: background 0.12s ease, color 0.12s ease;
                            }
                            .scarcord-win-btn:hover {
                                background: rgba(255, 255, 255, 0.08);
                                color: #f2f3f5;
                            }
                            .scarcord-win-close:hover {
                                background: #da373c !important;
                                color: #ffffff !important;
                            }

                            /* Shift Discord's top right header/toolbar (Inbox, Help, Member list, Search) to the left */
                            /* 138px = 3 buttons x 46px */
                            [class*="toolbar_"],
                            [class*="upperContainer_"] > [class*="children_"] + div,
                            section[class*="title_"] [class*="toolbar_"],
                            header[class*="header_"] [class*="toolbar_"],
                            div[class*="trailing_"],
                            div[class*="subtitleContainer_"] + div {
                                margin-right: 142px !important;
                                padding-right: 0 !important;
                            }

                            /* Clean scrollbars */
                            ::-webkit-scrollbar {
                                width: 8px !important;
                                height: 8px !important;
                            }
                            ::-webkit-scrollbar-thumb {
                                background: rgba(255, 255, 255, 0.18) !important;
                                border-radius: 4px !important;
                            }
                            ::-webkit-scrollbar-thumb:hover {
                                background: rgba(255, 255, 255, 0.3) !important;
                            }
                        `;

                        if (document.head) {
                            document.head.appendChild(style);
                        }
                        document.body.appendChild(controls);

                        // Attach button handlers - use built-in window plugin cmds (already allowed for remote origin)
                        document.getElementById('scarcord-min').addEventListener('click', (e) => {
                            e.preventDefault();
                            e.stopPropagation();
                            winCmd('minimize');
                        });
                        document.getElementById('scarcord-max').addEventListener('click', (e) => {
                            e.preventDefault();
                            e.stopPropagation();
                            winCmd('toggle_maximize');
                        });
                        document.getElementById('scarcord-close').addEventListener('click', (e) => {
                            e.preventDefault();
                            e.stopPropagation();
                            winCmd('hide');
                        });
                    }

                    // Dynamically enforce right margin on Discord toolbar elements across views
                    function adjustDiscordHeader() {
                        // Find the topmost right toolbar container in Discord's header
                        const candidates = document.querySelectorAll('[class*="toolbar_"], [aria-label="Help"], [aria-label="Inbox"]');
                        candidates.forEach(el => {
                            const toolbar = el.closest('[class*="toolbar_"]') || (el.parentElement && el.parentElement.parentElement);
                            if (toolbar && toolbar !== document.body) {
                                toolbar.style.setProperty('margin-right', '142px', 'important');
                            }
                        });
                    }

                    // 4. Native dragging on top header area (excluding interactive icons, inputs, buttons)
                    // Use pointerdown + preventDefault so WebView2 starts the native drag reliably
                    window.addEventListener('pointerdown', (e) => {
                        if (e.target.closest('#scarcord-window-controls')) return;
                        if (e.button !== 0 || e.isPrimary === false) return;

                        const inTopRegion = e.clientY <= 28;
                        if (!inTopRegion) return;
                        const isInteractive = e.target.closest('button, a, input, textarea, select, [role="button"], [role="tab"], [tabindex], img, video, .scarcord-win-btn');
                        if (isInteractive) return;

                        if (e.detail === 2) {
                            winCmd('toggle_maximize');
                        } else {
                            e.preventDefault();
                            winCmd('start_dragging');
                        }
                    });

                    if (document.readyState === 'loading') {
                        document.addEventListener('DOMContentLoaded', () => {
                            injectNativeControls();
                            adjustDiscordHeader();
                        });
                    } else {
                        injectNativeControls();
                        adjustDiscordHeader();
                    }

                    // Observe DOM changes to maintain toolbar margin dynamically on navigation
                    const observer = new MutationObserver(() => {
                        adjustDiscordHeader();
                        if (!document.getElementById('scarcord-window-controls') && document.body) {
                            injectNativeControls();
                        }
                    });
                    observer.observe(document.documentElement, { childList: true, subtree: true });
                })();
            "#;

            let discord_url: tauri::Url = "https://discord.com/app"
                .parse()
                .expect("Failed to parse Discord URL");

            let mut builder = WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External(discord_url),
            )
            .title("Discord")
            .decorations(false)
            .inner_size(1280.0, 800.0)
            .min_inner_size(800.0, 600.0)
            .initialization_script(init_script)
            .on_navigation(move |nav_url| {
                let host = nav_url.host_str().unwrap_or_default();
                let is_discord = host == "discord.com" || host.ends_with(".discord.com") || host == "discordapp.com" || host.ends_with(".discordapp.com");

                if !is_discord && (nav_url.scheme() == "http" || nav_url.scheme() == "https") {
                    let _ = app_handle.opener().open_url(nav_url.as_str(), None::<&str>);
                    false
                } else {
                    true
                }
            });

            #[cfg(target_os = "windows")]
            {
                builder = builder.additional_browser_args(
                    "--use-fake-ui-for-media-stream --autoplay-policy=no-user-gesture-required",
                );
            }

            builder.build()?;

            setup_tray(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running ScarCord");
}
