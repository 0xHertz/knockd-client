use crate::models::SshClient;
#[cfg(any(target_os = "windows", target_os = "macos"))]
use std::path::Path;
use std::process::Command;

// ─── Detection ───────────────────────────────────────────────────

#[cfg(target_os = "windows")]
pub fn detect_ssh_clients() -> Vec<SshClient> {
    let mut clients = Vec::new();

    clients.push(SshClient {
        name: "Windows Terminal".into(),
        path: "wt".into(),
        installed: which_installed("wt"),
    });

    clients.push(SshClient {
        name: "OpenSSH (ssh)".into(),
        path: "ssh".into(),
        installed: which_installed("ssh"),
    });

    clients.push(SshClient {
        name: "PuTTY".into(),
        path: "putty".into(),
        installed: check_putty(),
    });

    clients.push(SshClient {
        name: "KiTTY".into(),
        path: detect_kitty(),
        installed: !detect_kitty().is_empty(),
    });

    clients.push(SshClient {
        name: "MobaXterm".into(),
        path: detect_mobaxterm(),
        installed: !detect_mobaxterm().is_empty(),
    });

    clients.push(SshClient {
        name: "Xshell".into(),
        path: detect_xshell(),
        installed: !detect_xshell().is_empty(),
    });

    clients.push(SshClient {
        name: "NxShell".into(),
        path: detect_nxshell(),
        installed: !detect_nxshell().is_empty(),
    });

    clients.push(SshClient {
        name: "Bitvise SSH".into(),
        path: detect_bitvise(),
        installed: !detect_bitvise().is_empty(),
    });

    clients.push(SshClient {
        name: "SecureCRT".into(),
        path: detect_securecrt(),
        installed: !detect_securecrt().is_empty(),
    });

    clients.push(SshClient {
        name: "Termius".into(),
        path: "termius".into(),
        installed: which_installed("termius"),
    });

    clients
}

#[cfg(target_os = "macos")]
pub fn detect_ssh_clients() -> Vec<SshClient> {
    vec![
        SshClient {
            name: "Terminal + ssh".into(),
            path: "ssh".into(),
            installed: true,
        },
        SshClient {
            name: "iTerm2".into(),
            path: "iterm2".into(),
            installed: app_installed("iTerm"),
        },
        SshClient {
            name: "Termius".into(),
            path: "termius".into(),
            installed: app_installed("Termius"),
        },
    ]
}

#[cfg(target_os = "linux")]
pub fn detect_ssh_clients() -> Vec<SshClient> {
    vec![
        SshClient {
            name: "OpenSSH (ssh)".into(),
            path: "ssh".into(),
            installed: which_installed("ssh"),
        },
        SshClient {
            name: "PuTTY".into(),
            path: "putty".into(),
            installed: which_installed("putty"),
        },
        SshClient {
            name: "Termius".into(),
            path: "termius".into(),
            installed: which_installed("termius"),
        },
    ]
}

// ─── Launch ──────────────────────────────────────────────────────

pub fn launch_ssh(client: &str, host: &str, port: u16, username: &str) -> Result<String, String> {
    let user_host = if username.is_empty() {
        host.to_string()
    } else {
        format!("{}@{}", username, host)
    };

    let client_lower = client.to_lowercase();

    // ── CLI / PATH-based clients ──
    match client_lower.as_str() {
        "putty" => {
            Command::new("putty")
                .args(["-ssh", &user_host, "-P", &port.to_string()])
                .spawn()
                .map_err(|e| format!("Failed to launch PuTTY: {}", e))?;
        }
        "kitty" => {
            let path = detect_kitty();
            if path.is_empty() {
                return Err("KiTTY not found".into());
            }
            Command::new(&path)
                .args(["-ssh", &user_host, "-P", &port.to_string()])
                .spawn()
                .map_err(|e| format!("Failed to launch KiTTY: {}", e))?;
        }
        "xshell" => {
            let path = detect_xshell();
            if path.is_empty() {
                return Err("Xshell not found".into());
            }
            Command::new(&path)
                .args(["-url", &format!("ssh://{}:{}", user_host, port)])
                .spawn()
                .map_err(|e| format!("Failed to launch Xshell: {}", e))?;
        }
        "mobaxterm" => {
            let path = detect_mobaxterm();
            if path.is_empty() {
                return Err("MobaXterm not found".into());
            }
            Command::new(&path)
                .arg("-newtab")
                .arg(format!("ssh://{}:{}", user_host, port))
                .spawn()
                .map_err(|e| format!("Failed to launch MobaXterm: {}", e))?;
        }
        "nxshell" => {
            let path = detect_nxshell();
            if path.is_empty() {
                return Err("NxShell not found".into());
            }
            Command::new(&path)
                .arg(format!("ssh://{}:{}", user_host, port))
                .spawn()
                .map_err(|e| format!("Failed to launch NxShell: {}", e))?;
        }
        "bitvise ssh" | "bitvise" => {
            let path = detect_bitvise();
            if path.is_empty() {
                return Err("Bitvise SSH not found".into());
            }
            Command::new(&path)
                .args([
                    &format!("-host={}", host),
                    &format!("-port={}", port),
                    &format!("-user={}", username),
                ])
                .spawn()
                .map_err(|e| format!("Failed to launch Bitvise: {}", e))?;
        }
        "securecrt" => {
            let path = detect_securecrt();
            if path.is_empty() {
                return Err("SecureCRT not found".into());
            }
            Command::new(&path)
                .args(["/SSH2", "/L", username, host, "/P", &port.to_string()])
                .spawn()
                .map_err(|e| format!("Failed to launch SecureCRT: {}", e))?;
        }
        "termius" => {
            Command::new("termius")
                .arg("ssh")
                .arg(&user_host)
                .spawn()
                .map_err(|e| format!("Failed to launch Termius: {}", e))?;
        }
        "windows terminal" | "wt" => {
            Command::new("wt")
                .args(["ssh", &user_host, "-p", &port.to_string()])
                .spawn()
                .map_err(|e| format!("Failed to launch Windows Terminal: {}", e))?;
        }
        _ => {
            launch_terminal_ssh(&user_host, port)?;
        }
    }

    Ok(format!("Launched {} connection to {}", client, host))
}

pub fn launch_url(url: &str) -> Result<String, String> {
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        Command::new("cmd")
            .args(["/c", "start", "", url])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    Ok(format!("Opened {}", url))
}

// ─── Terminal-based fallback ─────────────────────────────────────

fn launch_terminal_ssh(user_host: &str, port: u16) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let terminals = [
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
            "x-terminal-emulator",
        ];
        let ssh_args = vec![
            "ssh".to_string(),
            user_host.to_string(),
            "-p".to_string(),
            port.to_string(),
        ];
        for term in &terminals {
            if which_installed(term) {
                let mut cmd = if *term == "gnome-terminal" {
                    let mut c = Command::new(term);
                    c.arg("--").args(&ssh_args);
                    c
                } else {
                    let mut c = Command::new(term);
                    c.arg("-e").arg("ssh").args(&ssh_args[1..]);
                    c
                };
                return cmd
                    .spawn()
                    .map(|_| ())
                    .map_err(|e| format!("Failed: {}", e));
            }
        }
        Command::new("ssh")
            .args([user_host, "-p", &port.to_string()])
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("Failed: {}", e))
    }

    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "tell application \"Terminal\" to do script \"ssh {} -p {}\"",
            user_host, port
        );
        Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("Failed: {}", e))
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("ssh")
            .args([user_host, "-p", &port.to_string()])
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("Failed: {}", e))
    }
}

// ─── Helpers ─────────────────────────────────────────────────────

fn which_installed(cmd: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        Command::new("where")
            .arg(cmd)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

#[cfg(target_os = "windows")]
fn check_paths(paths: &[&str]) -> String {
    for p in paths {
        let expanded = expand_env_vars(p);
        if Path::new(&expanded).exists() {
            return expanded;
        }
    }
    for p in paths {
        if p.contains('*') {
            if let Ok(entries) = glob::glob(p) {
                if let Some(Ok(path)) = entries.into_iter().next() {
                    return path.to_string_lossy().to_string();
                }
            }
        }
    }
    String::new()
}

#[cfg(target_os = "windows")]
fn expand_env_vars(s: &str) -> String {
    let mut result = s.to_string();
    for (var, env_key) in &[
        ("%LOCALAPPDATA%", "LOCALAPPDATA"),
        ("%APPDATA%", "APPDATA"),
        ("%USERPROFILE%", "USERPROFILE"),
        ("%PROGRAMFILES%", "ProgramFiles"),
        ("%PROGRAMFILES(X86)%", "ProgramFiles(x86)"),
    ] {
        if result.contains(var) {
            if let Ok(val) = std::env::var(env_key) {
                result = result.replace(var, &val);
            }
        }
    }
    result
}

// ── Per-client detectors (Windows) ───────────────────────────────

#[cfg(target_os = "windows")]
fn check_putty() -> bool {
    let paths = [
        r"C:\Program Files\PuTTY\putty.exe",
        r"C:\Program Files (x86)\PuTTY\putty.exe",
    ];
    paths.iter().any(|p| Path::new(p).exists()) || which_installed("putty")
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
fn check_putty() -> bool {
    which_installed("putty")
}

#[cfg(target_os = "windows")]
fn detect_kitty() -> String {
    check_paths(&[
        r"C:\Program Files\KiTTY\kitty.exe",
        r"C:\Program Files (x86)\KiTTY\kitty.exe",
        r"%LOCALAPPDATA%\KiTTY\kitty.exe",
    ])
}
#[cfg(not(target_os = "windows"))]
fn detect_kitty() -> String {
    String::new()
}

#[cfg(target_os = "windows")]
fn detect_mobaxterm() -> String {
    check_paths(&[
        r"C:\Program Files (x86)\Mobatek\MobaXterm\MobaXterm.exe",
        r"C:\Program Files\Mobatek\MobaXterm\MobaXterm.exe",
        r"%USERPROFILE%\MobaXterm\MobaXterm.exe",
    ])
}
#[cfg(not(target_os = "windows"))]
fn detect_mobaxterm() -> String {
    String::new()
}

#[cfg(target_os = "windows")]
fn detect_xshell() -> String {
    check_paths(&[
        r"C:\Program Files\NetSarang\Xshell*\Xshell.exe",
        r"C:\Program Files (x86)\NetSarang\Xshell*\Xshell.exe",
    ])
}
#[cfg(not(target_os = "windows"))]
fn detect_xshell() -> String {
    String::new()
}

#[cfg(target_os = "windows")]
fn detect_nxshell() -> String {
    check_paths(&[
        r"C:\Program Files\NxShell\NxShell.exe",
        r"C:\Program Files (x86)\NxShell\NxShell.exe",
        r"%LOCALAPPDATA%\Programs\NxShell\NxShell.exe",
        r"%USERPROFILE%\AppData\Local\NxShell\NxShell.exe",
    ])
}
#[cfg(not(target_os = "windows"))]
fn detect_nxshell() -> String {
    String::new()
}

#[cfg(target_os = "windows")]
fn detect_bitvise() -> String {
    check_paths(&[
        r"C:\Program Files\Bitvise SSH Client\BvSsh.exe",
        r"C:\Program Files (x86)\Bitvise SSH Client\BvSsh.exe",
    ])
}
#[cfg(not(target_os = "windows"))]
fn detect_bitvise() -> String {
    String::new()
}

#[cfg(target_os = "windows")]
fn detect_securecrt() -> String {
    check_paths(&[
        r"C:\Program Files\VanDyke Software\Clients\SecureCRT.exe",
        r"C:\Program Files\VanDyke Software\SecureCRT\SecureCRT.exe",
        r"C:\Program Files (x86)\VanDyke Software\SecureCRT\SecureCRT.exe",
    ])
}
#[cfg(not(target_os = "windows"))]
fn detect_securecrt() -> String {
    String::new()
}

// ── macOS ────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
fn app_installed(app_name: &str) -> bool {
    Path::new(&format!("/Applications/{}.app", app_name)).exists()
}
