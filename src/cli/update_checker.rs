use colored::Colorize;
use ctrlc::set_handler;
use tailcall_version::VERSION;
use update_informer::{registry, Check, Version};
use which::which;

#[derive(Default)]
enum InstallationMethod {
    Npm,
    Npx,
    Brew,
    #[default]
    Direct,
}

impl InstallationMethod {
    /// figure out the installation method is used by user.
    pub fn get_installation_method() -> Self {
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

        InstallationMethod::default()
    }

    fn format_upgrade_message(&self, command: &str) -> String {
        format!("{} {}", "Please run:".white(), command.yellow())
    }

    /// displays the message to upgrade the tailcall depending on the
    /// installation method used.
    pub fn display_message(&self) -> String {
        match self {
            InstallationMethod::Npx => {
                self.format_upgrade_message("npx @tailcallhq/tailcall@latest")
            }
            InstallationMethod::Npm => {
                self.format_upgrade_message("npm update -g @tailcallhq/tailcall")
            }
            InstallationMethod::Brew => self.format_upgrade_message("brew upgrade tailcall"),
            InstallationMethod::Direct => {
                "Please update by downloading the latest release from GitHub".to_string()
            }
        }
    }
}

fn show_update_message(name: &str, latest_version: Version) {
    let github_release_url = format!("https://github.com/{name}/releases/tag/{latest_version}",);
    tracing::warn!(
        "{} {} {} {}. {}. Release notes: {}",
        "A new release of tailcall is available:",
        VERSION.as_str().cyan(),
        "➜",
        latest_version.to_string().cyan(),
        InstallationMethod::get_installation_method().display_message(),
        github_release_url.yellow()
    );
}

pub async fn check_for_update() {
    if VERSION.is_dev() {
        // skip validation if it's not a release
        return;
    }

    let name: &str = "tailcallhq/tailcall";

    let informer = update_informer::new(registry::GitHub, name, VERSION.as_str());

    if let Some(latest_version) = informer.check_version().ok().flatten() {
        // schedules the update message to be shown when the user presses Ctrl+C on cli.
        let _ = set_handler(move || {
            show_update_message(name, latest_version.clone());
            std::process::exit(exitcode::OK);
        });
    }
}
