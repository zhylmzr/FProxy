/// A simple http server.
/// Use:
/// ```bash
/// cargo run --example http_server
/// open browser locate to "http://localhost:2233"
/// ```

use httparse::Header;
use std::net::TcpStream;
use FProxy::http::{Response, Server};

fn main() {
    let mut server = Server::create_http_server("127.0.0.1:2233");

    server.set_stream_handler(|stream: TcpStream| -> TcpStream {
        let mut response = Response::new(stream);
        response.set_status_code(200);
        response.write_headers(&[Header {
            name: "Content-Type",
            value: b"text/html",
        }]);
        response.write_body(b"<h1>hello world</h1>");
        response.flush().unwrap();
        response.stream
    });

    println!("HTTP Server is running: http://127.0.0.1:2233");
    server.start();
}
