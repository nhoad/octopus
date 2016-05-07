#[macro_use]
extern crate mioco;

#[macro_use]
extern crate octopus;

use std::str::FromStr;
use std::io;
use std::net;

use mioco::tcp::TcpListener;

use octopus::server::handle_client;

fn main() {
    mioco::start(|| -> io::Result<()> {
        let ip = net::IpAddr::from_str("127.0.0.1").unwrap();
        let port = 8000;
        let addr = net::SocketAddr::new(ip, port);

        let listener = match TcpListener::bind(&addr) {
            Ok(v) => v,
            Err(e) => fatal!("Could not bind listener to port {}: {}", port, e)
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
    }).unwrap().unwrap();

}
