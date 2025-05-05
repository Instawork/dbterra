use std::env;

use crate::local::Root;

#[derive(Debug, Clone)]
pub struct Config {
    pub account_id: i64,
    pub project_id: Option<i64>,
    pub token: String,
    pub base_url: String,
}

const BASE_URL: &str = "DBT_CLOUD_BASE_URL";
const ACCOUNT_ENV: &str = "DBT_CLOUD_ACCOUNT_ID";
const TOKEN_ENV: &str = "DBT_CLOUD_TOKEN";

impl Config {
    pub fn build(yaml: &Root) -> Result<Config, &'static str> {
        let account_id = env::var(ACCOUNT_ENV).unwrap_or_else(|_| {
            if let Some(account) = &yaml.account {
                account.id.to_string()
            } else {
                panic!(
                    "{} must be set or declared via `account` in dbt_cloud.yml",
                    ACCOUNT_ENV
                )
            }
        });
        let token = env::var(TOKEN_ENV).unwrap_or_else(|_| panic!("{} must be set", TOKEN_ENV));
        let base_url = env::var(BASE_URL).unwrap_or_else(|_| "https://cloud.getdbt.com".to_string());
        Ok(Config {
            account_id: account_id.parse().expect("account_id must be a number"),
            project_id: None,
            token,
            base_url,
        })
    }
    pub fn with_project_id(&self, project_id: i64) -> Self {
        let clone = self.clone();
        Self {
            project_id: Some(project_id),
            ..clone
        }
    }
}
