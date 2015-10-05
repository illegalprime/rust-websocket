use dataframe::{DataFrame, Opcode};

pub struct Message<'a>(DataFrame);

impl<'a> Message<'a> {
    pub fn string(msg: &'a str) -> Self {
        Message(DataFrame {
            finished: true,
            op:       Opcode::Text,
            data:     Some(msg.as_bytes()),
        })
    }

    pub fn binary(msg: &'a [u8]) -> Self {
        Message(DataFrame {
            finished: true,
            op:       Opcode::Binary,
            data:     Some(msg),
        })
    }

    pub fn ping(id: &'a [u8]) -> Self {
        Message(DataFrame {
            finished: true,
            op:       Opcode::Ping,
            data:     Some(id),
        })
    }

    pub fn pong(id: &'a [u8]) -> Self {
        Message(DataFrame {
            finished: true,
            op:       Opcode::Pong, 
            data:     Some(id),
        })
    }

    pub fn close() -> Self {
        Message(DataFrame::new())
    }
}


