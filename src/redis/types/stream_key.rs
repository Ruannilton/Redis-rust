use std::cmp::Ordering;

use crate::{redis::redis_error::RedisError, utils};

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
    ) -> Result<Self, RedisError> {
        if key == "$" {
            return Ok(last_key.unwrap());
        }
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
            .ok_or(RedisError::InvalidStreamEntryId(key.to_owned()))?;

        let time_u128 = u128::from_str_radix(time, 10)
            .map_err(|_| RedisError::InvalidStreamEntryId(key.to_owned()))?;

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

            u64::from_str_radix(sequence, 10)
                .map_err(|_| RedisError::InvalidStreamEntryId(key.to_owned()))?
        } else {
            sequence.ok_or(RedisError::InvalidStreamEntryId(key.to_owned()))?
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
