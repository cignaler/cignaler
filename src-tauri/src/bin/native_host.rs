use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Request {
    action: String,
    name: String,
    project: String,
    #[serde(rename = "ref")]
    reference: String,
    ci_server: String,
}

#[derive(Serialize)]
struct IpcRequest<'a> {
    action: &'a str,
    name: &'a str,
    project: &'a str,
    #[serde(rename = "ref")]
    reference: &'a str,
    ci_server: &'a str,
}

#[derive(Serialize, Deserialize)]
struct Response {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn read_message() -> io::Result<Vec<u8>> {
    let mut len_bytes = [0u8; 4];
    io::stdin().read_exact(&mut len_bytes)?;
    let len = u32::from_ne_bytes(len_bytes) as usize;
    let mut buf = vec![0u8; len];
    io::stdin().read_exact(&mut buf)?;
    Ok(buf)
}

fn write_message(msg: &[u8]) -> io::Result<()> {
    let len = (msg.len() as u32).to_ne_bytes();
    io::stdout().write_all(&len)?;
    io::stdout().write_all(msg)?;
    io::stdout().flush()?;
    Ok(())
}

fn send_response(success: bool, error: Option<String>) {
    let resp = Response { success, error };
    let json = serde_json::to_vec(&resp).unwrap_or_else(|_| b"{}".to_vec());
    let _ = write_message(&json);
}

fn get_socket_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("com.ostwi.dev").join("cignaler.sock"))
}

fn send_to_app(req: &Request) -> Result<Response, String> {
    let socket_path = get_socket_path().ok_or("Could not resolve socket path")?;

    let mut stream = UnixStream::connect(&socket_path)
        .map_err(|e| format!("Failed to connect to Cignaler app (is it running?): {}", e))?;

    // Send request as JSON line
    let ipc_req = IpcRequest {
        action: &req.action,
        name: &req.name,
        project: &req.project,
        reference: &req.reference,
        ci_server: &req.ci_server,
    };
    let json = serde_json::to_string(&ipc_req)
        .map_err(|e| format!("Failed to serialize request: {}", e))?;

    stream
        .write_all(json.as_bytes())
        .map_err(|e| format!("Failed to send request: {}", e))?;
    stream
        .write_all(b"\n")
        .map_err(|e| format!("Failed to send newline: {}", e))?;

    // Read response
    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let response: Response = serde_json::from_str(&response_line)
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(response)
}

fn run() -> Result<Response, String> {
    let data = read_message().map_err(|e| format!("Failed to read message: {}", e))?;
    let req: Request =
        serde_json::from_slice(&data).map_err(|e| format!("Invalid JSON: {}", e))?;

    if req.action != "add-watcher" {
        return Err(format!("Unknown action: {}", req.action));
    }

    send_to_app(&req)
}

fn main() {
    match run() {
        Ok(response) => send_response(response.success, response.error),
        Err(e) => send_response(false, Some(e)),
    }
}
