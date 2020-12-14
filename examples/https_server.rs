use tiny_http::{Response, Server, SslConfig};
use FProxy::cert::{make_cert_from_ca, Certificate};

extern crate FProxy;

pub fn create_https_server(cert: Vec<u8>, key: Vec<u8>) {
    let server = Server::https(
        "0.0.0.0:8001",
        SslConfig {
            certificate: cert,
            private_key: key,
        },
    )
    .unwrap();

    println!(
        "listening: https://localhost:{}",
        server.server_addr().port()
    );

    for request in server.incoming_requests() {
        assert!(request.secure());

        let response = Response::from_string("hello world");
        request.respond(response).unwrap();
    }
}

fn main() {
    let ca = Certificate::new("../ca/cert.pem", "../ca/key.pem").unwrap();
    let cert = make_cert_from_ca("localhost", &ca).unwrap();
    let key = ca.key.private_key_to_pem_pkcs8().unwrap();
    let cert = cert.to_pem().unwrap();
    create_https_server(cert, key);
}
