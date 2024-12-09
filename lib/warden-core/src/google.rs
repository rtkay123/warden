pub mod protobuf {
    include!(concat!(env!("OUT_DIR"), "/google.protobuf.rs"));
}

pub mod r#type {
    tonic::include_proto!("google.r#type");
}
