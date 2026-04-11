// Build as Windows GUI application (no console window)
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use gpui_component_assets::Assets;
use gpui::{px, AppContext, Styled, WindowOptions};
use gpui_component::ActiveTheme;

fn main() {
    // Single instance: exit if already running
    let instance = single_instance::SingleInstance::new("com.keyman.app").unwrap();
    if !instance.is_single() {
        return;
    }

    // Initialize i18n (detect system language)
    keyman_core::i18n::init();

    // Check accessibility permission on macOS
    #[cfg(target_os = "macos")]
    {
        check_accessibility_permission();
    }

    let app = gpui_platform::application().with_assets(Assets);
    app.run(move |cx| {
        gpui_component::init(cx);
        cx.activate(true);

        // Create main window
        let window_bounds = {
            gpui::Bounds::centered(
                None,
                gpui::size(px(640.0), px(480.0)),
                cx,
            )
        };

        cx.spawn(async move |cx| {
            let mut options = WindowOptions::default();
            options.window_bounds = Some(gpui::WindowBounds::Windowed(window_bounds));
            options.is_resizable = false;
            options.titlebar = Some(gpui::TitlebarOptions {
                title: Some(keyman_core::i18n::t_app_title().into()),
                ..Default::default()
            });

            cx.open_window(options, |window, cx| {
                window.on_window_should_close(cx, |window, _cx| {
                    if is_window_minimized(window) {
                        return true; // Already minimized → taskbar close → allow exit
                    }
                    window.minimize_window();
                    false // X button → minimize instead
                });

                let view = cx.new(|cx| keyman_ui::app::KeymanApp::new(window, cx));
                cx.new(|cx| {
                    gpui_component::Root::new(view, window, cx)
                        .bg(cx.theme().background)
                })
            })
            .expect("Failed to open window");
        })
        .detach();
    });
}

#[cfg(target_os = "windows")]
fn is_window_minimized(window: &gpui::Window) -> bool {
    use raw_window_handle::HasWindowHandle;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::IsIconic;

    if let Ok(handle) = HasWindowHandle::window_handle(window) {
        if let raw_window_handle::RawWindowHandle::Win32(win32) = handle.as_raw() {
            unsafe { IsIconic(HWND(win32.hwnd.get() as *mut _)).as_bool() }
        } else {
            false
        }
    } else {
        false
    }
}

#[cfg(not(target_os = "windows"))]
fn is_window_minimized(_window: &gpui::Window) -> bool {
    false
}

#[cfg(target_os = "macos")]
fn check_accessibility_permission() {
    // On macOS, CGEventTap requires accessibility permission.
    // We check at startup and print a warning if not granted.
    use core_graphics::event::{
        CGEventTap, CGEventTapLocation, CGEventTapPlacement,
        CGEventTapOptions, CGEventType,
    };

    let tap = CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::ListenOnly,
        vec![CGEventType::KeyDown],
        |_, _, _| None,
    );

    if tap.is_err() {
        eprintln!("WARNING: Accessibility permission not granted.");
        eprintln!("Please go to System Settings > Privacy & Security > Accessibility");
        eprintln!("and enable Keyman (键盘侠) (or your terminal) to use keyboard monitoring.");
    }
}
