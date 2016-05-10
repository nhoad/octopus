extern crate mioco;

#[macro_use]
extern crate octopus;

use std::io;

fn main() {
    mioco::start(|| -> io::Result<()> {
        let server = octopus::http::server::Server::new("127.0.0.1", 8000);
        server.start()
    }).unwrap().unwrap();
}
