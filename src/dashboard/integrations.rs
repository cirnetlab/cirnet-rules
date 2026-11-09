//! Read-only interoperability with explicitly selected, version-pinned local engines.
use super::storage::{AppError, Empty, decode};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    io,
    sync::mpsc::{self, Receiver, SyncSender, TryRecvError},
    thread::{self, JoinHandle},
};

mod formats;
mod transport;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Adapter {
    OpenHands,
    OpenCode,
}

impl Adapter {
    const fn id(self) -> &'static str {
        match self {
            Self::OpenHands => "openhands",
            Self::OpenCode => "opencode",
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::OpenHands => "OpenHands",
            Self::OpenCode => "OpenCode",
        }
    }

    const fn version(self) -> &'static str {
        match self {
            Self::OpenHands => "1.60.1",
            Self::OpenCode => "2.2.5",
        }
    }

    const fn endpoint(self) -> &'static str {
        match self {
            Self::OpenHands => "/server_info",
            Self::OpenCode => "/api/info",
        }
    }

    fn parse_version(self, body: &[u8]) -> Option<String> {
        let data: Value = serde_json::from_slice(body).ok()?;
        let version = match self {
            Self::OpenHands => data["version"]
                .as_str()
                .or_else(|| data["sdk_version"].as_str())?,
            Self::OpenCode => {
                data["pid"].as_u64()?;
                if !data["urls"].as_array()?.iter().all(Value::is_string) {
                    return None;
                }
                data["paths"]["tmp"].as_str()?;
                data["version"].as_str()?
            }
        }
        .trim();
        let core = version.split(['-', '+']).next()?;
        if version.len() > 48
            || core.split('.').count() != 3
            || !core
                .split('.')
                .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
            || !version
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-+".contains(&b))
        {
            return None;
        }
        Some(version.to_owned())
    }
}

fn capabilities(status: &str, probe: &str) -> Value {
    json!({
        "probe":probe,"create_session":status,"instructions":status,"skills":status,
        "models":status,"tools":status,"approvals":status,"streaming":status,"resume":status,
        "isolation":"BLOCKED","diff":"BLOCKED","usage":"BLOCKED"
    })
}

pub(super) fn inventory() -> Value {
    let mut adapters: Vec<Value> = [Adapter::OpenHands, Adapter::OpenCode]
        .into_iter()
        .map(|adapter| {
            json!({
                "id":adapter.id(),"name":adapter.name(),"version":adapter.version(),
                "dialect": match adapter {
                    Adapter::OpenHands => "Agent Server",
                    Adapter::OpenCode => "Server V2",
                },
                "capabilities":capabilities("BLOCKED", "AVAILABLE"),
                "reason":"The version probe is read-only. Governed native execution requires verified isolation and approval enforcement."
            })
        })
        .collect();
    for (id, name) in [
        ("codex", "Codex"),
        ("claude", "Claude"),
        ("antigravity", "Antigravity"),
    ] {
        adapters.push(json!({
            "id":id,"name":name,"version":null,"dialect":"Portable handoff",
            "capabilities":capabilities("BLOCKED", "BLOCKED"),"handoff":"HANDOFF_ONLY",
            "reason":"The local runtime can prepare a handoff. A native execution backend is unavailable."
        }));
    }
    json!({"adapters":adapters})
}

pub(crate) fn dispatch(method: &str, params: Value) -> Result<Value, AppError> {
    match method {
        "integration.list" => {
            let _: Empty = decode(params)?;
            Ok(inventory())
        }
        "integration.import_preview" => formats::preview(params),
        _ => Err(AppError::new(
            "METHOD_NOT_FOUND",
            "Unknown integration operation.",
        )),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Probe {
    adapter: Adapter,
    port: u16,
}

impl Probe {
    fn parse(params: Value) -> Result<Self, AppError> {
        let probe: Self = decode(params)?;
        if probe.port == 0 {
            return Err(AppError::new(
                "INVALID_PARAMS",
                "Select a port between 1 and 65535.",
            ));
        }
        Ok(probe)
    }

    fn run(self) -> Value {
        let expected_version = self.adapter.version();
        let mut view = json!({
            "adapter":self.adapter.id(),"reachable":false,"authentication":"NOT_VERIFIED",
            "contract":"UNAVAILABLE","expected_version":expected_version,"native_execution":"BLOCKED"
        });
        let response = match transport::get(self.port, self.adapter.endpoint()) {
            Ok(response) => response,
            Err(transport::Failure::Connect) => return view,
            Err(transport::Failure::Response) => {
                view["reachable"] = json!(true);
                view["contract"] = json!("INVALID_RESPONSE");
                return view;
            }
        };
        view["reachable"] = json!(true);
        view["contract"] = json!(match response.status {
            401 | 403 => {
                view["authentication"] = json!("REQUIRED");
                "AUTH_REQUIRED"
            }
            200 => match self.adapter.parse_version(&response.body) {
                Some(version) => {
                    let matches = version == expected_version;
                    view["version"] = json!(version);
                    if matches {
                        "MATCHED"
                    } else {
                        "VERSION_MISMATCH"
                    }
                }
                None => "INVALID_RESPONSE",
            },
            _ => "HTTP_ERROR",
        });
        view
    }
}

pub(crate) struct ProbeQueue {
    commands: Option<SyncSender<Probe>>,
    results: Receiver<Value>,
    worker: Option<JoinHandle<()>>,
    pending: Option<String>,
}

impl ProbeQueue {
    pub(crate) fn new() -> io::Result<Self> {
        let (commands, probes) = mpsc::sync_channel::<Probe>(1);
        let (results, completed) = mpsc::sync_channel(1);
        let worker = thread::Builder::new()
            .name("openrules-integration-probe".into())
            .spawn(move || {
                for probe in probes {
                    if results.send(probe.run()).is_err() {
                        break;
                    }
                }
            })?;
        Ok(Self {
            commands: Some(commands),
            results: completed,
            worker: Some(worker),
            pending: None,
        })
    }

    pub(crate) fn busy(&self) -> bool {
        self.pending.is_some()
    }

    pub(crate) fn submit(&mut self, id: &str, params: Value) -> Result<(), AppError> {
        let probe = Probe::parse(params)?;
        if self.busy() {
            return Err(AppError::new(
                "PROBE_BUSY",
                "Wait for the current integration probe to finish.",
            ));
        }
        self.commands
            .as_ref()
            .expect("commands are present while the queue is live")
            .try_send(probe)
            .map_err(|_| {
                AppError::new(
                    "PROBE_UNAVAILABLE",
                    "The integration worker is unavailable.",
                )
            })?;
        self.pending = Some(id.to_owned());
        Ok(())
    }

    pub(crate) fn reap(&mut self) -> Option<(String, Result<Value, AppError>)> {
        let result = match self.results.try_recv() {
            Ok(value) => Ok(value),
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => Err(AppError::new(
                "PROBE_UNAVAILABLE",
                "The integration worker stopped.",
            )),
        };
        self.pending.take().map(|id| (id, result))
    }
}

impl Drop for ProbeQueue {
    fn drop(&mut self) {
        self.commands.take();
        if let Some(worker) = self.worker.take() {
            // Only one probe can be admitted. Its socket has an overall deadline.
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests;
