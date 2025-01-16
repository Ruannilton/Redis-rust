use std::{collections::HashMap, sync::Arc};

use crate::{
    resp::resp_serializer,
    resp_desserializer::RespTk,
    server::redis_app::RedisApp,
    types::{execution_response::ExecResponse, value_container::ValueContainer},
};

pub async fn execute_set(app: Arc<RedisApp>, token: &RespTk) -> ExecResponse {
    let mut args = token.get_command_args();

    if let (Some(key), Some(value)) = (
        args.next().and_then(|tk| tk.get_content_string()),
        args.next().and_then(|tk| Some(tk.get_value())),
    ) {
        let opts = get_optional_args(&mut args);
        let exp = get_expiration_time(opts);

        app.put_entry(key, value, exp).await;
        app.broadcast_command(token).await;
        return resp_serializer::to_resp_string("OK".to_owned()).into();
    }
    resp_serializer::null_resp_string().into()
}

fn get_optional_args<'a>(
    args: &mut impl Iterator<Item = &'a RespTk>,
) -> HashMap<String, ValueContainer> {
    let mut result = HashMap::new();
    while let (Some(key), val) = (
        args.next().and_then(|tk| tk.get_content_string()),
        args.next().and_then(|tk| Some(tk.get_value())),
    ) {
        _ = result.insert(key, val.unwrap_or(ValueContainer::Null));
    }
    result
}

fn get_expiration_time(map: HashMap<String, ValueContainer>) -> Option<u128> {
    for (name, val) in map {
        match (name.as_str(), val) {
            ("PX", ValueContainer::String(exp)) => return exp.parse::<u128>().ok(),
            ("EX", ValueContainer::String(exp)) => {
                return exp.parse::<u128>().map(|x| x * 1000).ok()
            }
            _ => {}
        };
    }

    None
}
