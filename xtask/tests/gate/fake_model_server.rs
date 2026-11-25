use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::JoinHandle;

pub enum Script {
    Respond(String),
    Status(u16),
    StallAfterHeaders,
}

pub struct FakeModelServer {
    pub url: String,
    handle: JoinHandle<usize>,
}

impl FakeModelServer {
    pub fn start(scripts: Vec<Script>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/chat/completions", listener.local_addr().unwrap());
        let handle = std::thread::spawn(move || serve_all(&listener, scripts));
        Self { url, handle }
    }

    pub fn finish(self) -> usize {
        self.handle.join().unwrap()
    }
}

fn serve_all(listener: &TcpListener, scripts: Vec<Script>) -> usize {
    let count = scripts.len();
    let workers: Vec<JoinHandle<()>> = scripts
        .into_iter()
        .map(|script| {
            let (stream, _) = listener.accept().unwrap();
            std::thread::spawn(move || serve_one(stream, &script))
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
    count
}

fn serve_one(mut stream: TcpStream, script: &Script) {
    drain_request(&mut stream);
    match script {
        Script::Respond(body) => write_response(&mut stream, 200, body),
        Script::Status(status) => write_response(&mut stream, *status, "{}"),
        Script::StallAfterHeaders => stall(&mut stream),
    }
}

fn drain_request(stream: &mut TcpStream) {
    let mut received = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        let read = stream.read(&mut buffer).unwrap();
        received.extend_from_slice(&buffer[..read]);
        if request_is_complete(&received) {
            return;
        }
    }
}

fn request_is_complete(received: &[u8]) -> bool {
    let text = String::from_utf8_lossy(received);
    let Some((headers, body)) = text.split_once("\r\n\r\n") else {
        return false;
    };
    let length = headers
        .lines()
        .find_map(|line| {
            line.to_ascii_lowercase()
                .strip_prefix("content-length:")
                .map(|value| value.trim().parse::<usize>().unwrap())
        })
        .unwrap_or(0);
    body.len() >= length
}

fn write_response(stream: &mut TcpStream, status: u16, body: &str) {
    let response = format!(
        "HTTP/1.1 {status} X\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes()).unwrap();
}

fn stall(stream: &mut TcpStream) {
    stream
        .write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n",
        )
        .unwrap();
    stream.flush().unwrap();
    let mut buffer = [0_u8; 64];
    while stream.read(&mut buffer).is_ok_and(|read| read > 0) {}
}

pub fn content_event(text: &str) -> String {
    format!(
        "data: {{\"choices\":[{{\"delta\":{{\"content\":{}}},\"finish_reason\":null}}]}}\n\n",
        serde_json::to_string(text).unwrap()
    )
}

pub fn reasoning_event() -> String {
    "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"thinking\"},\"finish_reason\":null}]}\n\n".to_owned()
}

pub fn finish_event(reason: &str) -> String {
    format!(
        "data: {{\"choices\":[{{\"delta\":{{}},\"finish_reason\":\"{reason}\"}}]}}\n\ndata: [DONE]\n\n"
    )
}
