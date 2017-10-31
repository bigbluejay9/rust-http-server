extern crate getopts;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate mio;

use std::io::*;
use getopts::Options;
use mio::*;
use mio::net::{TcpListener, TcpStream};
use std::env;
use std::net::SocketAddr;
use std::str;

const SERVER: Token = Token(0);

fn print_usage(prog: &str, opts: Options) {
  let brief = format!("Usage: {} LISTEN-ADDRESS", prog);
  print!("{}", opts.usage(&brief));
}

fn start_server(addr: &SocketAddr) -> Result<std::io::Error> {
  let server = try!(TcpListener::bind(addr));
  let poll = try!(Poll::new());
  let mut events = Events::with_capacity(1024);
  poll.register(&server, SERVER, Ready::readable(), PollOpt::edge());
  debug!("Listening on {}.", addr);

  loop {
    poll.poll(&mut events, None);

    for event in events.iter() {
      match event.token() {
        SERVER => {
          handle_connection(server.accept().unwrap());
        }
        _ => unreachable!(),
      }
    }
  }
}

fn handle_connection((mut c, remote): (TcpStream, SocketAddr)) {
  debug!("Accepted on {:?}.", c);

  let mut buf = [0; 512];
  let read_bytes = c.read(&mut buf).unwrap();
  debug!("Read {} bytes: {}", read_bytes, str::from_utf8(&buf).unwrap());

  let response = "HTTP/1.1 200 OK\r\n\r\n";
  debug!("Responding with: {}", response);
  c.write(response.as_bytes()).unwrap();
  c.flush().unwrap();
}

fn main() {
  env_logger::init().unwrap();

  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();

  let mut opts = Options::new();
  opts.optflag("h", "help", "print help menu");
  let matches = match opts.parse(&args[1..]) {
    Ok(m) => {m}
    Err(f) => { panic!(f.to_string()); }
  };

  if matches.opt_present("h") || matches.free.len() != 1 {
    print_usage(&program, opts);
    return;
  }

  let addr: SocketAddr;
  match matches.free[0].parse::<SocketAddr>() {
    Err(e) => {
      println!("Bad address to listen on {}: {}.", matches.free[0],
        e.to_string());
      return;
    }
    Ok(a) => { addr = a; }
  }

  match start_server(&addr) {
    Err(e) => { error!("Server loop failed: {}.", e.to_string()); }
    _ => {}
  }
}
