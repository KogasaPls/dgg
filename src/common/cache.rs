use anyhow::Result;
use cached::{Cached, UnboundCache};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::collections::HashMap;

use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonCache {
    cache_path: PathBuf,

    #[serde(skip)]
    cache: Option<UnboundCache<String, Value>>,
}

impl JsonCache {
    pub fn new(cache_path: PathBuf) -> Self {
        if !cache_path.exists() {
            debug!("Creating cache directory: {}", cache_path.display());
            std::fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
            std::fs::write(&cache_path, "{}").unwrap();
            Self {
                cache_path,
                cache: Some(UnboundCache::new()),
            }
        } else {
            info!("Loading cache from: {}", cache_path.display());
            Self::load(cache_path).unwrap()
        }
    }

    pub fn save(&self) -> Result<()> {
        if !self.cache_path.exists() {
            std::fs::create_dir_all(&self.cache_path)?;
        }

        let store = self.cache.as_ref().unwrap().get_store();
        let store_json = serde_json::to_string_pretty(&store)?;
        std::fs::write(&self.cache_path, store_json)?;
        Ok(())
    }

    pub fn load(cache_path: PathBuf) -> Result<Self> {
        let store_json = std::fs::read_to_string(&cache_path)?;
        let store: HashMap<String, Value> = serde_json::from_str(&store_json)?;
        let mut cache = UnboundCache::new();
        for (k, v) in store {
            cache.cache_set(k, v);
        }

        Ok(Self {
            cache_path,
            cache: Some(cache),
        })
    }

    pub fn get<T>(&mut self, key: String) -> Result<Option<T>>
    where
        T: Clone + serde::Serialize + DeserializeOwned,
    {
        self.cache
            .as_mut()
            .unwrap()
            .cache_get(&key)
            .map(|v| serde_json::from_value::<T>(v.clone()))
            .transpose()
            .map_err(|e| e.into())
    }

    pub fn set<T>(&mut self, key: String, value: T) -> Result<Option<T>>
    where
        T: Clone + serde::Serialize + DeserializeOwned,
    {
        let value = serde_json::to_value(value)?;
        let old_value = self.cache.as_mut().unwrap().cache_set(key, value);
        self.save()?;

        Ok(old_value.map(|v| serde_json::from_value::<T>(v).unwrap()))
    }
}
