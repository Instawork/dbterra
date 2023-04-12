use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    pub projects: HashMap<String, Project>,
    pub environments: HashMap<String, Environment>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub jobs: HashMap<String, Job>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Job {
    pub name: Option<String>,
    pub environment: String,
    pub target: String,
    pub timeout: Option<i64>,
    pub threads: Option<i64>,
    pub ci: Option<CI>,
    pub schedule: Option<Schedule>,
    pub steps: Vec<String>,
    pub generate_docs: Option<bool>,
    pub defer_to_job_id: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CI {
    pub run_on_pr: Option<bool>,
    pub custom_branch_only: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schedule {
    pub cron: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Environment {
    pub id: i64,
}
