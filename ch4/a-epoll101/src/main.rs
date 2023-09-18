use std::io::Result;
use std::os::unix::io::AsRawFd;
use std::net::TcpStream;

mod ffi;
mod net;
mod poll;

fn main() -> Result<()> {
    let poll = poll::Poll::new()?;
    let mut streams = vec![];
    let mut event_counter = 0;
    
    
    let addr = "localhost:8080";
    
    for i in 1..6 {
        let delay = (5 - i) * 1000;
        let request = format!(
            "GET /{}/request-{} HTTP/1.1\r\n\
             Host: localhost\r\n\
             Connection: close\r\n\
             \r\n",
            delay,
            i
        );
        let stream = net::TcpStream::write(addr, request.as_bytes())?;
        poll.registry().register(&stream, i, ffi::EPOLLIN | ffi::EPOLLONESHOT)?;

        streams.push(stream);
        event_counter += 1;
    }

    while event_counter > 0 {
        let mut events = Vec::with_capacity(10);
        poll.poll(&mut events, None)?;
        
        for event in events {
            println!("RECEIVED: {:?}", event);
            event_counter -= 1;
        }
    }

    println!("FINISHED");
    Ok(())
}
