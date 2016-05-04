extern crate httparse;
extern crate octopus;

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

use octopus::request::Request;

fn read_into_buffer<R: Read>(stream: &mut R, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
    let len = buffer.len();
    let capacity = buffer.capacity();

    let resize_amount = 4096;

    match len {
        0 => {
            println!("buffer is empty, growing {} -> {}", len, len + resize_amount);
            buffer.resize(len + resize_amount, 0);
        },
        _ if len == capacity => {
            println!("buffer is at capacity, growing {} -> {}", len, len + resize_amount);
            buffer.resize(len + resize_amount, 0);
        }
        _ => {
            assert!(0 < len && len < capacity);
        }
    }

    stream.read(&mut buffer[len..])
}

fn handle_request<'buf, S: Write + Read>(stream: &mut S, request: Request<'buf>, body: Vec<u8>) {
    let mut body = body;

    match request.headers.content_length() {
        Some(n) => {
            if body.len() == n {
                println!("We have it all, no need to read");
            } else if body.len() > n {
                println!("We have too much, what?!");
            } else {
                println!("We have {} bytes of {}", body.len(), n);
                // FIXME: What if it's a 50gb upload! It should read up to a
                // maximum of 65535 bytes or something, otherwise stream it.
                let i = body.len();

                body.reserve(n);
                body.resize(n, 0);
                stream.read_exact(&mut body[i..n-i]).unwrap();
            }
        },
        None => {
            assert!(body.len() == 0);
        }
    }

    println!("Handle this: {:?} {:?}", request, request.headers.content_length());

    // XXX: definitely 1.0 candidate code.
    stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nCool: header\r\n\r\nhey dude").unwrap();
    stream.flush().unwrap();
}

fn handle_client(mut stream: TcpStream) {
    let default_size = 65535;
    let mut buffer = Vec::with_capacity(default_size);
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut total_read = 0;

    loop {
        match read_into_buffer(&mut stream, &mut buffer) {
            Ok(0) => {
                println!("empty read, bailing");
                return
            },
            Ok(n) => {
                total_read += n;
                println!("Did a read {}", n);
            },
            Err(e) => {
                println!("Error occurred while reading {}", e);
                return;
            }
        }

        {
            let mut request = httparse::Request::new(&mut headers);
            match request.parse(&buffer).unwrap() {
                httparse::Status::Complete(n) => {
                    let body = buffer[n..total_read].iter().cloned().collect();
                    let request = Request::from_raw(request);
                    handle_request(&mut stream, request, body);
                },
                _ => {
                    continue;
                }
            }
        }

        // reset the buffer so we have a clean slate for keep-alive.
        buffer.truncate(0);
    }
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
