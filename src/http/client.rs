extern crate mioco;
extern crate url;

use std::io::{self, Write, Read};
use std::net;
use std::str::FromStr;

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
            Err(_) => {
                downstream.write_all(b"HTTP/1.1 501 Internal Server Error\r\nContent-Length: 6\r\n\r\nSorry\n").unwrap();
            }
        }
    }

    pub fn connect(&self, url: &url::Url) -> io::Result<mioco::tcp::TcpStream> {
        let domain = url.host_str().unwrap();
        let port = match url.port() {
            Some(port) => port,
            None => {
                match url.scheme() {
                    "http" => 80,
                    "https" => 443,
                    _ => panic!("Unknown scheme {}", url.scheme())
                }
            }
        };

        // FIXME: DNS Lookup. net::lookup_addrs is unstable and also blocking.
        let ip = net::IpAddr::from_str(domain).unwrap();
        let addr = net::SocketAddr::new(ip, port);

        // FIXME: configurable retry count
        for _ in 0..2 {
            match mioco::tcp::TcpStream::connect(&addr) {
                Ok(conn) => {
                    return Ok(conn);
                },
                Err(e) => {
                    println!("failed to connect: {}", e);
                }
            }
        }

        mioco::tcp::TcpStream::connect(&addr)
    }
}
