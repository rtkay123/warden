#[cfg(any(feature = "message", feature = "pseudonyms"))]
enum Entity {
    #[cfg(feature = "message")]
    ISO2022,
    #[cfg(feature = "pseudonyms")]
    Pseudonyms,
}

#[cfg(any(feature = "message", feature = "pseudonyms"))]
impl Entity {
    fn protos(&self) -> Vec<&'static str> {
        let mut res: Vec<&'static str> = vec![];

        #[cfg(feature = "message")]
        fn iso20022_protos() -> Vec<&'static str> {
            vec![
                "proto/warden_message.proto",
            ]
        }

        #[cfg(feature = "pseudonyms")]
        fn pseudonyms_protos() -> Vec<&'static str> {
            vec![
                "proto/pseudonyms/account.proto",
                "proto/pseudonyms/entity.proto",
                "proto/pseudonyms/account_holder.proto",
                "proto/pseudonyms/transaction_relationship.proto",
            ]
        }

        match self {
            #[cfg(feature = "message")]
            Entity::ISO2022 => {
                res.extend(iso20022_protos());
            }
            #[cfg(feature = "pseudonyms")]
            Entity::Pseudonyms => {
                res.extend(pseudonyms_protos());
            }
        }
        res
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../../proto");

#[cfg(any(feature = "message", feature = "pseudonyms"))]
    let mut protos: Vec<&'static str> = vec![];

    #[cfg(feature = "message")]
    protos.extend(Entity::ISO2022.protos());

    #[cfg(feature = "pseudonyms")]
    protos.extend(Entity::Pseudonyms.protos());

    #[cfg(any(feature = "message", feature = "pseudonyms"))]
    build_proto(&protos)?;

    Ok(())
}

#[cfg(any(feature = "message", feature = "pseudonyms"))]
fn build_proto(protos: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let config = tonic_prost_build::configure();

    #[cfg(feature = "serde")]
    let config = add_serde(config);

    #[cfg(feature = "openapi")]
    let config = add_openapi(config);

    config
        .file_descriptor_set_path(out_dir.join("warden_descriptor.bin"))
        .protoc_arg("-I=../..")
        .compile_well_known_types(true)
        .compile_protos(
            protos,
            &["../../proto", "../../proto/googleapis"], // specify the root location to search proto dependencies
        )?;

    Ok(())
}

#[cfg(all(feature = "serde", any(feature = "pseudonyms", feature = "message")))]
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

#[cfg(all(feature = "openapi", any(feature = "message", feature = "pseudonyms")))]
fn add_openapi(config: tonic_prost_build::Builder) -> tonic_prost_build::Builder {
    config.type_attribute(".", "#[derive(utoipa::ToSchema)]")
}
