use std::time::Duration;

use colored::Colorize;
use update_informer::{registry, Check};
use which::which;

enum InstallationMethod {
    Npm,
    Npx,
    Brew,
    Direct,
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
    tokio::task::spawn_blocking(|| {
        let name: &str = "tailcallhq/tailcall";
        let Some(current_version) = option_env!("APP_VERSION") else {
            return;
        };

        let informer =
            update_informer::new(registry::GitHub, name, current_version).interval(Duration::ZERO);

        if let Some(latest_version) = informer.check_version().ok().flatten() {
            let github_release_url = format!(
                "https://github.com/tailcallhq/tailcall/releases/tag/{}",
                latest_version
            );
            let installation_method = get_installation_method();
            log::warn!(
                "{}",
                format!(
                    "A new release of tailcall is available: {} {} {}",
                    current_version.to_string().cyan(),
                    "➜".white(),
                    latest_version.to_string().cyan()
                )
                .yellow()
            );
            match installation_method {
                InstallationMethod::Npx => log::warn!(
                    "{}",
                    "You're running an outdated version, run: npx @tailcallhq/tailcall@latest"
                ),
                InstallationMethod::Npm => {
                    log::warn!("{}", "To upgrade, run: npm update -g @tailcallhq/tailcall")
                }
                InstallationMethod::Brew => {
                    log::warn!("{}", "To upgrade, run: brew upgrade tailcall")
                }
                InstallationMethod::Direct => log::warn!(
                    "{}",
                    "Please update by downloading the latest release from GitHub"
                ),
            }
            log::warn!("{}", github_release_url.to_string().yellow());
        }
    });
}
