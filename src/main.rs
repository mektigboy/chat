use std::net::SocketAddr;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    let listener: TcpListener = TcpListener::bind("localhost:8080").await.unwrap();
    let (transaction, _reception) = broadcast::channel::<(String, SocketAddr)>(10);

    loop {
        let (mut socket, address) = listener.accept().await.unwrap();
        let transaction = transaction.clone();
        let mut reception = transaction.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    _ = reader.read_line(&mut line) => {
                        transaction.send((line.clone(), address)).unwrap();
                        line.clear()
                    }

                    result = reception.recv() => {
                        let (message, message_address) = result.unwrap();

                        if address != message_address {
                            writer.write_all(message.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
