use mockall::automock;
use std::collections::{HashMap, HashSet};
use mockall::predicate::str;
use anyhow::Context;
use crate::providers::modio::{LoggingMiddleware, MODIO_DRG_ID, ModioMod, ModioModResponse};

#[cfg_attr(test, automock)]
#[async_trait::async_trait]
pub trait DrgModio: Sync + Send {
    fn with_parameters(parameters: &HashMap<String, String>) -> anyhow::Result<Self>
    where
        Self: Sized;
    async fn check(&self) -> anyhow::Result<()>;
    async fn fetch_mod(&self, id: u32) -> anyhow::Result<ModioMod>;
    async fn fetch_files(&self, mod_id: u32) -> anyhow::Result<ModioMod>;
    async fn fetch_file(&self, mod_id: u32, modfile_id: u32) -> anyhow::Result<modio::files::File>;
    async fn fetch_dependencies(&self, mod_id: u32) -> anyhow::Result<Vec<u32>>;
    async fn fetch_mods_by_name(&self, name_id: &str) -> anyhow::Result<Vec<ModioModResponse>>;
    async fn fetch_mods_by_ids(&self, filter_ids: Vec<u32>) -> anyhow::Result<Vec<modio::mods::Mod>>;
    async fn fetch_mod_updates_since(
        &self,
        mod_ids: Vec<u32>,
        last_update: u64,
    ) -> anyhow::Result<HashSet<u32>>;
    fn download<A: 'static>(&self, action: A) -> modio::download::Downloader
    where
        modio::download::DownloadAction: From<A>;
}

#[async_trait::async_trait]
impl DrgModio for modio::Modio {
    fn with_parameters(parameters: &HashMap<String, String>) -> anyhow::Result<Self> {
        let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
            .with::<LoggingMiddleware>(Default::default())
            .build();
        let modio = modio::Modio::new(
            modio::Credentials::with_token(
                "".to_owned(), // TODO patch modio to not use API key at all
                parameters
                    .get("oauth")
                    .context("missing OAuth token param")?,
            ),
            client,
        )?;

        Ok(modio)
    }
    async fn check(&self) -> anyhow::Result<()> {
        use modio::filter::Eq;
        use modio::mods::filters::Id;
        use crate::providers::modio::MODIO_DRG_ID;

        self.game(MODIO_DRG_ID)
            .mods()
            .search(Id::eq(0))
            .collect()
            .await?;
        Ok(())
    }

    /// fetch_mod fetches a mod description from mod.io
    async fn fetch_mod(&self, id: u32) -> anyhow::Result<ModioMod> {
        use modio::filter::NotEq;
        use modio::mods::filters::Id;
        use crate::providers::modio::MODIO_DRG_ID;

        let files = self
            .game(MODIO_DRG_ID)
            .mod_(id)
            .files()
            .search(Id::ne(0))
            .collect()
            .await?;
        let mod_ = self.game(MODIO_DRG_ID).mod_(id).get().await?;

        Ok(ModioMod::new(mod_, files))
    }

    // seems to be a duplicate of fetch_mod
    // shouldn't this return a Vec<File>?
    async fn fetch_files(&self, mod_id: u32) -> anyhow::Result<ModioMod> {
        use modio::filter::NotEq;
        use modio::mods::filters::Id;
        use crate::providers::modio::MODIO_DRG_ID;

        let files = self
            .game(MODIO_DRG_ID)
            .mod_(mod_id)
            .files()
            .search(Id::ne(0))
            .collect()
            .await?;
        let mod_ = self.game(MODIO_DRG_ID).mod_(mod_id).get().await?;

        Ok(ModioMod::new(mod_, files))
    }
    async fn fetch_file(&self, mod_id: u32, modfile_id: u32) -> anyhow::Result<modio::files::File> {
        Ok(self
            .game(MODIO_DRG_ID)
            .mod_(mod_id)
            .file(modfile_id)
            .get()
            .await?)
    }
    async fn fetch_dependencies(&self, mod_id: u32) -> anyhow::Result<Vec<u32>> {
        Ok(self
            .game(MODIO_DRG_ID)
            .mod_(mod_id)
            .dependencies()
            .list()
            .await?
            .into_iter()
            .map(|d| d.mod_id)
            .collect::<Vec<_>>())
    }
    async fn fetch_mods_by_name(&self, name_id: &str) -> anyhow::Result<Vec<ModioModResponse>> {
        use modio::filter::{Eq, In};
        use modio::mods::filters::{NameId, Visible};

        let filter = NameId::eq(name_id).and(Visible::_in(vec![0, 1]));
        Ok(self
            .game(MODIO_DRG_ID)
            .mods()
            .search(filter)
            .collect()
            .await?
            .into_iter()
            .map(|m| m.into())
            .collect())
    }
    async fn fetch_mods_by_ids(&self, filter_ids: Vec<u32>) -> anyhow::Result<Vec<modio::mods::Mod>> {
        use modio::filter::In;
        use modio::mods::filters::Id;

        let filter = Id::_in(filter_ids);

        Ok(self
            .game(MODIO_DRG_ID)
            .mods()
            .search(filter)
            .collect()
            .await?)
    }
    async fn fetch_mod_updates_since(
        &self,
        mod_ids: Vec<u32>,
        last_update: u64,
    ) -> anyhow::Result<HashSet<u32>> {
        use modio::filter::Cmp;
        use modio::filter::In;
        use modio::filter::NotIn;

        use modio::mods::filters::events::EventType;
        use modio::mods::filters::events::ModId;
        use modio::mods::filters::DateAdded;
        use modio::mods::EventType as EventTypes;

        let events = self
            .game(MODIO_DRG_ID)
            .mods()
            .events(
                EventType::not_in(vec![
                    EventTypes::ModCommentAdded,
                    EventTypes::ModCommentDeleted,
                ])
                .and(ModId::_in(mod_ids))
                .and(DateAdded::gt(last_update)),
            )
            .collect()
            .await?;
        Ok(events.iter().map(|e| e.mod_id).collect::<HashSet<_>>())
    }
    fn download<A>(&self, action: A) -> modio::download::Downloader
    where
        modio::download::DownloadAction: From<A>,
    {
        self.download(action)
    }
}

#[cfg(test)]
mod test {
    use std::sync::{OnceLock, RwLock};

    use crate::{providers::VersionAnnotatedCache, state::config::ConfigWrapper};

    use super::*;
}
