//! J-18 LEG2 slice 2: synchronous rust-analyzer stdio client implementing
//! `elai_knowledge::lsp_channel::LspChannelAdapter`, called by
//! `crate::knowledge_stack::build_base_pipeline`.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use elai_knowledge::lsp_channel::{LspChannelAdapter, LspChannelError, LspChannelHit};
use serde_json::{json, Value};

pub(crate) struct LspStdioClient {
    binary: String,
    timeout: Duration,
}

impl LspStdioClient {
    /// Default client: rust-analyzer with a timeout from the environment.
    pub(crate) fn new() -> Self {
        let timeout_ms = std::env::var("ELAI_LSP_CHANNEL_TIMEOUT_MS")
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(5_000);
        Self {
            binary: "rust-analyzer".to_owned(),
            timeout: Duration::from_millis(timeout_ms),
        }
    }

}

fn binary_on_path(binary: &str) -> bool {
    #[cfg(windows)]
    let cmd = "where";
    #[cfg(not(windows))]
    let cmd = "which";
    std::process::Command::new(cmd)
        .arg(binary)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn write_message<W: Write>(writer: &mut W, value: &Value) -> Result<(), LspChannelError> {
    let body = serde_json::to_vec(value)
        .map_err(|error| LspChannelError::Internal(format!("serialize message: {error}")))?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())
        .map_err(|error| LspChannelError::Internal(format!("write header: {error}")))?;
    writer
        .write_all(&body)
        .map_err(|error| LspChannelError::Internal(format!("write body: {error}")))?;
    writer
        .flush()
        .map_err(|error| LspChannelError::Internal(format!("flush message: {error}")))
}

fn read_message<R: Read>(reader: &mut BufReader<R>) -> Result<Value, LspChannelError> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        let bytes = reader
            .read_line(&mut line)
            .map_err(|error| LspChannelError::Internal(format!("read header: {error}")))?;
        if bytes == 0 {
            return Err(LspChannelError::Internal("unexpected end of headers".to_owned()));
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        let (name, value) = trimmed
            .split_once(':')
            .ok_or_else(|| LspChannelError::Internal("malformed header".to_owned()))?;
        if name.eq_ignore_ascii_case("Content-Length") {
            content_length = Some(value.trim().parse::<usize>().map_err(|error| {
                LspChannelError::Internal(format!("invalid Content-Length: {error}"))
            })?);
        }
    }
    let length = content_length
        .ok_or_else(|| LspChannelError::Internal("missing Content-Length".to_owned()))?;
    let mut body = vec![0_u8; length];
    reader
        .read_exact(&mut body)
        .map_err(|error| LspChannelError::Internal(format!("read body: {error}")))?;
    serde_json::from_slice(&body)
        .map_err(|error| LspChannelError::Internal(format!("parse JSON: {error}")))
}

fn response_with_id(message: &Value, id: i64) -> bool {
    message.get("id").and_then(Value::as_i64) == Some(id)
}

fn exchange(
    mut stdin: std::process::ChildStdin,
    stdout: std::process::ChildStdout,
    query: String,
    project_root: String,
    limit: usize,
    child_ref: Arc<Mutex<Child>>,
) -> Result<Vec<LspChannelHit>, LspChannelError> {
    let mut reader = BufReader::new(stdout);
    let result = (|| {
        write_message(
            &mut stdin,
            &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":format!("file://{}", project_root),"capabilities":{}}}),
        )?;
        loop {
            if response_with_id(&read_message(&mut reader)?, 1) { break; }
        }
        write_message(&mut stdin, &json!({"jsonrpc":"2.0","method":"initialized","params":{}}))?;
        write_message(
            &mut stdin,
            &json!({"jsonrpc":"2.0","id":2,"method":"workspace/symbol","params":{"query":query}}),
        )?;
        let reply = loop {
            let message = read_message(&mut reader)?;
            if response_with_id(&message, 2) { break message; }
        };
        if let Some(error) = reply.get("error") {
            return Err(LspChannelError::Internal(format!("server error: {error}")));
        }
        let mut hits = Vec::new();
        if let Some(items) = reply.get("result").and_then(Value::as_array) {
            for item in items {
                let Some(name) = item.get("name").and_then(Value::as_str) else { continue };
                let Some(uri) = item.pointer("/location/uri").and_then(Value::as_str) else { continue };
                let Some(line) = item.pointer("/location/range/start/line").and_then(Value::as_u64) else { continue };
                let Ok(line) = u32::try_from(line) else { continue };
                hits.push(LspChannelHit { symbol: name.to_owned(), path: uri.strip_prefix("file://").unwrap_or(uri).to_owned(), line });
            }
        }
        hits.truncate(limit);
        Ok(hits)
    })();
    let _ = write_message(&mut stdin, &json!({"jsonrpc":"2.0","id":3,"method":"shutdown","params":null}));
    let _ = write_message(&mut stdin, &json!({"jsonrpc":"2.0","method":"exit","params":null}));
    drop(stdin);
    if let Ok(mut child) = child_ref.lock() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

impl LspChannelAdapter for LspStdioClient {
    fn workspace_symbol(
        &self,
        query: &str,
        project_root: &Path,
        limit: usize,
    ) -> Result<Vec<LspChannelHit>, LspChannelError> {
        if query.trim().is_empty() {
            return Err(LspChannelError::InvalidInput("query must not be empty".to_owned()));
        }
        if !binary_on_path(&self.binary) {
            return Err(LspChannelError::ClientUnavailable("rust-analyzer not on PATH".to_owned()));
        }
        let child = Command::new(&self.binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| LspChannelError::ClientUnavailable(format!("spawn failed: {error}")))?;
        let mut child = child;
        let stdin = child.stdin.take().ok_or_else(|| LspChannelError::Internal("server stdin unavailable".to_owned()))?;
        let stdout = child.stdout.take().ok_or_else(|| LspChannelError::Internal("server stdout unavailable".to_owned()))?;
        let child_ref = Arc::new(Mutex::new(child));
        let (sender, receiver) = mpsc::channel();
        let query = query.to_owned();
        let project_root = project_root.display().to_string();
        let child_for_worker = child_ref.clone();
        std::thread::spawn(move || {
            let result = exchange(stdin, stdout, query, project_root, limit, child_for_worker);
            let _ = sender.send(result);
        });
        match receiver.recv_timeout(self.timeout) {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Ok(mut child) = child_ref.lock() {
                    let _ = child.kill();
                }
                let ms = self.timeout.as_millis();
                Err(LspChannelError::Timeout(format!("{ms} ms")))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                Err(LspChannelError::Internal("worker disconnected".to_owned()))
            }
        }
    }
}
