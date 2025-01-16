use crate::{resp::resp_serializer, types::execution_response::ExecResponse};

pub fn execute_ping() -> ExecResponse {
    resp_serializer::to_resp_string("PONG".into()).into()
}
