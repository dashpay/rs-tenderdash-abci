use std::time::Duration;

use bollard::{
    container::{Config, RemoveContainerOptions},
    service::{CreateImageInfo, HostConfig},
    Docker, API_DEFAULT_VERSION,
};
use futures::StreamExt;
use tenderdash_abci::Error;
use tokio::{runtime::Runtime, time::timeout};
use tracing::{debug, info};

pub struct TenderdashDocker {
    id: String,
    docker: Docker,
    image: String,
    runtime: Runtime,
}
impl TenderdashDocker {
    /// new() creates and starts new Tenderdash docker container for provided tag.
    /// If tag is "", we use tenderdash proto version.
    pub(crate) fn new(tag: String) -> Result<TenderdashDocker, Error> {
        // let tag = String::from(tenderdash_proto::VERSION);
        let tag = match tag.is_empty() {
            true => String::from(tenderdash_proto::VERSION),
            false => tag,
        };
        let image = format!("dashpay/tenderdash:{}", tag);

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        info!("starting Tenderdash docker container");

        let mut td: TenderdashDocker = TenderdashDocker {
            id: Default::default(),
            docker: runtime.block_on(Self::connect())?,
            image,
            runtime,
        };

        td.id = td.runtime.block_on(td.start())?;

        // Handle ctrl+c
        let id = td.id.clone();
        let docker = td.docker.clone();
        // let runtime = &td.runtime;
        let _ctrlc = td.runtime.spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            debug!("Received ctrl+c, removing Tenderdash container");
            timeout(Duration::from_secs(15), Self::stop(id, &docker))
                .await
                .expect("timeout removing tenderdash container");
            std::process::exit(1);
        });

        Ok(td)
    }

    async fn connect() -> Result<Docker, Error> {
        debug!("Connecting to Docker server");
        let docker = Docker::connect_with_socket("/var/run/docker.sock", 120, API_DEFAULT_VERSION)?;

        let info = docker.info().await?;
        debug!(
            "Connected to Docker server version {}",
            info.server_version.unwrap()
        );

        Ok(docker)
    }

    async fn image_pull(&self) -> Result<(), Error> {
        debug!("Fetching image {}", self.image);
        let image_responses = self.docker.create_image(
            Some(bollard::image::CreateImageOptions {
                from_image: self.image.clone(),
                ..Default::default()
            }),
            None,
            None,
        );

        let mut image_responses = image_responses
            .collect::<Vec<Result<CreateImageInfo, bollard::errors::Error>>>()
            .await;

        let image_response = image_responses.pop().unwrap()?;
        debug!("Image fetch status: {}", image_response.status.unwrap());
        Ok(())
    }

    async fn create_container(&self) -> Result<String, Error> {
        debug!("Creating container");
        let container_config = Config {
            image: Some(self.image.clone()),
            env: Some(vec![String::from("PROXY_APP=unix:\\/\\/\\/abci.sock")]),
            host_config: Some(HostConfig {
                binds: Some(vec![String::from("/tmp/socket:/abci.sock")]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let id = self
            .docker
            .create_container::<String, String>(
                Some(bollard::container::CreateContainerOptions {
                    name: String::from("tenderdash"),
                    ..Default::default()
                }),
                container_config,
            )
            .await?
            .id;

        debug!("Starting container");
        self.docker
            .start_container::<String>(
                &id,
                Some(bollard::container::StartContainerOptions {
                    ..Default::default()
                }),
            )
            .await?;

        Ok(id)
    }
    /// Start Tenderdash in Docker
    async fn start(&self) -> Result<String, Error> {
        self.image_pull().await?;

        let id = self.create_container().await?;

        info!("Tenderdash container started successfully");
        Ok(id)
    }

    async fn stop(id: String, docker: &Docker) {
        docker
            .remove_container(
                &id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await
            .expect("cannot remove container, leaving some garbage");
    }
}

impl Drop for TenderdashDocker {
    fn drop(&mut self) {
        if !self.id.is_empty() {
            self.runtime
                .block_on(Self::stop(self.id.clone(), &self.docker));
        }
    }
}
