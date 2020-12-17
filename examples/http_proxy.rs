/// A simple http forward proxy example.
/// Use:
/// ```bash
/// cargo run --example http_proxy
/// curl -x http://127.0.0.1:2233 http://www.baidu.com
/// ```

use std::{
    io,
    io::{BufRead, Read, Write},
    net::{TcpListener, TcpStream},
    thread, vec,
};

use io::BufReader;

fn parse_http(src: &[u8]) -> io::Result<Option<Vec<u8>>> {
    let mut parsed_headers = [httparse::EMPTY_HEADER; 16];
    let mut r = httparse::Request::new(&mut parsed_headers);
    let status = r.parse(src).map_err(|e| {
        let msg = format!("failed to parse http request: {:?}", e);
        io::Error::new(io::ErrorKind::Other, msg)
    })?;

    let amt = match status {
        httparse::Status::Complete(amt) => amt,
        httparse::Status::Partial => return Ok(None),
    };

    let src = src.split_at(amt).1;
    let path = r.path.unwrap().to_string();

    let mut ret = match r.method {
        Some("GET") => ureq::get(&path),
        Some("POST") => ureq::post(&path),
        Some("CONNECT") => panic!("doesn't support https tunnel"),
        _ => panic!(),
    };

    for header in r.headers.iter() {
        ret = ret.set(
            header.name,
            &String::from_utf8(header.value.to_vec()).unwrap(),
        );
    }

    let res = ret.send_bytes(src).unwrap();

    let mut buf = vec![];
    res.into_reader().read_to_end(&mut buf)?;

    Ok(Some(buf))
}

fn read_head(stream: &TcpStream) -> Option<Vec<u8>> {
    let mut reader = BufReader::new(stream);
    let mut ret = vec![];

    loop {
        let mut line = vec![];
        let read_size = reader.read_until(b'\n', &mut line);

        if let Err(_) = read_size {
            break;
        }
        if read_size.unwrap() == 2 && line[0] == b'\r' && line[1] == b'\n' {
            ret.push([b'\r', b'\n'].to_vec());
            break;
        }

        ret.push(line);
    }

    Some(ret.concat())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let local_http_server = TcpListener::bind("127.0.0.1:2233")?;

    for stream in local_http_server.incoming() {
        thread::spawn(|| -> Result<(), std::io::Error> {
            if let Err(_) = stream {
                return Ok(());
            }
            let mut stream = stream?;

            let buf = read_head(&stream);
            if let Some(buf) = buf {
                let ret = parse_http(&buf)?;

                if let Some(buf) = ret {
                    stream.write(&buf)?;
                } else {
                    stream.write(b"HTTP/1.1 200 OK\r\n\r\n")?;
                }
            }

            Ok(())
        });
    }

    Ok(())
}
