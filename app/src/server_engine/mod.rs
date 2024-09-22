use std::{collections::HashMap, sync::Arc};

use crate::{common::{ClientMessage, Message, ServerMessage}, socket::{packet::packet::{Event, SocketServerEvent}, server::{SocketServer, SocketServerConfig}}};

pub struct ServerEngine {
    players: HashMap<u32, (f32, f32, f32)>,
    socket_server: SocketServer,
    rx: tokio::sync::mpsc::Receiver<SocketServerEvent>,
}


impl ServerEngine {
    pub async fn start() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(200);
        let socket_server = SocketServer::start(SocketServerConfig::default(), tx).await.unwrap();
        
        Self {socket_server, rx, players: HashMap::new()}
    }

    pub fn tick(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            let client_id = event.client_id;
            match event.event {
                Event::Connection(nickname) => {
                    let message = Message::Server(ServerMessage::AddPlayer((client_id, nickname)));
                    let payload: Arc<[u8]> = bincode::serialize(&message).unwrap().into();
                    println!("connect!");
                    self.socket_server.send_exlude(client_id, payload);
                },
                Event::Disconnection => {
                    let message = Message::Server(ServerMessage::RemovePlayer(client_id));
                    let payload: Arc<[u8]> = bincode::serialize(&message).unwrap().into();
                    self.socket_server.send_exlude(client_id, payload);
                },
                Event::Packet(bytes) => {
                    let message = bincode::deserialize::<Message>(&bytes).unwrap();
                    let Message::Client(event) = message else {continue};
                    match event {
                        ClientMessage::Position(a, b, c) => {
                            self.players.insert(client_id, (a, b, c));
                        }
                    }
                }
            }
        }

        let message = Message::Server(ServerMessage::Players(self.players.clone()));
        let payload: Arc<[u8]> = bincode::serialize(&message).unwrap().into();
        self.socket_server.send_all(payload);
    }
}