extern crate mio;

use std::io::{Read, Write, Result};
use std::str::FromStr;
use mio::{Evented, Ready, Poll, PollOpt, Token};
use mio::net::TcpStream;

pub struct IrcStream
{
    sock: TcpStream,
    buf: String,
}

impl IrcStream
{
    pub fn new(sock: TcpStream) -> IrcStream
    {
        return IrcStream
        {
            sock,
            buf: String::new()
        }
    }

    pub fn read_frames(&mut self) -> Result<Vec<String>>
    {
        let mut data=String::new();
        let res=self.sock.read_to_string(&mut data);
        self.buf.push_str(&data);
        //println!("Data read: {}", &self.buf);
        if self.buf.is_empty()
        {
            match res
            {
                Ok(_) => Ok(Vec::new()),
                Err(e) => Err(e)
            }
        }
        else
        {
            let mut vec: Vec<String>=self.buf.split("\r\n").map(|s| String::from_str(s).unwrap()).collect();
            
            //Save the last element which can be an incomplete frame
            self.buf=vec.pop().unwrap_or("".to_owned());

            Ok(vec)
        }
    }
}

impl mio::Evented for IrcStream
{
    fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> Result<()>
    {
        self.sock.register(poll, token, interest, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> Result<()>
    {
        self.sock.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> Result<()>
    {
        self.sock.deregister(poll)
    }
}

impl Write for IrcStream
{
    fn write(&mut self, buf: &[u8]) -> Result<usize>
    {
        self.sock.write(buf)
    }

    fn flush(&mut self) -> Result<()>
    {
        self.sock.flush()
    }
}

impl Read for IrcStream
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>
    {
        self.sock.read(buf)
    }
}

