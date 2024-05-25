const DEFAULT_VERSION: &str = "0.1.0-dev";

pub struct Version {
    version: &'static str,
}

impl Version {
    pub const fn new(version: &'static str) -> Self {
        Version { version }
    }

    pub const fn as_str(&self) -> &'static str {
        self.version
    }

    pub fn is_dev(&self) -> bool {
        self.version.contains("dev")
    }
}

pub const VERSION: Version = match option_env!("APP_VERSION") {
    Some(version) => Version::new(version),
    None => Version::new(DEFAULT_VERSION),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_version() {
        assert_eq!(VERSION.as_str(), DEFAULT_VERSION);
        assert!(VERSION.is_dev());
    }

    #[test]
    fn test_custom_version() {
        const CUSTOM_VERSION: Version = Version::new("1.0.0-release");
        assert_eq!(CUSTOM_VERSION.as_str(), "1.0.0-release");
        assert!(!CUSTOM_VERSION.is_dev());
    }
}
