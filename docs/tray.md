**Tray integration — TODOs & Improvements**

Short list of follow-ups to implement or review later:

- **Threaded GTK integration**: keep the current `GtkTray::init()` threaded pattern; review edge cases and add tests.
- **Replace helper waiter with GLib channel**: use `glib::MainContext::channel` to signal shutdown from Rust into the GTK main loop (avoid an extra blocking thread).
- **Menu items and callbacks**: add example menu items (e.g., `Quit`) wired to the main app via channels (show sample code and channels to send messages to the main thread).
- **Shutdown and lifecycle docs**: document calling `TrayHandle::shutdown()` on app exit and graceful cleanup steps.
- **Error propagation**: consider returning early initialization errors from the GTK thread back to the caller (second channel or synchronous bootstrap) instead of only logging.
- **Dependency notes**: document distro-specific packages (Fedora/Cosmic, Debian/Ubuntu, Arch) required for appindicator and GTK, and note GNOME/COSMIC extension requirements.
- **Icon packaging**: document `include_bytes!` path expectations (src/tray/icon.ico) and how to replace the icon at build time.
- **Testing instructions**: add a manual QA checklist to verify tray behavior across DEs (GNOME+extension, COSMIC, KDE, X11 vs Wayland).
- **Logging & debugging**: add recommended run flags (`RUST_BACKTRACE=1 RUST_LOG=info`) and how to inspect libayatana/libappindicator warnings.
- **Optional improvements**: consider `libayatana-appindicator-glib` binding or DBus-based implementation for better modern support.

Notes
- Keep entries short; implement code changes first, then update this doc with code references and examples.
