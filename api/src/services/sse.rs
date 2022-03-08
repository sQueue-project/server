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
use crate::services::payload::ContentType;
use proto::{SsePacket, SsePacketEvent};
use prost::Message;

pub struct SseResponse(SseRxClient);

impl Responder for SseResponse {
    type Body = BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::Ok()
            .insert_header(("Content-Type", "text/event-stream"))
            .no_chunking(1024)
            .streaming(self.0)
    }
}

impl From<SseRxClient> for SseResponse {
    fn from(client: SseRxClient) -> Self {
        Self(client)
    }
}

#[derive(Debug, Error)]
pub enum SseError {
    #[error("Failed to send: {0}")]
    Send(#[from] TrySendError<Bytes>),
    #[error("Failed to serialize to json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Failed to serialize to protobuf: {0}")]
    Protobuf(#[from] prost::EncodeError),
}

#[derive(Debug)]
pub struct Broadcaster {
    clients: Vec<SseTxClient>,
}

#[derive(Debug)]
pub struct SseTxClient {
    sender: Sender<Bytes>,
    content_type: ContentType
}

pub type AMBroadcaster = Arc<Mutex<Broadcaster>>;

const HEARTBEAT_INTERVAL: u64 = 10;
const SSE_PACKET_DATA_PING: &str = "Ping";
const SSE_PACKET_DATA_CONNECTED: &str = "Connected";

#[derive(Serialize)]
struct SseJsonPacket<D: Serialize> {
    event: SsePacketEvent,
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
        let packet_json = Bytes::from(serde_json::to_string(&SseJsonPacket {
            event: SsePacketEvent::InternalStatus,
            data: SSE_PACKET_DATA_PING,
        })?);

        let packet_protobuf = Bytes::from(SsePacket {
            event: SsePacketEvent::InternalStatus.into(),
            data: SSE_PACKET_DATA_PING.as_bytes().to_vec(),
        }.encode_to_vec());

        self.clients.retain(|x| {
            let bytes = match x.content_type {
                ContentType::Json => packet_json.clone(),
                ContentType::Protobuf => packet_protobuf.clone(),
                _ => unreachable!()
            };

            x.sender.try_send(bytes).is_ok()
        });

        Ok(())
    }

    pub fn new_client(&mut self, content_type: ContentType) -> Result<SseRxClient, SseError> {
        let (tx, rx) = channel(100);
        let bytes = match content_type {
            ContentType::Json => Bytes::from(serde_json::to_string(&SseJsonPacket {
                event: SsePacketEvent::InternalStatus,
                data: SSE_PACKET_DATA_CONNECTED,
            })?),
            ContentType::Protobuf => Bytes::from(SsePacket {
                event: SsePacketEvent::InternalStatus.into(),
                data: SSE_PACKET_DATA_CONNECTED.as_bytes().to_vec()
            }.encode_to_vec()),
            ContentType::Other => unreachable!(),
        };
        tx.try_send(bytes)?;

        self.clients.push(SseTxClient {
            sender: tx,
            content_type: content_type.clone(),
        });

        Ok(SseRxClient {
            receiver: rx,
            content_type,
        })
    }

    #[allow(unused)]
    // TODO remove unused
    pub fn send<D: Serialize + Message + Clone>(&self, event: SsePacketEvent, data: D) -> Result<(), SseError> {
        let packet_json = Bytes::from(serde_json::to_string(&SseJsonPacket {
            event: event.clone(),
            data: data.clone(),
        })?);

        let packet_protobuf = Bytes::from(SsePacket {
            event: event.into(),
            data: data.encode_to_vec()
        }.encode_to_vec());

        self.clients.iter()
            .try_for_each(|x| {
                let bytes = match x.content_type {
                    ContentType::Json => packet_json.clone(),
                    ContentType::Protobuf => packet_protobuf.clone(),
                    ContentType::Other => unreachable!()
                };

                x.sender.try_send(bytes)
            })?;

        Ok(())
    }
}

pub struct SseRxClient {
    receiver: Receiver<Bytes>,
    pub content_type: ContentType,
}

impl Stream for SseRxClient {
    type Item = Result<Bytes, actix_web::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.receiver).poll_recv(cx) {
            Poll::Ready(Some(v)) => Poll::Ready(Some(Ok(v))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending
        }
    }
}