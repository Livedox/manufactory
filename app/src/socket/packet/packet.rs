use std::{default, net::SocketAddr};

pub struct PacketTest {
    size: u32,
    payload: Vec<u8>,
}

use super::header::{Header, HeaderId};

#[derive(Debug, Clone)]
pub struct Packet {
    header: Header,
    payload: Vec<u8>,
}

impl Packet {
    pub fn new_header(header_id: HeaderId) -> Self {
        Self { header: Header::new(header_id, 0), payload: Vec::new() }
    }

    pub fn new(header_id: HeaderId, payload: Vec<u8>) -> Self {
        Self { header: Header::new(header_id, payload.len() as u32), payload }
    }

    pub fn header(&self) -> Header {
        self.header
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn into_payload(self) -> Vec<u8> {
        self.payload
    }

    pub fn size(&self) -> u32 {
        self.header.size()
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Connection(String),
    Disconnection,
    Packet(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct SocketServerEvent {
    pub client_id: u32,
    pub event: Event
}

impl SocketServerEvent {
    pub fn new(client_id: u32, event: Event) -> Self {
        Self { client_id, event }
    }
}