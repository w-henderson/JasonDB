mod cli;
mod extract;
mod isam_mirror;
mod net;
mod request;

#[cfg(test)]
mod tests;

use jasondb::database::Database;
use jasondb::isam;

use parking_lot::RwLock;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use websocket::sync::Server;

#[tokio::main]
async fn main() {
    // Parse arguments
    let args = cli::load_args();

    match args {
        // If no subcommand was specified, run the program regularly
        cli::Args::Main {
            database,
            no_tcp,
            no_ws,
            tcp_port,
            ws_port,
            ws_cert,
            ws_key,
            mirror_interval,
            log_config,
        } => {
            cli::log("[INFO] Starting JasonDB...", &log_config.force());

            // Initialise the database to be mutable and thread-safe
            if let Ok(loaded_db) = isam::load(&database) {
                let db = Arc::new(RwLock::new(loaded_db));
                let isam_db_ref = db.clone();

                // Initialise a variable to store the state of the application
                // 0 - running, 1 - stopping, 2 - safe to terminate
                let application_state = Arc::new(AtomicU8::new(0));

                // Create a thread for each type of connection
                if !no_tcp {
                    let tcp_addr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
                    let tcp_socket_addr = SocketAddr::new(tcp_addr, tcp_port);
                    let tcp_socket = TcpListener::bind(tcp_socket_addr).await.unwrap();
                    let tcp_db_ref = db.clone();
                    let config_clone = log_config.clone();
                    tokio::spawn(async move {
                        net::tcp::handler(tcp_socket, &tcp_db_ref, config_clone).await;
                    });
                }

                if !no_ws {
                    if let Ok(tls) = net::ws::init_tls(&ws_cert, &ws_key) {
                        let ws_addr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
                        let ws_socket_addr = SocketAddr::new(ws_addr, ws_port);
                        let ws_socket = Server::bind_secure(ws_socket_addr, tls).unwrap();
                        let ws_db_ref = db.clone();
                        let config_clone = log_config.clone();
                        tokio::spawn(async move {
                            net::ws::handler(ws_socket, &ws_db_ref, config_clone).await;
                        });
                    } else {
                        return cli::log("[ERR]  Unspecified or invalid TLS certificate. If you're not using WebSocket, pass the `--no-ws` argument to ignore.", &log_config);
                    }
                }

                // Create a thread to asynchronously mirror the database to disk
                let isam_application_state = application_state.clone();
                let config_clone = log_config.clone();
                tokio::spawn(async move {
                    isam_mirror::mirror_handler(
                        isam_db_ref,
                        &database,
                        mirror_interval,
                        isam_application_state,
                        config_clone,
                    )
                    .await;
                });

                ctrlc::set_handler(move || {
                    application_state.store(1, Ordering::SeqCst);
                    cli::log(
                        "[DISK] Waiting for next save to complete...",
                        &log_config.force(),
                    );

                    // Wait for the state to change to 2 (safe to terminate).
                    // This change happens in the ISAM thread and can take up to 5 seconds.
                    while application_state.load(Ordering::SeqCst) == 1 {}

                    // Safely exit the program
                    cli::log("[INFO] Exiting the program.", &log_config.force());
                    std::process::exit(0);
                })
                .expect("[ERR]  Couldn't set exit handler");

                // Idles the main thread
                std::thread::park();
            } else {
                cli::log(
                    "[ERR]  Unspecified or invalid database.",
                    &log_config.force(),
                )
            }
        }

        // If the create subcommand was specified, create a database
        cli::Args::Create { name } => create_database(&name),

        // If the extract command was specified, run the extraction tool
        cli::Args::Extract { path } => {
            return if let Ok(()) = extract::extract(&path) {
                cli::log("[INFO] Database extracted.", &cli::LogConfig::default())
            } else {
                cli::log("[ERR]  An error occurred.", &cli::LogConfig::default())
            }
        }

        // If an error occurred while parsing arguments
        cli::Args::Error { message } => {
            cli::log(&format!("[ERR]  {}", message), &cli::LogConfig::default())
        }
    }
}

/// Create a new database.
/// Executed when the user runs `jasondb create <database name>`.
fn create_database(name: &str) {
    let db = Database::new(name);
    isam::save(name, &db);
    cli::log("[INFO] Empty database created.", &cli::LogConfig::default());
}
