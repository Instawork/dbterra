use convert_case::{Case, Casing};
use std::collections::HashMap;

use crate::config::Config;
use crate::diff::Diff;
use crate::local::{Environment, Job, Schedule as LocalSchedule};
use crate::remote::{Execution, Schedule, Settings, Triggers};
use crate::RemoteJob;

/// Used to handle cases like `ml_feautres -> ML Features`
fn captialize_short_words(w: &str) -> String {
    let words: Vec<_> = w
        .split_ascii_whitespace()
        .map(|word| {
            if word.len() <= 2 {
                word.to_ascii_uppercase()
            } else {
                word.to_string()
            }
        })
        .collect();
    words.join(" ")
}

impl RemoteJob {
    pub fn from_local_job(
        key: &str,
        job: Job,
        config: &Config,
        environments: &HashMap<String, Environment>,
    ) -> Self {
        let environment = environments
            .get(&job.environment)
            .unwrap_or_else(|| panic!("no environment declared for: {}", &job.environment));
        let has_schedule = job.schedule.is_some();
        let schedule = job.schedule.unwrap_or(LocalSchedule {
            cron: "0/10 * * * *".to_string(),
        });
        let ci = job.ci.unwrap_or_default();
        let name = job
            .name
            .unwrap_or_else(|| captialize_short_words(&key.to_case(Case::Title)));
        RemoteJob {
            id: None,
            account_id: config.account_id,
            project_id: config.project_id.expect("missing project_id for local job"),
            environment_id: environment.id,
            name,
            dbt_version: None,
            triggers: Triggers {
                github_webhook: ci.run_on_pr.unwrap_or_default(),
                git_provider_webhook: false,
                schedule: has_schedule,
                custom_branch_only: ci.custom_branch_only,
            },
            execute_steps: job.steps,
            execution: Execution {
                timeout_seconds: job.timeout.unwrap_or(0),
            },
            settings: Settings {
                threads: job.threads.unwrap_or(4),
                target_name: job.target,
            },
            state: 1, // TODO: computed value, add constants
            generate_docs: job.generate_docs.unwrap_or(false),
            schedule: Schedule::cron(&schedule.cron),
            deferring_job_definition_id: job.defer_to_job_id,
            deferring_environment_id: job.defer_to_env_id,
        }
    }

    pub fn merge(&self, existing: &RemoteJob) -> Self {
        let mut s = self.clone();

        // Always set ID to existing one since that won't change
        s.id = existing.id;

        // If we aren't set to schedule, use the existing values for schedule no matter what
        if !s.triggers.schedule {
            s.schedule = existing.schedule.clone();
        }

        s
    }

    pub fn diff<'a>(&'a self, job: &'a RemoteJob) -> Diff {
        let v1 = serde_json::to_value(self).unwrap();
        let v2 = serde_json::to_value(job).unwrap();
        Diff::from(v1, v2)
    }

    pub fn new_diff<'a>(&'a self, job: &'a RemoteJob) -> Diff {
        let v1 = serde_json::to_value(self).unwrap();
        let v2 = serde_json::to_value(job).unwrap();
        Diff::from_new(v1, v2)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        config::Config,
        local::{Environment, Job as LocalJob},
        remote::{Date, Execution, Job as RemoteJob, Schedule, Settings, Time, Triggers},
    };

    #[test]
    fn defaults() {
        let config = Config {
            account_id: 123,
            token: "abc123".to_string(),
            project_id: Some(456),
            base_url: "https://cloud.getdbt.com".to_string(),
        };
        let mut environments = HashMap::new();
        environments.insert("test".to_string(), Environment { id: 789 });
        let local_job = LocalJob {
            steps: vec!["dbt run".to_string()],
            environment: "test".to_string(),
            target: "production".to_string(),
            name: None,
            timeout: None,
            threads: None,
            ci: None,
            schedule: None,
            generate_docs: None,
            defer_to_job_id: None,
            defer_to_env_id: None,
        };
        let expected_remote = RemoteJob {
            id: None,
            account_id: config.account_id,
            project_id: config.project_id.unwrap(),
            environment_id: environments.get("test").unwrap().id,
            name: "Test".to_string(), // converted from key
            dbt_version: None,
            triggers: Triggers {
                github_webhook: false,
                git_provider_webhook: false,
                schedule: false,
                custom_branch_only: None,
            },
            execute_steps: vec!["dbt run".to_string()],
            settings: Settings {
                threads: 4,
                target_name: "production".to_string(),
            },
            state: 1,
            generate_docs: false,
            deferring_job_definition_id: None,
            deferring_environment_id: None,
            schedule: Schedule {
                cron: "0/10 * * * *".to_string(),
                date: Date {
                    type_field: "custom_cron".to_string(),
                    cron: Some("0/10 * * * *".to_string()),
                    days: None,
                },
                time: Time {
                    type_field: "every_hour".to_string(),
                    interval: Some(1),
                    hours: None,
                },
            },
            execution: Execution { timeout_seconds: 0 },
        };
        assert_eq!(
            RemoteJob::from_local_job("test", local_job, &config, &environments),
            expected_remote
        );
    }

    #[test]
    fn key() {
        let config = Config {
            account_id: 123,
            token: "abc123".to_string(),
            project_id: Some(456),
            base_url: "https://cloud.getdbt.com".to_string(),
        };
        let mut environments = HashMap::new();
        environments.insert("test".to_string(), Environment { id: 789 });
        let local_job = LocalJob {
            steps: vec!["dbt run".to_string()],
            environment: "test".to_string(),
            target: "production".to_string(),
            name: None,
            timeout: None,
            threads: None,
            ci: None,
            schedule: None,
            generate_docs: None,
            defer_to_job_id: None,
            defer_to_env_id: None,
        };
        let converted_job = RemoteJob::from_local_job(
            "test_some_snake_case_thing",
            local_job,
            &config,
            &environments,
        );
        assert_eq!(&converted_job.name, "Test Some Snake Case Thing");
    }

    #[test]
    fn key_is_overridden() {
        let config = Config {
            account_id: 123,
            token: "abc123".to_string(),
            project_id: Some(456),
            base_url: "https://cloud.getdbt.com".to_string(),
        };
        let mut environments = HashMap::new();
        environments.insert("test".to_string(), Environment { id: 789 });
        let local_job = LocalJob {
            steps: vec!["dbt run".to_string()],
            environment: "test".to_string(),
            target: "production".to_string(),
            name: Some("My Test Job".to_string()),
            timeout: None,
            threads: None,
            ci: None,
            schedule: None,
            generate_docs: None,
            defer_to_job_id: None,
            defer_to_env_id: None,
        };
        let converted_job = RemoteJob::from_local_job(
            "test_some_snake_case_thing",
            local_job,
            &config,
            &environments,
        );
        assert_eq!(&converted_job.name, "My Test Job");
    }
}
