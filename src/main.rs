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
    rx: mpsc::Receiver<String>
}

impl BufferView
{
    fn new(size: usize, rx: mpsc::Receiver<String>) -> Self
    {
        let mut content=VecDeque::new();
        content.resize(size, String::default());
        BufferView{
            content,
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
}

fn writer(s: Arc<Mutex<IrcStream>>)
{
    loop {
        let mut line=String::new();
        stdin().read_line(&mut line);
        line.push_str("\r\n");
        s.lock().unwrap().write_all(&line.into_bytes());
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

fn input_cb(tx: &mpsc::Sender<String>, ctx: &mut Cursive, input: &str)
{
    match input
    {
        "/quit" => ctx.quit(),
        _ => 
        {
            tx.send(input.to_owned());
            ctx.find_id::<EditView>("input").unwrap().set_content("");
        }
    }
}

fn main() -> Result<(), std::io::Error>
{
    let mut context=Cursive::default();
    let mut layout=LinearLayout::vertical();
    let (tx, rx)=std::sync::mpsc::channel();

    context.add_global_callback('q', Cursive::quit);
    layout.add_child(BoxView::with_full_screen(
        BufferView::new(100, rx).with_id("text")
    ));
    layout.add_child(EditView::new()
        .on_submit_mut(move |ctx, i| input_cb(&tx, ctx, i))
        .with_id("input")
    );

    context.add_layer(Panel::new(layout));

    context.run();

    Ok(())
}

fn main_old() {
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
    });
    let tx_stream=shared_stream.clone();
    thread::spawn(move || writer(tx_stream)).join();
}

