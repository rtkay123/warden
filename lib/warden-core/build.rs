fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(false)
        //.out_dir("src/google")  // you can change the generated code's location
        .protoc_arg("-I=../..")
        .compile_protos(
            &["proto/googleapis/google/pubsub/v1/pubsub.proto"],
            &["../../proto/googleapis"], // specify the root location to search proto dependencies
        )?;
    Ok(())
}
