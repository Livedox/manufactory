use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};

use crate::socket::{common::{Handshake, HandshakeId}, packet::header::PROTOCOL_VERSION};

use super::packet::header::Header;

pub struct ClientConfig {
    addr: String,
    max_message_size: usize,
    max_packet_count: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            addr: String::from("127.0.0.1:8090"),
            max_message_size: 2_097_152, // 2 megabytes
            max_packet_count: 200,
        }
    }
}


pub struct Client {
    writer: tokio::net::tcp::OwnedWriteHalf,
}


impl Client {
    pub async fn start(config: ClientConfig) -> tokio::io::Result<Self> {
        let mut stream = TcpStream::connect(config.addr).await?;
        let _client_id = Self::handshake(&mut stream).await.unwrap();

        let (mut reader, writer) = stream.into_split();

        tokio::spawn(async move {
            let mut count = 0;
            loop {
                let Ok(header_num) = reader.read_u64().await else {break};
                let header = Header::try_from(header_num).unwrap();
                count += 1;
                println!("Addr: {:?}, Header: {:?}, Count: {count}", reader.local_addr(), header);
            }
        });

        Ok(Self {writer})
    }

    pub async fn send_header(&mut self, header: Header) -> tokio::io::Result<()> {
        self.writer.write_u64(header.into()).await?;
        Ok(())
    }


    pub async fn handshake(stream: &mut TcpStream) -> Result<u32, Box<dyn std::error::Error>> {
        stream.write_u64(Handshake::new(HandshakeId::ProtocolVersion, PROTOCOL_VERSION as u32).into()).await?;
        let handshake: Handshake = stream.read_u64().await?.into();
        Self::check_reject(stream, handshake).await?;
        if handshake.id() != HandshakeId::ClientId {
            return Err("Incorrect Handshake Protocol".into());
        }
        let client_id = handshake.data();
        let nickname = "Big_Cock!".as_bytes();
        println!("{:?}", nickname.len() as u32);
        stream.write_u64(Handshake::new(HandshakeId::Nickname, nickname.len() as u32).into()).await?;
        stream.write_all(nickname).await?;
        let handshake: Handshake = stream.read_u64().await?.into();
        Self::check_reject(stream, handshake).await?;
        if handshake.id() != HandshakeId::Ok {
            return Err("Incorrect Handshake Protocol".into());
        }
        println!("Cool handshake client!");
        Ok(client_id)
    }

    async fn check_reject(stream: &mut TcpStream, handshake: Handshake) -> Result<(), Box<dyn std::error::Error>> {
        if handshake.id() == HandshakeId::Reject {
            let mut reject_msg = String::new();
            stream.read_to_string(&mut reject_msg).await?;
            return Err(reject_msg.into())
        }
        Ok(())
    } 
}