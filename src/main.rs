//! # ECSR
//!
//! `ecsr` is a tool that makes it easy to execute the ecs execute command.

use aws_config::{profile::ProfileFileCredentialsProvider, SdkConfig};
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input};
use regex::Regex;

struct EcsClient {
    client: aws_sdk_ecs::client::Client,
}

impl EcsClient {
    fn new(config: &SdkConfig) -> Self {
        Self {
            client: aws_sdk_ecs::Client::new(config),
        }
    }

    async fn fetch_clusters(&self) -> Result<Vec<String>, aws_sdk_ecs::Error> {
        let resp = self.client.list_clusters().send().await?;

        let re = Regex::new(r"^arn:aws:ecs:.*:[0-9]{12}:cluster/(.*)$").unwrap();

        let clusters = resp
            .cluster_arns()
            .unwrap()
            .iter()
            .map(|s| re.captures(s).unwrap().get(1).unwrap().as_str().to_string())
            .collect();

        Ok(clusters)
    }

    async fn fetch_services(&self, cluster: &str) -> Result<Vec<String>, aws_sdk_ecs::Error> {
        let resp = self.client.list_services().cluster(cluster).send().await?;

        let p = &format!("^arn:aws:ecs:.*:[0-9]{{12}}:service/{}/(.*)$", cluster);
        let re = Regex::new(p).unwrap();

        let services = resp
            .service_arns()
            .unwrap()
            .iter()
            .map(|s| re.captures(s).unwrap().get(1).unwrap().as_str().to_string())
            .collect();

        Ok(services)
    }

    async fn fetch_tasks(
        &self,
        cluster: &str,
        service: &str,
    ) -> Result<Vec<String>, aws_sdk_ecs::Error> {
        let resp = self
            .client
            .list_tasks()
            .cluster(cluster)
            .service_name(service)
            .send()
            .await?;

        let p = &format!("^arn:aws:ecs:.*:[0-9]{{12}}:task/{}/(.*)$", cluster);
        let re = Regex::new(p).unwrap();

        let tasks = resp
            .task_arns()
            .unwrap()
            .iter()
            .map(|s| re.captures(s).unwrap().get(1).unwrap().as_str().to_string())
            .collect();

        Ok(tasks)
    }

    async fn fetch_containers(
        &self,
        cluster: &str,
        task: &str,
    ) -> Result<Vec<String>, aws_sdk_ecs::Error> {
        let resp = self
            .client
            .describe_tasks()
            .cluster(cluster)
            .tasks(task)
            .send()
            .await?;

        let containers = resp
            .tasks()
            .unwrap()
            .iter()
            .flat_map(|t| t.containers().unwrap())
            .map(|c| c.name().unwrap().to_string())
            .collect();

        Ok(containers)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = dirs::home_dir().unwrap();
    path.push(".aws/credentials");
    println!("Read profile from {}", path.display());

    let result = std::fs::read_to_string(path.as_path());
    let content = match result {
        Ok(content) => content,
        Err(error) => {
            return Err(error.into());
        }
    };
    let mut profiles: Vec<String> = vec![];

    for line in content.lines() {
        if line.starts_with('[') {
            profiles.push(line.to_string().replace('[', "").replace(']', ""));
        }
    }
    let profile_name_idx = use_fuzzy_select(
        &format!("Select profile from {:?}", path.into_os_string()),
        &profiles,
    )
    .unwrap();
    let profile = &profiles[profile_name_idx];

    // This credentials provider will load credentials from ~/.aws/credentials.
    let credentials_provider = ProfileFileCredentialsProvider::builder()
        .profile_name(profile)
        .build();

    // Load the credentials
    let config = aws_config::from_env()
        .credentials_provider(credentials_provider)
        .load()
        .await;

    let ecs = EcsClient::new(&config);

    let clusters = ecs.fetch_clusters().await?;
    let cluster_arn_idx = use_fuzzy_select("Select cluster", &clusters).unwrap();
    let cluster = &clusters[cluster_arn_idx];

    let services = ecs.fetch_services(cluster).await?;
    let service_idx = use_fuzzy_select("Select service", &services).unwrap();
    let service = &services[service_idx];

    let tasks = ecs.fetch_tasks(cluster, service).await?;
    let task_idx = use_fuzzy_select("Select task", &tasks).unwrap();
    let task = &tasks[task_idx];

    let containers = ecs.fetch_containers(cluster, task).await?;
    let container_idx = use_fuzzy_select("Select container", &containers).unwrap();
    let container = &containers[container_idx];

    let command: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Command")
        .interact_text()
        .unwrap();

    // @see https://github.com/aws/amazon-ssm-agent/issues/354

    // let c = ecs
    //     .client
    //     .execute_command()
    //     .cluster(cluster)
    //     .task(task)
    //     .container(container)
    //     .interactive(true)
    //     .command("ls")
    //     .send()
    //     .await?;
    let command = format!("aws --profile {} ecs execute-command --cluster {} --container {} --interactive --command {} --task {}", profile, cluster, container, command, task);
    println!("{}", command);

    Ok(())
}

fn use_fuzzy_select(prompt: &str, list: &[String]) -> Result<usize, std::io::Error> {
    FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(0)
        .items(list)
        .interact()
}
