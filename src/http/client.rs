extern crate mioco;
extern crate url;

use std::io::{self, Write, Read};
use std::net::ToSocketAddrs;

use super::request::Request;
pub struct Client;

impl Client {
    pub fn forward<S: Write>(&self, downstream: &mut S, request: Request, body: Vec<u8>) {
        match self.connect(&request.url) {
            Ok(mut upstream) => {
                let serialized: Vec<u8> = request.into();
                upstream.write_all(&serialized).unwrap();
                upstream.write_all(&body).unwrap();

                let mut buffer = [0; 65536];

                // FIXME: actually parse the response here.
                loop {
                    match upstream.read(&mut buffer) {
                        Ok(0) => {
                            break
                        },
                        Ok(n) => {
                            downstream.write_all(&buffer[..n]).unwrap();
                        },
                        Err(e) => {
                            println!("Error {}", e);
                            break
                        }
                    }
                }
            },
            Err(e) => {
                println!("Error connecting upstream: {}", e);
                downstream.write_all(b"HTTP/1.1 501 Internal Server Error\r\nContent-Length: 6\r\n\r\nSorry\n").unwrap();
            }
        }
    }

    pub fn connect(&self, url: &url::Url) -> io::Result<mioco::tcp::TcpStream> {
        // FIXME: actual async DNS would be nice?
        for addrs in url.to_socket_addrs() {
            // Extract std::net::SocketAddr for this set
            for addr in addrs {
                match mioco::tcp::TcpStream::connect(&addr) {
                    Ok(conn) => {
                        return Ok(conn);
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }
        }

        Err(io::Error::new(io::ErrorKind::NotFound, "No suitable host could be found"))
    }
}
