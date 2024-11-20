use std::path::PathBuf;

use chrono::Utc;
use colored::Colorize;
use ctrlc::set_handler;
use dirs::cache_dir;
use tailcall_version::VERSION;
use update_informer::{registry, Check};
use which::which;

enum InstallationMethod {
    Npm,
    Npx,
    Brew,
    Direct,
}

fn get_state_file_path() -> Option<PathBuf> {
    let mut config_path = cache_dir()?; // Get the appropriate config directory
    config_path.push("tailcall"); // Your application name
    config_path.push(".state");
    Some(config_path)
}

fn get_installation_method() -> InstallationMethod {
    if std::env::var("npm_execpath").is_ok() {
        return InstallationMethod::Npx;
    }

    if let Ok(output) = std::process::Command::new("npm")
        .arg("ls")
        .arg("--global")
        .output()
    {
        if String::from_utf8_lossy(&output.stdout).contains("@tailcallhq/tailcall") {
            return InstallationMethod::Npm;
        }
    }

    if let Ok(result) = which("tailcall") {
        if result.to_str().map_or(false, |s| s.contains("homebrew")) {
            return InstallationMethod::Brew;
        }
    }

    InstallationMethod::Direct
}

pub async fn check_for_update() {
    tokio::task::spawn_blocking(move || {
        if VERSION.is_dev() {
            // skip validation if it's not a release
            return;
        }

        let state_file_path = get_state_file_path().unwrap();
        let epoch_time_now = Utc::now().timestamp();
        let show_update_message = match std::fs::read_to_string(&state_file_path) {
            Ok(data) => match data.trim().parse::<i64>() {
                Ok(epoch_time) => (epoch_time_now - epoch_time) > 24 * 60 * 60,
                Err(_) => true,
            },
            Err(_) => true,
        };

        if !show_update_message {
            // if it's been less than 24 hours since the last check, don't show or look for the update.
            return;
        }

        let name: &str = "tailcallhq/tailcall";

        let informer = update_informer::new(registry::GitHub, name, VERSION.as_str());

        if let Some(latest_version) = informer.check_version().ok().flatten() {
            let github_release_url =
                format!("https://github.com/{name}/releases/tag/{latest_version}",);
            let installation_method = get_installation_method();

            let path_exists = std::fs::metadata(state_file_path.clone()).is_ok();
            if !path_exists {
                std::fs::create_dir_all(state_file_path.parent().unwrap()).unwrap();
            }

            if let Ok(_) = std::fs::write(state_file_path, epoch_time_now.to_string()) {}

            if true {
                set_handler(move || {
                    tracing::warn!(
                        "{}",
                        format!(
                            "A new release of tailcall is available: {} {} {}",
                            VERSION.as_str().cyan(),
                            "➜".white(),
                            latest_version.to_string().cyan()
                        )
                        .yellow()
                    );
                    std::process::exit(exitcode::CONFIG);
                })
                .expect("Error setting Ctrl-C handler");
                return;
            }

            match installation_method {
                InstallationMethod::Npx => tracing::warn!(
                    "You're running an outdated version, run: npx @tailcallhq/tailcall@latest"
                ),
                InstallationMethod::Npm => {
                    tracing::warn!("To upgrade, run: npm update -g @tailcallhq/tailcall")
                }
                InstallationMethod::Brew => {
                    tracing::warn!("To upgrade, run: brew upgrade tailcall")
                }
                InstallationMethod::Direct => {
                    tracing::warn!("Please update by downloading the latest release from GitHub")
                }
            }
            tracing::warn!("{}", github_release_url.yellow());
        }
    });
}
