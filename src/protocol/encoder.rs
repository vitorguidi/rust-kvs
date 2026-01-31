use bytes::{BufMut, BytesMut};
use std::io;
use tokio_util::codec::Encoder;

use super::types::Response;
use super::CacheCodec;

impl Encoder<Response> for CacheCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Response, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            Response::Ok => {
                dst.put_u8(0x00);
            }
            Response::NotFound => {
                dst.put_u8(0x01);
            }
            Response::Found(data) => {
                dst.put_u8(0x02);
                dst.put_u32(data.len() as u32);
                dst.extend_from_slice(&data);
            }
            Response::Error(message) => {
                dst.put_u8(0xFF);
                dst.put_u32(message.len() as u32);
                dst.extend_from_slice(message.as_bytes());
            }
        }
        Ok(())
    }
}
