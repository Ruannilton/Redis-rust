use crate::{resp::resp_serializer, resp_desserializer::RespTk};

pub fn execute_echo(token: &RespTk) -> String {
    if let Some(val) = token
        .get_command_args()
        .next()
        .and_then(|t| t.get_content_string())
    {
        resp_serializer::to_resp_bulk(val)
    } else {
        resp_serializer::to_err_string("ERROR no value provided".into())
    }
}
