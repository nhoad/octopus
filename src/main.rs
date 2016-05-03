extern crate http_muncher;
extern crate url;

use std::collections::HashMap;
use std::io::prelude::*;
use std::mem;
use std::net::{TcpListener, TcpStream};
use std::thread;

use url::{Url, ParseError};

use http_muncher::{Parser, ParserHandler};

struct HttpParser {
    current_key: Option<String>,
    headers_complete: bool,
    headers: HashMap<String, String>,
    url: Option<String>
}


impl HttpParser {
    pub fn new() -> HttpParser {
        HttpParser {
            url: None,
            current_key: None,
            headers_complete: false,
            headers: HashMap::new(),
        }
    }
}

impl HttpParser {
    fn request(&mut self) -> Request {
        // FIXME: benchmark this.
        let unparsed = mem::replace(&mut self.url, None).unwrap();
        let url = match Url::parse(&unparsed) {
            Ok(v) => v,
            Err(ParseError::RelativeUrlWithoutBase) => {
                // FIXME: don't assume http! Gross! Scheme can be derived from
                // what the listening port is expecting.
                let mut s: String = "http://".into();
                s.push_str(self.headers.get("Host").expect("Host header was missing"));
                s.push_str(&unparsed);
                println!("full url is {}", s);
                Url::parse(&s).unwrap()
            },
            Err(s) => panic!(s),
        };

        Request {
            url: url,
            headers: mem::replace(&mut self.headers, HashMap::new()),
        }
    }
}

// FIXME: Would this be better as a state handler than a parser handler? i.e.
// make it connect upstream and all that?
impl ParserHandler for HttpParser {

    fn on_url(&mut self, url: &[u8]) -> bool {
        self.url = Some(std::str::from_utf8(url).unwrap().to_string());
        true
    }

    fn on_header_field(&mut self, s: &[u8]) -> bool {
        self.current_key = Some(std::str::from_utf8(s).unwrap().to_string());
        true
    }

    fn on_header_value(&mut self, header: &[u8]) -> bool {
        // FIXME: this can be called multiple times for large values?
        // https://github.com/nbaksalyar/rust-streaming-http-parser/issues/4
        let key = mem::replace(&mut self.current_key, None).unwrap();
        self.headers.insert(
            key,
            std::str::from_utf8(header).unwrap().to_string());
        true
    }

    fn on_headers_complete(&mut self) -> bool {
        self.headers_complete = true;
        false
    }

    //fn on_status(&mut self, status: &[u8]) -> bool { false }
    //fn on_body(&mut self, body: &[u8]) -> bool { false }
    //fn on_message_begin(&mut self) -> bool { false }
    //fn on_message_complete(&mut self) -> bool { false }
    //fn on_chunk_header(&mut self) -> bool { false }
    //fn on_chunk_complete(&mut self) -> bool { false }
}

#[derive(Debug)]
struct Request {
    url: Url,
    headers: HashMap<String, String>,
}

impl Request {
    fn write_reqline(&self, method: &'static str, version: (u16, u16), stream: &mut TcpStream) {
        let (major, minor) = version;
        // FIXME: cargo run --release makes the method "<unknown>" here?
        let reqline = format!("{} {} HTTP/{}.{}\r\n", method, self.url.path(), major, minor);
        println!("writing {:?}", reqline);
        stream.write_all(&(reqline).as_bytes()).unwrap();
    }

    fn write_headers(&self, stream: &mut TcpStream) {
        // iterate over everything.
        for (key, value) in &self.headers {
            let header = format!("{}: {}\r\n", key, value);
            stream.write_all(&(header).as_bytes()).unwrap();
        }
        stream.write_all(b"\r\n").unwrap();
    }

    fn port(&self) -> u16 {
        match self.url.port() {
            Some(port) => port,
            None => {
                match self.url.scheme() {
                    "http" => 80,
                    "https" => 443,
                    _ => panic!("Unknown scheme {}", self.url.scheme())
                }
            }
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 65536];
    let mut parser = Parser::request(HttpParser::new());

    loop {
        let s = stream.read(&mut buffer[..]).unwrap();
        if s == 0 {
            // FIXME: handle this better.
            println!("empty read, bailing");
            break
        }
        println!("feeding {} bytes", s);
        parser.parse(&buffer);

        let handler = parser.get();
        if handler.headers_complete {
            break
        } else {
            println!("not complete yet");
        }
    }

    let http_version = parser.http_version();
    let http_method = parser.http_method();
    println!("method is {}, {:?}", http_method, http_version);
    let handler = parser.get();

    assert!(handler.headers_complete);

    let request = handler.request();

    println!("request {:?}", request);

    let domain = request.url.host_str().unwrap();

    println!("Connecting upstream");

    // FIXME: perform and cache DNS lookup for the domain.
    let mut upstream = TcpStream::connect((domain, request.port())).unwrap();
    println!("Connected upstream");

    request.write_reqline(http_method, http_version, &mut upstream);
    request.write_headers(&mut upstream);

    // FIXME: check for a request body and start streaming that, using the
    // parser

    // FIXME: do something more sophisticated, involving parsing the response
    splice(&mut upstream, &mut stream);
}

fn splice(sender: &mut TcpStream, receiver: &mut TcpStream) {
    let mut buffer = [0; 65536];

    loop {
        let n = sender.read(&mut buffer[..]).unwrap();

        if n > 0 {
            match receiver.write_all(&buffer[..n]) {
                Ok(..) => {},
                Err(e) => {
                    println!("error writing to client {}", e);
                    return;
                }
            }
        } else {
            println!("sender went away");
            break
        }
    }

    receiver.flush().unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move|| {
                    // connection succeeded
                    handle_client(stream)
                });
            }
            Err(e) => { /* connection failed */ }
        }
    }

    // close the socket server
    drop(listener);
}
