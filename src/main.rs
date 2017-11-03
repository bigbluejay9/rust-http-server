extern crate getopts;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate mio;

use std::io::BufReader;
use std::io::Write;
use getopts::Options;
use mio::*;
use mio::net::{TcpListener, TcpStream};
use std::io::BufRead;
use std::env;
use std::net::SocketAddr;
use std::{time, thread, str};

const SERVER: Token = Token(0);

fn print_usage(prog: &str, opts: Options) {
    let brief = format!("Usage: {} LISTEN-ADDRESS", prog);
    print!("{}", opts.usage(&brief));
}

fn start_server(addr: &SocketAddr) -> Result<(), std::io::Error> {
    let server = try!(TcpListener::bind(addr));
    let poll = try!(Poll::new());
    let mut events = Events::with_capacity(1024);
    poll.register(
        &server,
        SERVER,
        Ready::readable(),
        PollOpt::edge(),
    )?;
    debug!("Listening on {}.", addr);

    loop {
        poll.poll(&mut events, None)?;

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

fn handle_connection((mut c, _remote): (TcpStream, SocketAddr)) {
    debug!("Accepted on {:?}.", c);

    let mut line = String::new();
    {
        let mut reader = BufReader::new(&c);
        loop {
            match reader.read_line(&mut line) {
                Ok(read) => {
                    if read <= 2 {
                        break;
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(time::Duration::from_millis(10));
                }
                Err(e) => panic!("Can't read: {}.", e.to_string()),
            };
        }
        debug!("Read {} bytes: {}", line.len(), line);
    }

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
        Ok(m) => m,
        Err(f) => {
            panic!(f.to_string());
        }
    };

    if matches.opt_present("h") || matches.free.len() != 1 {
        print_usage(&program, opts);
        return;
    }

    let addr = match matches.free[0]
        .replace("localhost", "127.0.0.1")
        .parse::<SocketAddr>() {
        Err(e) => {
            println!(
                "Bad address to listen on {}: {}.",
                matches.free[0],
                e.to_string()
            );
            return;
        }
        Ok(a) => a,
    };

    match start_server(&addr) {
        Err(e) => {
            error!("Server loop failed: {}.", e.to_string());
        }
        _ => {}
    }
}
