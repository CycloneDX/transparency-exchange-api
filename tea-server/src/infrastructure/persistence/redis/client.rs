use redis::Client;

pub struct RedisClient {
    #[allow(dead_code)]
    client: Client,
}

impl RedisClient {
    pub fn new(url: &str) -> Result<Self, redis::RedisError> {
        Ok(Self {
            client: Client::open(url)?,
        })
    }
}
