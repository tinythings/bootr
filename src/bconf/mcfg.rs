// Bootr Config: bconf

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, Error, ErrorKind},
    path::PathBuf,
};

// Part of main BootrConfig
#[derive(Debug, Serialize, Deserialize)]
pub struct CnfOciRegistry {
    pub image: String,
    pub login: IndexMap<String, String>,
}

// Part of main BootrConfig
#[derive(Debug, Serialize, Deserialize)]
pub struct CnfSystem {
    pub autoupdate: bool,
    pub check: String,
    #[serde(rename = "keep-data")]
    pub keep_data: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BootrConfig {
    #[serde(rename = "oci-registry")]
    pub oci_registry: CnfOciRegistry,
    pub system: CnfSystem,
}

/// Read bootr config
#[allow(dead_code, clippy::unnecessary_unwrap)]
pub fn get_bootr_config(pth: PathBuf) -> Result<BootrConfig, Error> {
    log::debug!("Loading main Bootr config from {:?}", pth);
    if !pth.exists() {
        return Err(Error::new(ErrorKind::NotFound, format!("Configuration file at {} is missing", pth.to_str().unwrap())));
    }

    match serde_yaml::from_reader::<BufReader<std::fs::File>, BootrConfig>(BufReader::new(File::open(pth)?)) {
        Ok(cfg) => Ok(cfg),
        Err(err) => Err(Error::new(std::io::ErrorKind::InvalidData, err)),
    }
}
