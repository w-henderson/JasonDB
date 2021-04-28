mod database;
mod isam;
mod net;
mod request;
mod tests;

use database::Database;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::net::TcpListener;
use websocket::sync::Server;

#[tokio::main]
async fn main() {
    // Initialise the database to be mutable and thread-safe
    let db = Arc::new(RwLock::new(Database::new("database")));

    // Initialise TLS for the `wss://` protocol
    let tls = net::ws::init_tls();

    // Initialise TCP sockets for both regular TCP and WebSocket and creates database references for them
    let tcp_socket = TcpListener::bind("127.0.0.1:1337").await.unwrap();
    let ws_socket = Server::bind_secure("127.0.0.1:1338", tls).unwrap();
    let tcp_db_ref = db.clone();
    let ws_db_ref = db.clone();

    // Create a thread for each type of connection
    tokio::spawn(async move {
        net::tcp::handler(tcp_socket, &tcp_db_ref).await;
    });
    tokio::spawn(async move {
        net::ws::handler(ws_socket, &ws_db_ref).await;
    });

    // Idles the main thread
    std::thread::park();
}
