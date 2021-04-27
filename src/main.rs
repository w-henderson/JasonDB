mod database;
mod isam;
mod request;
mod tests;

use database::Database;
use futures::{SinkExt, StreamExt};
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::codec::{Framed, LinesCodec};

#[tokio::main]
async fn main() {
    let db = Arc::new(RwLock::new(Database::new("database")));
    let tcp_listener = TcpListener::bind("127.0.0.1:1337").await.unwrap();
    let ws_listener = TcpListener::bind("127.0.0.1:1338").await.unwrap();

    let tcp_db_ref = db.clone();
    let ws_db_ref = db.clone();

    tokio::spawn(async move {
        tcp_handler(tcp_listener, &tcp_db_ref).await;
    });

    tokio::spawn(async move {
        ws_handler(ws_listener, &ws_db_ref).await;
    });

    loop {}
}

async fn tcp_handler(listener: TcpListener, db: &Arc<RwLock<Database>>) {
    loop {
        match listener.accept().await {
            Ok((socket, _)) => {
                let db_ref = db.clone();

                tokio::spawn(async move {
                    let mut lines = Framed::new(socket, LinesCodec::new());
                    while let Some(result) = lines.next().await {
                        match result {
                            Ok(line) => {
                                let request = request::parse(&line);
                                let response = request::execute(request, &db_ref);

                                lines.send(response).await.unwrap();
                            }
                            Err(_) => (),
                        }
                    }
                });
            }
            Err(_) => (),
        }
    }
}

async fn ws_handler(listener: TcpListener, db: &Arc<RwLock<Database>>) {
    loop {
        match listener.accept().await {
            Ok((socket, _)) => {
                let db_ref = db.clone();

                tokio::spawn(async move {
                    let ws_stream = tokio_tungstenite::accept_async(socket).await.unwrap();
                    let (mut outgoing, mut incoming) = ws_stream.split();

                    while let Some(result) = incoming.next().await {
                        match result {
                            Ok(msg) => {
                                let request = request::parse(&msg.to_text().unwrap());
                                let response = request::execute(request, &db_ref);

                                outgoing.send(Message::Text(response)).await.unwrap_or(());
                            }
                            Err(_) => (),
                        }
                    }
                });
            }
            Err(_) => (),
        }
    }
}
