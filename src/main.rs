mod database;
mod isam;
mod request;
mod tests;

use database::Database;
use futures::SinkExt;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

#[tokio::main]
async fn main() {
    let db = Arc::new(RwLock::new(Database::new("database")));
    listener_tcp(db).await;
}

async fn listener_tcp(db: Arc<RwLock<Database>>) {
    let listener = TcpListener::bind("127.0.0.1:1337").await.unwrap();

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

async fn listener_ws(db: Arc<RwLock<Database>>) {}
