// Bootr Config: bconf

use super::defaults;
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
    image: String,
    login: IndexMap<String, String>,
}

// Part of main BootrConfig
#[derive(Debug, Serialize, Deserialize)]
pub struct CnfSystem {
    autoupdate: bool,
    check: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BootrConfig {
    #[serde(rename = "oci-registry")]
    oci_registry: CnfOciRegistry,
    system: CnfSystem,
}

/// Read bootr config
pub fn get_bootr_config(pth: Option<PathBuf>) -> Result<BootrConfig, Error> {
    let p: PathBuf;
    if pth.is_some() {
        p = pth.unwrap();
    } else {
        p = PathBuf::from(defaults::C_BOOTR_CFG.to_string());
    }

    if !p.exists() {
        return Err(Error::new(ErrorKind::NotFound, format!("Configuration file at {} is missing", p.to_str().unwrap())));
    }

    match serde_yaml::from_reader::<BufReader<std::fs::File>, BootrConfig>(BufReader::new(File::open(p)?)) {
        Ok(cfg) => Ok(cfg),
        Err(err) => Err(Error::new(std::io::ErrorKind::InvalidData, err)),
    }
}
