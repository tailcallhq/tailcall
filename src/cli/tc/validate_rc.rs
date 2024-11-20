use std::path::Path;

use super::helpers::{TAILCALL_RC, TAILCALL_RC_SCHEMA};
use crate::core::runtime::TargetRuntime;

pub async fn validate_rc_config_files(runtime: TargetRuntime, file_paths: &[String]) {
    // base config files.
    let tailcallrc = include_str!("../../../generated/.tailcallrc.graphql");
    let tailcallrc_json = include_str!("../../../generated/.tailcallrc.schema.json");

    // Define the config files to check with their base contents
    let rc_config_files = vec![
        (TAILCALL_RC, tailcallrc),
        (TAILCALL_RC_SCHEMA, tailcallrc_json),
    ];

    for path in file_paths {
        let parent_dir = match Path::new(path).parent() {
            Some(dir) => dir,
            None => continue,
        };

        let mut outdated_files = Vec::with_capacity(2);

        for (file_name, base_content) in &rc_config_files {
            let config_path = parent_dir.join(file_name);
            if config_path.exists() {
                if let Ok(content) = runtime.file.read(&config_path.to_string_lossy()).await {
                    if &content != base_content {
                        // file content not same.
                        outdated_files.push(file_name);
                    }
                } else {
                    // unable to read file.
                    outdated_files.push(file_name);
                }
            }
        }

        if !outdated_files.is_empty() {
            let message = if outdated_files.len() == 2 {
                format!(
                    "[{}, {}] is outdated, reinitialize using tailcall init.",
                    outdated_files[0], outdated_files[1]
                )
            } else {
                format!(
                    "[{}] is outdated, reinitialize using tailcall init.",
                    outdated_files[0]
                )
            };
            tracing::warn!(message);
            return;
        }
    }
}
