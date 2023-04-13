use colored::*;
use serde_json::Value;
use treediff::tools::{ChangeType, Recorder};
use treediff::value::Key;

#[derive(Debug, Clone)]
pub struct Diff {
    _local: Value,
    _remote: Value,
    changes: Vec<Change>,
}

#[derive(Debug, Clone)]
pub enum Change {
    Added(String, String),
    Removed(String, String),
    Unchanged(String, String),
    Modified(String, String, String),
}

impl Diff {
    pub fn from(v1: Value, v2: Value) -> Self {
        let changes = Diff::diff(&v1, &v2);
        Self {
            _local: v1,
            _remote: v2,
            changes,
        }
    }

    pub fn from_new(v1: Value, v2: Value) -> Self {
        let changes = Diff::new_diff(&v1, &v2);
        Self {
            _local: v1,
            _remote: v2,
            changes,
        }
    }

    fn diff(v1: &Value, v2: &Value) -> Vec<Change> {
        let mut d = Recorder::default();
        treediff::diff(v1, v2, &mut d);
        let changes = d
            .calls
            .iter()
            .map(|c| match c {
                ChangeType::Added(k, v) => Change::Added(Diff::friendly_key(k), v.to_string()),
                ChangeType::Removed(k, v) => Change::Removed(Diff::friendly_key(k), v.to_string()),
                ChangeType::Unchanged(k, v) => {
                    Change::Unchanged(Diff::friendly_key(k), v.to_string())
                }
                ChangeType::Modified(k, old, new) => {
                    let old = old.to_string();
                    if old == "null" {
                        Change::Added(Diff::friendly_key(k), new.to_string())
                    } else {
                        Change::Modified(Diff::friendly_key(k), old, new.to_string())
                    }
                }
            })
            .collect();
        changes
    }

    fn new_diff(v1: &Value, v2: &Value) -> Vec<Change> {
        Diff::diff(v1, v2)
            .into_iter()
            .map(|c| match c {
                Change::Modified(k, _, new) => Change::Added(k, new),
                _ => c,
            })
            .collect()
    }

    fn friendly_key(keys: &[Key]) -> String {
        return keys
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(".");
    }

    pub fn has_changes(&self) -> bool {
        !self
            .changes
            .iter()
            .all(|f| matches!(*f, Change::Unchanged(_, _)))
    }

    pub fn pretty_print(&self, padding: &str) {
        for c in &self.changes {
            match c {
                Change::Added(k, v) => {
                    // TODO: Cleanup but for now we only support cron so no need to show this change
                    if k == "schedule.cron" || k.contains("schedule.time") {
                        continue;
                    }
                    println!("{}{} {} {}", padding, "+".green(), k.green(), v.green());
                }
                Change::Removed(k, v) => {
                    println!("{}{} {} {}", padding, "-".red(), k.red(), v.red());
                }
                Change::Unchanged(_, _) => {
                    // DO NOTHING
                }
                Change::Modified(k, old, new) => {
                    // TODO: Cleanup but for now we only support cron so no need to show this change
                    if k == "schedule.cron" || k.contains("schedule.time") {
                        continue;
                    }
                    println!(
                        "{}{} {} {} -> {}",
                        padding,
                        "~".yellow(),
                        k.yellow(),
                        old.to_string().yellow(),
                        new.to_string().yellow()
                    );
                }
            }
        }
    }
}
