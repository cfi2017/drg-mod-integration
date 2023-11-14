use std::collections::{HashMap, HashSet};
use lazy_static::lazy_static;
use modio::download::Downloader;
use modio::DownloadAction;
use modio::files::File;
use modio::mods::Mod;
use reqwest_middleware::ClientWithMiddleware;
use crate::providers::modio::drg_modio::DrgModio;
use crate::providers::modio::{LoggingMiddleware, ModioMod, ModioModResponse};

pub struct SwissDevModio {
    client: ClientWithMiddleware,
}

impl SwissDevModio {
    pub fn new(client: ClientWithMiddleware) -> Self {
        SwissDevModio {
            client
        }
    }
}

const API_URL: &str = "https://mods.swiss.dev/api/v1";

impl DrgModio for SwissDevModio {
    fn with_parameters(parameters: &HashMap<String, String>) -> anyhow::Result<Self> where Self: Sized {
        let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
            .with::<LoggingMiddleware>(Default::default())
            .build();
        let modio = SwissDevModio::new(client);

        Ok(modio)
    }

    async fn check(&self) -> anyhow::Result<()> {
        self.client.get(format!("{}/status", API_URL))
            .send()
            .await?;
        Ok(())
    }

    async fn fetch_mod(&self, id: u32) -> anyhow::Result<ModioMod> {
        self.client.get(format!("{}/mods/{}", API_URL, id))
            .send()
            .await?
            .json()
            .await.map_err(|e| e.into())
    }

    async fn fetch_files(&self, mod_id: u32) -> anyhow::Result<ModioMod> {
        self.fetch_mod(mod_id).await
    }

    async fn fetch_file(&self, mod_id: u32, modfile_id: u32) -> anyhow::Result<File> {
        self.client.get(format!("{}/mods/{}/files/{}", API_URL, mod_id, modfile_id))
            .send()
            .await?
            .json()
            .await.map_err(|e| e.into())
    }

    async fn fetch_dependencies(&self, mod_id: u32) -> anyhow::Result<Vec<u32>> {
        self.client.get(format!("{}/mods/{}/dependencies", API_URL, mod_id))
            .send()
            .await?
            .json()
            .await.map_err(|e| e.into())
    }

    async fn fetch_mods_by_name(&self, name_id: &str) -> anyhow::Result<Vec<ModioModResponse>> {
        self.client.get(format!("{}/mods?name_id={}", API_URL, name_id))
            .send()
            .await?
            .json()
            .await.map_err(|e| e.into())
    }

    // json post to accommodate more ids
    async fn fetch_mods_by_ids(&self, filter_ids: Vec<u32>) -> anyhow::Result<Vec<Mod>> {
        self.client.post(format!("{}/mods", API_URL))
            .json(&filter_ids)
            .send()
            .await?
            .json()
            .await.map_err(|e| e.into())
    }

    async fn fetch_mod_updates_since(&self, mod_ids: Vec<u32>, last_update: u64) -> anyhow::Result<HashSet<u32>> {
        self.client.post(format!("{}/mods?last_update={}", API_URL, last_update))
            .json(&mod_ids)
            .send()
            .await?
            .json()
            .await.map_err(|e| e.into())
    }

    fn download<A: 'static>(&self, action: A) -> Downloader where DownloadAction: From<A> {
        todo!()
    }
}
