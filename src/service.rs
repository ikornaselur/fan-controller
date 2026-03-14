use anyhow::{bail, Result};
use std::fs;
use std::process::Command;

const SERVICE_NAME: &str = "fan-controller";
const SERVICE_PATH: &str = "/etc/systemd/system/fan-controller.service";
const INSTALL_DIR: &str = "/usr/local/bin";

fn systemctl(args: &[&str]) -> Result<()> {
    let status = Command::new("systemctl").args(args).status()?;
    if !status.success() {
        bail!("systemctl {} failed", args.join(" "));
    }
    Ok(())
}

/// Build the ExecStart line from the provided run args.
fn build_exec_start(args: &[String]) -> String {
    let binary = format!("{}/{}", INSTALL_DIR, SERVICE_NAME);
    if args.is_empty() {
        format!("{} run", binary)
    } else {
        format!("{} run {}", binary, args.join(" "))
    }
}

/// Build Environment= lines for the service file.
fn build_env_lines(mqtt_username: &Option<String>, mqtt_password: &Option<String>) -> String {
    let mut lines = Vec::new();
    if let Some(user) = mqtt_username {
        lines.push(format!("Environment=\"MQTT_USERNAME={}\"", user));
    }
    if let Some(pass) = mqtt_password {
        lines.push(format!("Environment=\"MQTT_PASSWORD={}\"", pass));
    }
    if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    }
}

pub fn install(
    run_args: &[String],
    mqtt_username: &Option<String>,
    mqtt_password: &Option<String>,
) -> Result<()> {
    // Copy binary to install dir
    let current_exe = std::env::current_exe()?;
    let dest = format!("{}/{}", INSTALL_DIR, SERVICE_NAME);
    println!("Copying binary to {}", dest);
    fs::copy(&current_exe, &dest)?;

    // Make sure it's executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&dest, fs::Permissions::from_mode(0o755))?;
    }

    let exec_start = build_exec_start(run_args);
    let env_lines = build_env_lines(mqtt_username, mqtt_password);

    let service = format!(
        "[Unit]\n\
         Description=Fan Controller\n\
         After=network.target\n\
         \n\
         [Service]\n\
         Type=simple\n\
         {env_lines}\
         ExecStart={exec_start}\n\
         KillSignal=SIGINT\n\
         Restart=on-failure\n\
         RestartSec=5\n\
         \n\
         [Install]\n\
         WantedBy=multi-user.target\n"
    );

    println!("Writing service file to {}", SERVICE_PATH);
    fs::write(SERVICE_PATH, service)?;

    println!("Enabling and starting service");
    systemctl(&["daemon-reload"])?;
    systemctl(&["enable", "--now", SERVICE_NAME])?;

    println!("Service installed and running.");
    Ok(())
}

pub fn uninstall() -> Result<()> {
    println!("Stopping service");
    // Don't fail if the service isn't running
    let _ = systemctl(&["stop", SERVICE_NAME]);

    println!("Disabling service");
    let _ = systemctl(&["disable", SERVICE_NAME]);

    if fs::metadata(SERVICE_PATH).is_ok() {
        println!("Removing service file");
        fs::remove_file(SERVICE_PATH)?;
        systemctl(&["daemon-reload"])?;
    }

    println!("Service uninstalled.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_start_no_args() {
        let result = build_exec_start(&[]);
        assert_eq!(result, "/usr/local/bin/fan-controller run");
    }

    #[test]
    fn exec_start_with_args() {
        let args = vec![
            "--target-temp".to_string(),
            "50".to_string(),
            "--kp".to_string(),
            "0.03".to_string(),
        ];
        let result = build_exec_start(&args);
        assert_eq!(
            result,
            "/usr/local/bin/fan-controller run --target-temp 50 --kp 0.03"
        );
    }

    #[test]
    fn env_lines_empty() {
        assert_eq!(build_env_lines(&None, &None), "");
    }

    #[test]
    fn env_lines_with_creds() {
        let result = build_env_lines(
            &Some("user".to_string()),
            &Some("pass".to_string()),
        );
        assert_eq!(
            result,
            "Environment=\"MQTT_USERNAME=user\"\nEnvironment=\"MQTT_PASSWORD=pass\"\n"
        );
    }
}
