use crate::redis::command::command_trait::Command;
use crate::redis::redis_app::RedisApp;
use crate::redis::redis_error::RedisError;
use crate::redis::types::command_token::CommandToken;
use crate::redis::types::transactions::ClientId;

use config_get_command::ConfiGet;
use discard_command::DiscardCommand;
use echo_command::EchoCommand;
use exec_command::ExecCommand;
use get_command::GetCommand;
use inc_command::IncCommand;
use keys_command::KeysCommand;
use multi_command::MultiCommand;
use ping_command::PingCommand;
use set_command::SetCommand;
use type_command::TypeCommand;
use xadd_command::XAddCommand;
use xrange_command::XRangeCommand;
use xread_command::XReadCommand;

use super::*;

pub async fn execute_command(
    app: &RedisApp,
    client_id: ClientId,
    cmd: CommandToken,
) -> Result<String, RedisError> {
    match cmd {
        CommandToken::Ping => PingCommand::new().execute(app).await,
        CommandToken::Echo(arg) => EchoCommand::new(arg.into()).execute(app).await,
        CommandToken::Get(key) => GetCommand::new(key).execute(app).await,
        CommandToken::Set(key, value, expires_at) => {
            SetCommand::new(key, value, expires_at).execute(app).await
        }
        CommandToken::ConfigGet(cfg) => ConfiGet::new(cfg).execute(app).await,
        CommandToken::Keys(arg) => KeysCommand::new(arg).execute(app).await,
        CommandToken::Type(tp) => TypeCommand::new(tp).execute(app).await,
        CommandToken::XAdd(key, id, fields) => XAddCommand::new(key, id, fields).execute(app).await,
        CommandToken::XRange(key, start, end) => {
            XRangeCommand::new(key, start, end).execute(app).await
        }
        CommandToken::XRead(block_time, stream_keys, ids) => {
            XReadCommand::new(block_time, stream_keys, ids)
                .execute(app)
                .await
        }
        CommandToken::Inc(key) => IncCommand::new(key).execute(app).await,
        CommandToken::Multi => MultiCommand::new(client_id).execute(app).await,
        CommandToken::Exec => ExecCommand::new(client_id).execute(app).await,
        CommandToken::Discard => DiscardCommand::new(client_id).execute(app).await,
    }
}
