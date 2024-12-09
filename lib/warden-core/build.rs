enum Entity {
    Date,
    RuleConfig,
    Pain001,
    Pain013,
    Pacs008,
    Pacs002,
}

impl Entity {
    fn path(&self) -> String {
        match self {
            Entity::Date => "proto/google/date.proto",
            Entity::RuleConfig => "proto/config/rule.proto",
            Entity::Pain001 => "proto/iso20022/pain.001.001.12.proto",
            Entity::Pain013 => "proto/iso20022/pain.013.001.11.proto",
            Entity::Pacs008 => "proto/iso20022/pacs.008.001.12.proto",
            Entity::Pacs002 => "proto/iso20022/pacs.002.001.12.proto",
        }
        .into()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto");

    let protos = vec![
        Entity::Date,
        Entity::RuleConfig,
        Entity::Pain001,
        Entity::Pain013,
        Entity::Pacs008,
        Entity::Pacs002,
    ];

    for proto in protos {
        let path = proto.path();

        let config = tonic_build::configure();

        #[cfg(feature = "serde")]
        let config = config.type_attribute(
            ".",
            "#[derive(serde::Serialize, serde::Deserialize)] #[serde(rename_all = \"snake_case\")]",
        );

        #[cfg(feature = "openapi")]
        let config = config.type_attribute(".", "#[derive(utoipa::ToSchema)]");

        config
            .compile_well_known_types(true)
            .compile_protos(&[path], &[""])?;
    }

    Ok(())
}
