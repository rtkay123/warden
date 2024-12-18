enum Entity {
    Date,
    Pacs008,
    Pacs002,
    TransactionRelationship,
    Account,
    Entity,
    AccountHolder,
}

impl Entity {
    fn path(&self) -> String {
        match self {
            Entity::Date => "proto/google/date.proto",
            Entity::Pacs008 => "proto/iso20022/pacs.008.001.12.proto",
            Entity::Pacs002 => "proto/iso20022/pacs.002.001.12.proto",
            Entity::TransactionRelationship => "proto/pseudonyms/transaction_relationship.proto",
            Entity::Account => "proto/pseudonyms/account.proto",
            Entity::Entity => "proto/pseudonyms/entity.proto",
            Entity::AccountHolder => "proto/pseudonyms/account_holder.proto",
        }
        .into()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto");

    let protos = vec![
        Entity::Date,
        Entity::Pacs008,
        Entity::Pacs002,
        Entity::TransactionRelationship,
        Entity::Account,
        Entity::AccountHolder,
        Entity::Entity,
    ];

    generate(&protos, None)?;

    Ok(())
}

fn generate(protos: &[Entity], server_mod: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    for proto in protos {
        let path = proto.path();

        let mut config = tonic_build::configure();

        #[cfg(feature = "serde")]
        {
            config = config.type_attribute(
                ".",
                "#[derive(serde::Serialize, serde::Deserialize)] #[serde(rename_all = \"snake_case\")]",
            );
        }

        #[cfg(feature = "openapi")]
        {
            config = config.type_attribute(".", "#[derive(utoipa::ToSchema)]");
        }

        if let Some(feature) = server_mod {
            config = config
                .server_mod_attribute("attrs", format!("#[cfg(feature = \"server-{feature}\")]"))
                .client_mod_attribute("attrs", format!("#[cfg(feature = \"client-{feature}\")]"));
        }

        config = config.compile_well_known_types(true);

        config.compile_protos(&[path], &[""])?;
    }
    Ok(())
}
