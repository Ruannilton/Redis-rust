use std::{cmp::Ordering, error::Error, fmt, u128, u64};

use crate::{
    resp_serializer::{to_resp_array, to_resp_bulk, RespSerializer},
    resp_type::RespToken,
    utils,
};

#[derive(Debug)]
pub enum Command {
    Ping,
    Echo(ValueContainer),
    Set(String, ValueContainer, Option<u128>),
    Get(String),
    ConfigGet(String),
    Keys(String),
    Type(String),
    XAdd(String, String, Vec<(String, String)>),
    XRange(String, String, String),
    XRead(Option<u64>, Vec<String>, Vec<String>),
}

#[derive(Debug, Clone)]
pub struct StreamEntry {
    pub id: StreamKey,
    pub fields: Vec<(String, String)>,
}

impl Into<String> for &StreamEntry {
    fn into(self) -> String {
        let fields = self
            .fields
            .iter()
            .map(|i| format!("{}: {}", i.0, i.1))
            .collect::<Vec<String>>()
            .join(", ");
        let id_str: String = self.id.clone().into();
        format!("{{{} [{}]}}", id_str, fields)
    }
}

impl RespSerializer for StreamEntry {
    fn to_resp(&self) -> String {
        let fields_array: Vec<String> = self
            .fields
            .iter()
            .map(|x| [x.0.clone(), x.1.clone()])
            .flatten()
            .collect();
        let fields_resp = to_resp_array(fields_array);
        let id_resp = to_resp_bulk(self.id.into());
        format!("*2\r\n{id_resp}{fields_resp}")
    }
}

#[derive(Debug, Clone)]
pub enum ValueContainer {
    String(String),
    Stream(Vec<StreamEntry>),
    Integer(i64),
    Array(Vec<ValueContainer>),
}

impl Into<String> for ValueContainer {
    fn into(self) -> String {
        to_string(&self)
    }
}

impl Into<String> for &ValueContainer {
    fn into(self) -> String {
        to_string(self)
    }
}

impl From<RespToken> for ValueContainer {
    fn from(value: RespToken) -> Self {
        from_aux(&value)
    }
}

impl From<&RespToken> for ValueContainer {
    fn from(value: &RespToken) -> Self {
        from_aux(value)
    }
}

fn to_string(container: &ValueContainer) -> String {
    match container {
        ValueContainer::String(s) => s.to_owned(),
        ValueContainer::Integer(i) => i.to_string(),
        ValueContainer::Stream(a) => a
            .iter()
            .map(|x| x.into())
            .collect::<Vec<String>>()
            .join(", "),
        ValueContainer::Array(a) => a
            .iter()
            .map(|x| to_string(x))
            .collect::<Vec<String>>()
            .join(", "),
    }
}

fn from_aux(value: &RespToken) -> ValueContainer {
    match value {
        RespToken::String(s) => ValueContainer::String(s.to_owned()),
        RespToken::Integer(i) => ValueContainer::Integer(i.to_owned()),
        RespToken::Error(s) => ValueContainer::String(s.to_owned()),
        RespToken::Array(a) => ValueContainer::Array(a.iter().map(|x| from_aux(x)).collect()),
    }
}

#[derive(Debug, Clone)]
pub struct StreamKeyDesserializerError {
    message: String,
}

impl StreamKeyDesserializerError {
    // Constructor to create a new error with a message
    pub fn new(msg: &str) -> StreamKeyDesserializerError {
        StreamKeyDesserializerError {
            message: msg.to_string(),
        }
    }
}

impl fmt::Display for StreamKeyDesserializerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RespDesserializerError: {}", self.message)
    }
}

impl Error for StreamKeyDesserializerError {}

#[derive(Debug, Clone, Copy)]
pub struct StreamKey {
    pub miliseconds_time: u128,
    pub sequence_number: u64,
}

impl Into<String> for StreamKey {
    fn into(self) -> String {
        format!("{}-{}", self.miliseconds_time, self.sequence_number)
    }
}

impl StreamKey {
    pub fn new(miliseconds_time: u128, sequence_number: u64) -> Self {
        Self {
            miliseconds_time,
            sequence_number,
        }
    }

    pub fn from_now(sequence_number: u64) -> Self {
        let ms = utils::get_current_time_ms();
        Self {
            miliseconds_time: ms,
            sequence_number,
        }
    }

    pub fn from_string(
        key: &String,
        last_key: &Option<StreamKey>,
        sequence: Option<u64>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if key == "-" {
            return Ok(Self::new(0, 1));
        }
        if key == "+" {
            return Ok(Self::new(u128::MAX, u64::MAX));
        }
        if key == "*" {
            return Ok(Self::from_now(0));
        }

        let splited: Vec<&str> = key.split('-').collect();

        let time = splited
            .get(0)
            .ok_or(StreamKeyDesserializerError::new("Id inválido"))?;

        let time_u128 = u128::from_str_radix(time, 10)?;

        let sequence = if let Some(sequence) = splited.get(1) {
            if *sequence == "*" {
                if let Some(key) = last_key {
                    if key.miliseconds_time == time_u128 {
                        return Ok(key.inc_sequence());
                    }
                }
                let new_seq = if time_u128 == 0 { 1 } else { 0 };
                return Ok(StreamKey::new(time_u128, new_seq));
            }

            u64::from_str_radix(sequence, 10)?
        } else {
            sequence.ok_or(StreamKeyDesserializerError::new("Id inválido"))?
        };

        Ok(StreamKey::new(time_u128, sequence))
    }

    pub fn from_time_string(
        time: &String,
        sequence: u64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let time_u128 = u128::from_str_radix(time, 10)?;
        Ok(Self {
            miliseconds_time: time_u128,
            sequence_number: sequence,
        })
    }

    fn inc_sequence(&self) -> Self {
        Self {
            miliseconds_time: self.miliseconds_time,
            sequence_number: self.sequence_number + 1,
        }
    }
}

impl Ord for StreamKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for StreamKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let cmp = (
            self.miliseconds_time.cmp(&other.miliseconds_time),
            self.sequence_number.cmp(&other.sequence_number),
        );

        let cmp = match cmp {
            (Ordering::Greater, _) => Some(Ordering::Greater),
            (Ordering::Less, _) => Some(Ordering::Less),
            (Ordering::Equal, Ordering::Less) => Some(Ordering::Less),
            (Ordering::Equal, Ordering::Greater) => Some(Ordering::Greater),
            (Ordering::Equal, Ordering::Equal) => Some(Ordering::Equal),
        };

        cmp
    }

    fn lt(&self, other: &Self) -> bool {
        std::matches!(self.partial_cmp(other), Some(std::cmp::Ordering::Less))
    }

    fn le(&self, other: &Self) -> bool {
        std::matches!(
            self.partial_cmp(other),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
        )
    }

    fn gt(&self, other: &Self) -> bool {
        std::matches!(self.partial_cmp(other), Some(std::cmp::Ordering::Greater))
    }

    fn ge(&self, other: &Self) -> bool {
        std::matches!(
            self.partial_cmp(other),
            Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
        )
    }
}

impl PartialEq for StreamKey {
    fn eq(&self, other: &Self) -> bool {
        self.miliseconds_time == other.miliseconds_time
            && self.sequence_number == other.sequence_number
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for StreamKey {}
