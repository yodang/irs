extern crate bytes;
extern crate resolve;
extern crate mio;

use std::net::SocketAddr;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::io::{Read, Write, Result};
use std::str::FromStr;
use resolve::resolver;
use mio::net::TcpStream;
use mio::{Events, Ready, Poll, PollOpt, Token};

mod ircstream;
use ircstream::IrcStream;

mod ircframe;
use ircframe::IrcFrame;

static SERVER_HOST: &'static str = "irc.iiens.net";
static SERVER_PORT: u16 = 6666;

fn main() {
    let resolved: std::net::IpAddr=resolver::resolve_host(SERVER_HOST).unwrap_or_else(|_|panic!("Could not resolve")).last().unwrap();
    let stream=IrcStream::new(TcpStream::connect(&SocketAddr::new(resolved, SERVER_PORT)).unwrap());

    let poller=Poll::new().unwrap();

    let LISTENER=Token::from(1);

    poller.register(&stream, LISTENER, Ready::readable()|Ready::writable(), PollOpt::edge());

    let shared_stream=Arc::new(Mutex::new(stream));
    let rx_stream=shared_stream.clone();
    thread::spawn(move ||
    {
        let mut events=Events::with_capacity(1024);
        
        loop {
            poller.poll(&mut events, None);

            for e in &events
            {
                let mut stream=rx_stream.lock().unwrap();
                match (e.readiness().is_readable(), e.readiness().is_writable(), e.token())
                {
                    (true, _, Token(1)) =>
                    {
                        //let mut data=String::new();
                        //rx_stream.lock().unwrap().read_to_string(&mut data);
                        //println!("Received (writable {}): {}", e.readiness().is_writable(), data);
                        for line in stream.read_frames().unwrap()
                        {
                            println!("Received: {:?}", IrcFrame::from_str(&line));
                            if line.starts_with("PING")
                            {
                                let mut pong="PONG :".to_owned();
                                pong.push_str(line.split(":").last().unwrap());
                                pong.push_str("\r\n");
                                println!("Send: {}", pong);
                                stream.write_all(&pong.into_bytes());
                            }
                        }
                    }
                    (_, true, Token(1)) =>
                    {
                        let nick="NICK Yooda \r\n".to_owned();
                        println!("Send: {}", nick);
                        stream.write_all(&nick.into_bytes());
                        let user="USER guest 0 * :guest\r\n".to_owned();
                        println!("Send: {}", user);
                        stream.write_all(&user.into_bytes());
                    }
                    _ => {println!("Unhandled event {:?}", e);}
                }
            }
        }
    }).join();
}

