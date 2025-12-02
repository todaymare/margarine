use crate::reader::Reader;


#[derive(Clone, Copy, Debug)]
pub enum Leb128Error {
    Overflow,
    EOI,
}

type Result<T> = core::result::Result<T, Leb128Error>;


pub const MAX_SIZE_64: usize = 10;


const CONT: u8 = 0x80;
const SIGN: u8 = 0x40;
const MASK: u8 = 0x7f;


pub fn decode_i64(reader: &mut Reader<u8>) -> Result<i64> {
    let mut result = 0;
    let mut shift  = 0;
    while let Some(byte) = reader.next() {
        // overflow.
        if shift == 63 && !(byte == 0x00 || byte == 0x7f) {
            if !reader.consume_while(|byte| byte & CONT != 0) {
                return Err(Leb128Error::EOI);
            }
            return Err(Leb128Error::Overflow);
        }

        result |= ((byte & MASK) as i64) << shift;
        shift  += 7;

        if byte & CONT == 0 {
            // sign extend.
            if shift < 64 && (byte & SIGN) == SIGN {
                result |= !0 << shift;
            }
            return Ok(result);
        }
    }

    return Err(Leb128Error::EOI);
}

pub fn decode_i32(reader: &mut Reader<u8>) -> Result<i32> {
    decode_i64(reader)?
    .try_into().ok().ok_or(Leb128Error::Overflow)
}


pub fn decode_u64(reader: &mut Reader<u8>) -> Result<u64> {
    let mut result = 0;
    let mut shift  = 0;
    while let Some(byte) = reader.next() {
        // overflow.
        if shift == 63 && !(byte == 0x00 || byte == 0x01) {
            if !reader.consume_while(|byte| byte & CONT != 0) {
                return Err(Leb128Error::EOI);
            }
            return Err(Leb128Error::Overflow);
        }

        result |= ((byte & MASK) as u64) << shift;
        shift  += 7;

        if byte & CONT == 0 {
            return Ok(result);
        }
    }

    return Err(Leb128Error::EOI);
}

pub fn decode_u32(reader: &mut Reader<u8>) -> Result<u32> {
    decode_u64(reader)?
    .try_into().ok().ok_or(Leb128Error::Overflow)
}

