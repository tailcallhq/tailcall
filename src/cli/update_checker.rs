use std::path::PathBuf;

use chrono::Utc;
use colored::Colorize;
use ctrlc::set_handler;
use dirs::{cache_dir, config_dir};
use tailcall_version::VERSION;
use update_informer::{registry, Check};
use which::which;

enum InstallationMethod {
    Npm,
    Npx,
    Brew,
    Direct,
}

fn get_state_file_path() -> PathBuf {
    let mut config_path = cache_dir().unwrap_or_else(|| config_dir().unwrap_or_default());
    config_path.push(".tailcall");
    config_path
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

/// Checks if more than 24 hours have passed since the last update check.
fn should_show_update_message() -> bool {
    let state_file_path = get_state_file_path();
    let epoch_time_now = Utc::now().timestamp();
    let show_update_message = match std::fs::read_to_string(&state_file_path) {
        Ok(data) => match data.trim().parse::<i64>() {
            Ok(epoch_time) => (epoch_time_now - epoch_time) > 24 * 60 * 60,
            Err(_) => true,
        },
        Err(_) => true,
    };
    show_update_message
}

/// Updates the state file with the current timestamp.
fn update_version_check_time() {
    let state_file_path = get_state_file_path();
    let epoch_time_now = Utc::now().timestamp();
    let path_exists = std::fs::metadata(state_file_path.clone()).is_ok();
    if !path_exists {
        if let Some(parent) = state_file_path.parent() {
            // it's okay if it's fails to create the directory. We'll try again next time.
            let _ = std::fs::create_dir_all(parent).is_ok();
        } else {
            let mut state_file_path = state_file_path.clone();
            state_file_path.pop(); // remove the file name, so that we can create directories.
                                   // it's okay if it's fails to create the directory. We'll try again next time.
            let _ = std::fs::create_dir_all(state_file_path.clone()).is_ok();
        }
    }

    // it's okay if it's fails to write. We'll try again next time.
    let _ = std::fs::write(state_file_path, epoch_time_now.to_string()).is_ok();
}

pub async fn check_for_update() {
    tokio::task::spawn_blocking(move || {
        if VERSION.is_dev() {
            // skip validation if it's not a release
            return;
        }

        if !should_show_update_message() {
            // if it's been less than 24 hours since the last check, don't show or look for
            // the update.
            return;
        }

        let name: &str = "tailcallhq/tailcall";

        let informer = update_informer::new(registry::GitHub, name, VERSION.as_str());

        if let Some(latest_version) = informer.check_version().ok().flatten() {
            update_version_check_time();
            let _ = set_handler(move || {
                let github_release_url =
                    format!("https://github.com/{name}/releases/tag/{latest_version}",);
                let installation_method = get_installation_method();
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
                        tracing::warn!(
                            "Please update by downloading the latest release from GitHub"
                        )
                    }
                }
                tracing::warn!("{}", github_release_url.yellow());
                std::process::exit(exitcode::OK);
            });
        }
    });
}
