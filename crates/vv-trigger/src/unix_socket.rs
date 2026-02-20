use crate::{Trigger, VvError, VvResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::AsyncReadExt;
use tokio::sync::broadcast;
use tracing::{debug, error, info};

/// Unix socket trigger that responds to JSON commands over a Unix domain socket.
///
/// This listens for JSON commands on a Unix socket:
/// - `{"cmd": "start"}` - Begins recording
/// - `{"cmd": "stop"}` - Ends recording
#[derive(Debug, Clone)]
pub struct UnixSocket {
    socket_path: PathBuf,
    tx: broadcast::Sender<SocketCommand>,
}

/// Commands that can be sent via Unix socket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketCommand {
    pub cmd: String,
}

impl UnixSocket {
    /// Create a new `UnixSocket` trigger.
    ///
    /// # Arguments
    /// * `socket_path` - Path to the Unix socket (default: /tmp/voicevault.sock)
    ///
    /// # Returns
    /// A Result containing the `UnixSocket` or a `VvError`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let socket = UnixSocket::new("/tmp/voicevault.sock")?;
    /// ```
    pub fn new(socket_path: &str) -> VvResult<Self> {
        let (tx, _rx) = broadcast::channel(16);

        Ok(Self {
            socket_path: PathBuf::from(socket_path),
            tx,
        })
    }

    /// Start the Unix socket listener in the background
    fn start_listener(&self) -> VvResult<()> {
        let socket_path = self.socket_path.clone();
        let tx = self.tx.clone();

        info!("Unix socket listening on: {}", socket_path.display());

        // Clean up existing socket file if it exists
        if socket_path.exists() {
            debug!("Removing existing socket file: {}", socket_path.display());
            std::fs::remove_file(&socket_path).map_err(|e| {
                VvError::Trigger(format!("Failed to remove existing socket: {}", e))
            })?;
        }

        // Spawn the listener in the background
        tokio::spawn(async move {
            Self::run_listener(socket_path, tx).await;
        });

        Ok(())
    }

    /// Run the socket listener
    async fn run_listener(socket_path: PathBuf, tx: broadcast::Sender<SocketCommand>) {
        use tokio::net::UnixListener;

        // Bind the socket
        let listener = match UnixListener::bind(&socket_path) {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind to socket {}: {}", socket_path.display(), e);
                return;
            },
        };

        info!("Unix socket listener started on: {}", socket_path.display());

        // Accept connections
        loop {
            match listener.accept().await {
                Ok((mut stream, _addr)) => {
                    let tx = tx.clone();

                    tokio::spawn(async move {
                        // Read the command
                        let mut buffer = vec![0u8; 1024];
                        match stream.read(&mut buffer).await {
                            Ok(n) => {
                                if n == 0 {
                                    return;
                                }

                                // Parse the JSON command
                                let json_str = String::from_utf8_lossy(&buffer[..n]);
                                debug!("Received socket command: {}", json_str);

                                match serde_json::from_str::<SocketCommand>(&json_str) {
                                    Ok(cmd) => {
                                        if let Err(e) = tx.send(cmd) {
                                            error!("Failed to send socket command: {}", e);
                                        }
                                    },
                                    Err(e) => {
                                        error!("Failed to parse socket command: {}", e);
                                    },
                                }
                            },
                            Err(e) => {
                                error!("Failed to read from socket: {}", e);
                            },
                        }
                    });
                },
                Err(e) => {
                    error!("Failed to accept socket connection: {}", e);
                },
            }
        }
    }

    /// Subscribe to socket commands
    fn subscribe(&self) -> broadcast::Receiver<SocketCommand> {
        self.tx.subscribe()
    }
}

#[async_trait::async_trait]
impl Trigger for UnixSocket {
    /// Wait for socket start command.
    async fn wait_for_start(&self) -> VvResult<()> {
        // Start the listener if not already running
        self.start_listener()?;

        info!(
            "Waiting for socket start command on: {}",
            self.socket_path.display()
        );

        // Subscribe to commands
        let mut rx = self.subscribe();

        // Wait for a start command
        loop {
            match rx.recv().await {
                Ok(cmd) if cmd.cmd == "start" => {
                    info!("Received socket start command");
                    return Ok(());
                },
                Ok(cmd) => {
                    debug!("Ignoring socket command: {}", cmd.cmd);
                },
                Err(e) => {
                    return Err(VvError::Trigger(format!(
                        "Socket command channel error: {}",
                        e
                    )));
                },
            }
        }
    }

    /// Wait for socket stop command.
    async fn wait_for_stop(&self) -> VvResult<()> {
        info!(
            "Waiting for socket stop command on: {}",
            self.socket_path.display()
        );

        // Subscribe to commands
        let mut rx = self.subscribe();

        // Wait for a stop command
        loop {
            match rx.recv().await {
                Ok(cmd) if cmd.cmd == "stop" => {
                    info!("Received socket stop command");
                    return Ok(());
                },
                Ok(cmd) => {
                    debug!("Ignoring socket command: {}", cmd.cmd);
                },
                Err(e) => {
                    return Err(VvError::Trigger(format!(
                        "Socket command channel error: {}",
                        e
                    )));
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_socket_creation() {
        let socket = UnixSocket::new("/tmp/voicevault_test.sock").unwrap();
        assert_eq!(
            socket.socket_path,
            PathBuf::from("/tmp/voicevault_test.sock")
        );
    }

    #[test]
    fn test_socket_command_serialization() {
        let cmd = SocketCommand {
            cmd: "start".to_string(),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("start"));

        let parsed: SocketCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.cmd, "start");
    }

    #[test]
    fn test_send_sync() {
        // Verify UnixSocket implements Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<UnixSocket>();
    }
}
