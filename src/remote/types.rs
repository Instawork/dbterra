use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response<T> {
    pub data: Option<T>,
    pub status: Status,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Job {
    pub id: Option<i64>,
    pub account_id: i64,
    pub project_id: i64,
    pub environment_id: i64,
    pub name: String,
    pub dbt_version: Option<String>, // defaults to environment in dbt cloud
    pub triggers: Triggers,
    pub execute_steps: Vec<String>,
    pub settings: Settings,
    pub state: i64,
    pub generate_docs: bool,
    pub deferring_job_definition_id: Option<i64>,
    pub schedule: Schedule,
    pub execution: Execution,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Triggers {
    pub github_webhook: bool,
    pub git_provider_webhook: bool,
    pub schedule: bool,
    pub custom_branch_only: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub threads: i64,
    pub target_name: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            threads: 4,
            target_name: "production".to_string(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schedule {
    pub cron: String,
    pub date: Date,
    pub time: Time,
}

impl Schedule {
    /// These defaults seem odd but it's what dbt Cloud uses when `custom_cron` is set
    pub fn cron(schedule: &str) -> Self {
        Schedule {
            cron: schedule.to_string(),
            date: Date {
                type_field: "custom_cron".to_string(),
                cron: Some(schedule.to_string()),
                days: None,
            },
            time: Time {
                type_field: "every_hour".to_string(),
                interval: Some(1),
                hours: None,
            },
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Date {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub days: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Time {
    #[serde(rename = "type")]
    pub type_field: String,
    pub interval: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hours: Option<Vec<i64>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Execution {
    pub timeout_seconds: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pub code: i64,
    pub is_success: bool,
    pub user_message: String,
    pub developer_message: String,
}
