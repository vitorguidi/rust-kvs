use bytes::Bytes;

pub const OP_GET: u8 = 0x01;
pub const OP_SET: u8 = 0x02;
pub const OP_PING: u8 = 0x03;

#[derive(Debug, Clone)]
pub enum Command {
    Get { key: String },
    Set {
        key: String,
        value: Bytes,
        ttl_sec: u64,
    },
    Ping,
}

#[derive(Debug)]
pub enum Response {
    Ok,
    Found(Bytes),
    NotFound,
    Error(String),
}
