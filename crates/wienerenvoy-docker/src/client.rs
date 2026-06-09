//! Connection to the local Docker daemon and read-only container/stack listing.

use std::collections::BTreeMap;

use bollard::Docker;
use bollard::container::ListContainersOptions;
use wienerenvoy_core::service::{ContainerInfo, DockerStatus, ServicesView, StackSummary};

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
}
