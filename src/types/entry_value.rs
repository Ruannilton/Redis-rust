use crate::utils;

use super::value_container::ValueContainer;

#[derive(Debug)]
pub struct EntryValue {
    pub(crate) value: ValueContainer,
    pub(crate) expires_at: Option<u128>,
}

impl EntryValue {
    pub fn get_value(&self) -> Option<ValueContainer> {
        if let Some(exp) = self.expires_at {
            let current_time = utils::get_current_time_ms();
            if current_time < exp {
                return Some(self.value.clone());
            }
            return None;
        }
        return Some(self.value.clone());
    }
}
