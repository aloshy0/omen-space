use clap::Subcommand;
use anyhow::Result;
use crossterm::style::Stylize;
use crate::dbus_proxy::PlatformProxy;

#[derive(Subcommand, Debug, Clone)]
pub enum OverlayCommand {
    /// Toggle HUD overlay visibility (Shift+F2)
    Toggle,
    /// Launch overlay in background daemon mode
    Daemon,
}

fn ensure_overlay_running() {
    let is_running = std::process::Command::new("pgrep")
        .arg("-x")
        .arg("omen-overlay")
        .output()
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false);

    if !is_running {
        let spawned = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|dir| dir.join("omen-overlay")))
            .and_then(|overlay_path| {
                if overlay_path.exists() {
                    std::process::Command::new(overlay_path)
                        .arg("--daemon")
                        .stdin(std::process::Stdio::null())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn()
                        .ok()
                } else {
                    None
                }
            });

        if spawned.is_none() {
            let _ = std::process::Command::new("omen-overlay")
                .arg("--daemon")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .or_else(|_| {
                    std::process::Command::new("/usr/bin/omen-overlay")
                        .arg("--daemon")
                        .stdin(std::process::Stdio::null())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn()
                });
        }
        // Brief pause for the process to bind to DBus if newly spawned
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

pub async fn handle(cmd: &OverlayCommand, conn: &zbus::Connection) -> Result<()> {
    match cmd {
        OverlayCommand::Toggle => {
            ensure_overlay_running();
            let proxy = PlatformProxy::new(conn).await?;
            let res = proxy.toggle_overlay().await?;
            if res == "OK" {
                println!("{} Overlay toggle signal sent.", "✓".green().bold());
            } else {
                println!("{} Failed to toggle overlay: {}", "✗".red().bold(), res);
            }
        }
        OverlayCommand::Daemon => {
            println!("{} Starting OMEN Overlay in daemon mode...", "🎮".cyan());
            ensure_overlay_running();
        }
    }
    Ok(())
}
