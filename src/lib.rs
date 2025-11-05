#[derive(Debug, Clone)]
pub struct Packet {
    pub id: u32,
    pub data: Vec<u8>,
}

impl Packet {
    pub fn new(id: u32, data: &[u8]) -> Self {
        Self {
            id,
            data: data.to_vec(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(4 + self.data.len());
        buf.extend_from_slice(&self.id.to_le_bytes());
        buf.extend_from_slice(&self.data);
        buf
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 4 { return None; }
        let id = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let data = bytes[4..].to_vec();
        Some(Self { id, data })
    }
}

pub fn protocol_init() -> i32 {
    println!("[Rhizome] Protocol initialized");
    42
}

pub fn protocol_send(data: &[u8]) -> i32 {
    let packet = Packet::new(1, data);
    let encoded = packet.encode();
    println!("[Rhizome] Sent {} bytes", encoded.len());
    0
}

pub fn protocol_receive(bytes: &[u8]) -> i32 {
    if let Some(packet) = Packet::decode(bytes) {
        println!("[Rhizome] Received packet #{} with {} bytes", packet.id, packet.data.len());
        0
    } else {
        -1
    }
}