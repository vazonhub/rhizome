use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::net::UdpSocket;
use tokio::sync::{Mutex, oneshot};
use tracing::{error, info};

use crate::exceptions::{NetworkError, RhizomeError};
use crate::utils::time::get_now_f64;

/// Raw Message
///
/// Data without serialization yet
#[derive(Debug, Clone)]
pub struct Message {
    /// Transferred data
    pub data: Vec<u8>,
    /// IP + port of node
    pub address: SocketAddr,
    /// Time of getting message
    pub timestamp: f64,
}

/// Main UDP structure
pub struct UDPTransport {
    /// IP for connection _(0.0.0.0)_
    pub host: String,
    /// Connection port _(8080)_
    pub port: u16,
    /// Active socket store
    ///
    /// - `Arc` - for multiconnection from node and listen thread
    /// - `Mutex` - for safety manipulate with thread
    /// - `Option` - because before call `.start()` socket doesn't exist
    pub socket: Arc<Mutex<Option<Arc<UdpSocket>>>>,
    /// Change for sending stop signal
    pub stop_tx: Mutex<Option<oneshot::Sender<()>>>,
    /// Thread safety status value
    pub is_running: AtomicBool,
}

impl UDPTransport {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            socket: Arc::new(Mutex::new(None)),
            stop_tx: Mutex::new(None),
            is_running: AtomicBool::new(false),
        }
    }

    /// Start UDP transport
    pub async fn start<F>(&self, handler: F) -> Result<(), RhizomeError>
    where
        F: Fn(Message) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send>>
            + Send
            + Sync
            + 'static,
    {
        if self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }

        let addr = format!("{}:{}", self.host, self.port);
        let socket = UdpSocket::bind(&addr).await.map_err(|e| {
            error!("Failed to bind socket: {}", e);
            RhizomeError::Network(NetworkError::General)
        })?;

        let socket_arc = Arc::new(socket);

        {
            let mut socket_lock = self.socket.lock().await;
            *socket_lock = Some(socket_arc.clone());
        }

        let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
        {
            let mut stop_tx_lock = self.stop_tx.lock().await;
            *stop_tx_lock = Some(stop_tx);
        }

        let handler = Arc::new(handler);

        tokio::spawn(async move {
            let mut buf = vec![0u8; 65535];

            loop {
                tokio::select! {
                    _ = &mut stop_rx => {
                        break;
                    }
                    result = socket_arc.recv_from(&mut buf) => {
                        match result {
                            Ok((size, addr)) => {
                                let data = buf[..size].to_vec();
                                let timestamp = get_now_f64();

                                let msg = Message { data, address: addr, timestamp };
                                let h = handler.clone();

                                tokio::spawn(async move {
                                    h(msg).await;
                                });
                            }
                            Err(e) => {
                                error!("UDP receive error: {}", e);
                            }
                        }
                    }
                }
            }
        });

        self.is_running.store(true, Ordering::SeqCst);
        info!(host = %self.host, port = self.port, "UDP transport started");
        Ok(())
    }

    /// Stop the UDP transport
    pub async fn stop(&self) {
        if !self.is_running.load(Ordering::SeqCst) {
            return;
        }

        {
            let mut stop_tx_lock = self.stop_tx.lock().await;
            if let Some(tx) = stop_tx_lock.take() {
                let _ = tx.send(());
            }
        }

        {
            let mut socket_lock = self.socket.lock().await;
            *socket_lock = None;
        }

        self.is_running.store(false, Ordering::SeqCst);
        info!("UDP transport stopped");
    }

    /// Send message
    pub async fn send(&self, data: &[u8], address: SocketAddr) -> Result<bool, RhizomeError> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err(RhizomeError::Network(NetworkError::General));
        }

        let socket_lock = self.socket.lock().await;
        if let Some(socket) = socket_lock.as_ref() {
            match socket.send_to(data, address).await {
                Ok(_) => Ok(true),
                Err(e) => {
                    error!(error = %e, address = %address, "Error sending message");
                    Ok(false)
                }
            }
        } else {
            error!("No socket available for sending");
            Ok(false)
        }
    }

    /// Get transport address
    pub async fn get_address(&self) -> SocketAddr {
        let socket_lock = self.socket.lock().await;
        if let Some(socket) = socket_lock.as_ref() {
            socket.local_addr().unwrap_or_else(|_| {
                format!("{}:{}", self.host, self.port)
                    .parse()
                    .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap())
            })
        } else {
            format!("{}:{}", self.host, self.port)
                .parse()
                .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap())
        }
    }
}
