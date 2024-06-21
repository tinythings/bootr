// Status config

use chrono::{DateTime, FixedOffset};
use indexmap::IndexMap;
use nix::errno::Errno;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use std::io::{BufReader, Error, ErrorKind, Write};
use std::{fs::File, path::PathBuf};

/// The execution parameters which SHOULD be used as a base
/// when running a container using the image. This field can
/// be null, in which case any execution parameters should
/// be specified at creation of the container.
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusConfigCfg {
    /// Default arguments to the entrypoint of the container.
    pub cmd: Option<String>,

    /// Arbitrary metadata for the container.
    pub labels: Option<IndexMap<String, String>>,
}

/// The rootfs key references the layer content addresses used by the image.
/// This makes the image config hash depend on the filesystem hash.
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusConfigRootfs {
    #[serde(rename = "type")]
    /// Type of the rootfs, currently always set to "layers"
    pub rootfs_type: String,

    /// An array of layer content hashes (DiffIDs), in order from first to last.
    pub diff_ids: Vec<String>,
}

/// Describes the history of each layer. The array is ordered from first to last.
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusConfigHistory {
    /// A combined date and time at which the layer was created, formatted
    /// as defined by RFC 3339, section 5.6.
    pub created: Option<String>,

    /// The author of the build point.
    pub author: Option<String>,

    /// The command which created the layer.
    pub created_by: Option<String>,

    /// A custom message set when creating the layer.
    pub comment: Option<String>,

    /// This field is used to mark if the history item created a filesystem diff.
    /// It is set to true if this history item doesn't correspond to an actual
    /// layer in the rootfs section (for example, Dockerfile's ENV command
    /// results in no change to the filesystem).
    pub empty_layer: Option<bool>,
}

/// Partial configuration structure of OCI config format.
/// https://github.com/opencontainers/image-spec/blob/main/config.md
///
/// DISCLAIMER: This struct only partial at the moment and
///             does not implements all the properties,
///             described in the specification, at least for now.
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusConfig {
    /// Combined date and time at which the image was created,
    /// formatted as defined by RFC 3339, section 5.6.
    created: Option<String>,

    /// Gives the name and/or email address of the person
    /// or entity which created and is responsible
    /// for maintaining the image.
    pub author: Option<String>,

    /// The CPU architecture which the binaries in this
    /// image are built to run on. The attribute is required.
    pub architecture: String,

    /// The name of the operating system which the image
    /// is built to run on. The attribute is required.
    pub os: String,

    /// Specifies the version of the operating system targeted
    /// by the referenced blob.
    #[serde(rename = "os.version")]
    pub os_version: Option<String>,

    /// Partial configuration structure of OCI config format.
    pub config: Option<StatusConfigCfg>,

    /// The rootfs key references the layer content addresses used by the image.
    /// This makes the image config hash depend on the filesystem hash.
    pub rootfs: StatusConfigRootfs,

    /// Describes the history of each layer. The array is ordered from first to last.
    pub history: Option<Vec<StatusConfigHistory>>,
}

impl StatusConfig {
    /// Get combined date and time as DateTime object, if specified.
    pub fn created(&self) -> Option<DateTime<FixedOffset>> {
        if let Some(created) = &self.created {
            return Some(DateTime::parse_from_str(&created, "%Y-%m-%dT%H:%M:%SZ").unwrap());
        }
        None
    }
}

/// Read status configuration from the given path.
pub async fn get_status_config(pth: PathBuf) -> Result<StatusConfig, Error> {
    if !pth.exists() {
        return Err(Error::new(ErrorKind::NotFound, format!("Configuration file at {} is missing", pth.to_str().unwrap())));
    }

    match serde_yaml::from_reader(BufReader::new(File::open(pth)?)) {
        Ok(cfg) => Ok(cfg),
        Err(err) => Err(Error::new(std::io::ErrorKind::InvalidData, err)),
    }
}

/// Writes OCI config to a file
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
