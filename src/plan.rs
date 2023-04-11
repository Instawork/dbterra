use std::collections::HashMap;

use colored::Colorize;

use crate::{
    config::Config,
    diff::Diff,
    local::Root,
    local::{Job as LocalJob, Project},
    remote::{DbtCloudClient, Job as RemoteJob},
};

pub struct Plan {
    projects: Vec<ProjectPlan>,
    // environments: Vec<EnvironmentPlan>, TODO: Implement for environments as well
}

impl Plan {
    pub fn from(yaml: Root, client: &DbtCloudClient, config: &Config) -> Self {
        let mut changes = Vec::new();
        for (k, project) in yaml.projects.iter() {
            // Grab the local YAML jobs and the remote jobs for the project
            let local_config = config.with_project_id(project.id);
            let local_jobs: Vec<&LocalJob> = project.jobs.values().collect();
            let remote_jobs = client
                .get_jobs_for_project(project.id)
                .expect("failed to get remote jobs");

            // Convert our local jobs to look like remote ones
            let converted_local_jobs: Vec<_> = local_jobs
                .into_iter()
                .map(|j| RemoteJob::from_local_job(j.clone(), &local_config, &yaml.environments))
                .collect();

            // Figure out which are updates, creates, and deletes
            let job_types: HashMap<String, JobPlanType> = determine_job_actions_by_name(
                converted_local_jobs.as_slice(),
                remote_jobs.data.unwrap().as_slice(),
            );

            // Create job plans
            let job_diffs: Vec<_> = job_types
                .values()
                .map(|plan_type| match plan_type {
                    JobPlanType::Update(remote, local) => JobPlan {
                        plan_type: plan_type.clone(),
                        diff: local.diff(remote),
                    },
                    JobPlanType::Create(local) => JobPlan {
                        plan_type: plan_type.clone(),
                        diff: RemoteJob::default().new_diff(local),
                    },
                    JobPlanType::Delete(remote) => JobPlan {
                        plan_type: plan_type.clone(),
                        diff: remote.diff(&RemoteJob::default()),
                    },
                })
                .collect();

            // Add our job plans
            changes.push(ProjectPlan {
                project_name: k.clone(),
                project: project.clone(),
                jobs: job_diffs,
            });
        }
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
    project_name: String,
    project: Project,
    jobs: Vec<JobPlan>,
}

impl ProjectPlan {
    pub fn has_changes(&self) -> bool {
        self.jobs.iter().any(|p| p.has_changes())
    }
    pub fn pretty_print(&self) {
        let mut temp_jobs: Vec<_> = self.jobs.iter().collect(); // vec w/ reference to change order but not clone data
        temp_jobs.sort_by(|a, b| a.name().partial_cmp(b.name()).unwrap());
        println!("{} ({}):\n", self.project_name, self.project.id);
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

// TODO: make more efficient
fn determine_job_actions_by_name<'a>(
    converted_jobs: &'a [RemoteJob],
    remote_jobs: &'a [RemoteJob],
) -> HashMap<String, JobPlanType> {
    let mut matched = HashMap::new();
    for c in converted_jobs {
        let mut did_match = false;
        for r in remote_jobs {
            if c.name == r.name {
                matched.insert(c.name.clone(), JobPlanType::Update(c.merge(&r), r.clone()));
                did_match = true;
                continue;
            }
        }
        if !did_match {
            matched.insert(c.name.clone(), JobPlanType::Create(c.clone()));
        }
    }
    for r in remote_jobs {
        let mut did_match = false;
        for c in converted_jobs {
            if c.name == r.name {
                did_match = true;
                continue;
            }
        }
        if !did_match {
            matched.insert(r.name.clone(), JobPlanType::Delete(r.clone()));
        }
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
                client.create_job(&local).expect("failed to create job");
            }
            JobPlanType::Update(local, _) => {
                println!("updating job: {}", local.id.unwrap());
                client.update_job(&local).expect("failed to update job");
            }
            JobPlanType::Delete(remote) => {
                println!("deleting job: {}", remote.id.unwrap());
            }
        }
    }
}
