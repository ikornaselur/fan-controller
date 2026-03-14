use anyhow::{bail, Result};
use serde::Deserialize;
use std::fs;

const REPO: &str = "ikornaselur/fan-controller";
const BINARY_NAME: &str = "fan-controller";

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub fn update() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: {}", current_version);

    println!("Fetching latest release...");
    let url = format!("https://api.github.com/repos/{}/releases/latest", REPO);
    let release: Release = ureq::get(&url)
        .header("User-Agent", "fan-controller")
        .call()?
        .body_mut()
        .read_json()?;

    let latest = release.tag_name.trim_start_matches('v');
    println!("Latest version: {}", latest);

    if latest == current_version {
        println!("Already up to date.");
        return Ok(());
    }

    // Find the binary asset
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == BINARY_NAME)
        .ok_or_else(|| anyhow::anyhow!("No '{}' asset found in release", BINARY_NAME))?;

    println!("Downloading {}...", asset.browser_download_url);
    let bytes = ureq::get(&asset.browser_download_url)
        .header("User-Agent", "fan-controller")
        .call()?
        .body_mut()
        .read_to_vec()?;

    // Replace the current binary
    let current_exe = std::env::current_exe()?;
    let tmp_path = current_exe.with_extension("tmp");

    fs::write(&tmp_path, &bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o755))?;
    }

    fs::rename(&tmp_path, &current_exe)?;

    println!("Updated: {} -> {}", current_version, latest);

    // Check if the service is installed and offer to restart
    if fs::metadata("/etc/systemd/system/fan-controller.service").is_ok() {
        println!("Service is installed. Restarting...");
        let status = std::process::Command::new("systemctl")
            .args(["restart", "fan-controller"])
            .status()?;
        if !status.success() {
            bail!("Failed to restart service");
        }
        println!("Service restarted.");
    }

    Ok(())
}
