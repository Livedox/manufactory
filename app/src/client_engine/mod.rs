use std::{collections::HashMap, sync::Arc, time::Duration};

use graphics_engine::{player_mesh::PlayerMesh, state};
use winit::{dpi::PhysicalSize, event::WindowEvent, event_loop::{EventLoop}, window::{Fullscreen}};

use crate::{camera, common::{ClientMessage, Message, ServerMessage}, content_loader::indices::{load_animated_models, load_blocks_textures, load_models, GamePath, Indices}, gui::gui_controller::GuiController, input_event::{self, input_service::{InputService, Key}, KeypressState, State}, my_time::{self, Timer}, player::player::Player, save_load::Save, socket::{client::{Client, ClientConfig}, packet::packet::{Event, SocketServerEvent}, server::{SocketClient, SocketServer, SocketServerConfig}}, CAMERA_FAR, CAMERA_FOV, CAMERA_NEAR};
use crate::glm;

pub struct ClientEngine {
    player: Player,
    players: HashMap<u32, (f32, f32, f32)>,
    client: Client,
    rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
}


impl ClientEngine {
    pub async fn start() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(200);
        let client = Client::start(ClientConfig::default(), tx).await.unwrap();
        let camera = camera::camera_controller::CameraController::new(
            glm::vec3(0.0, 20.0, 0.0), CAMERA_FOV, CAMERA_NEAR, CAMERA_FAR);
        let player = Player::new(camera, glm::vec3(0.0, 20.0, 0.0));
    
        Self {client, rx, players: HashMap::new(), player}
    }

    pub fn tick(&mut self) {
        while let Ok(bytes) = self.rx.try_recv() {
            let message = bincode::deserialize::<Message>(&bytes).unwrap();
            let Message::Server(event) = message else {continue};
            match event {
                ServerMessage::AddPlayer((id, nickname)) => {
                    self.players.insert(id, (0.0, 0.0, 0.0));
                    println!("Player {nickname}");
                },
                ServerMessage::Players(players) => {
                    for (id, position) in players {
                        if self.client.id() != id {
                            self.players.insert(id, position);
                        }
                    }
                },
                ServerMessage::RemovePlayer(client_id) => {
                    self.players.remove(&client_id);
                }
            }
        }
        let pos = self.player.position();
        let message = Message::Client(ClientMessage::Position(pos.x, pos.y, pos.z));
        let payload: Vec<u8> = bincode::serialize(&message).unwrap().into();
        self.client.send_payload(payload);
    }

    pub fn positions(&self) -> Vec<[f32; 3]> {
        self.players.values().map(|(a, b, c)| [*a, *b, *c]).collect()
    }

    pub fn proj_view(&self, width: f32, height: f32) -> glm::Mat4 {
        self.player.camera().proj_view(width, height)
    }

    pub fn player(&mut self) -> &mut Player {
        &mut self.player
    }
}