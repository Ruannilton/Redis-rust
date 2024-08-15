use std::collections::VecDeque;

use super::command_token::CommandToken;

pub type Transaction = VecDeque<CommandToken>;
pub type ClientId = u64;
