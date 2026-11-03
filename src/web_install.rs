// izone/src/web_install.rs
//
// `izone web install-service` (Linux, systemd user service) and `izone web uninstall`,
// which removes everything `izone web` stores locally.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use colored::Colorize;

use crate::web_auth;

const UNIT_NAME: &str = "izone-web.service";

fn unit_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("systemd").join("user").join(UNIT_NAME))
}

fn systemctl(args: &[&str]) -> bool {
    Command::new("systemctl").arg("--user").args(args).status().map(|s| s.success()).unwrap_or(false)
}

/// The unit file contents for running `izone web` as a systemd user service.
pub fn unit_file(exe: &str, host: &str, port: u16) -> String {
    format!(
        "[Unit]
Description=iZone web UI
Documentation=https://github.com/rufusshrestha/izone
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart={exe} web --host {host} --port {port}
Restart=on-failure
RestartSec=5
# Optional: set the controller address here instead of ~/.config/izone/config.toml
#Environment=IZONE_IP=http://192.168.1.130
NoNewPrivileges=true
UMask=0077

[Install]
WantedBy=default.target
"
    )
}

pub fn install_service(host: &str, port: u16) -> Result<(), String> {
    if !cfg!(target_os = "linux") {
        return Err("install-service uses systemd and is only available on Linux. \
                    On macOS/Windows, run `izone web` directly or use Docker (see README)."
            .to_string());
    }
    let exe = std::env::current_exe().map_err(|e| format!("could not find the izone binary: {}", e))?;
    let exe = exe.canonicalize().unwrap_or(exe);
    let path = unit_path().ok_or("could not find the user config directory")?;
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| format!("could not create {}: {}", path.parent().unwrap().display(), e))?;
    fs::write(&path, unit_file(&exe.display().to_string(), host, port))
        .map_err(|e| format!("could not write {}: {}", path.display(), e))?;
    println!("Wrote {}", path.display());
    if !systemctl(&["daemon-reload"]) || !systemctl(&["enable", "--now", UNIT_NAME]) {
        return Err(format!(
            "the unit file was written, but systemctl failed. Try: systemctl --user enable --now {}",
            UNIT_NAME
        ));
    }
    println!("{} izone web is running as a user service on {}:{}", "✓".green(), host, port);
    println!("  Status: systemctl --user status {}", UNIT_NAME);
    println!("  Logs:   journalctl --user -u {} -f", UNIT_NAME);
    println!("To keep it running when you're logged out: loginctl enable-linger $USER");
    Ok(())
}

fn remove_dir_if_empty(dir: Option<PathBuf>, removed: &mut Vec<String>) {
    if let Some(d) = dir {
        if fs::read_dir(&d).map(|mut it| it.next().is_none()).unwrap_or(false) && fs::remove_dir(&d).is_ok() {
            removed.push(format!("empty folder {}", d.display()));
        }
    }
}

/// Remove everything `izone web` stored locally. Returns a list of what was removed.
pub fn uninstall() -> Result<Vec<String>, String> {
    let mut removed = Vec::new();

    // 1. systemd user service (Linux)
    if let Some(path) = unit_path().filter(|p| p.exists()) {
        systemctl(&["disable", "--now", UNIT_NAME]);
        fs::remove_file(&path).map_err(|e| format!("could not delete {}: {}", path.display(), e))?;
        systemctl(&["daemon-reload"]);
        removed.push(format!("systemd service {}", path.display()));
    }

    // 2. Changed web password (encrypted file) and its encryption key
    let cred = web_auth::credential_path()?;
    let had_cred = cred.exists();
    if web_auth::reset_password()? || had_cred {
        removed.push(format!("web password and its key ({})", web_auth::storage_description()));
    }

    // 3. Folders izone created, if now empty (config.toml is left alone)
    remove_dir_if_empty(dirs::data_local_dir().map(|p| p.join("izone")), &mut removed);
    remove_dir_if_empty(dirs::config_dir().map(|p| p.join("izone")), &mut removed);
    Ok(removed)
}

/// Files `izone web uninstall` deliberately leaves (they're not created by `izone web`).
pub fn kept() -> Vec<String> {
    let mut kept = Vec::new();
    for p in [
        dirs::config_dir().map(|p| p.join("izone").join("config.toml")),
        dirs::home_dir().map(|p| p.join(".izone.toml")),
    ]
    .into_iter()
    .flatten()
    {
        if p.exists() {
            kept.push(format!("{} (your controller address, used by the CLI too)", p.display()));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        kept.push(format!("{} (the izone program itself; delete it to remove izone completely)", exe.display()));
    }
    kept
}
