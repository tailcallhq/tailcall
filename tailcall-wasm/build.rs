use std::path::Path;

fn main() {
    let version = std::env::var("APP_VERSION").unwrap_or_else(|_| "0.1.0".to_string());
    let cargo_toml_path = Path::new("Cargo.toml");
    let cargo_toml_content =
        std::fs::read_to_string(cargo_toml_path).expect("Unable to read Cargo.toml");

    let mut cargo_toml: toml::Value = cargo_toml_content
        .parse()
        .expect("Unable to parse Cargo.toml");

    if let Some(package) = cargo_toml.get_mut("package") {
        package["version"] = toml::Value::String(version);
    }

    std::fs::write(
        cargo_toml_path,
        toml::to_string_pretty(&cargo_toml).unwrap(),
    )
    .expect("Unable to write to Cargo.toml");
}
