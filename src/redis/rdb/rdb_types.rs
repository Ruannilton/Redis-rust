use crate::redis::redis_error::RedisError;

pub(crate) enum SizeEncodedValue {
    Size(usize),
    IntegerString(IntegerStringLen),
    LZFString,
}

pub(crate) enum IntegerStringLen {
    SingleWord = 0,
    DoubleWord = 1,
    QuadWord = 2,
}

#[repr(u8)]
#[derive(PartialEq, Eq)]
pub(crate) enum OpCodes {
    EOF = 0xFF,
    SelectDb = 0xFE,
    ExpireTime = 0xFD,
    ExpireTimeMs = 0xFC,
    ResizeDb = 0xFB,
    Metadata = 0xFA,
    StringValue = 0x00,
}

impl TryInto<OpCodes> for u8 {
    type Error = RedisError;

    fn try_into(self) -> Result<OpCodes, Self::Error> {
        match self {
            0xFF => Ok(OpCodes::EOF),
            0xFE => Ok(OpCodes::SelectDb),
            0xFD => Ok(OpCodes::ExpireTime),
            0xFC => Ok(OpCodes::ExpireTimeMs),
            0xFB => Ok(OpCodes::ResizeDb),
            0xFA => Ok(OpCodes::Metadata),
            0x00 => Ok(OpCodes::StringValue),
            _ => Err(RedisError::InvalidOpCode),
        }
    }
}
