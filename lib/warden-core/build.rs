enum Entity {
    RuleConfig,
}

impl Entity {
    fn path(&self) -> String {
        match self {
            Entity::RuleConfig => "proto/config/rule.proto",
        }
        .into()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto");

    let protos = vec![Entity::RuleConfig];

    for proto in protos {
        let path = proto.path();

        let config = tonic_build::configure();

        #[cfg(feature = "serde")]
        let config = config.type_attribute(
            ".",
            "#[derive(serde::Serialize, serde::Deserialize)] #[serde(rename_all = \"snake_case\")]",
        );

        config
            .compile_well_known_types(true)
            .compile_protos(&[path], &[""])?;
    }

    Ok(())
}
