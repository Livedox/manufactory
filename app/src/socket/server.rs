use std::hash::Hash;
use std::io;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64};
use std::sync::Arc;
use std::{collections::HashMap, net::SocketAddr};

use thiserror::Error;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::TcpStream;
use tokio::net::{tcp::OwnedWriteHalf, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc::Sender;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_util::sync::CancellationToken;

use crate::socket::packet::header::{Header, HeaderId, PROTOCOL_VERSION};
use crate::socket::packet::packet::{Packet};

use super::common::{Handshake, HandshakeId};
use super::packet::packet::{Event, SocketServerEvent};


#[derive(Debug, Clone)]
pub struct SocketServerConfig {
    pub addr: String,
    pub max_message_size: usize,
    pub max_connection: usize,
    pub max_packet_count: usize,
}

impl Default for SocketServerConfig {
    fn default() -> Self {
        Self {
            addr: String::from("127.0.0.1:8090"),
            max_message_size: 2_097_152, // 2 megabytes
            max_connection: 100,
            max_packet_count: 200,
        }
    }
}

#[derive(Error, Debug)]
pub enum SocketClientError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    SendError(#[from] tokio::sync::mpsc::error::SendError<SocketServerEvent>),
    #[error("Incorrect protocol version")]
    IncorrectProtocolVersion,
    #[error("Incorrect HandshakeId")]
    IncorrectHandshakeId,
    #[error("Too many connections")]
    TooManyConnections,
    #[error("Too big nickname")]
    TooBigNickname
}

#[derive(Debug)]
pub struct SocketClient {
    is_disconnect_send: Arc<AtomicBool>,
    token: CancellationToken,
    event_sender: tokio::sync::mpsc::Sender<SocketServerEvent>,
    tx: tokio::sync::mpsc::Sender<Arc<[u8]>>,
    addr: SocketAddr,
    id: u32,
}

impl SocketClient {
    pub async fn new(
        mut socket: TcpStream,
        addr: SocketAddr,
        id: u32,
        is_max_connection: bool,
        max_message_size: usize,
        max_packet_count: usize,
        event_sender: tokio::sync::mpsc::Sender<SocketServerEvent>,
    ) -> Result<Self, SocketClientError> {
        let nickname = Self::handshake(&mut socket, id, is_max_connection).await?;
        let token = CancellationToken::new();
        let (reader, writer) = socket.into_split();
        let (tx, rx) = mpsc::channel::<Arc<[u8]>>(max_packet_count);
        let is_disconnect_send = Arc::new(AtomicBool::new(false));
        

        let cloned_event_sender = event_sender.clone();
        let cloned_token = token.clone();
        let cloned_is_send = Arc::clone(&is_disconnect_send);
        tokio::spawn(async move {
            tokio::select! {
                _ = cloned_token.cancelled() => {},
                _ = Self::proccess_incoming(
                    cloned_event_sender.clone(), reader, max_message_size, id) => {},
            }
            Self::disconnection(cloned_is_send, cloned_event_sender, id).await;
            cloned_token.cancel();
        });

        let cloned_event_sender = event_sender.clone();
        let cloned_token = token.clone();
        let cloned_is_send = Arc::clone(&is_disconnect_send);
        tokio::spawn(async move {
            let _ = Self::proccess_outgoing(writer, rx, cloned_token.clone()).await;
            Self::disconnection(cloned_is_send, cloned_event_sender, id).await;
            cloned_token.cancel();
        });

        event_sender.send(SocketServerEvent::new(id, Event::Connection(nickname))).await?;
        Ok(Self {is_disconnect_send, event_sender, token, tx, addr, id})
    }

    pub fn addr(&self) -> SocketAddr { self.addr }
    pub fn id(&self) -> u32 { self.id }
    pub fn is_active(&self) -> bool {!self.token.is_cancelled()}
    pub fn sender(&self) -> &tokio::sync::mpsc::Sender<Arc<[u8]>> {
        &self.tx
    }

    async fn disconnection(is_send: Arc<AtomicBool>, event_sender: Sender<SocketServerEvent>, id: u32) {
        if !is_send.swap(true, std::sync::atomic::Ordering::Release) {
            let _ = event_sender.send(SocketServerEvent::new(id, Event::Disconnection)).await;
        }
    }

    async fn proccess_incoming(
        event_sender: tokio::sync::mpsc::Sender<SocketServerEvent>,
        mut reader: OwnedReadHalf,
        max_message_size: usize,
        id: u32,
    ) -> Result<(), SocketClientError> {
        let mut buf = vec![0; max_message_size];
        loop {
            let size = reader.read_u32().await?;
            if size == 0 {
                eprintln!("Size == 0!");
                break;
            };
            if size as usize > max_message_size {
                eprintln!("Message too big!");
                break;
            }
            let buf = &mut buf[0..size as usize];
            reader.read_exact(buf).await?;
            event_sender.send(SocketServerEvent::new(id, Event::Packet(Vec::from(buf)))).await?;
        }
        Ok(())
    }

    async fn proccess_outgoing(
        mut writer: OwnedWriteHalf,
        mut rx: tokio::sync::mpsc::Receiver<Arc<[u8]>>,
        token: CancellationToken,
    ) -> tokio::io::Result<()> {
        loop {
            if token.is_cancelled() && rx.is_empty() {break};
            let v = tokio::select! {
                _ = token.cancelled() => {continue},
                v = rx.recv() => {
                    let Some(v) = v else {break};
                    v
                }
            };
            let size = v.len();
            writer.write_u32(size as u32).await?;
            writer.write_all(&v).await?;
        }
        Ok(())
    }

    async fn handshake(socket: &mut TcpStream, id: u32, is_max_connection: bool) -> Result<String, SocketClientError> {
        let handshake = Handshake::from(socket.read_u64().await?);
        if handshake.id() != HandshakeId::ProtocolVersion {
            Self::reject(socket, "Incorrect HandshakeId").await?;
            return Err(SocketClientError::IncorrectHandshakeId);
        }
        if handshake.data() != PROTOCOL_VERSION as u32 {
            Self::reject(socket, "Incorrect protocol version").await?;
            return Err(SocketClientError::IncorrectProtocolVersion);
        }
        if is_max_connection {
            Self::reject(socket, "Too many connections").await?;
            return Err(SocketClientError::TooManyConnections);
        }
        socket.write_u64(Handshake::new(HandshakeId::ClientId, id).into()).await?;
        let handshake = Handshake::from(socket.read_u64().await?);
        if handshake.id() != HandshakeId::Nickname {
            Self::reject(socket, "Incorrect HandshakeId").await?;
            return Err(SocketClientError::IncorrectHandshakeId);
        }
        if handshake.data() > 200 {
            Self::reject(socket, "Too big nickname").await?;
            return Err(SocketClientError::TooBigNickname);
        }
        let mut buf = vec![0; handshake.data() as usize];
        println!("{}", handshake.data());
        socket.read_exact(&mut buf).await?;
        let nickname = String::from_utf8(buf)?;

        socket.write_u64(Handshake::new(HandshakeId::Ok, 0).into()).await?;
        println!("Cool handshake server! {nickname}");
        Ok(nickname)
    }

    async fn reject(socket: &mut TcpStream, msg: &str) -> tokio::io::Result<()> {
        let msg = msg.as_bytes();
        let answer = Handshake::new(HandshakeId::Reject, msg.len() as u32);
        socket.write_u64(answer.into()).await?;
        socket.write_all(msg).await?;
        Ok(())
    }
}

impl Drop for SocketClient {
    fn drop(&mut self) {
        if !self.is_disconnect_send.swap(true, std::sync::atomic::Ordering::Release) {
            let _ = self.event_sender
                .blocking_send(SocketServerEvent::new(self.id, Event::Disconnection));
        }
        self.token.cancel();
    }
}

#[derive(Debug, Clone)]
pub struct SocketClients(pub Arc<RwLock<HashMap<u32, Arc<SocketClient>>>>);

impl SocketClients {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(HashMap::new())))
    }

    pub async fn get(&self, id: u32) -> Option<Arc<SocketClient>> {
        self.0.read().await.get(&id).cloned()
    }

    pub async fn insert(&self, id: u32, socket_client: SocketClient) -> Option<Arc<SocketClient>> {
        self.0.write().await.insert(id, Arc::new(socket_client))
    }

    pub async fn len(&self) -> usize {
        self.0.read().await.len()
    }
}

pub struct SocketServer {
    socket_clients: SocketClients,
    token: CancellationToken,
    config: SocketServerConfig,
}

impl SocketServer {
    pub async fn start(config: SocketServerConfig, event_sender: Sender<SocketServerEvent>) -> tokio::io::Result<Self> {
        let listener = TcpListener::bind(&config.addr).await?;
        
        let token = CancellationToken::new();
        let socket_clients = SocketClients::new();

        let cloned_token = token.clone();
        let cloned_socket_clients = socket_clients.clone();
        tokio::spawn(async move {
            tokio::select! {
                _  = cloned_token.cancelled() => {},
                _ = Self::proccess_incoming(
                        listener,
                        config.max_connection,
                        config.max_message_size,
                        config.max_packet_count,
                        event_sender,
                        cloned_socket_clients) => {},
            }
        });

        Ok(Self {socket_clients, token, config})
    }

    pub fn config(&self) -> &SocketServerConfig {
        &self.config
    }

    async fn proccess_incoming(
        listener: TcpListener,
        max_connection: usize,
        max_message_size: usize,
        client_max_packet_count: usize,
        tx: Sender<SocketServerEvent>,
        socket_clients: SocketClients,
    ) {
        let mut client_id = 0u32;
        loop {
            let (socket, addr) = listener.accept().await.unwrap();
            let Ok(socket_client) = SocketClient::new(
                socket,
                addr,
                client_id,
                socket_clients.len().await >= max_connection,
                max_message_size,
                client_max_packet_count,
                tx.clone()).await else {continue};
            socket_clients.insert(client_id, socket_client).await;
            
            client_id += 1;
        }
    }

    fn send(&self, client_id: u32, payload: Arc<[u8]>) {
        let socket_clients = self.socket_clients.clone();
        tokio::spawn(async move {
            if let Some(client) = socket_clients.get(client_id).await {
                let _ = client.sender().send(payload).await;
            };
        });
    }

    fn send_exlude(&self, client_id: u32, payload: Arc<[u8]>) {
        let socket_clients = self.socket_clients.clone();
        tokio::spawn(async move {
            let guard = socket_clients.0.read().await;
            for client in guard.values() {
                if client.id() != client_id {
                    let _ = client.sender().send(Arc::clone(&payload)).await;
                }
            }
        });
    }

    fn send_all(&self, payload: Arc<[u8]>) {
        let socket_clients = self.socket_clients.clone();
        tokio::spawn(async move {
            let guard = socket_clients.0.read().await;
            for client in guard.values() {
                let _ = client.sender().send(Arc::clone(&payload)).await;
            }
        });
    }
}


impl Drop for SocketServer {
    fn drop(&mut self) {
        self.token.cancel();
    }
}