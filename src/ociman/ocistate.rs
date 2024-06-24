use crate::bconf::defaults;
use log::debug;
use oci_distribution::manifest::OciImageManifest;
use serde::{Deserialize, Serialize};
use std::io::{self, BufReader, Write};
use std::{
    fs::File,
    io::Error,
    path::{Path, PathBuf},
};

/// OCI State
///
/// This object is imported from an OCI manifest but keeps only required fields.
/// It can be serialised to the disk and imported back.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OCIMeta {
    /// The property specifies the image manifest schema version.
    #[serde(rename = "schema-version")]
    pub schema_version: u8,

    /// This property references a configuration object for a container, by digest
    pub config: OCIStateArtefact,

    /// The array MUST have the base layer at index 0.
    /// Subsequent layers MUST then follow in stack order (i.e. from layers[0]
    /// to layers[len(layers)-1]). The final filesystem layout MUST match
    /// the result of applying the layers to an empty directory.
    pub layers: Vec<OCIStateArtefact>,
}

/// OCI State Artefact contains media type, digest and the size
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OCIStateArtefact {
    #[serde(rename = "media-type")]
    pub media_type: String,
    pub digest: String,
    pub size: i64,
}

pub trait OCIMetaTryFrom<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Error;

    /// Performs the conversion.
    fn try_from(value: T) -> Result<Self, Self::Error>;
}

impl OCIMeta {
    /// Write OCI state to the file
    pub fn save<P: AsRef<Path>>(class: &OCIMeta, pth: P) -> Result<(), Error> {
        let pth = PathBuf::from(pth.as_ref());
        if !pth.exists() {
            return Err(Error::new(std::io::ErrorKind::NotFound, format!("Path {:?} not found", pth.parent().unwrap())));
        }
        let pth = pth.join(defaults::C_BOOTR_SECT_OCI_META);
        debug!("Writing OCI config to a file at {:?}", pth);

        match serde_yaml::to_string(class) {
            Ok(yml) => {
                let mut f = File::create(pth).unwrap();
                f.write_all(yml.as_bytes()).unwrap();
            }
            Err(err) => {
                return Err(Error::new(std::io::ErrorKind::InvalidData, format!("{}", err)));
            }
        }

        Ok(())
    }

    /// True, if layers contain SHA256 digest.
    pub fn has_layer(&self, digest: &str) -> bool {
        for aft in &self.layers {
            if aft.digest.eq(digest) {
                return true;
            }
        }
        false
    }

    /// Collect and return already known layer digests.
    /// The layers are ordered from base (0) to newest (n).
    pub fn get_layers_as_digests(&self) -> Vec<String> {
        let mut out = Vec::<String>::default();
        for aft in &self.layers {
            out.push(aft.digest.to_owned());
        }
        out
    }
}

impl From<OciImageManifest> for OCIMeta {
    /// Import OCI image manifest from the OciImageManifest object
    fn from(mfst: OciImageManifest) -> Self {
        let mut layers = Vec::<OCIStateArtefact>::default();
        for lyr in mfst.layers {
            layers.push(OCIStateArtefact { media_type: lyr.media_type, digest: lyr.digest, size: lyr.size })
        }

        OCIMeta {
            schema_version: mfst.schema_version,
            config: OCIStateArtefact { media_type: mfst.config.media_type, digest: mfst.config.digest, size: mfst.config.size },
            layers,
        }
    }
}

impl<T: ?Sized + AsRef<Path>> OCIMetaTryFrom<&T> for OCIMeta {
    type Error = io::Error;

    /// Try load from a file
    fn try_from(pth: &T) -> Result<Self, Self::Error> {
        let pth = PathBuf::from(pth.as_ref());
        match serde_yaml::from_reader(BufReader::new(File::open(pth)?)) {
            Ok(state) => Ok(state),
            Err(err) => Err(Error::new(std::io::ErrorKind::InvalidData, err)),
        }
    }
}
