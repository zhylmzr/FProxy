use std::{
    io::{self, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use httparse::Header;

fn default_stream_handler(stream: TcpStream) -> TcpStream {
    stream
}

pub struct Server {
    listener: TcpListener,
    stream_handler: Arc<dyn Fn(TcpStream) -> TcpStream + Send + Sync>,
}

impl Server {
    pub fn create_http_server(addr: &str) -> Self {
        let listener = TcpListener::bind(addr).unwrap();

        Self {
            listener,
            stream_handler: Arc::new(default_stream_handler),
        }
    }

    pub fn start(&self) {
        for stream in self.listener.incoming() {
            let stream_handler = Arc::clone(&self.stream_handler);
            thread::spawn(move || -> Result<(), std::io::Error> {
                if let Err(_) = stream {
                    return Ok(());
                }
                stream_handler(stream?);
                Ok(())
            });
        }
    }

    pub fn set_stream_handler(&mut self, f: fn(TcpStream) -> TcpStream) {
        self.stream_handler = Arc::new(f);
    }
}

pub struct Response<'a> {
    pub stream: TcpStream,
    pub status_code: u32,
    raw_header: &'a [Header<'a>],
    raw_body: Vec<u8>,
}

fn write_crlf(target: &mut Vec<u8>) {
    target.append(&mut b"\r\n".to_vec());
}

impl<'a> Response<'a> {
    pub fn new(stream: TcpStream) -> Self {
        Response {
            stream,
            status_code: 404,
            raw_header: &[],
            raw_body: vec![],
        }
    }

    pub fn set_status_code(&mut self, code: u32) {
        self.status_code = code;
    }

    pub fn write_headers(&mut self, headers: &'a [Header]) {
        self.raw_header = headers;
    }

    pub fn write_body(&mut self, body: &[u8]) {
        self.raw_body.append(&mut body.to_vec());
    }

    pub fn flush(&mut self) -> Result<(), io::Error> {
        let mut buff = vec![];

        // 1. Write status code.
        buff.append(&mut self.get_status_code());

        // 2. Write Response Headers(include Content-Length).
        self.write_content_length(&mut buff);
        for header in self.raw_header {
            buff.append(&mut format!("{}: ", header.name).as_bytes().to_vec());
            buff.append(&mut header.value.to_vec());
            write_crlf(&mut buff);
        }
        write_crlf(&mut buff);

        // 3. Write Response Body.
        buff.append(&mut self.raw_body);
        write_crlf(&mut buff);
        write_crlf(&mut buff);

        // 4. Flush stream.
        self.stream.write(&buff)?;
        Ok(())
    }

    fn get_status_code(&self) -> Vec<u8> {
        let desc = match self.status_code {
            100 => "Continue",
            101 => "Switching Protocols",
            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            203 => "Non-Authoritative ",
            204 => "No Content",
            205 => "Reset Content",
            206 => "Partial Content",
            300 => "Multiple Choices",
            301 => "Moved Permanently",
            302 => "Found",
            303 => "See Other",
            304 => "Not Modified",
            305 => "Use Proxy",
            307 => "Temporary Redirect",
            400 => "Bad Request",
            401 => "Unauthorized",
            402 => "Payment Required",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            406 => "Not Acceptable",
            407 => "Proxy Authentication ",
            408 => "Request Timeout",
            409 => "Conflict",
            410 => "Gone",
            411 => "Length Required",
            412 => "Precondition Failed",
            413 => "Payload Too Large",
            414 => "URI Too Long",
            415 => "Unsupported Media Type",
            416 => "Range Not Satisfiable",
            417 => "Expectation Failed",
            426 => "Upgrade Required",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Timeout",
            505 => "HTTP Version Not Supported",
            _ => panic!(format!("{} status code error", self.status_code)),
        };

        format!("HTTP/1.1 {} {}\r\n", self.status_code, desc)
            .as_bytes()
            .to_vec()
    }

    fn write_content_length(&self, target: &mut Vec<u8>) {
        target.append(
            &mut format!("Content-Length: {}\r\n", self.raw_body.len())
                .as_bytes()
                .to_vec(),
        );
    }
}
