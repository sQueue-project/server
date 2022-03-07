use std::pin::Pin;
use std::sync::Arc;
use parking_lot::Mutex;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::{interval_at, Instant};
use std::task::{Context, Poll};
use std::time::Duration;
use actix_web::body::BoxBody;
use actix_web::{HttpRequest, HttpResponse, Responder};
use actix_web::web::Bytes;
use tokio_stream::Stream;
use serde::Serialize;
use thiserror::Error;
use tokio::sync::mpsc::error::TrySendError;
use tracing::warn;

pub struct SseResponse(SseClient);

impl Responder for SseResponse {
    type Body = BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::Ok()
            .insert_header(("Content-Type", "text/event-stream"))
            .no_chunking(1024)
            .streaming(self.0)
    }
}

impl From<SseClient> for SseResponse {
    fn from(client: SseClient) -> Self {
        Self(client)
    }
}

#[derive(Debug, Error)]
pub enum SseError {
    #[error("Failed to send: {0}")]
    Send(#[from] TrySendError<Bytes>),
    #[error("Failed to serialize: {0}")]
    Serialize(#[from] serde_json::Error),
}

#[derive(Debug)]
pub struct Broadcaster {
    clients: Vec<Sender<Bytes>>
}

pub type AMBroadcaster = Arc<Mutex<Broadcaster>>;

const HEARTBEAT_INTERVAL: u64 = 10;
const SSE_PACKET_EVENT_INTERNAL_STATUS: &str = "InternalStatus";
const SSE_PACKET_DATA_PING: &str = "Ping";
const SSE_PACKET_DATA_CONNECTED: &str = "Connected";

#[derive(Serialize)]
struct SsePacket<E: Serialize, D: Serialize> {
    event: E,
    data: D,
}

impl Broadcaster {
    pub fn new() -> AMBroadcaster {
        let this = Self {
            clients: Vec::new()
        };

        let am_this = Arc::new(Mutex::new(this));
        Self::spawn_ping(am_this.clone());
        am_this
    }

    fn spawn_ping(this: AMBroadcaster) {
        actix_rt::spawn(async move {
            let mut task = interval_at(Instant::now(), Duration::from_secs(HEARTBEAT_INTERVAL));
            loop {
                task.tick().await;
                if let Err(e) = this.lock().remove_stale_clients() {
                    warn!("Failed to remove stale clients: {:?}", e);
                }
            }
        });
    }

    fn remove_stale_clients(&mut self) -> Result<(), SseError> {
        let packet = serde_json::to_string(&SsePacket {
            event: SSE_PACKET_EVENT_INTERNAL_STATUS,
            data: SSE_PACKET_DATA_PING,
        })?;

        self.clients.retain(|x| x.try_send(Bytes::from(packet.clone())).is_ok());

        Ok(())
    }

    pub fn new_client(&mut self) -> Result<SseClient, SseError> {
        let (tx, rx) = channel(100);
        let packet = serde_json::to_string(&SsePacket {
            event: SSE_PACKET_EVENT_INTERNAL_STATUS,
            data: SSE_PACKET_DATA_CONNECTED,
        })?;
        tx.try_send(Bytes::from(packet))?;

        self.clients.push(tx);

        Ok(SseClient(rx))
    }

    pub fn send<E: Serialize, D: Serialize>(&self, event: E, data: D) -> Result<(), SseError> {
        let packet = serde_json::to_string(&SsePacket {
            event,
            data
        }).unwrap();

        self.clients.iter()
            .try_for_each(|x| x.try_send(Bytes::from(packet.clone())))?;

        Ok(())
    }
}

pub struct SseClient(Receiver<Bytes>);

impl Stream for SseClient {
    type Item = Result<Bytes, actix_web::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.0).poll_recv(cx) {
            Poll::Ready(Some(v)) => Poll::Ready(Some(Ok(v))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending
        }
    }
}