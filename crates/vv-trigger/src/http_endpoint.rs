use crate::{Trigger, VvError, VvResult};
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info};

/// HTTP endpoint trigger that responds to HTTP requests.
///
/// This spawns an HTTP server with three endpoints:
/// - `POST /start` - Begins recording
/// - `POST /stop` - Ends recording
/// - `GET /status` - Returns recording status
#[derive(Debug, Clone)]
pub struct HttpEndpoint {
    port: u16,
    tx: Arc<broadcast::Sender<HttpCommand>>,
}

/// Commands that can be sent via HTTP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpCommand {
    Start,
    Stop,
}

/// Response for the /status endpoint
#[derive(Debug, Clone, Serialize)]
struct StatusResponse {
    recording: bool,
}

/// Success response
#[derive(Debug, Serialize)]
struct SuccessResponse {
    status: String,
}

/// Error response
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

impl HttpEndpoint {
    /// Create a new `HttpEndpoint` trigger.
    ///
    /// # Arguments
    /// * `port` - Port to listen on (default: 8765)
    ///
    /// # Returns
    /// A Result containing the `HttpEndpoint` or a `VvError`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let http = HttpEndpoint::new(8765)?;
    /// ```
    pub fn new(port: u16) -> VvResult<Self> {
        let (tx, _rx) = broadcast::channel(16);

        Ok(Self {
            port,
            tx: Arc::new(tx),
        })
    }

    /// Start the HTTP server in the background
    fn start_server(&self) -> VvResult<()> {
        let tx = Arc::clone(&self.tx);
        let port = self.port;

        let app = Router::new()
            .route("/start", post(start_recording))
            .route("/stop", post(stop_recording))
            .route("/status", get(get_status))
            .with_state(tx);

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        info!("HTTP endpoint listening on http://{}", addr);

        // Spawn the server in the background
        tokio::spawn(async move {
            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(l) => l,
                Err(e) => {
                    error!("Failed to bind to port {}: {}", port, e);
                    return;
                },
            };

            if let Err(e) = axum::serve(listener, app).await {
                error!("HTTP server error: {}", e);
            }
        });

        Ok(())
    }

    /// Subscribe to HTTP commands
    fn subscribe(&self) -> broadcast::Receiver<HttpCommand> {
        self.tx.subscribe()
    }
}

/// Handler for POST /start
async fn start_recording(
    State(tx): State<Arc<broadcast::Sender<HttpCommand>>>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Received HTTP start command");

    if let Err(e) = tx.send(HttpCommand::Start) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to send start command: {}", e),
            }),
        ));
    }

    Ok(Json(SuccessResponse {
        status: "started".to_string(),
    }))
}

/// Handler for POST /stop
async fn stop_recording(
    State(tx): State<Arc<broadcast::Sender<HttpCommand>>>,
) -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Received HTTP stop command");

    if let Err(e) = tx.send(HttpCommand::Stop) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to send stop command: {}", e),
            }),
        ));
    }

    Ok(Json(SuccessResponse {
        status: "stopped".to_string(),
    }))
}

/// Handler for GET /status
async fn get_status() -> impl IntoResponse {
    debug!("Received HTTP status request");

    // For now, we don't track state, so we return a placeholder
    // In a full implementation, we'd track recording state
    Json(StatusResponse { recording: false })
}

#[async_trait::async_trait]
impl Trigger for HttpEndpoint {
    /// Wait for HTTP POST /start command.
    async fn wait_for_start(&self) -> VvResult<()> {
        // Start the server if not already running
        self.start_server()?;

        info!("Waiting for HTTP POST /start command on port {}", self.port);

        // Subscribe to commands
        let mut rx = self.subscribe();

        // Wait for a Start command
        loop {
            match rx.recv().await {
                Ok(HttpCommand::Start) => {
                    info!("Received HTTP start command");
                    return Ok(());
                },
                Ok(HttpCommand::Stop) => {
                    debug!("Ignoring stop command while waiting for start");
                },
                Err(e) => {
                    return Err(VvError::Trigger(format!(
                        "HTTP command channel error: {}",
                        e
                    )));
                },
            }
        }
    }

    /// Wait for HTTP POST /stop command.
    async fn wait_for_stop(&self) -> VvResult<()> {
        info!("Waiting for HTTP POST /stop command on port {}", self.port);

        // Subscribe to commands
        let mut rx = self.subscribe();

        // Wait for a Stop command
        loop {
            match rx.recv().await {
                Ok(HttpCommand::Stop) => {
                    info!("Received HTTP stop command");
                    return Ok(());
                },
                Ok(HttpCommand::Start) => {
                    debug!("Ignoring start command while waiting for stop");
                },
                Err(e) => {
                    return Err(VvError::Trigger(format!(
                        "HTTP command channel error: {}",
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
    fn test_http_endpoint_creation() {
        let http = HttpEndpoint::new(8765).unwrap();
        assert_eq!(http.port, 8765);
    }

    #[test]
    fn test_send_sync() {
        // Verify HttpEndpoint implements Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<HttpEndpoint>();
    }

    #[test]
    fn test_http_command_serialization() {
        let start = HttpCommand::Start;
        let json = serde_json::to_string(&start).unwrap();
        assert!(json.contains("Start"));

        let stop = HttpCommand::Stop;
        let json = serde_json::to_string(&stop).unwrap();
        assert!(json.contains("Stop"));
    }
}
