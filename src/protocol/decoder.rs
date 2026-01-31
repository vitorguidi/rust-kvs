use bytes::{Buf, BytesMut};
use std::io;
use std::str;
use tokio_util::codec::Decoder;

use super::types::{Command, OP_GET, OP_PING, OP_SET};
use super::CacheCodec;

impl Decoder for CacheCodec {
    type Item = Command;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }
        let opcode = src[0];

        match opcode {
            OP_PING => {
                // Layout: [OpCode: 1]
                src.advance(1);
                Ok(Some(Command::Ping))
            }
            OP_GET => {
                // Layout: [Opcode: 1][KeyLen: 4][Key: KeyLen]
                if src.len() < 5 {
                    return Ok(None);
                }

                let mut temp_buf = &src[1..5];
                let key_len = temp_buf.get_u32() as usize;

                if src.len() < 1 + 4 + key_len {
                    src.reserve(key_len);
                    return Ok(None);
                }

                src.advance(5);

                let key_bytes = src.split_to(key_len);
                let key = str::from_utf8(&key_bytes)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                    .to_string();
                Ok(Some(Command::Get { key }))
            }
            OP_SET => {
                // Layout: [OpCode: 1][KeyLen: 4][Key: KeyLen][ValLen: 4][Val: ValLen][TTL: 8]
                if src.len() < 5 {
                    return Ok(None);
                }
                let mut p = &src[1..];
                let key_len = p.get_u32() as usize;
                if src.len() < 1 + 4 + key_len + 4 {
                    src.reserve(key_len + 50);
                    return Ok(None);
                }
                let mut p = &src[1 + 4 + key_len..];
                let val_len = p.get_u32() as usize;

                let total_required = 1 + 4 + key_len + 4 + val_len + 8;
                if src.len() < total_required {
                    src.reserve(val_len);
                    return Ok(None);
                }

                src.advance(1 + 4);
                let key_bytes = src.split_to(key_len);
                let key = String::from_utf8_lossy(&key_bytes).to_string();
                src.advance(4);
                let val = src.split_to(val_len).freeze();
                let ttl = src.get_u64();
                Ok(Some(Command::Set {
                    key,
                    value: val,
                    ttl_sec: ttl,
                }))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Opcode",
            )),
        }
    }
}
