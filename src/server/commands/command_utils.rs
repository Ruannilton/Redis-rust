use crate::resp_desserializer::RespTk;

pub fn get_next_arg_string<'a>(args: &mut impl Iterator<Item = &'a RespTk>) -> Option<String> {
    args.next().and_then(|t| t.get_content_string())
}
