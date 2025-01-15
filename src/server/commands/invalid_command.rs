use crate::resp::resp_serializer;

pub fn execute_invalid() -> String {
    resp_serializer::to_err_string("INVALID".into())
}
