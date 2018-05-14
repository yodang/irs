extern crate bytes;
extern crate resolve;
extern crate mio;
extern crate cursive;

use std::net::SocketAddr;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::io::{Read, Write};
use std::io::stdin;
use std::str::FromStr;
use std::collections::VecDeque;
use resolve::resolver;
use mio::net::TcpStream;
use mio::{Events, Ready, Poll, PollOpt, Token};
use cursive::Cursive;
use cursive::view::View;
use cursive::traits::Identifiable;
use cursive::views::*;

mod ircstream;
use ircstream::IrcStream;

mod ircframe;
use ircframe::IrcFrame;

static SERVER_HOST: &'static str = "irc.iiens.net";
static SERVER_PORT: u16 = 6666;

struct BufferView {
    content: VecDeque<String>,
    tx: mpsc::Sender<String>,
    rx: mpsc::Receiver<String>
}

impl BufferView
{
    fn new(size: usize) -> Self
    {
        let mut content=VecDeque::new();
        let (tx, rx)=mpsc::channel();
        content.resize(size, String::default());
        BufferView{
            content,
            tx,
            rx
        }
    }

    fn update(&mut self)
    {
        while let Ok(line) = self.rx.try_recv()
        {
            self.content.push_back(line);
            self.content.pop_front();
        }
    }

    fn get_tx(&self) -> mpsc::Sender<String>
    {
        self.tx.clone()
    }
}

impl View for BufferView
{
    fn layout(&mut self, _: cursive::vec::Vec2)
    {
        self.update();
    }

    fn draw(&self, printer: &cursive::Printer)
    {
        for (i, line) in self.content.iter().rev().take(printer.size.y).enumerate()
        {
            printer.print((0, printer.size.y - (i+1)), line);
        }
    }
}

fn input_cb(tx: &mpsc::Sender<String>, ctx: &mut Cursive, input: &str, streams: &mut Vec<Arc<Mutex<IrcStream>>>)
{
    match input
    {
        "/connect" => 
        {
            streams.push(
                connect(SERVER_HOST, ctx.find_id::<BufferView>("text").unwrap().get_tx()).unwrap()
            );
            ctx.find_id::<EditView>("input").unwrap().set_content("");
        },
        "/quit" => ctx.quit(),
        _ => 
        {
            ctx.find_id::<BufferView>("text").unwrap().get_tx().send(input.to_owned());
            if streams.len()!=0
            {
                streams[0].lock().unwrap().write_all(&format!("{}\r\n", input).into_bytes());
            }
            ctx.find_id::<EditView>("input").unwrap().set_content("");
        }
    }
}

fn main() -> Result<(), std::io::Error>
{
    let mut context=Cursive::default();
    let mut layout=LinearLayout::vertical();
    let (tx, rx)=std::sync::mpsc::channel();
    let mut streams = Vec::default();

    context.set_fps(10);

    context.add_global_callback('q', Cursive::quit);
    layout.add_child(BoxView::with_full_screen(
        BufferView::new(100).with_id("text")
    ));
    layout.add_child(EditView::new()
        .on_submit_mut(move |ctx, i| input_cb(&tx, ctx, i, &mut streams))
        .with_id("input")
    );

    context.add_layer(Panel::new(layout));

    context.run();

    Ok(())
}

fn connect(host: &str, tx: mpsc::Sender<String>) -> Result<Arc<Mutex<IrcStream>>, std::io::Error> {
    let resolved: std::net::IpAddr=resolver::resolve_host(host).unwrap_or_else(|_|panic!("Could not resolve")).last().unwrap();
    let stream=IrcStream::new(TcpStream::connect(&SocketAddr::new(resolved, SERVER_PORT)).unwrap());

    let poller=Poll::new().unwrap();

    let LISTENER=Token::from(1);

    poller.register(&stream, LISTENER, Ready::readable()|Ready::writable(), PollOpt::edge())?;

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
                            tx.send(format!("Received: {:?}", IrcFrame::from_str(&line)));
                            if line.starts_with("PING")
                            {
                                let mut pong="PONG :".to_owned();
                                pong.push_str(line.split(":").last().unwrap());
                                pong.push_str("\r\n");
                                tx.send(format!("Send: {}", pong));
                                stream.write_all(&pong.into_bytes());
                            }
                        }
                    }
                    (_, true, Token(1)) =>
                    {
                        let nick="NICK Yooda \r\n".to_owned();
                        tx.send(format!("Send: {}", nick));
                        stream.write_all(&nick.into_bytes());
                        let user="USER guest 0 * :guest\r\n".to_owned();
                        tx.send(format!("Send: {}", user));
                        stream.write_all(&user.into_bytes());
                    }
                    _ => {println!("Unhandled event {:?}", e);}
                }
            }
        }
    });

    Ok(shared_stream.clone())
}

