use std::sync::Arc;

use crate::{
    resp::resp_serializer, resp_desserializer::RespTk, server::redis_app::RedisApp,
    types::execution_response::ExecResponse,
};

pub async fn execute_info(app: Arc<RedisApp>, _token: &RespTk) -> ExecResponse {
    let mut response_str = String::new();

    response_str.push_str("# Replication\n");

    let instance_type = match app.settings.instance_type {
        crate::types::instance_type::InstanceType::Master => "role:master",
        crate::types::instance_type::InstanceType::Slave => "role:slave",
    };

    response_str.push_str(instance_type);

    if let Some(master_replid) = &app.settings.master_replid {
        let replid = format!("\nmaster_replid:{}", master_replid);
        response_str.push_str(replid.as_str());
    }

    let master_repl_offset = format!("\nmaster_repl_offset:{}", app.settings.master_repl_offset);
    response_str.push_str(master_repl_offset.as_str());

    let resp = resp_serializer::to_resp_bulk(response_str);

    resp.into()
}
