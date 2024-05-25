use std::env;

use tailcall_version::VERSION;

const LONG_ENV_FILTER_VAR_NAME: &str = "TAILCALL_TRACKER";
const SHORT_ENV_FILTER_VAR_NAME: &str = "TC_TRACKER";

/// Checks if tracking is enabled
pub fn check_tracking() -> bool {
    let is_prod = !VERSION.is_dev();
    let usage_enabled = env::var(LONG_ENV_FILTER_VAR_NAME)
        .or(env::var(SHORT_ENV_FILTER_VAR_NAME))
        .map(|v| !v.eq_ignore_ascii_case("false"))
        .ok();
    check_tracking_inner(is_prod, usage_enabled)
}

fn check_tracking_inner(is_prod_build: bool, tracking_enabled: Option<bool>) -> bool {
    if let Some(usage_enabled) = tracking_enabled {
        usage_enabled
    } else {
        is_prod_build
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn usage_enabled_true() {
        assert!(check_tracking_inner(true, Some(true)));
        assert!(check_tracking_inner(false, Some(true)));
    }

    #[test]
    fn usage_enabled_false() {
        assert!(!check_tracking_inner(true, Some(false)));
        assert!(!check_tracking_inner(false, Some(false)));
    }

    #[test]
    fn usage_enabled_none_is_prod_true() {
        assert!(check_tracking_inner(true, None));
    }

    #[test]
    fn usage_enabled_none_is_prod_false() {
        assert!(!check_tracking_inner(false, None));
    }
}
