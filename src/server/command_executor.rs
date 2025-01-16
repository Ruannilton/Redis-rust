use std::sync::Arc;

use crate::{
    resp::resp_serializer,
    resp_desserializer::RespTk,
    types::{
        connection_context::ConnectionContext, execution_response::ExecResponse,
        transactions::TransactionMap,
    },
};

use super::{commands, redis_app::RedisApp};

pub async fn execute_command(
    app: Arc<RedisApp>,
    token: &RespTk,
    context: ConnectionContext,
) -> ExecResponse {
    let cmd_name = token.get_command_name();

    let mut transations = app.transactions.lock().await;
    let transaction_id: u64 = context.connection_id;

    if let Some(_) = transations.get(transaction_id) {
        match cmd_name {
            "DISCARD" => {
                transations.discard(transaction_id);
            }
            "MULTI" => {
                transations.begin(transaction_id);
            }
            "EXECUTE" => {
                execute_transaction(app.clone(), transaction_id, &mut transations, context).await;
            }
            _ => {
                transations.push(transaction_id, token);
                return resp_serializer::to_resp_string("QUEUED".to_owned()).into();
            }
        }
        return resp_serializer::to_resp_string("OK".to_owned()).into();
    } else {
        return process_command(app.clone(), token, context).await;
    }
}

async fn process_command(
    app: Arc<RedisApp>,
    token: &RespTk,
    context: ConnectionContext,
) -> ExecResponse {
    let cmd_name = token.get_command_name();

    match cmd_name {
        "PING" => commands::ping_command::execute_ping(),
        "ECHO" => commands::echo_command::execute_echo(token),
        "GET" => commands::get_command::execute_get(app, token).await,
        "SET" => commands::set_command::execute_set(app, token).await,
        "CONFIG" => commands::config_command::execute_config(app, token).await,
        "KEYS" => commands::keys_command::execute_keys(app, token).await,
        "TYPE" => commands::type_command::execute_type(app, token).await,
        "XADD" => commands::xadd_command::execute_xadd(app, token).await,
        "XRANGE" => commands::xrange_command::execute_xrange(app, token).await,
        "XREAD" => commands::xread_command::execute_xread(app, token).await,
        "INC" => commands::command_inc::execute_inc(app, token).await,
        "INFO" => commands::info_command::execute_info(app, token).await,
        "REPLCONF" => commands::replconf_command::execute_replconf(app, token, context).await,
        "PSYNC" => commands::psync_command::execute_psync(app, token).await,
        _ => commands::invalid_command::execute_invalid(),
    }
}

async fn execute_transaction(
    app: Arc<RedisApp>,
    transaction_id: u64,
    transaction_map: &mut TransactionMap,
    context: ConnectionContext,
) -> String {
    if let Some(tx) = transaction_map.get(transaction_id) {
        for tk in tx {
            process_command(app.clone(), tk, context.clone()).await;
        }
    }
    transaction_map.discard(transaction_id);
    return resp_serializer::to_resp_string("OK".to_owned());
}
