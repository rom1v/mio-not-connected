extern crate mio;

use std::io::{ErrorKind, Read};
use std::net::{Ipv4Addr, SocketAddr};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use mio::{Events, Poll, PollOpt, Ready, Token};

fn main() {
    let poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(16);
    let mut buf = [0; 4096];

    for i in 0..2 {
        println!("[{}] Attempt {}...", timestamp(), i);

        // some random yahoo.com IP
        let addr = SocketAddr::new(Ipv4Addr::new(98, 139, 180, 149).into(), 80);
        let mut stream = mio::tcp::TcpStream::connect(&addr).unwrap();
        let addr = SocketAddr::new(Ipv4Addr::new(98, 138, 253, 109).into(), 80);
        let mut stream2 = mio::tcp::TcpStream::connect(&addr).unwrap();

        let timeout = Some(Duration::from_secs(1));

        println!("[{}] Interest = {{writable}}", timestamp());
        poll.register(&stream, Token(i), Ready::writable(), PollOpt::level()).unwrap();
        poll.register(&stream2, Token(42), Ready::readable(), PollOpt::level()).unwrap();

        if cfg!(feature = "pollwritable") {
            // polling here avoids the problem once, but not twice
            do_poll(&poll, &mut stream, &mut events, &mut buf, timeout);
        }

        if cfg!(feature = "temporize") {
            thread::sleep(Duration::from_millis(200));
        }

        //println!("[{}] Interest = {{readable}}", timestamp());
//        poll.reregister(&stream, Token(i), Ready::readable(), PollOpt::level()).unwrap();

        do_poll(&poll, &mut stream, &mut events, &mut buf, timeout);

        println!("[{}] Attempt {} complete", timestamp(), i);
        poll.deregister(&stream).unwrap();
    }
}

fn do_poll(poll: &Poll, stream: &mut mio::tcp::TcpStream, events: &mut Events, buf: &mut [u8], timeout: Option<Duration>) {
    poll.poll(events, timeout).unwrap();
    for event in &*events {
        println!("[{}] event={:?}", timestamp(), event);
        if event.readiness().is_readable() {
            match stream.read(&mut buf[..]) {
                Ok(0) => {
                    println!("EOF");
                }
                Ok(len) => {
                    let content = String::from_utf8_lossy(&buf[..len]);
                    println!("content: {}", content);
                }
                Err(err) => {
                    if err.kind() == ErrorKind::WouldBlock {
                        println!("spurious event");
                    } else {
                        println!("error: [{:?}]: {}", err.kind(), err);
                        return;
                    }
                }
            }
        }
    }
}

fn timestamp() -> String {
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).unwrap();
    format!("{}.{}", duration.as_secs() * 1000, duration.subsec_nanos() / 1000000)
}
