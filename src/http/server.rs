extern crate httparse;
extern crate mioco;

use std::io::{self, Read, Write};
use std::net;
use std::str::FromStr;

use super::client::Client;
use super::request::Request;

pub struct Server<'interface> {
    interface: &'interface str,
    port: u16,
}

impl<'interface> Server<'interface> {
    pub fn new(interface: &'interface str, port: u16) -> Server {
        Server {
            interface: interface,
            port: port,
        }
    }

    pub fn start(&self) -> io::Result<()> {
        let ip = net::IpAddr::from_str(self.interface).unwrap();
        let addr = net::SocketAddr::new(ip, self.port);

        let listener = match mioco::tcp::TcpListener::bind(&addr) {
            Ok(v) => v,
            Err(e) => fatal!("Could not bind listener to port {}: {}", self.port, e)
        };

        println!("Starting tcp echo server on {:?}", try!(listener.local_addr()));

        loop {
            let conn = try!(listener.accept());

            mioco::spawn(move || -> io::Result<()> {
                handle_client(conn);

                Ok(())
            });

            println!("spawned");
        }
    }
}


fn handle_request<'buf, S: Write + Read>(mut stream: &mut S, request: Request<'buf>, mut body: Vec<u8>) {
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

    let client = Client;
    client.forward(&mut stream, request, body);
}

fn handle_client<S: Write + Read>(mut stream: S) {
    let mut buffer = Vec::with_capacity(65536);
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

/// Read from the given stream into the given buffer.
/// Interally this will perform a read for up to 65536 bytes of data, and
/// append it to the end of the given buffer.
fn read_into_buffer<R: Read>(stream: &mut R, buffer: &mut Vec<u8>) -> io::Result<usize> {

    // XXX: it would be nice to benchmark how this compares to reading directly
    // into the given buffer.

    // We do it this way to ensure we only put data into `buffer` that was
    // actually read from the stream, and not accidentally leave the buffer
    // filled with nulls from a resize.

    let mut read_buf = vec![0; 65536];
    match stream.read(&mut read_buf) {
        Ok(n) if n > 0 => {
            unsafe {
                read_buf.set_len(n);
            }
            buffer.append(&mut read_buf);
            Ok(n)
        }
        r => r,
    }
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::read_into_buffer;

    #[test]
    fn test_read_into_buffer() {
        let mut s = Cursor::new("Hello world");
        let mut buf = Vec::with_capacity(5);

        assert_eq!(11, read_into_buffer(&mut s, &mut buf).unwrap());
        assert_eq!(&buf[..], b"Hello world");

        let mut s = Cursor::new("!");

        assert_eq!(1, read_into_buffer(&mut s, &mut buf).unwrap());

        assert_eq!(&buf, b"Hello world!");

        assert_eq!(0, read_into_buffer(&mut s, &mut buf).unwrap());

        assert_eq!(&buf, b"Hello world!");
    }
}
