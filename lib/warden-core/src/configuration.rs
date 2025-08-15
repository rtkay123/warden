#[cfg(feature = "serde")]
pub(crate) mod conv;

tonic::include_proto!("configuration");

pub mod routing {
    tonic::include_proto!("configuration.routing");
}

pub mod rule {
    tonic::include_proto!("configuration.rule");
}
