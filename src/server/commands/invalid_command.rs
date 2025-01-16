use crate::{resp::resp_serializer, types::execution_response::ExecResponse};

pub fn execute_invalid() -> ExecResponse {
    resp_serializer::to_err_string("INVALID".into()).into()
}
