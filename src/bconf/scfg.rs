// Status config

use nix::errno::Errno;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use std::io::Write;
use std::{fs::File, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusConfig {}

/// Writes OCI config to a file
#[allow(dead_code)]
pub async fn ociconf_to_file(cfg: Vec<u8>, pth: PathBuf) -> Result<(), Errno> {
    if !pth.parent().unwrap().exists() {
        return Err(Errno::ENOENT);
    }

    let data = from_slice::<serde_json::Value>(&cfg);
    if data.is_err() {
        return Err(Errno::ENODATA);
    }
    match serde_yaml::to_string(&data.unwrap()) {
        Ok(yml) => {
            let mut f = File::create(pth).unwrap();
            f.write_all(&yml.into_bytes()).unwrap();
        }
        Err(_) => {
            return Err(Errno::EILSEQ);
        }
    }

    Ok(())
}
