use crate::{
    resp::resp_serializer, resp_desserializer::RespTk, types::execution_response::ExecResponse,
};

pub fn execute_echo(token: &RespTk) -> ExecResponse {
    if let Some(val) = token
        .get_command_args()
        .next()
        .and_then(|t| t.get_content_string())
    {
        resp_serializer::to_resp_bulk(val).into()
    } else {
        resp_serializer::to_err_string("ERROR no value provided".into()).into()
    }
}
