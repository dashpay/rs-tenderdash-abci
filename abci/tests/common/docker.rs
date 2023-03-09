use std::time::Duration;

use bollard::{
    container::{Config, RemoveContainerOptions},
    service::{CreateImageInfo, HostConfig},
    Docker, API_DEFAULT_VERSION,
};
use flex_error::{define_error, DisplayError};
use futures::StreamExt;
use tokio::{runtime::Runtime, time::timeout};
use tracing::{debug, error, info};
use url::Url;

pub struct TenderdashDocker {
    id: String,
    docker: Docker,
    image: String,
    runtime: Runtime,
}
impl TenderdashDocker {
    /// new() creates and starts new Tenderdash docker container for provided tag.
    ///
    /// Panics on error.
    ///
    /// When using with socket server, it should be called after the server starts listening.
    ///
    /// # Arguments
    ///
    /// * `tag` - Docker tag to use; provide empty string to use default
    /// * `app_address` - address of ABCI app server; either 'tcp://1.2.3.4:4567' or 'unix:///path/to/file'
    ///
    pub(crate) fn new(tag: &str, app_address: &str) -> Result<TenderdashDocker, Error> {
        // let tag = String::from(tenderdash_proto::VERSION);
        let tag = if tag.is_empty() {
            tenderdash_proto::VERSION
        } else {
            tag
        };

        let app_address = url::Url::parse(app_address).expect("invalid app address");
        if app_address.scheme() != "tcp" && app_address.scheme() != "unix" {
            panic!("app_address must be either tcp:// or unix://");
        }

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("cannot initialize tokio runtime");

        info!("Starting Tenderdash docker container");

        let docker = runtime.block_on(Self::connect())?;

        let mut td: TenderdashDocker = TenderdashDocker {
            id: Default::default(),
            docker,
            image: format!("dashpay/tenderdash:{}", tag),
            runtime,
        };

        td.id = td.runtime.block_on(td.start(app_address))?;

        info!("Tenderdash docker container started successfully");
        td.handle_ctrlc();
        Ok(td)
    }

    fn handle_ctrlc(&self) {
        // Handle ctrl+c
        let id = self.id.clone();
        let docker = self.docker.clone();
        // let runtime = &td.runtime;
        let _ctrlc = self.runtime.spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            error!("Received ctrl+c, removing Tenderdash container");

            let stopped = timeout(Duration::from_secs(15), Self::stop(id, &docker))
                .await
                .expect("timeout removing tenderdash container");
            if stopped.is_err() {
                error!(
                    "failed to remove tenderdash container: {}",
                    stopped.err().unwrap()
                )
            }
            std::process::exit(1);
        });
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

        image_responses.pop().unwrap()?;
        debug!("Image fetch completed");

        Ok(())
    }

    async fn create_container(&self, app_address: Url) -> Result<String, Error> {
        debug!("Creating container");
        let binds = if app_address.scheme() == "unix" {
            let path = app_address.path();
            Some(vec![format!("{}:{}", path, path)])
        } else {
            None
        };

        let app_address = app_address.to_string().replace("/", "\\/");

        debug!("Tenderdash will connect to ABCI address: {}", app_address);
        let container_config = Config {
            image: Some(self.image.clone()),
            env: Some(vec![format!("PROXY_APP={}", app_address)]),
            host_config: Some(HostConfig {
                binds: binds,
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
    async fn start(&self, app_address: Url) -> Result<String, Error> {
        self.image_pull().await?;
        self.create_container(app_address).await
    }

    async fn stop(id: String, docker: &Docker) -> Result<(), Error> {
        docker
            .remove_container(
                &id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await?;
        Ok(())
    }
}

impl Drop for TenderdashDocker {
    fn drop(&mut self) {
        if !self.id.is_empty() {
            let _ = self
                .runtime
                .block_on(Self::stop(self.id.clone(), &self.docker));
        }
    }
}
define_error!(
Error {
    Docker
        [DisplayError<bollard::errors::Error>]
        | _ | { "Docker error" },
});

// FIXME: I think this should be generated somehow by the define_error! macro above, but it is not
impl From<bollard::errors::Error> for Error {
    fn from(value: bollard::errors::Error) -> Self {
        Error::docker(value)
    }
}
