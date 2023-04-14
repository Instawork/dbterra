// API Docs
// https://docs.getdbt.com/dbt-cloud/api-v2

use std::error::Error;

use reqwest::blocking::{Client, RequestBuilder};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use reqwest::Method;
use serde::Serialize;

use crate::config::Config;
use crate::remote::types::{Job, Response};

pub struct DbtCloudClient<'a> {
    pub client: Client,
    pub config: &'a Config,
}

impl<'a> DbtCloudClient<'a> {
    pub fn new(config: &'a Config) -> Self {
        let client = reqwest::blocking::Client::builder().build().unwrap();
        Self { client, config }
    }

    fn construct_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let token_value = HeaderValue::from_str(&format!("Token {}", &self.config.token)).unwrap();
        headers.insert(USER_AGENT, HeaderValue::from_static("dbterra/0.1"));
        headers.insert(AUTHORIZATION, token_value);
        headers
    }

    fn request<T>(&self, method: Method, url: &str, body: Option<T>) -> RequestBuilder
    where
        T: Serialize,
    {
        let mut builder = self
            .client
            .request(method, url)
            .headers(self.construct_headers());

        if let Some(v) = body {
            builder = builder.json(&v);
        }
        builder
    }

    pub fn get_jobs(&self) -> Result<Response<Vec<Job>>, Box<dyn Error>> {
        let url = format!(
            "https://cloud.getdbt.com/api/v2/accounts/{}/jobs/",
            self.config.account_id,
        );

        let response = self.request::<Job>(Method::GET, &url, None).send()?;
        let dbt_response = response.json()?;
        Ok(dbt_response)
    }

    pub fn get_jobs_for_project(
        &self,
        project_id: i64,
    ) -> Result<Response<Vec<Job>>, Box<dyn Error>> {
        let dbt_response = self.get_jobs()?;
        let filtered_response = Response {
            status: dbt_response.status,
            data: Some(
                dbt_response
                    .data
                    .expect("error gettings jobs, check `status`")
                    .into_iter()
                    .filter(|j| j.project_id == project_id)
                    .collect(),
            ),
        };
        Ok(filtered_response)
    }

    pub fn create_job(&self, job: &Job) -> Result<Response<Job>, Box<dyn Error>> {
        let url = format!(
            "https://cloud.getdbt.com/api/v2/accounts/{}/jobs/",
            self.config.account_id,
        );
        let response = self.request(Method::POST, &url, Some(job)).send()?;
        let dbt_response = response.json()?;
        Ok(dbt_response)
    }

    pub fn update_job(&self, job: &Job) -> Result<Response<Job>, Box<dyn Error>> {
        let url = format!(
            "https://cloud.getdbt.com/api/v2/accounts/{}/jobs/{}/",
            self.config.account_id,
            job.id.expect("id is required to update a job"),
        );
        let response = self.request(Method::POST, &url, Some(job)).send()?;
        let dbt_response = response.json()?;
        Ok(dbt_response)
    }
}
