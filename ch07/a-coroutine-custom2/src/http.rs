use crate::future::{Future, PollState};
use std::{
    io::{ErrorKind, Read, Write},
    mem,
    net::SocketAddr,
    sync::Mutex,
};

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
        Host: localhost\r\n\
        Connection: close\r\n\
        \r\n\
    "
    )
}

/// Not multi-threaded so not mutex needed but let's pretend.
pub static N_POLLS: Mutex<usize> = Mutex::new(0);

pub struct Http;

impl Http {
    pub fn get(path: &str) -> impl Future<Output = String> {
        HttpGetFuture::new(path)
    }
}

struct HttpGetFuture {
    stream: mio::net::TcpStream,
    request: Box<[u8]>,
    n_bytes_written: usize,
    reply: Vec<u8>,
}

impl HttpGetFuture {
    fn new(path: &str) -> Self {
        Self {
            stream: mio::net::TcpStream::connect(SocketAddr::V4("127.0.0.1:8080".parse().unwrap()))
                .unwrap(),
            request: get_req(path).as_bytes().into(),
            n_bytes_written: 0,
            reply: Vec::new(),
        }
    }
}

impl Future for HttpGetFuture {
    type Output = String;

    fn poll(&mut self) -> PollState<Self::Output> {
        *N_POLLS.lock().unwrap() += 1;

        if let Err(e) = self.stream.peer_addr() {
            if e.kind() == ErrorKind::NotConnected {
                println!("TCP conn not ready");
                return PollState::NotReady;
            }

            panic!("error connecting: {e}");
        }

        let buf = &self.request[self.n_bytes_written..];
        // let buf = &buf[..3.min(buf.len())];
        match self.stream.write(buf) {
            Ok(0) => (),
            Ok(n) => {
                println!("Wrote out: {:?}", String::from_utf8_lossy(&buf[..n]));

                self.n_bytes_written += n;
                return PollState::NotReady;
            }
            Err(e) => panic!("write broken: {e}"),
        }

        let mut buf = [0; 4096];
        match self.stream.read(&mut buf) {
            Ok(0) => PollState::Ready(String::from_utf8(mem::take(&mut self.reply)).unwrap()),
            Ok(n) => {
                println!("Read: {:?}", String::from_utf8_lossy(&buf[..n]));
                self.reply.extend(buf[..n].iter());
                PollState::NotReady
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                // eprintln!("read would block: {e}",);
                PollState::NotReady
            }
            Err(e) => {
                panic!("read broken: {e}, {}", e.kind());
            }
        }
    }
}
