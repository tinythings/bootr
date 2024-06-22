// OCI data.
// Download blobs and manifests

use futures_util::stream;
use futures_util::StreamExt;
use futures_util::TryStreamExt;
use oci_distribution::{
    client::{ClientConfig, Config, ImageData, ImageLayer},
    errors::OciDistributionError,
    secrets::RegistryAuth,
    Client, Reference,
};

pub struct OciClient {
    client: Client,
    auth: RegistryAuth,
    mtypes: Vec<String>,
}

/// OCI client implement pulling layers.
/// NOTE: it only pulls them, but the placement meant to be handled elsewhere.
///
/// TODO: Add custom writables. Currently only vector in memory.
#[allow(dead_code, clippy::unnecessary_unwrap)]
impl OciClient {
    pub fn new(auth: Option<RegistryAuth>) -> Self {
        OciClient {
            client: Client::new(ClientConfig::default()),
            auth: if auth.is_some() { auth.unwrap() } else { RegistryAuth::Anonymous },
            mtypes: vec![],
        }
    }

    /// Authenticate to the registry, if anonymous (default) is not enough
    pub async fn auth_basic(&mut self, usr: &str, pwd: &str) -> &mut Self {
        self.auth = RegistryAuth::Basic(usr.to_string(), pwd.to_string());
        self
    }

    /// Add media type to restrict accepted type
    pub fn add_media_type(&mut self, mtype: &str) -> &mut Self {
        self.mtypes.push(mtype.to_owned());
        self
    }

    /// Pull data
    pub async fn pull(&self, uri: &str) -> Result<ImageData, nix::errno::Errno> {
        let imgref = Reference::try_from(uri);
        if imgref.is_err() {
            return Err(nix::errno::Errno::EINVAL);
        }
        let imgref = imgref.unwrap();

        let mdcr = self.client.pull_manifest_and_config(&imgref, &self.auth).await;
        if mdcr.is_err() {
            return Err(nix::errno::Errno::ENXIO);
        }

        let (manifest, digest, cfg) = mdcr.unwrap();

        // Check media types, if needed
        if !self.mtypes.is_empty() {
            for layer in &manifest.layers {
                if !self.mtypes.contains(&layer.media_type) {
                    log::debug!("Wrong media type found: {}", &layer.media_type);
                    return Err(nix::errno::Errno::EINVAL);
                }
            }
        }

        let r_imgref = &imgref;
        let layers = stream::iter(&manifest.layers)
            .map(|layer| {
                let this = &self.client;
                log::debug!("Media type: {}", layer.media_type);
                async move {
                    let mut out: Vec<u8> = Vec::new();
                    this.pull_blob(r_imgref, layer, &mut out).await?;
                    Ok::<_, OciDistributionError>(ImageLayer::new(out, layer.media_type.clone(), layer.annotations.clone()))
                }
            })
            .boxed()
            .buffer_unordered(16)
            .try_collect::<Vec<ImageLayer>>()
            .await;

        if layers.is_err() {
            return Err(nix::errno::Errno::ENXIO);
        }

        let mtype = manifest.clone().media_type.unwrap();
        let annotations = manifest.annotations.to_owned();

        Ok(ImageData {
            layers: layers.unwrap(),
            manifest: Some(manifest),
            config: Config::new(cfg.into_bytes(), mtype, annotations),
            digest: Some(digest),
        })
    }
}
