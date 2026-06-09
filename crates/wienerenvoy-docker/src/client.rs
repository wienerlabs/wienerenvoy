//! Connection to the local Docker daemon: discovery, read-only listing, and
//! container/stack lifecycle control plus log tailing.

use std::collections::{BTreeMap, HashMap};

use bollard::Docker;
use bollard::container::{
    ListContainersOptions, LogsOptions, RestartContainerOptions, StartContainerOptions,
    StopContainerOptions,
};
use futures::StreamExt;
use wienerenvoy_core::service::{
    ContainerInfo, DockerStatus, ServiceAction, ServicesView, StackSummary,
};

use crate::error::DockerError;

const COMPOSE_PROJECT_LABEL: &str = "com.docker.compose.project";

/// A handle to the local Docker daemon, or a no-op when Docker is absent.
pub struct DockerHandle {
    docker: Option<Docker>,
}

impl DockerHandle {
    /// A handle with no Docker connection, for tests and degraded mode.
    #[must_use]
    pub fn disconnected() -> Self {
        Self { docker: None }
    }

    /// Connect to the local Docker. Never fails: an unreachable Docker yields a
    /// handle that reports `available: false` and an empty services view.
    #[must_use]
    pub async fn connect() -> Self {
        match Docker::connect_with_local_defaults() {
            Ok(docker) => match docker.version().await {
                Ok(_) => {
                    tracing::info!("connected to Docker");
                    Self {
                        docker: Some(docker),
                    }
                }
                Err(err) => {
                    tracing::warn!(error = %err, "Docker socket present but not responding");
                    Self { docker: None }
                }
            },
            Err(err) => {
                tracing::info!(error = %err, "Docker not available; service control disabled");
                Self { docker: None }
            }
        }
    }

    fn client(&self) -> Result<&Docker, DockerError> {
        self.docker.as_ref().ok_or(DockerError::Unavailable)
    }

    /// Docker availability and version.
    pub async fn status(&self) -> DockerStatus {
        match &self.docker {
            Some(docker) => match docker.version().await {
                Ok(version) => DockerStatus {
                    available: true,
                    version: version.version,
                },
                Err(_) => DockerStatus::default(),
            },
            None => DockerStatus::default(),
        }
    }

    /// List all containers grouped into Compose stacks by project label.
    pub async fn services(&self) -> ServicesView {
        let docker = self.status().await;
        let Some(client) = &self.docker else {
            return ServicesView {
                docker,
                ..Default::default()
            };
        };

        let options = ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        };
        let containers = match client.list_containers(Some(options)).await {
            Ok(containers) => containers,
            Err(err) => {
                tracing::warn!(error = %err, "list_containers failed");
                return ServicesView {
                    docker,
                    ..Default::default()
                };
            }
        };

        let mut stacks: BTreeMap<String, Vec<ContainerInfo>> = BTreeMap::new();
        let mut standalone: Vec<ContainerInfo> = Vec::new();

        for container in containers {
            let project = container
                .labels
                .as_ref()
                .and_then(|labels| labels.get(COMPOSE_PROJECT_LABEL).cloned());
            let info = ContainerInfo {
                id: container.id.unwrap_or_default().chars().take(12).collect(),
                name: container
                    .names
                    .and_then(|names| names.into_iter().next())
                    .map(|n| n.trim_start_matches('/').to_string())
                    .unwrap_or_default(),
                image: container.image.unwrap_or_default(),
                state: container.state.unwrap_or_default(),
                status: container.status.unwrap_or_default(),
            };
            match project {
                Some(project) => stacks.entry(project).or_default().push(info),
                None => standalone.push(info),
            }
        }

        let stacks = stacks
            .into_iter()
            .map(|(name, containers)| {
                let running = containers.iter().filter(|c| c.state == "running").count();
                StackSummary {
                    name,
                    running,
                    total: containers.len(),
                    containers,
                }
            })
            .collect();

        ServicesView {
            docker,
            stacks,
            standalone,
        }
    }

    /// Start, stop, or restart a single container.
    pub async fn container_action(
        &self,
        id: &str,
        action: ServiceAction,
    ) -> Result<(), DockerError> {
        let client = self.client()?;
        apply_action(client, id, action).await
    }

    /// Apply an action to every container in a Compose stack. Returns the count.
    pub async fn stack_action(
        &self,
        project: &str,
        action: ServiceAction,
    ) -> Result<usize, DockerError> {
        let client = self.client()?;
        let mut filters = HashMap::new();
        filters.insert(
            "label".to_string(),
            vec![format!("{COMPOSE_PROJECT_LABEL}={project}")],
        );
        let options = ListContainersOptions::<String> {
            all: true,
            filters,
            ..Default::default()
        };
        let containers = client.list_containers(Some(options)).await?;
        let mut affected = 0;
        for container in containers {
            if let Some(id) = container.id {
                apply_action(client, &id, action).await?;
                affected += 1;
            }
        }
        Ok(affected)
    }

    /// Tail the last `tail` log lines of a container.
    pub async fn container_logs(&self, id: &str, tail: usize) -> Result<Vec<String>, DockerError> {
        let client = self.client()?;
        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            timestamps: false,
            tail: tail.to_string(),
            ..Default::default()
        };
        let mut stream = client.logs(id, Some(options));
        let mut lines = Vec::new();
        while let Some(item) = stream.next().await {
            match item {
                Ok(output) => {
                    for line in output.to_string().split('\n') {
                        let trimmed = line.trim_end();
                        if !trimmed.is_empty() {
                            lines.push(trimmed.to_string());
                        }
                    }
                }
                Err(err) => {
                    tracing::warn!(error = %err, "log stream error");
                    break;
                }
            }
        }
        Ok(lines)
    }
}

async fn apply_action(client: &Docker, id: &str, action: ServiceAction) -> Result<(), DockerError> {
    match action {
        ServiceAction::Start => {
            client
                .start_container(id, None::<StartContainerOptions<String>>)
                .await?;
        }
        ServiceAction::Stop => {
            client
                .stop_container(id, None::<StopContainerOptions>)
                .await?;
        }
        ServiceAction::Restart => {
            client
                .restart_container(id, None::<RestartContainerOptions>)
                .await?;
        }
    }
    Ok(())
}
