use std::{sync::Arc, time::Duration};

use bollard::{
    container::{Config, RemoveContainerOptions},
    service::{CreateImageInfo, HostConfig},
    Docker, API_DEFAULT_VERSION,
};
use futures::StreamExt;
use tenderdash_abci::ServerRuntime;
use tokio::{io::AsyncWriteExt, time::timeout};
use tracing::{debug, error, info};
use url::Url;

pub struct TenderdashDocker {
    id: String,
    /// human-readable name of the container
    name: String,
    docker: Docker,
    image: String,
    runtime: ServerRuntime,
}
impl TenderdashDocker {
    /// new() creates and starts new Tenderdash docker container for provided
    /// tag.
    ///
    /// Panics on error.
    ///
    /// When using with socket server, it should be called after the server
    /// starts listening.
    ///
    /// # Arguments
    ///
    /// * `tag` - Docker tag to use; provide empty string to use default
    /// * `app_address` - address of ABCI app server; for example,
    ///   `tcp://172.17.0.1:4567`, `tcp://[::ffff:ac11:1]:5678`,
    ///   `grpc://172.17.01:5678` or `unix:///path/to/file`
    pub(crate) fn new(
        container_name: &str,
        tag: Option<&str>,
        app_address: &str,
    ) -> TenderdashDocker {
        // let tag = String::from(tenderdash_proto::VERSION);
        let tag = match tag.unwrap_or_default() {
            "" => tenderdash_proto::meta::TENDERDASH_VERSION,
            tag => tag,
        };

        let app_address = url::Url::parse(app_address).expect("invalid app address");
        if app_address.scheme() != "tcp"
            && app_address.scheme() != "unix"
            && app_address.scheme() != "grpc"
        {
            panic!("app_address must be either grpc://, tcp:// or unix://");
        }

        let runtime = tenderdash_abci::ServerRuntime::default();

        info!("Starting Tenderdash docker container");

        let docker = runtime
            .block_on(Self::connect())
            .expect("docker daemon connection failure");

        let mut td: TenderdashDocker = TenderdashDocker {
            id: Default::default(),
            name: container_name.to_string(),
            docker,
            image: format!("dashpay/tenderdash:{}", tag),
            runtime,
        };

        // Create container; we do it separately to retrieve `id` early and clean up if
        // needed
        td.id = td
            .runtime
            .block_on(td.create_container(app_address))
            .expect("creating docker container failed");

        td.runtime
            .block_on(td.start_container())
            .expect("starting docker container failed");

        info!("Tenderdash docker container started successfully");
        td.handle_ctrlc();

        td
    }

    fn handle_ctrlc(&self) {
        // Handle ctrl+c
        let id = self.id.clone();
        let docker = self.docker.clone();
        // let runtime = &td.runtime;
        let _ctrlc = self.runtime.spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            error!("Received ctrl+c, removing Tenderdash container");

            let stopped = timeout(Duration::from_secs(15), Self::stop(id, docker))
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

    async fn connect() -> Result<Docker, anyhow::Error> {
        debug!("Connecting to Docker server");
        let docker = Docker::connect_with_socket("/var/run/docker.sock", 120, API_DEFAULT_VERSION)?;

        let info = docker.info().await?;
        debug!(
            "Connected to Docker server version {}",
            info.server_version.unwrap()
        );

        Ok(docker)
    }

    async fn image_pull(&self) -> Result<(), anyhow::Error> {
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

    async fn create_container(&self, app_address: Url) -> Result<String, anyhow::Error> {
        self.image_pull().await?;

        debug!("Creating container");
        let binds = if app_address.scheme() == "unix" {
            let path = app_address.path();
            Some(vec![format!("{}:{}", path, path)])
        } else {
            None
        };

        let (abci, app_address) = match app_address.scheme() {
            "grpc" => {
                let address = app_address
                    .to_string()
                    .replace("grpc://", "")
                    .replace('/', "\\/");
                ("grpc", address)
            },
            _ => ("socket", app_address.to_string().replace('/', "\\/")),
        };

        debug!("Tenderdash will connect to ABCI address: {}", app_address);
        let container_config = Config {
            image: Some(self.image.clone()),
            env: Some(vec![
                format!("PROXY_APP={}", app_address),
                format!("ABCI={}", abci),
            ]),
            host_config: Some(HostConfig {
                binds,
                ..Default::default()
            }),
            ..Default::default()
        };

        let id = self
            .docker
            .create_container::<String, String>(
                Some(bollard::container::CreateContainerOptions {
                    name: self.name.clone(),
                    ..Default::default()
                }),
                container_config,
            )
            .await?
            .id;

        Ok(id)
    }

    async fn start_container(&self) -> Result<(), anyhow::Error> {
        debug!("Starting container");
        self.docker
            .start_container::<String>(
                &self.id,
                Some(bollard::container::StartContainerOptions {
                    ..Default::default()
                }),
            )
            .await?;

        Ok(())
    }

    /// Print 200 most recent logs from Tenderdash on standard error.
    #[allow(dead_code)]
    pub fn print_logs(&self) {
        let id = self.id.clone();

        if !id.is_empty() {
            debug!("Printing Tenderdash logs");
            let rt = &self.runtime;
            let docker = self.docker.clone();

            // We use `pollster` to block on the future, as tokio might not block_on()
            // when called inside a running runtime.
            let fut = rt.spawn(Self::emit_logs(id, docker));
            pollster::block_on(fut)
                .expect("cannot spawn log emitting fn")
                .expect("cannot emit logs");
        }
    }

    async fn emit_logs(id: String, docker: Docker) -> Result<(), anyhow::Error> {
        let stderror = tokio::io::stderr();
        let mut dest = tokio::io::BufWriter::new(stderror);

        let mut logs = docker.logs(
            &id,
            Some(bollard::container::LogsOptions {
                follow: false,
                stdout: true,
                stderr: true,
                tail: "200",
                ..Default::default()
            }),
        );

        while let Some(log) = logs.next().await {
            let log = log.unwrap();

            let data = log.to_string() + "\n";
            let data = data.as_bytes();

            dest.write_all(data).await.expect("cannot write logs");
        }

        dest.flush().await.expect("cannot flush logs");

        Ok(())
    }

    async fn stop(id: String, docker: Docker) -> Result<(), anyhow::Error> {
        debug!("Stopping Tenderdash container");
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
            let id = self.id.clone();
            let docker = self.docker.clone();

            let fut = self.runtime.spawn(Self::stop(id, docker));
            pollster::block_on(fut)
                .expect("cannot stop container")
                .expect("cannot stop container");
        }
    }
}

/// Use custom panic handler to dump logs on panic
#[allow(dead_code)]
pub fn setup_td_logs_panic(td_docker: &Arc<TenderdashDocker>) {
    let weak_ref = Arc::downgrade(td_docker);
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(td) = weak_ref.upgrade() {
            td.print_logs()
        }
        original(info);
    }));
}
