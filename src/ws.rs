use crate::{database::Database, request};
use dotenv::var;
use native_tls::{Identity, TlsAcceptor};
use parking_lot::RwLock;
use std::{fs::File, io::Read, sync::Arc};
use std::{net::TcpListener, thread};
use websocket::{server::WsServer, OwnedMessage};

pub fn init_tls() -> TlsAcceptor {
    let mut file = File::open(var("CERT").unwrap()).unwrap();
    let mut bytes: Vec<u8> = Vec::new();
    file.read_to_end(&mut bytes).unwrap();
    let identity = Identity::from_pkcs12(&bytes, &var("KEY").unwrap()).unwrap();
    TlsAcceptor::new(identity).unwrap()
}

pub async fn ws_handler(server: WsServer<TlsAcceptor, TcpListener>, db: &Arc<RwLock<Database>>) {
    for request in server.filter_map(Result::ok) {
        let db_ref = db.clone();

        thread::spawn(move || {
            let mut client = request.accept().unwrap();

            loop {
                let msg = client.recv_message().unwrap();

                match msg {
                    OwnedMessage::Text(text) => {
                        let request = request::parse(&text);
                        let response = request::execute(request, &db_ref);
                        client.send_message(&OwnedMessage::Text(response)).unwrap();
                    }

                    OwnedMessage::Close(_) => {
                        break;
                    }

                    _ => (),
                }
            }
        });
    }
}
