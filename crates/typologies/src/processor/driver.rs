use tonic::IntoRequest;
use warden_core::configuration::typology::{TypologyConfiguration, TypologyConfigurationRequest};

use crate::state::AppHandle;

pub trait GetTypologyConfiguration {
    fn get_typology_config(
        &self,
        typology_key: TypologyConfigurationRequest,
    ) -> impl std::future::Future<Output = anyhow::Result<TypologyConfiguration>> + Send;
}

impl GetTypologyConfiguration for AppHandle {
    async fn get_typology_config(
        &self,
        typology_key: TypologyConfigurationRequest,
    ) -> anyhow::Result<TypologyConfiguration> {
        {
            let local_cache = self.local_cache.read().await;
            if let Some(result) = local_cache.get(&typology_key).await.map(Ok) {
                return result;
            }
        }

        let local_cache = self.local_cache.write().await;
        let mut client = self.query_typology_client.clone();

        let value = client
            .get_typology_configuration(typology_key.clone().into_request())
            .await?
            .into_inner()
            .configuration
            .ok_or_else(|| anyhow::anyhow!("configuration unavailable"))?;
        local_cache
            .insert(typology_key.clone(), value.clone())
            .await;

        Ok(value)
    }
}
