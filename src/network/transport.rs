use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::{Mutex, oneshot};
use tracing::{error, info};

use crate::exceptions::{NetworkError, RhizomeError};

/// Сетевое сообщение (аналог dataclass Message)
#[derive(Debug, Clone)]
pub struct Message {
    pub data: Vec<u8>,
    pub address: SocketAddr,
    pub timestamp: f64,
}

pub struct UDPTransport {
    host: String,
    port: u16,
    socket: Arc<Mutex<Option<Arc<UdpSocket>>>>, // Use Mutex for interior mutability
    stop_tx: Mutex<Option<oneshot::Sender<()>>>,
    is_running: AtomicBool,
}

impl UDPTransport {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            socket: Arc::new(Mutex::new(None)), // Initialize as None
            stop_tx: Mutex::new(None),
            is_running: AtomicBool::new(false),
        }
    }

    /// Запуск UDP транспорта (аналог start)
    pub async fn start<F>(&self, handler: F) -> Result<(), RhizomeError>
    where
        F: Fn(Message) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync
            + 'static,
    {
        if self.is_running.load(Ordering::SeqCst) {
            return Ok(());
        }

        // Bind socket
        let addr = format!("{}:{}", self.host, self.port);
        let socket = UdpSocket::bind(&addr).await.map_err(|e| {
            error!("Failed to bind socket: {}", e);
            RhizomeError::Network(NetworkError::General)
        })?;

        let socket_arc = Arc::new(socket);

        // Store socket in Mutex-protected field
        {
            let mut socket_lock = self.socket.lock().await;
            *socket_lock = Some(socket_arc.clone());
        }

        // Setup stop channel
        let (stop_tx, mut stop_rx) = oneshot::channel::<()>();
        {
            let mut stop_tx_lock = self.stop_tx.lock().await;
            *stop_tx_lock = Some(stop_tx);
        }

        let handler = Arc::new(handler);

        // Start listening task
        tokio::spawn(async move {
            let mut buf = vec![0u8; 65535];

            loop {
                tokio::select! {
                    // Проверка сигнала остановки
                    _ = &mut stop_rx => {
                        break;
                    }
                    // Получение данных
                    result = socket_arc.recv_from(&mut buf) => {
                        match result {
                            Ok((size, addr)) => {
                                let data = buf[..size].to_vec();
                                let timestamp = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs_f64();

                                let msg = Message { data, address: addr, timestamp };
                                let h = handler.clone();

                                // Запускаем обработчик в отдельной задаче (аналог loop.create_task)
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

    /// Остановка UDP транспорта
    pub async fn stop(&self) {
        if !self.is_running.load(Ordering::SeqCst) {
            return;
        }

        // Send stop signal
        {
            let mut stop_tx_lock = self.stop_tx.lock().await;
            if let Some(tx) = stop_tx_lock.take() {
                let _ = tx.send(());
            }
        }

        // Clear socket
        {
            let mut socket_lock = self.socket.lock().await;
            *socket_lock = None;
        }

        self.is_running.store(false, Ordering::SeqCst);
        info!("UDP transport stopped");
    }

    /// Отправка сообщения
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

    /// Получение адреса транспорта
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
