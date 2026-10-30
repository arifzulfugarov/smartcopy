#![allow(unexpected_cfgs)]
#![allow(non_snake_case)] // Native Apple C APIs use their published names.

use std::ffi::{c_char, c_long, c_void, CStr, CString};
use objc::{class, msg_send, sel, sel_impl, runtime::Object};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

type CFRef = *const c_void;
type CFMutRef = *mut c_void;
type CFIndex = c_long;
type CGEventRef = *mut c_void;
type EventCallback = unsafe extern "C" fn(*mut c_void, u32, CGEventRef, *mut c_void) -> CGEventRef;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: CFRef) -> bool;
    fn AXUIElementCreateApplication(pid: i32) -> CFRef;
    fn AXUIElementCopyAttributeValue(element: CFRef, attribute: CFRef, result: *mut CFRef) -> i32;
    fn AXUIElementSetAttributeValue(element: CFRef, attribute: CFRef, value: CFRef) -> i32;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(value: CFRef);
    fn CFStringCreateWithCString(allocator: CFRef, text: *const c_char, encoding: u32) -> CFRef;
    fn CFGetTypeID(value: CFRef) -> usize;
    fn CFStringGetTypeID() -> usize;
    fn CFStringGetLength(value: CFRef) -> CFIndex;
    fn CFStringGetMaximumSizeForEncoding(length: CFIndex, encoding: u32) -> CFIndex;
    fn CFStringGetCString(value: CFRef, buffer: *mut c_char, buffer_size: CFIndex, encoding: u32) -> bool;
    fn CFMachPortCreateRunLoopSource(allocator: CFRef, port: CFMutRef, order: CFIndex) -> CFRef;
    fn CFRunLoopGetCurrent() -> CFMutRef;
    fn CFRunLoopAddSource(loop_ref: CFMutRef, source: CFRef, mode: CFRef);
    fn CFRunLoopRun();
    static kCFRunLoopCommonModes: CFRef;
    static kCFBooleanTrue: CFRef;
    fn CFDictionaryCreate(allocator: CFRef, keys: *const CFRef, values: *const CFRef, count: CFIndex, key_callbacks: CFRef, value_callbacks: CFRef) -> CFRef;
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(location: u32, placement: u32, options: u32, mask: u64, callback: EventCallback, user_data: *mut c_void) -> CFMutRef;
    fn CGEventTapEnable(tap: CFMutRef, enabled: bool);
}

// NSWorkspace is implemented in AppKit. Linking the framework is required
// when using Objective-C's runtime from a Rust binary.
#[link(name = "AppKit", kind = "framework")]
extern "C" {}

const UTF8: u32 = 0x0800_0100;
const MOUSE_DOWN: u32 = 1;
const MOUSE_UP: u32 = 2;
const MOUSE_DRAG: u32 = 6;
const EVENT_TAP_DISABLED_TIMEOUT: u32 = 0xffff_fffe;
const EVENT_TAP_DISABLED_BY_USER_INPUT: u32 = 0xffff_ffff;

static DRAGGED: AtomicBool = AtomicBool::new(false);

struct OwnedCF(CFRef);
impl Drop for OwnedCF {
    fn drop(&mut self) {
        if !self.0.is_null() { unsafe { CFRelease(self.0); } }
    }
}

fn cf_string(value: &str) -> Option<OwnedCF> {
    let c = CString::new(value).ok()?;
    let pointer = unsafe { CFStringCreateWithCString(ptr::null(), c.as_ptr(), UTF8) };
    if pointer.is_null() { None } else { Some(OwnedCF(pointer)) }
}

fn attribute(element: CFRef, key: &str) -> Option<OwnedCF> {
    let cf_key = cf_string(key)?;
    let mut value: CFRef = ptr::null();
    let status = unsafe { AXUIElementCopyAttributeValue(element, cf_key.0, &mut value) };
    if status == 0 && !value.is_null() { Some(OwnedCF(value)) } else {
        if std::env::var_os("SMARTCOPY_DEBUG").is_some() { eprintln!("[debug] {key} unavailable (AX error {status})"); }
        None
    }
}

fn to_string(value: CFRef) -> Option<String> {
    unsafe {
        if CFGetTypeID(value) != CFStringGetTypeID() { return None; }
        let size = CFStringGetMaximumSizeForEncoding(CFStringGetLength(value), UTF8);
        if !(0..=2_000_000).contains(&size) { return None; }
        let mut buf = vec![0i8; size as usize + 1];
        if !CFStringGetCString(value, buf.as_mut_ptr(), buf.len() as CFIndex, UTF8) { return None; }
        Some(CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned())
    }
}

fn frontmost_chrome_pid() -> Option<i32> {
    unsafe {
        let workspace: *mut Object = msg_send![class!(NSWorkspace), sharedWorkspace];
        if workspace.is_null() { return None; }
        let app: *mut Object = msg_send![workspace, frontmostApplication];
        if app.is_null() { return None; }

        let bundle: *mut Object = msg_send![app, bundleIdentifier];
        if bundle.is_null() { return None; }
        let id_ptr: *const c_char = msg_send![bundle, UTF8String];
        if id_ptr.is_null() { return None; }
        let bundle_id = CStr::from_ptr(id_ptr).to_string_lossy();
        if std::env::var_os("SMARTCOPY_DEBUG").is_some() { eprintln!("[debug] Frontmost bundle: {bundle_id}"); }
        if bundle_id != "com.google.Chrome" { return None; }

        let pid: i32 = msg_send![app, processIdentifier];
        if std::env::var_os("SMARTCOPY_DEBUG").is_some() { eprintln!("[debug] Chrome PID: {pid}"); }
        Some(pid)
    }
}


// Chromium accepts AXEnhancedUserInterface on its application accessibility
// element. Keep one request per Chrome PID, rather than incrementing its
// assistive-technology request count on every mouse selection.
fn request_chrome_accessibility(pid: i32) {
    let app = OwnedCF(unsafe { AXUIElementCreateApplication(pid) });
    if app.0.is_null() { return; }

    // Querying the app role also triggers native accessibility activation in
    // current Chromium builds. A failed query is not fatal.
    let _ = attribute(app.0, "AXRole");

    if let Some(key) = cf_string("AXEnhancedUserInterface") {
        let status = unsafe { AXUIElementSetAttributeValue(app.0, key.0, kCFBooleanTrue) };
        if std::env::var_os("SMARTCOPY_DEBUG").is_some() {
            eprintln!("[debug] Chrome accessibility activation request: {status}");
        }
    }
}

fn selected_chrome_text() -> Option<String> {
    let pid = frontmost_chrome_pid()?;
    let chrome = OwnedCF(unsafe { AXUIElementCreateApplication(pid) });
    if chrome.0.is_null() {
        eprintln!("[debug] Could not create Chrome accessibility element");
        return None;
    }
    let focused = attribute(chrome.0, "AXFocusedUIElement")?;
    if std::env::var_os("SMARTCOPY_DEBUG").is_some() {
        for key in ["AXRole", "AXSubrole"] {
            if let Some(value) = attribute(focused.0, key).and_then(|v| to_string(v.0)) {
                eprintln!("[debug] {key}: {value}");
            }
        }
    }

    // Do not read from password/secure text fields.
    if let Some(subrole) = attribute(focused.0, "AXSubrole").and_then(|v| to_string(v.0)) {
        if subrole.contains("Secure") || subrole.contains("Password") { return None; }
    }
    let text = attribute(focused.0, "AXSelectedText").and_then(|v| to_string(v.0))?;
    if text.trim().is_empty() { None } else { Some(text) }
}

// Callback only sends a notification; blocking accessibility calls happen on a worker.
unsafe extern "C" fn event_callback(
    _proxy: *mut c_void,
    event_type: u32,
    event: CGEventRef,
    context: *mut c_void,
) -> CGEventRef {
    match event_type {
        MOUSE_DOWN => DRAGGED.store(false, Ordering::Relaxed),
        MOUSE_DRAG => DRAGGED.store(true, Ordering::Relaxed),
        MOUSE_UP => {
            if DRAGGED.swap(false, Ordering::Relaxed) {
                let sender = &*(context as *const Sender<()>);
                let _ = sender.send(());
            }
        }
        EVENT_TAP_DISABLED_TIMEOUT | EVENT_TAP_DISABLED_BY_USER_INPUT => {
            // The event tap should be reenabled in a future hardened release.
        }
        _ => {}
    }
    event
}

pub fn run() -> Result<(), String> {
    if !unsafe { AXIsProcessTrusted() } {
        // Request macOS's standard permission prompt from the installed app process.
        let key = cf_string("AXTrustedCheckOptionPrompt").ok_or("Cannot create permission key")?;
        let options = unsafe { CFDictionaryCreate(ptr::null(), &key.0, &kCFBooleanTrue, 1, ptr::null(), ptr::null()) };
        if !options.is_null() {
            unsafe { AXIsProcessTrustedWithOptions(options); CFRelease(options); }
        }
        return Err("Accessibility access required for Smart Copy.app. Enable it in System Settings > Privacy & Security > Accessibility, then run `smartcopy on`.".into());
    }
    let (tx, rx) = mpsc::channel::<()>();
    thread::spawn(move || {
        let mut clipboard = match arboard::Clipboard::new() {
            Ok(c) => c,
            Err(e) => { eprintln!("Cannot open clipboard: {e}"); return; }
        };
        let mut initialized_pid: Option<i32> = None;
        // No polling: the worker sleeps until a completed mouse drag wakes it.
        while rx.recv().is_ok() {
            if let Some(pid) = frontmost_chrome_pid() {
                if initialized_pid != Some(pid) {
                    request_chrome_accessibility(pid);
                    initialized_pid = Some(pid);
                }
            } else {
                initialized_pid = None;
                continue;
            }
            thread::sleep(Duration::from_millis(35));
            if let Some(text) = selected_chrome_text() {
                if let Err(e) = clipboard.set_text(text) {
                    eprintln!("Clipboard error: {e}");
                }
            }
        }
    });

    // Session event tap, at head of event chain, listen-only; never modifies input.
    let sender = Box::new(tx);
    let sender_ptr = Box::into_raw(sender);
    let mask = (1u64 << MOUSE_DOWN) | (1u64 << MOUSE_UP) | (1u64 << MOUSE_DRAG);
    let tap = unsafe { CGEventTapCreate(1, 0, 1, mask, event_callback, sender_ptr.cast()) };
    if tap.is_null() {
        unsafe { drop(Box::from_raw(sender_ptr)); }
        return Err("Unable to create a mouse event tap. Check Accessibility / Input Monitoring permissions, then restart Smart Copy.".into());
    }
    let source = unsafe { CFMachPortCreateRunLoopSource(ptr::null(), tap, 0) };
    if source.is_null() { return Err("Unable to create the macOS event loop source.".into()); }
    unsafe {
        CFRunLoopAddSource(CFRunLoopGetCurrent(), source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);
    }
    crate::service::announce_ready()?;
    if std::env::var_os("SMARTCOPY_DEBUG").is_some() { println!("Smart Copy listening for Chrome selections. Press Ctrl+C to stop."); }
    unsafe { CFRunLoopRun(); }
    Ok(())
}
