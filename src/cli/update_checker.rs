use colored::Colorize;
use update_informer::{registry, Check};
use which::which;

enum InstallationMethod {
  Npm,
  Brew,
  Direct,
}

fn get_installation_method() -> InstallationMethod {
  let output = std::process::Command::new("npm").arg("ls").arg("--global").output();

  if let Ok(output) = output {
    let output_str = String::from_utf8_lossy(&output.stdout);
    if output_str.contains("@tailcallhq/tailcall") {
      return InstallationMethod::Npm;
    }
  }

  if let Ok(result) = which("tailcall") {
    if result.to_str().unwrap().contains("homebrew") {
      return InstallationMethod::Brew;
    }
  }

  InstallationMethod::Direct
}

pub async fn check_for_update() {
  let name: &str = "tailcallhq/tailcall";
  let current_version: &str = match option_env!("APP_VERSION") {
    Some(version) => version,
    _ => return,
  };

  let informer = update_informer::new(registry::GitHub, name, current_version);

  if let Some(latest_version) = informer.check_version().ok().flatten() {
    let github_release_url = format!("https://github.com/tailcallhq/tailcall/releases/tag/{}", latest_version);

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

    let installation_method = get_installation_method();
    match installation_method {
      InstallationMethod::Npm => log::warn!("{}", "To upgrade, run: npm update -g @tailcallhq/tailcall"),
      InstallationMethod::Brew => log::warn!("{}", "To upgrade, run: brew upgrade tailcall"),
      InstallationMethod::Direct => log::warn!("{}", "Please update by downloading the latest release from GitHub"),
    }
    log::warn!("{}", github_release_url.to_string().yellow());
  }
}
