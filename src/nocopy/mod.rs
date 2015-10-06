use ws::util::header::DataFrameHeader;

pub struct Message<'a>(DataFrame);

impl<'a> Message<'a> {
    pub fn string(msg: &'a str) -> Self {
        Message(DataFrame::oneshot(Opcode::Text, Some(msg.as_bytes())))
    }

    pub fn binary(msg: &'a [u8]) -> Self {
        Message(DataFrame::oneshot(Opcode::Binary, Some(msg)))
    }

    pub fn ping(id: &'a [u8]) -> Self {
        Message(DataFrame::oneshot(Opcode::Ping, Some(msg)))
    }

    pub fn pong(id: &'a [u8]) -> Self {
        Message(DataFrame::oneshot(Opcode::Pong, Some(msg)))
    }

    pub fn close() -> Self {
        Message(DataFrame::oneshot(Opcode::Close, None))
    }

    pub fn close_because(code: u16, reason: &str) -> Self {
        Message(DataFrame::oneshot(Opcode::Close, Some(msg)))
    }
}

pub struct DataFrame<'a> {
    /// Whether or no this constitutes the end of a message
    pub finished: bool,
    /// The reserved portion of the data frame (RFC6455 5.2)
    pub reserved: [bool; 3],
    /// The opcode associated with this data frame
    pub opcode: Opcode,
    /// The payload associated with this data frame
    pub data: Option<&'a [u8]>,
}

impl<'a> WritableDataFrame for DataFrame<'a> {
    fn opcode(&self) -> Opcode {
        return self.opcode;
    }

    fn is_last(&self) -> bool {
        return self.finished;
    }

    fn reserved(&self) -> [bool; 3] {
        return self.reserved;
    }

    fn data(&self) -> &[u8] {
        return self.data;
    }
}

impl<'a> DataFrame<'a> {
    fn oneshot(op: Opcode, data: Option<&'a [u8]>) -> Self {
        DataFrame {
            finished: true,
            reserved: [false; 3],
            opcode:   op,
            data:     data,
        }

        let header = 
    }
}
