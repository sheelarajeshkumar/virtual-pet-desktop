# Linux status

Linux is not a supported public-release target yet.

CI compiles the native Tauri binary on Ubuntu with the current WebKitGTK prerequisites. That catches portability regressions in Rust, Tauri configuration and bundled assets without claiming that the desktop overlay works on real Linux desktops.

Before adding Linux installers, test the running app on current GNOME and KDE sessions under both Wayland and X11. Verify always-on-top behavior, click-through, tray support, multi-monitor movement, audio, notifications, package installation and uninstall. Add an installer job only after those checks are recorded for a supported distribution matrix.
