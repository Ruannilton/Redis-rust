use crate::redis::{redis_app::RedisApp, redis_error::RedisError};

pub trait Command {
    async fn execute(&self, app: &RedisApp) -> Result<String, RedisError>;
}
