mod parser;

/// Well known types
pub mod protobuf {
    include!(concat!(env!("OUT_DIR"), "/google.protobuf.rs"));
}

pub mod r#type {
    include!(concat!(env!("OUT_DIR"), "/google.r#type.rs"));
}
