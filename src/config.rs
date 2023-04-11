use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub account_id: i64,
    pub project_id: Option<i64>,
    pub token: String,
}

const ACCOUNT_ENV: &str = "DBT_CLOUD_ACCOUNT_ID";
const TOKEN_ENV: &str = "DBT_CLOUD_TOKEN";

impl Config {
    pub fn build() -> Result<Config, &'static str> {
        let account_id =
            env::var(ACCOUNT_ENV).unwrap_or_else(|_| panic!("{} must be set", ACCOUNT_ENV));
        let token = env::var(TOKEN_ENV).unwrap_or_else(|_| panic!("{} must be set", TOKEN_ENV));
        Ok(Config {
            account_id: account_id.parse().expect("account_id must be a number"),
            project_id: None,
            token,
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
