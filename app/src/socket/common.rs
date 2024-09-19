#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeId {
    UnknownError = 0,
    ProtocolVersion = 1,
    ClientId = 2,

    Reject = 3,
    Nickname = 4,
    Ok = 5,
}

impl From<u32> for HandshakeId {
    fn from(id: u32) -> Self {
        match id {
            1 => Self::ProtocolVersion,
            2 => Self::ClientId,
            3 => Self::Reject,
            4 => Self::Nickname,
            5 => Self::Ok,
            _ => Self::UnknownError
        }
    }
}

impl From<HandshakeId> for u32 {
    fn from(id: HandshakeId) -> Self {id as u32}
}

/// | HandshakeId | Meaning of data |
/// |--------------|------------|
/// | UnknownError | doesn't matter |
/// | ProtocolVersion | ProtocolVersion |
/// | ClientId | ClientId |
/// | Reject | Reject string byte size |
/// | Nickname | Nickname string byte size |
/// | Ok | doesn't matter |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Handshake {
    id: HandshakeId,
    data: u32,
}

impl Handshake {
    pub fn new(id: HandshakeId, data: u32) -> Self {
        Self { id, data }
    }

    pub fn data(&self) -> u32 {
        self.data
    }

    pub fn id(&self) -> HandshakeId {
        self.id
    }
}


impl From<u64> for Handshake {
    fn from(mut value: u64) -> Self {
        let data = (value & 0xffff_ffff) as u32;
        value >>= 32;
        let id = (value & 0xffff_ffff) as u32;

        Self {id: HandshakeId::from(id), data}
    }
}

impl From<Handshake> for u64 {
    fn from(h: Handshake) -> Self {
        let mut result = 0u64;
        result |= h.id as u64;
        result <<= 32;
        result |= h.data as u64;
        result
    }
}