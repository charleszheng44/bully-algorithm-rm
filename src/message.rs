use std::convert::TryFrom;
use std::num::TryFromIntError;
use std::io::{Read, Write};
use std::error::Error;
use std::mem::size_of;
use std::cmp::PartialEq;

#[derive(Debug)]
pub struct Message {
    pub message_type: MessageType,
    pub content: String
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum MessageType {
    Election = 0,
    Answer = 1,
    Coordinator = 2,
}

impl Message {
    /// to_u8_vec serializes the Message to an u8 vector
    pub fn to_u8_vec(&self) -> Result<Vec<u8>, TryFromIntError> { 
        let mut tmp_vec = Vec::new();
        // 1. type block (the second data block containing 8 bits/1 byte)
        let msg_type_blk = (self.message_type as u8).to_be_bytes();
        tmp_vec.extend_from_slice(&msg_type_blk);
        // 2. content block (the third data block containing 
        // self.content.len() bytes)
        let content_blk = self.content.as_bytes();
        tmp_vec.extend_from_slice(content_blk);
        Ok(tmp_vec)
    }
    
    /// from_u8_vec deserializes an u8 vector to a Message
    pub fn from_u8_vec(inp: Vec<u8>) -> Result<Message, String>{
        let mut msg = Message{
            message_type: MessageType::Election, 
            content: String::new(),
        };
        // 1. read the type block
        msg.message_type = match &inp[0] {
            0 => MessageType::Election,
            1 => MessageType::Answer,
            2 => MessageType::Coordinator,
            _ => return Err(format!("unknown message type{}", &inp[0])),
        };
        // 2. read the content block
        msg.content = String::from_utf8_lossy(&inp[1..]).into_owned();
        Ok(msg)
    }
    
    /// send_message sends message through the T
    pub fn send_message<T: Write>(
        msg: &Self, 
        writer: &mut T) -> Result<usize, Box<dyn Error>> {
        // 1. serialize the Message to an u8 array
        let u8_vec = msg.to_u8_vec()?;
        // 2. get the length of the Message and generate a length block
        let msg_len = u64::try_from(u8_vec.len())?;
        let msg_len_blk = msg_len.to_be_bytes();
        // 3. generate the data, which including length block and the message block
        let mut msg_vec = Vec::new();
        msg_vec.extend_from_slice(&msg_len_blk);
        msg_vec.extend(u8_vec.iter().cloned());
        // 4. write the message
        let byts_write = writer.write(msg_vec.as_slice())?;
        Ok(byts_write)
    }
    
    /// receive_message receives message from the reader
    pub fn receive_message<T: Read>(reader: &mut T) -> Result<Message, Box<dyn Error>>{
        // 1. get the first 8 bytes of data 
        let mut msg_len_blk = [0; size_of::<u64>()/size_of::<u8>()];
        let mut byts_read = reader.read(&mut msg_len_blk)?;
        if byts_read == 0 {
            return Err("read nothing from the stream".into());
        }
        // 2. get the length of the message data
        let msg_len = u64::from_be_bytes(msg_len_blk) as usize;

        // 3. get the message data
        let mut msg_blk_vec = vec![0; msg_len];
        byts_read = reader.read(&mut msg_blk_vec)?;
        if byts_read == 0 {
            return Err("read nothing from the stream".into());
        }
        // 4. deserialize the message from the vector
        let msg = Self::from_u8_vec(msg_blk_vec)?;
        Ok(msg)
    }
}

impl PartialEq for MessageType {
    fn eq(&self, other: &Self) -> bool {
        (*self as u8) == (*other as u8)
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.message_type == other.message_type && 
            self.content == other.content
    }
}

#[cfg(test)]
mod test {
    use super::MessageType;
    use super::Message;
    use std::io::{Cursor, Seek, SeekFrom};
    
    #[test]
    fn test_encode_decode() {
        let msg = Message {
            message_type: MessageType::Election,
            content: String::from("this is an election message"),
        };
        let u8_vec = msg.to_u8_vec().unwrap();
        let decode_msg = Message::from_u8_vec(u8_vec).unwrap();
        assert_eq!(msg, decode_msg);
    }

    #[test]
    fn test_send_receive_msg() {
        let msg = Message {
            message_type: MessageType::Election,
            content: String::from("this is an election message"),
        };
        let mut tunnel = Cursor::new(Vec::<u8>::new());
        Message::send_message(&msg, &mut tunnel).unwrap();
        tunnel.seek(SeekFrom::Start(0)).unwrap();
        let rcv_msg = Message::receive_message(&mut tunnel).unwrap();
        assert_eq!(msg, rcv_msg);
    }
}
