use std::collections::HashSet;
use std::process::Output;

use chrono::{DateTime, Utc};
use machineid_rs::{Encryption, HWIDComponent, IdBuilder};
use sysinfo::System;
use tokio::process::Command;
use tokio::time::Duration;

use super::Result;
use crate::can_track::can_track;
use crate::collect::{ga, posthog, Collect};
use crate::{Event, EventKind};

const GA_API_SECRET: &str = match option_env!("GA_API_SECRET") {
    Some(val) => val,
    None => "dev",
};
const GA_MEASUREMENT_ID: &str = match option_env!("GA_MEASUREMENT_ID") {
    Some(val) => val,
    None => "dev",
};
const POSTHOG_API_SECRET: &str = match option_env!("POSTHOG_API_SECRET") {
    Some(val) => val,
    None => "dev",
};

const PARAPHRASE: &str = "tc_key";

const DEFAULT_CLIENT_ID: &str = "<anonymous>";

pub struct Tracker {
    collectors: Vec<Box<dyn Collect>>,
    can_track: bool,
    start_time: DateTime<Utc>,
}

impl Default for Tracker {
    fn default() -> Self {
        let ga_tracker = Box::new(ga::Tracker::new(
            GA_API_SECRET.to_string(),
            GA_MEASUREMENT_ID.to_string(),
        ));
        let posthog_tracker = Box::new(posthog::Tracker::new(POSTHOG_API_SECRET));
        let start_time = Utc::now();
        let can_track = can_track();
        Self {
            collectors: vec![ga_tracker, posthog_tracker],
            can_track,
            start_time,
        }
    }
}

impl Tracker {
    pub async fn init_ping(&'static self, duration: Duration) {
        let mut interval = tokio::time::interval(duration);
        tokio::task::spawn(async move {
            loop {
                interval.tick().await;
                let _ = self.dispatch(EventKind::Ping).await;
            }
        });
    }

    pub async fn dispatch(&'static self, event_kind: EventKind) -> Result<()> {
        if self.can_track {
            // Create a new event
            let event = Event {
                event_name: event_kind.name(),
                start_time: self.start_time,
                cores: cores(),
                client_id: client_id(),
                os_name: os_name(),
                up_time: up_time(self.start_time),
                args: args(),
                path: path(),
                cwd: cwd(),
                user: user(),
                version: version(),
                email: email().await.into_iter().collect(),
            };

            // Dispatch the event to all collectors
            for collector in &self.collectors {
                collector.collect(event.clone()).await?;
            }

            tracing::debug!("Event dispatched: {:?}", event);
        }

        Ok(())
    }
}

// Get the email address
async fn email() -> HashSet<String> {
    fn parse(output: Output) -> Option<String> {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }

        return None;
    }

    // From Git
    async fn git() -> Option<String> {
        let output = Command::new("git")
            .args(&["config", "--global", "user.email"])
            .output()
            .await
            .ok()?;

        parse(output)
    }

    // From SSH Keys
    async fn ssh() -> Option<HashSet<String>> {
        let home_dir = std::env::var("HOME").ok()?;
        let ssh_dir = format!("{}/.ssh", home_dir);

        // List all files in .ssh directory using ls command
        let ls_output = Command::new("ls").arg(&ssh_dir).output().await.ok()?;

        let files = parse(ls_output)?;
        let mut email_ids = HashSet::default();

        // Process each file
        for file in files.lines() {
            // if it's pub ssh file, read it to collect email id.
            if file.ends_with(".pub") {
                let key_path = format!("{}/{}", ssh_dir, file);
                if let Ok(output) = Command::new("cat").arg(&key_path).output().await {
                    if let Some(pub_key) = parse(output) {
                        let parts: Vec<&str> = pub_key.trim().split_whitespace().collect();

                        // SSH public keys typically have at least three parts
                        if parts.len() >= 3 {
                            // The comment part is usually the third element
                            let comment = parts[2];

                            // Validate the email format using a simple check
                            if comment.contains('@') && comment.contains('.') {
                                let email_id = comment.to_string();
                                email_ids.insert(email_id);
                            }
                        }
                    }
                }
            }
        }

        if email_ids.is_empty() {
            None
        } else {
            Some(email_ids)
        }
    }

    let git_email = git().await;
    let ssh_emails = ssh().await;

    let mut email_ids = HashSet::new();

    if let Some(email) = git_email {
        if !email.trim().is_empty() {
            email_ids.insert(email.trim().to_string());
        }
    }

    if let Some(emails) = ssh_emails {
        for email in emails {
            if !email.trim().is_empty() {
                email_ids.insert(email.trim().to_string());
            }
        }
    }

    email_ids
}

// Generates a random client ID
fn client_id() -> String {
    let mut builder = IdBuilder::new(Encryption::SHA256);
    builder
        .add_component(HWIDComponent::SystemID)
        .add_component(HWIDComponent::CPUCores);
    builder
        .build(PARAPHRASE)
        .unwrap_or(DEFAULT_CLIENT_ID.to_string())
}

// Get the number of CPU cores
fn cores() -> usize {
    let sys = System::new_all();
    sys.physical_core_count().unwrap_or(0)
}

// Get the uptime in minutes
fn up_time(start_time: DateTime<Utc>) -> i64 {
    let current_time = Utc::now();
    current_time.signed_duration_since(start_time).num_minutes()
}

fn version() -> String {
    tailcall_version::VERSION.as_str().to_string()
}

fn user() -> String {
    whoami::username()
}

fn cwd() -> Option<String> {
    std::env::current_dir()
        .ok()
        .and_then(|path| path.to_str().map(|s| s.to_string()))
}

fn path() -> Option<String> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.to_str().map(|s| s.to_string()))
}

fn args() -> Vec<String> {
    std::env::args().skip(1).collect()
}

fn os_name() -> String {
    System::long_os_version().unwrap_or("Unknown".to_string())
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use std::process::Command;

    use super::*;

    lazy_static! {
        static ref TRACKER: Tracker = Tracker::default();
    }

    #[tokio::test]
    async fn test_tracker() {
        if let Err(e) = TRACKER
            .dispatch(EventKind::Command("ping".to_string()))
            .await
        {
            panic!("Tracker dispatch error: {:?}", e);
        }
    }
}
