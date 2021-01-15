use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufStream};
use tokio::net::TcpListener;

fn http_check(buffer: &BytesMut) -> bool {
    let len = buffer.len();
    if len <= 4 {
        return false;
    }
    buffer[len - 1] == b'\n'
        && buffer[len - 2] == b'\r'
        && buffer[len - 3] == b'\n'
        && buffer[len - 4] == b'\r'
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:2233").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let mut socket = BufStream::new(socket);
            let mut buffer = BytesMut::new();

            loop {
                if http_check(&buffer) {
                    break;
                }

                if 0 == socket.read_buf(&mut buffer).await.unwrap() {
                    if !buffer.is_empty() {
                        println!("connection closed by peer");
                    }
                    return;
                }
            }

            let content = "hello world";
            socket
                .write_all(
                    format!(
                        "HTTP/1.1 200\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n",
                        content.len(),
                        content
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
            socket.flush().await.unwrap();
        });
    }
}
