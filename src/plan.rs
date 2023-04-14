use std::collections::{HashMap, HashSet};

use colored::Colorize;

use crate::{
    config::Config,
    diff::Diff,
    local::Job as LocalJob,
    local::Root,
    remote::{DbtCloudClient, Job as RemoteJob},
};

pub struct Plan {
    projects: Vec<ProjectPlan>,
    // environments: Vec<EnvironmentPlan>, TODO: Implement for environments as well
}

impl Plan {
    pub fn from(yaml: Root, client: &DbtCloudClient, config: &Config) -> Self {
        let changes: Vec<_> = yaml
            .projects
            .into_iter()
            .map(|(k, project)| {
                // Fetch our remote jobs
                let remote_jobs = client
                    .get_jobs_for_project(project.id)
                    .expect("failed to get remote jobs");

                // Convert our local jobs to look like remote ones// Grab the local YAML jobs and the remote jobs for the project
                let local_config = config.with_project_id(project.id);
                let local_jobs: Vec<(String, LocalJob)> = project.jobs.into_iter().collect();
                let converted_local_jobs: Vec<_> = local_jobs
                    .into_iter()
                    .map(|(k, j)| {
                        RemoteJob::from_local_job(&k, j, &local_config, &yaml.environments)
                    })
                    .collect();

                // Figure out which are updates, creates, and deletes
                let job_types: HashMap<String, JobPlanType> = determine_job_plan_types_by_name(
                    converted_local_jobs,
                    remote_jobs.data.unwrap(),
                );

                // Create job plans
                let job_diffs: Vec<_> = job_types
                    .into_values()
                    .map(|plan_type| match &plan_type {
                        JobPlanType::Update(remote, local) => {
                            let diff = local.diff(remote);
                            JobPlan { plan_type, diff }
                        }
                        JobPlanType::Create(local) => {
                            let diff = RemoteJob::default().new_diff(local);
                            JobPlan { plan_type, diff }
                        }
                        JobPlanType::Delete(remote) => {
                            let diff = remote.diff(&RemoteJob::default());
                            JobPlan { plan_type, diff }
                        }
                    })
                    .collect();

                // Add our job plans
                ProjectPlan {
                    project_id: project.id,
                    project_name: k,
                    jobs: job_diffs,
                }
            })
            .collect();

        Self { projects: changes }
    }

    pub fn has_changes(&self) -> bool {
        self.projects.iter().any(|p| p.has_changes())
    }

    pub fn pretty_print(&self) {
        for p in &self.projects {
            p.pretty_print();
        }
    }

    pub fn apply(&self, client: &DbtCloudClient) {
        for p in &self.projects {
            p.apply(client);
        }
    }
}

pub struct ProjectPlan {
    project_id: i64,
    project_name: String,
    jobs: Vec<JobPlan>,
}

impl ProjectPlan {
    pub fn has_changes(&self) -> bool {
        self.jobs.iter().any(|p| p.has_changes())
    }
    pub fn pretty_print(&self) {
        let mut temp_jobs: Vec<_> = self.jobs.iter().collect(); // vec w/ reference to change order but not clone data
        temp_jobs.sort_by(|a, b| a.name().partial_cmp(b.name()).unwrap());
        println!("{} ({}):\n", self.project_name, self.project_id);
        for j in temp_jobs {
            j.pretty_print();
        }
    }
    pub fn apply(&self, client: &DbtCloudClient) {
        for j in &self.jobs {
            j.apply(client);
        }
    }
}

fn determine_job_plan_types_by_name(
    local_jobs: Vec<RemoteJob>,
    remote_jobs: Vec<RemoteJob>,
) -> HashMap<String, JobPlanType> {
    let local_keys: HashSet<String> = local_jobs.iter().map(|j| j.name.to_string()).collect();
    let remote_keys: HashSet<String> = remote_jobs.iter().map(|j| j.name.to_string()).collect();
    let create_keys: Vec<_> = local_keys.difference(&remote_keys).collect();
    let delete_keys: Vec<_> = remote_keys.difference(&local_keys).collect();
    let update_keys: Vec<_> = local_keys.intersection(&remote_keys).collect();
    let mut local_jobs_by_name: HashMap<_, _> = local_jobs
        .into_iter()
        .map(|j| (j.name.to_string(), j))
        .collect();
    let mut remote_jobs_by_name: HashMap<_, _> = remote_jobs
        .into_iter()
        .map(|j| (j.name.to_string(), j))
        .collect();

    let mut matched = HashMap::new();
    for k in create_keys {
        let c = local_jobs_by_name.remove(k.as_str()).unwrap();
        matched.insert(k.to_string(), JobPlanType::Create(c));
    }
    for k in update_keys {
        let c = local_jobs_by_name.remove(k.as_str()).unwrap();
        let r = remote_jobs_by_name.remove(k.as_str()).unwrap();
        matched.insert(k.to_string(), JobPlanType::Update(c.merge(&r), r));
    }
    for k in delete_keys {
        let r = remote_jobs_by_name.remove(k.as_str()).unwrap();
        matched.insert(k.to_string(), JobPlanType::Delete(r));
    }
    matched
}

#[derive(Debug, Clone)]
enum JobPlanType {
    Create(RemoteJob),
    Update(RemoteJob, RemoteJob),
    Delete(RemoteJob),
}

struct JobPlan {
    plan_type: JobPlanType,
    diff: Diff,
}

impl JobPlan {
    pub fn has_changes(&self) -> bool {
        self.diff.has_changes()
    }
    pub fn name(&self) -> &str {
        match &self.plan_type {
            JobPlanType::Create(new) => &new.name,
            JobPlanType::Update(_, remote) => &remote.name,
            JobPlanType::Delete(remote) => &remote.name,
        }
    }
    pub fn pretty_print(&self) {
        if self.has_changes() {
            match &self.plan_type {
                JobPlanType::Create(new) => {
                    println!("{}    \"{}\" (Computed)", "+".green(), new.name);
                    self.diff.pretty_print("      ");
                }
                JobPlanType::Update(_, remote) => {
                    println!(
                        "{}  \"{}\" ({})",
                        "+/-".yellow(),
                        remote.name,
                        remote.id.unwrap()
                    );
                    self.diff.pretty_print("      ");
                }
                JobPlanType::Delete(remote) => {
                    println!(
                        "{}    \"{}\" ({})",
                        "-".red(),
                        remote.name,
                        remote.id.unwrap()
                    );
                }
            }
        }
    }

    pub fn apply(&self, client: &DbtCloudClient) {
        if !self.has_changes() {
            return;
        }
        match &self.plan_type {
            JobPlanType::Create(local) => {
                println!("creating job: {}", &local.name);
                client.create_job(local).expect("failed to create job");
            }
            JobPlanType::Update(local, _) => {
                println!("updating job: {}", local.id.unwrap());
                client.update_job(local).expect("failed to update job");
            }
            JobPlanType::Delete(remote) => {
                println!("deleting job: {}", remote.id.unwrap());
            }
        }
    }
}
