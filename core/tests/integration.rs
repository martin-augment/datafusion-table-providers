use crate::docker::RunningContainer;
use rand::RngExt;

#[cfg(feature = "adbc")]
mod adbc;
mod arrow_record_batch_gen;
#[cfg(feature = "clickhouse")]
mod clickhouse;
#[cfg(any(
    feature = "clickhouse",
    feature = "mongodb",
    feature = "mysql",
    feature = "postgres"
))]
mod docker;
#[cfg(feature = "duckdb")]
mod duckdb;
#[cfg(feature = "flight")]
mod flight;
#[cfg(feature = "mongodb")]
mod mongodb;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "oracle")]
mod oracle;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

struct ContainerManager {
    port: u16,
    claimed: bool,
    running_container: Option<RunningContainer>,
}

impl Drop for ContainerManager {
    fn drop(&mut self) {
        tracing::info!("ContainerManager dropped");
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(drop_container(self.running_container.take(), self.port));
    }
}

impl Default for ContainerManager {
    fn default() -> Self {
        ContainerManager {
            port: crate::get_random_port(),
            claimed: false,
            running_container: None,
        }
    }
}

async fn drop_container(running_container: Option<RunningContainer>, port: u16) {
    if let Some(running_container) = running_container {
        match std::env::var("DF_TABLE_PROVIDERS_DEBUG").ok() {
            Some(_) => {
                // Just stop the container, so the developer could re-start if needed
                tracing::info!("Stopping Docker container on port {port}");
                if let Err(e) = running_container.stop().await {
                    tracing::error!("Error stopping Docker container: {e}");
                }
            },
            None => {
                tracing::info!("Removing Docker container on port {port}");
                if let Err(e) = running_container.remove().await {
                    tracing::error!("Error removing Docker container: {e}");
                }
            }
        }

    }
}

fn container_registry() -> String {
    std::env::var("CONTAINER_REGISTRY")
        .unwrap_or_else(|_| "public.ecr.aws/docker/library/".to_string())
}

fn get_random_port() -> u16 {
    rand::rng().random_range(15432..65535)
}
