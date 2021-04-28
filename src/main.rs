mod database;
mod isam;
mod request;
mod tests;
mod ws;

use database::Database;
use futures::{SinkExt, StreamExt};
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::codec::{Framed, LinesCodec};
use websocket::sync::Server;

#[tokio::main]
async fn main() {
    let db = Arc::new(RwLock::new(Database::new("database")));
    let tls = ws::init_tls();
    let tcp_listener = TcpListener::bind("127.0.0.1:1337").await.unwrap();
    let ws_listener = Server::bind_secure("127.0.0.1:1338", tls).unwrap();

    let tcp_db_ref = db.clone();
    let ws_db_ref = db.clone();

    tokio::spawn(async move {
        tcp_handler(tcp_listener, &tcp_db_ref).await;
    });

    tokio::spawn(async move {
        ws::ws_handler(ws_listener, &ws_db_ref).await;
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
