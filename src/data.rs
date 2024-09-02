use std::collections::HashMap;

use log::{info, warn};
use serde::{Deserialize, Serialize};

use crate::constants;

#[derive(Serialize, Deserialize)]
pub struct Data(HashMap<String, String>);

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

impl Data {
    pub fn load_or_default() -> Self {
        std::fs::File::open(constants::DATA_PATH)
            .map_err(|err| Error::Io(err))
            .and_then(|file| {
                let result = serde_json::from_reader::<_, Data>(file).map_err(|err| err.into());
                if result.is_ok() {
                    info!("Successfully loaded data from disk.");
                }
                result
            })
            .unwrap_or_else(|err| {
                warn!("Failed to load data: {}. Creating empty data.", err);
                Data(HashMap::new())
            })
    }

    pub fn set(&mut self, key: &str, value: String) {
        self.0.insert(key.to_string(), value);
    }

    pub fn get<'a>(&'a self, key: &str) -> Option<&'a str> {
        self.0.get(key).map(|s| s.as_str())
    }

    pub fn save(&self) {
        let file = std::fs::File::create(constants::DATA_PATH).unwrap();
        serde_json::to_writer(file, self).unwrap();
    }
}

impl Drop for Data {
    fn drop(&mut self) {
        info!("Data dropped, saving to disk.");
        self.save();
    }
}