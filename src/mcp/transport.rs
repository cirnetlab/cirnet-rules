use futures::{SinkExt, StreamExt};
use rmcp::{
    RoleServer,
    model::{ClientJsonRpcMessage, ClientNotification, ErrorData, RequestId, ServerJsonRpcMessage},
    transport::{
        Transport,
        async_rw::JsonRpcMessageCodec,
        io::{StdioReader, StdioWriter, stdio_pipes},
    },
};
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use std::{
    collections::{HashSet, VecDeque},
    fmt, io,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::Mutex as AsyncMutex;
use tokio_util::{
    bytes::BytesMut,
    codec::{Decoder, FramedRead, FramedWrite, LinesCodec, LinesCodecError},
    sync::CancellationToken,
};

const FRAME_BYTES: usize = 64 * 1024;
const IN_FLIGHT: usize = 16;
const WRITE_DEADLINE: Duration = Duration::from_secs(3);
const JSON_DEPTH: u8 = 24;

type Writer = FramedWrite<StdioWriter, JsonRpcMessageCodec<ServerJsonRpcMessage>>;

/// Bounds transport admission until the associated response has been flushed.
/// The SDK owns JSON-RPC encoding, decoding and both protocol lifecycles.
pub(super) struct BoundedStdio {
    reader: FramedRead<StdioReader, FrameLines>,
    writer: Arc<AsyncMutex<Writer>>,
    admission: Arc<Mutex<Admission>>,
    closed: CancellationToken,
}

impl BoundedStdio {
    pub(super) fn new() -> Self {
        let (reader, writer) = stdio_pipes();
        Self {
            reader: FramedRead::new(
                reader,
                FrameLines(LinesCodec::new_with_max_length(FRAME_BYTES)),
            ),
            writer: Arc::new(AsyncMutex::new(FramedWrite::new(
                writer,
                JsonRpcMessageCodec::default(),
            ))),
            admission: Arc::new(Mutex::new(Admission::default())),
            closed: CancellationToken::new(),
        }
    }
}

impl Transport<RoleServer> for BoundedStdio {
    type Error = io::Error;

    fn send(
        &mut self,
        message: ServerJsonRpcMessage,
    ) -> impl Future<Output = io::Result<()>> + Send + 'static {
        let writer = self.writer.clone();
        let admission = self.admission.clone();
        let closed = self.closed.clone();
        async move {
            let id = match &message {
                ServerJsonRpcMessage::Response(response) => Some(response.id.clone()),
                ServerJsonRpcMessage::Error(error) => error.id.clone(),
                _ => None,
            };
            let send = async {
                let mut writer = writer.lock().await;
                if id.as_ref().is_some_and(|id| {
                    !admission
                        .lock()
                        .expect("admission lock")
                        .pending
                        .contains(id)
                }) {
                    // The client cancelled while this response was waiting to write.
                    return Ok(());
                }
                writer.send(message).await.map_err(io::Error::other)
            };
            let result = tokio::select! {
                biased;
                () = closed.cancelled() => Err(io::ErrorKind::BrokenPipe.into()),
                result = tokio::time::timeout(WRITE_DEADLINE, send) => {
                    result.unwrap_or_else(|_| Err(io::ErrorKind::TimedOut.into()))
                }
            };
            if result.is_err() {
                closed.cancel();
            }
            if let Some(id) = id {
                admission
                    .lock()
                    .expect("admission lock")
                    .pending
                    .remove(&id);
            }
            result
        }
    }

    async fn receive(&mut self) -> Option<ClientJsonRpcMessage> {
        loop {
            let line = tokio::select! {
                biased;
                () = self.closed.cancelled() => return None,
                line = self.reader.next() => line?,
            };
            let Ok(line) = line else {
                self.closed.cancel();
                return None;
            };
            if line.is_empty() {
                continue;
            }
            let mut decoder = JsonRpcMessageCodec::<ClientJsonRpcMessage>::default();
            let mut data = BytesMut::from(line.as_bytes());
            data.extend_from_slice(b"\n");
            let message = match unique_json(line.as_bytes()) {
                Ok(()) => decoder.decode(&mut data).ok().flatten(),
                Err(error) if error.is_data() => None,
                Err(_) => continue,
            };
            let Some(message) = message else {
                self.send(ServerJsonRpcMessage::error(
                    ErrorData::invalid_request("Invalid MCP message", None),
                    None,
                ))
                .await
                .ok()?;
                continue;
            };
            match &message {
                ClientJsonRpcMessage::Request(request) => {
                    if !self
                        .admission
                        .lock()
                        .expect("admission lock")
                        .accept(&request.id)
                    {
                        self.closed.cancel();
                        return None;
                    }
                }
                ClientJsonRpcMessage::Notification(notification) => {
                    if let ClientNotification::CancelledNotification(cancelled) =
                        &notification.notification
                        && let Some(id) = &cancelled.params.request_id
                    {
                        self.admission.lock().expect("admission lock").cancel(id);
                    }
                }
                _ => {}
            }
            return Some(message);
        }
    }

    async fn close(&mut self) -> io::Result<()> {
        self.closed.cancel();
        Ok(())
    }
}

pub(crate) fn unique_json(bytes: &[u8]) -> Result<(), serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    UniqueJson(JSON_DEPTH)
        .deserialize(&mut deserializer)
        .and_then(|()| deserializer.end())
}

#[derive(Default)]
struct Admission {
    pending: HashSet<RequestId>,
    cancelled: VecDeque<RequestId>,
}

impl Admission {
    fn accept(&mut self, id: &RequestId) -> bool {
        self.pending.len() < IN_FLIGHT
            && !self.cancelled.contains(id)
            && self.pending.insert(id.clone())
    }

    fn cancel(&mut self, id: &RequestId) {
        if self.pending.remove(id) {
            if self.cancelled.len() == IN_FLIGHT {
                self.cancelled.pop_front();
            }
            self.cancelled.push_back(id.clone());
        }
    }
}
struct FrameLines(LinesCodec);

impl Decoder for FrameLines {
    type Item = String;
    type Error = LinesCodecError;

    fn decode(&mut self, data: &mut BytesMut) -> Result<Option<String>, Self::Error> {
        self.0.decode(data)
    }

    fn decode_eof(&mut self, data: &mut BytesMut) -> Result<Option<String>, Self::Error> {
        let frame = self.0.decode(data)?;
        if frame.is_none() {
            data.clear();
        }
        Ok(frame)
    }
}
struct UniqueJson(u8);

impl<'de> DeserializeSeed<'de> for UniqueJson {
    type Value = ();

    fn deserialize<D: de::Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        if self.0 == 0 {
            return Err(de::Error::custom("MCP JSON depth limit exceeded"));
        }
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for UniqueJson {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("bounded JSON with unique member names")
    }

    fn visit_bool<E: de::Error>(self, _: bool) -> Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: de::Error>(self, _: i64) -> Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: de::Error>(self, _: u64) -> Result<(), E> {
        Ok(())
    }
    fn visit_f64<E: de::Error>(self, _: f64) -> Result<(), E> {
        Ok(())
    }
    fn visit_str<E: de::Error>(self, _: &str) -> Result<(), E> {
        Ok(())
    }
    fn visit_unit<E: de::Error>(self) -> Result<(), E> {
        Ok(())
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<(), M::Error> {
        let mut keys = HashSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key) {
                return Err(de::Error::custom("Duplicate JSON member"));
            }
            map.next_value_seed(UniqueJson(self.0 - 1))?;
        }
        Ok(())
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut sequence: S) -> Result<(), S::Error> {
        while sequence
            .next_element_seed(UniqueJson(self.0 - 1))?
            .is_some()
        {}
        Ok(())
    }
}
