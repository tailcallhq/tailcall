use colored::Colorize;
use serde_json::Value;
use which::which;

enum InstallationMethod {
  Npm,
  Brew,
  Direct,
}

const RELEASE_URL: &str = "https://api.github.com/repos/tailcallhq/tailcall/releases";

const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

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

async fn get_latest_version() -> Result<String, reqwest::Error> {
  let client = reqwest::Client::builder().user_agent(APP_USER_AGENT).build()?;
  let response = client.get(RELEASE_URL).send().await?;
  let json: Value = serde_json::from_str(&response.text().await?).unwrap();
  let latest_version = json[0]["tag_name"].as_str().unwrap().to_string();
  Ok(latest_version)
}

pub async fn check_for_update() {
  let current_version: &str = match option_env!("APP_VERSION") {
    Some(version) => version,
    _ => return,
  };

  let latest_version = get_latest_version().await.unwrap();

  let latest_version = match semver::Version::parse(&latest_version.replace('v', "")) {
    Ok(version) => version,
    Err(_) => {
      return;
    }
  };

  let current_version = match semver::Version::parse(&current_version.replace('v', "")) {
    Ok(version) => version,
    Err(_) => {
      return;
    }
  };

  let needs_update = latest_version > current_version;

  if needs_update {
    let github_release_url = format!(
      "https://github.com/tailcallhq/tailcall/releases/tag/v{}",
      latest_version
    );

    log::warn!(
      "{}",
      format!(
        "A new release of tailcall is available: {} -> {}",
        current_version.to_string().cyan(),
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
