use crate::resp::resp_serializer;

pub fn execute_ping() -> String {
    resp_serializer::to_resp_string("PONG".into())
}
