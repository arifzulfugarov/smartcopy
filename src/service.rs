use std::{env, fs, io::{self, IsTerminal, Write}, path::{Path, PathBuf}, process::Command, thread, time::Duration};
const LABEL: &str = "dev.smartcopy.chrome";
const BUNDLE_ID: &str = "dev.smartcopy.app";
extern "C" { fn getuid() -> u32; }
fn home() -> Result<PathBuf,String> { env::var_os("HOME").map(PathBuf::from).ok_or("HOME is missing".into()) }
fn app() -> Result<PathBuf,String> { Ok(home()?.join("Applications/Smart Copy.app")) }
fn worker() -> Result<PathBuf,String> { Ok(app()?.join("Contents/MacOS/smartcopy")) }
fn plist() -> Result<PathBuf,String> { Ok(home()?.join(format!("Library/LaunchAgents/{LABEL}.plist"))) }
fn logs_path() -> Result<PathBuf,String> { Ok(home()?.join("Library/Logs/SmartCopy/smartcopy.log")) }
fn ready_path() -> Result<PathBuf,String> { Ok(home()?.join("Library/Application Support/SmartCopy/ready")) }
fn domain() -> String { format!("gui/{}", unsafe { getuid() }) }
fn service_name() -> String { format!("{}/{LABEL}",domain()) }
fn ctl(args:&[&str]) -> Result<std::process::Output,String> { Command::new("/bin/launchctl").args(args).output().map_err(|e|e.to_string()) }
fn registered() -> Result<bool,String> { Ok(ctl(&["print", &service_name()])?.status.success()) }
fn xml(s:&str)->String { s.replace('&',"&amp;").replace('<',"&lt;").replace('>',"&gt;").replace('"',"&quot;").replace('\'',"&apos;") }
fn ensure_dir(p:&Path)->Result<(),String>{fs::create_dir_all(p).map_err(|e|e.to_string())}
fn fail(output:&std::process::Output)->String { String::from_utf8_lossy(&output.stderr).trim().to_owned() }
fn write_plist()->Result<(),String>{
 let path=plist()?;ensure_dir(path.parent().unwrap())?;
 let exe=xml(&worker()?.to_string_lossy());let log=xml(&logs_path()?.to_string_lossy());
 let content=format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>Label</key><string>{LABEL}</string>
<key>ProgramArguments</key><array><string>{exe}</string><string>run</string></array>
<key>RunAtLoad</key><true/>
<key>KeepAlive</key><false/>
<key>StandardOutPath</key><string>{log}</string>
<key>StandardErrorPath</key><string>{log}</string>
</dict></plist>"#);
 fs::write(&path,content).map_err(|e|format!("Could not write LaunchAgent: {e}"))
}
fn bundle_info() -> String { format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>{BUNDLE_ID}</string>
<key>CFBundleName</key><string>Smart Copy</string>
<key>CFBundleDisplayName</key><string>Smart Copy</string>
<key>CFBundleExecutable</key><string>smartcopy</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleVersion</key><string>1</string>
<key>CFBundleShortVersionString</key><string>0.3.1</string>
<key>LSUIElement</key><true/>
</dict></plist>"#) }
fn install_bundle()->Result<(),String>{
 let dest=worker()?;ensure_dir(dest.parent().unwrap())?;
 let src=env::current_exe().map_err(|e|e.to_string())?;
 if src != dest { fs::copy(&src,&dest).map_err(|e|format!("Cannot install executable: {e}"))?; }
 fs::write(app()?.join("Contents/Info.plist"),bundle_info()).map_err(|e|e.to_string())?;
 // Ad-hoc signing is for development only, not a stable identity across releases.
 let out=Command::new("/usr/bin/codesign").args(["--force","--deep","--sign","-"]).arg(app()?).output().map_err(|e|format!("codesign failed: {e}"))?;
 if !out.status.success(){return Err(format!("Could not sign app bundle: {}",fail(&out)));}
 Ok(())
}
fn animate(label:&str){if io::stdout().is_terminal(){for ch in ["◐","◓","◑","◒"]{print!("\r\x1b[2K{ch} {label}");let _=io::stdout().flush();thread::sleep(Duration::from_millis(55));}print!("\r\x1b[2K");let _=io::stdout().flush();}}
fn stop_agent()->Result<(),String>{
 if registered()? {let output=ctl(&["bootout",&domain(),&plist()?.to_string_lossy()])?;
 if !output.status.success(){return Err(format!("Cannot stop service: {}",fail(&output)));}}
 Ok(())
}
fn live() -> bool {
 let path=match ready_path(){Ok(p)=>p,Err(_)=>return false};
 let Ok(contents)=fs::read_to_string(path) else{return false};
 let Ok(pid)=contents.trim().parse::<i32>() else{return false};
 // kill(pid, 0) checks for a process without sending a signal.
 unsafe { kill(pid,0)==0 }
}
extern "C" {fn kill(pid:i32,sig:i32)->i32;}
pub fn on()->Result<(),String>{
 stop_agent()?;
 let _=fs::remove_file(ready_path()?);
 if !worker()?.exists() { install_bundle()?; }
 ensure_dir(logs_path()?.parent().unwrap())?;
 ensure_dir(ready_path()?.parent().unwrap())?;
 write_plist()?;
 let output=ctl(&["bootstrap",&domain(),&plist()?.to_string_lossy()])?;
 if !output.status.success(){return Err(format!("Could not start service: {}",fail(&output)));}
 for _ in 0..18 {if live(){animate("Starting Smart Copy");println!("✓ Smart Copy is ON — highlight text in Chrome.");return Ok(());}thread::sleep(Duration::from_millis(200));}
 eprintln!("⚠ Smart Copy needs attention: the worker did not start.");
 eprintln!("  Open ~/Applications/Smart Copy.app and approve Accessibility access.");
 eprintln!("  Then run `smartcopy on`. See `smartcopy logs` for details.");
 Err("Background service is not ready".into())
}
pub fn update()->Result<(),String>{
 stop_agent()?;
 let _=fs::remove_file(ready_path()?);
 install_bundle()?;
 println!("Smart Copy updated. macOS may require renewed Accessibility approval after an update.");
 on()
}
pub fn off()->Result<(),String>{stop_agent()?;let _=fs::remove_file(plist()?);let _=fs::remove_file(ready_path()?);animate("Stopping Smart Copy");println!("✓ Smart Copy is OFF.");Ok(())}
pub fn status()->Result<(),String>{if live(){println!("● Smart Copy is ON.");}else if registered()?{println!("! Smart Copy is installed but not running. Run `smartcopy logs`.");}else{println!("○ Smart Copy is OFF.");}Ok(())}
pub fn logs()->Result<(),String>{match fs::read_to_string(logs_path()?) {Ok(s)=>{for line in s.lines().rev().take(35).collect::<Vec<_>>().into_iter().rev(){println!("{line}");}},Err(e)=>println!("No diagnostics available: {e}")};Ok(())}
pub fn announce_ready()->Result<(),String>{let path=ready_path()?;ensure_dir(path.parent().unwrap())?;fs::write(path,std::process::id().to_string()).map_err(|e|e.to_string())}
