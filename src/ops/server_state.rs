use crate::context::McContext;
use crate::minecraft::server_properties::ServerProperties;
use crate::ops::lock::InstanceLocks;
use crate::utils::errors::McResult;

/// Whether the instance's server is running, and its remote console when it
/// is. Anything a running server cannot apply is deferred with a warning.
pub enum ServerState {
    Stopped,
    Running(Option<minecraft_client_rs::Client>)
}

impl ServerState {
    pub async fn detect(context: &McContext, rcon_port: u16) -> McResult<ServerState> {
        let locks = InstanceLocks::new(&context.cwd);
        let mut world_lock = locks.world()?;

        if world_lock.try_acquire()?.is_some() {
            return Ok(ServerState::Stopped);
        }

        let instance_path = context.cwd.join("instance");
        let rcon = ServerProperties::read_rcon_password(&instance_path)
            .await?
            .and_then(|password| connect_rcon(rcon_port, &password));

        Ok(ServerState::Running(rcon))
    }

    pub fn send(&mut self, context: &mut McContext, command: String) {
        match self {
            ServerState::Stopped => {}
            ServerState::Running(Some(rcon)) => match rcon.send_command(command) {
                Ok(response) => {
                    _ = context.shell().status("Server", response.body.trim());
                }
                Err(error) => {
                    _ = context.shell().warn(format!(
                        "could not apply the change to the running server: {}; it takes effect at the next restart",
                        error
                    ));
                }
            },
            ServerState::Running(None) => {
                _ = context.shell().warn(
                    "the server is running but rcon is unavailable; the change takes effect at the next restart"
                );
            }
        }
    }

    pub fn defer(&self, context: &mut McContext, what: &str) {
        if matches!(self, ServerState::Running(_)) {
            _ = context.shell().warn(format!(
                "{} takes effect at the next restart; the running server cannot apply it",
                what
            ));
        }
    }

    pub fn close(self) {
        if let ServerState::Running(Some(mut rcon)) = self {
            let _ = rcon.close();
        }
    }
}

pub fn connect_rcon(port: u16, password: &str) -> Option<minecraft_client_rs::Client> {
    let rcon_address = format!("127.0.0.1:{}", port);

    let mut client = minecraft_client_rs::Client::new(rcon_address).ok()?;

    if client.authenticate(password.to_string()).is_err() {
        let _ = client.close();

        None
    } else {
        Some(client)
    }
}
