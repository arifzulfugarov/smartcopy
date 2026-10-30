#![allow(unexpected_cfgs)]
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
mod service;

#[cfg(target_os = "macos")]
fn main() {
    let arg = std::env::args().nth(1);
    let result = match arg.as_deref() {
        Some("on") => service::on(),
        Some("off") => service::off(),
        Some("update") => service::update(),
        Some("status") => service::status(),
        Some("logs") => service::logs(),
        Some("run") => macos::run(),
        // Finder launch uses no arguments: launch the worker and request permission.
        None if std::env::current_exe().map(|p| p.to_string_lossy().contains(".app/Contents/MacOS/")).unwrap_or(false) => macos::run(),
        _ => { println!("Smart Copy\n  smartcopy on      Enable at login\n  smartcopy off     Disable\n  smartcopy status  Check status\n  smartcopy logs    View diagnostics\n  smartcopy update  Explicitly update the app"); Ok(()) }
    };
    if let Err(err) = result {
        eprintln!("Smart Copy: {err}");
        std::process::exit(1);
    }
}
#[cfg(not(target_os = "macos"))]
fn main() { eprintln!("This version currently supports macOS only."); std::process::exit(1); }
