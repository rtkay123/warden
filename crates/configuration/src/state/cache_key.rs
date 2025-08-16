use warden_stack::redis::ToRedisArgs;

#[derive(Clone, Copy, Debug)]
pub enum CacheKey<'a> {
    ActiveRouting,
    Routing(&'a uuid::Uuid),
    Rule { id: &'a str, version: &'a str },
    Typology { id: &'a str, version: &'a str },
}

impl ToRedisArgs for CacheKey<'_> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + warden_stack::redis::RedisWrite,
    {
        let value = match self {
            CacheKey::ActiveRouting => "routing.active".into(),
            CacheKey::Routing(uuid) => format!("routing.{uuid}"),
            CacheKey::Rule { id, version } => format!("rule.{id}.{version}"),
            CacheKey::Typology { id, version } => format!("typology.{id}.{version}"),
        };

        out.write_arg(value.as_bytes());
    }
}
