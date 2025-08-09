#[cfg(feature = "iso20022")]
enum Entity {
    #[cfg(feature = "iso20022")]
    ISO2022,
}

#[cfg(feature = "iso20022")]
impl Entity {
    fn protos(&self) -> Vec<&'static str> {
        let mut res: Vec<&'static str> = vec![];

        #[cfg(feature = "iso20022")]
        fn iso20022_protos() -> Vec<&'static str> {
            vec![
                "proto/iso20022/pacs_008_001_12.proto",
                "proto/iso20022/pacs_002_001_12.proto",
                "proto/googleapis/google/type/money.proto",
                "proto/warden_message.proto",
            ]
        }

        match self {
            #[cfg(feature = "iso20022")]
            Entity::ISO2022 => {
                res.extend(iso20022_protos());
            }
        }
        res
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../../proto");

    #[cfg(feature = "iso20022")]
    build_proto("iso20022", Entity::ISO2022)?;

    Ok(())
}

#[cfg(feature = "iso20022")]
fn build_proto(package: &str, entity: Entity) -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let config = tonic_prost_build::configure()
         .server_mod_attribute(
                package,
                format!("#[cfg(feature = \"rpc-server-{package}\")] #[cfg_attr(docsrs, doc(cfg(feature = \"rpc-server-{package}\")))]"),
            )
            .client_mod_attribute(
                package,
                format!("#[cfg(feature = \"rpc-client-{package}\")] #[cfg_attr(docsrs, doc(cfg(feature = \"rpc-client-{package}\")))]"),
            );

    #[cfg(feature = "serde")]
    let config = add_serde(config);

    #[cfg(feature = "openapi")]
    let config = add_openapi(config);

    config
        .file_descriptor_set_path(out_dir.join(format!("{package}_descriptor.bin")))
            .server_mod_attribute(
                package,
                format!("#[cfg(feature = \"rpc-server-{package}\")] #[cfg_attr(docsrs, doc(cfg(feature = \"rpc-server-{package}\")))]"),
            )
            .client_mod_attribute(
                package,
                format!("#[cfg(feature = \"rpc-client-{package}\")] #[cfg_attr(docsrs, doc(cfg(feature = \"rpc-client-{package}\")))]"),
            )
        .protoc_arg("-I=../..")
        .compile_well_known_types(true)
        .compile_protos(
            &entity.protos(),
            &["../../proto/googleapis", "../../proto"], // specify the root location to search proto dependencies
        )?;

    Ok(())
}

#[cfg(all(feature = "serde", feature = "iso20022"))]
fn add_serde(config: tonic_prost_build::Builder) -> tonic_prost_build::Builder {
    let config = config.type_attribute(
        ".",
        "#[derive(serde::Serialize, serde::Deserialize)] #[serde(rename_all = \"snake_case\")]",
    );

    #[cfg(feature = "serde-time")]
    let config = config.type_attribute(
        ".google.protobuf.Timestamp",
        "#[serde(try_from = \"time::OffsetDateTime\")] #[serde(into = \"String\")]",
    );

    config
}

#[cfg(all(feature = "openapi", feature = "iso20022"))]
fn add_openapi(config: tonic_prost_build::Builder) -> tonic_prost_build::Builder {
    config.type_attribute(".", "#[derive(utoipa::ToSchema)]")
}
