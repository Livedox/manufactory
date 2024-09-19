pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum HeaderId {
    SomethingIsWrong = 0,
    Packet = 1,
    Heartbeat = 2,
    IncorrectProtocolVersion = 3,
    IncorrectHeader = 4,
    TooManyConnections = 5,
    PacketSizeTooBig = 6,
}


impl HeaderId {
    pub fn is_close(self) -> bool {
        match self {
            Self::IncorrectHeader => true,
            Self::IncorrectProtocolVersion => true,
            Self::TooManyConnections => true,
            Self::PacketSizeTooBig => true,
            Self::SomethingIsWrong => true,
            _ => false
        }
    }
}

impl TryFrom<u16> for HeaderId {
    type Error = &'static str;
    
    fn try_from(id: u16) -> Result<Self, Self::Error> {
        match id {
            1 => Ok(HeaderId::Packet),
            2 => Ok(HeaderId::Heartbeat),
            3 => Ok(HeaderId::IncorrectProtocolVersion),
            4 => Ok(HeaderId::IncorrectHeader),
            5 => Ok(HeaderId::TooManyConnections),
            _ => Err("There is no given id")
        }
    }
}

impl From<HeaderId> for u16 {
    fn from(id: HeaderId) -> Self {
        id as u16
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Header {
    protocol_version: u16,
    id: HeaderId,
    /// Packet size
    size: u32
}

impl TryFrom<u64> for Header {
    type Error = &'static str;
    
    fn try_from(mut value: u64) -> Result<Self, Self::Error> {
        let size = (value & 0xffff_ffff) as u32;
        value >>= 32;
        let id = (value & 0xffff) as u16;
        value >>= 16;
        let pv = (value & 0xffff) as u16;

        Ok(Self {id: HeaderId::try_from(id)?, protocol_version: pv, size})
    }
}

impl From<Header> for u64 {
    fn from(h: Header) -> Self {
        let mut result = 0u64;
        result |= h.protocol_version as u64;
        result <<= 16;
        result |= h.id as u64;
        result <<= 32;
        result |= h.size as u64;
        result
    }
}

impl Header {
    pub fn new(id: HeaderId, size: u32) -> Self {
        Self { protocol_version: PROTOCOL_VERSION, id, size }
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn protocol_version(&self) -> u16 {
        self.protocol_version
    }

    pub fn id(&self) -> HeaderId {
        self.id
    }

    pub fn is_close(&self) -> bool {
        self.id.is_close()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header() {
        let header = Header::new(HeaderId::Packet, 2);
        let num = <u64>::from(header);
        let header_two = Header::try_from(num).unwrap();

        assert_eq!(header, header_two);
    }
}