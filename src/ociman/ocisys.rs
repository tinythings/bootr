// OCI meta
// Stores system status and hash sums of used layers

use super::installer;
use crate::{
    bconf::{
        defaults,
        mcfg::BootrConfig,
        scfg::{self, StatusConfig},
    },
    ociman::ocidata::OciClient,
};
use nix::{
    fcntl::{renameat2, RenameFlags},
    unistd::symlinkat,
};
use std::{collections::HashMap, fs, io::Error, path::PathBuf};
use tokio::{
    sync::{self, futures},
    task::{self, spawn_blocking},
};

/// OCISysroot is an object that contains all the structure of
/// the container-related metadata and an actual sysroot.
#[derive(Default)]
pub struct OCISysroot {
    /// Path the sysroot.
    /// This yields to the following schema:
    ///
    ///   /bootr/system/<NAME>
    path: PathBuf,

    /// True if the sysroot is currently active.
    is_active: bool,

    /// True, if the sysroot is just a "scaffold", i.e. nothing there yet.
    is_empty: bool,

    /// Status Config
    statconf: StatusConfig,
}

impl OCISysroot {
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        let mut osr = OCISysroot { path, is_empty: true, is_active: false, ..Default::default() };
        osr.load()?;

        Ok(osr)
    }

    /// Load all the information about current sysroot
    fn load(&mut self) -> Result<(), Error> {
        // Load status file of the rootfs slot
        let status_file = self.path.join(defaults::C_BOOTR_SECT_STATUS);
        self.is_empty = status_file.exists();
        if !self.is_empty {
            self.statconf = scfg::get_status_config(status_file)?;
        }

        // Check if there is a symlink, pointing to it as current.
        // This yields the following schema:
        //
        // /bootr/system/current -> /bootr/system/<NAME>
        if let Ok(ptr) = fs::read_link(PathBuf::from(defaults::C_BOOTR_CURRENT_LNK.to_string())) {
            self.is_active = ptr == self.path;
        }

        Ok(())
    }
}

/// OCI Manager manages known sysroots:
///   - returns their information
///   - sets which one is active and ready to boot
///   - physical path to the sysroot etc
pub struct OCISysMgr {
    cfg: BootrConfig,

    /// Sysroot partitions. Stored in a hashmap for
    /// keeping track of several locations,
    /// if more than A, B and temp.
    sysparts: HashMap<String, OCISysroot>,
}

impl OCISysMgr {
    /// Manager constructor
    pub fn new(cfg: BootrConfig) -> Result<Self, Error> {
        let mut mgr = OCISysMgr { cfg, sysparts: HashMap::default() };
        mgr.init()?;

        Ok(mgr)
    }

    /// Init always checks the directory structure,
    /// creates missing, cleaning up junk if any, reports it etc.
    fn init(&mut self) -> Result<(), Error> {
        // pre-create all required directories in the rootfs
        for s in [defaults::C_BOOTR_SYSDIR.as_str(), defaults::C_BOOTR_SECT_A.as_str(), defaults::C_BOOTR_SECT_B.as_str()] {
            let p = PathBuf::from(s);
            if !p.exists() {
                log::debug!("Directory {:?} was not found, creating", p);
                fs::create_dir_all(p)?;
            }
        }

        // Load sysroots idempotently
        self.sysparts.clear();
        for p in [defaults::C_BOOTR_SECT_A.as_str(), defaults::C_BOOTR_SECT_B.as_str()] {
            log::debug!("Loading sysroot at {}", p);
            self.load_sysroot(PathBuf::from(p))?;
        }

        Ok(())
    }

    /// Scans all sysroots
    fn load_sysroot(&mut self, pth: PathBuf) -> Result<(), Error> {
        if !pth.exists() {
            return Err(Error::new(std::io::ErrorKind::NotFound, format!("Path at {:?} not found", pth.to_str())));
        }

        let sr = OCISysroot::new(pth.to_owned());
        if sr.is_ok() {
            self.sysparts.insert(pth.to_str().unwrap().to_owned(), sr.unwrap());
        } else {
            log::warn!("Skipping sysroot: {}", sr.err().unwrap());
        }

        Ok(())
    }

    /// Download latest update for the known image, taken from the Bootr Config.
    /// This can be used by installer and updater, in case a whole new refresh
    /// is required. It will download all layers all over again, discarding the
    /// previous state.
    async fn download(&self) -> Result<(), Error> {
        let slot_path = PathBuf::from(defaults::C_BOOTR_SECT_TMP.as_str());
        log::debug!("Downloading artifacts to {:?}", slot_path);
        let oci_cnt = OciClient::new(None);

        match oci_cnt.pull(&self.cfg.oci_registry.image).await {
            Ok(img) => {
                println!("Manifest: {}", img.manifest.unwrap());
                println!("{} layers found:", &img.layers.len());
                for layer in &img.layers {
                    println!("   Type: {}, size: {}", layer.media_type, layer.data.len());
                }
            }
            Err(x) => println!("Error: {}", x),
        }

        Ok(())
    }

    /// Download only delta layers, i.e. what are new from the manifest, merging
    /// to the existing ones. This is used only by updates.
    fn download_delta(&self) -> Result<(), Error> {
        Ok(())
    }

    /// Scans all sysroots and sets active pointer to the latest
    pub fn set_active_latest(&mut self) -> Result<(), Error> {
        // Reload everything
        self.init()?;

        Ok(())
    }

    /// Sets active pointer to the sysroot by the ID.
    /// The ID is the name of the sysroot, following the schema:
    ///
    ///     /bootr/system/<NAME>
    ///
    /// This method either creates a new symlink or flips it, if
    /// it is currently points elsewhere.
    pub fn set_active_by_id(&mut self, id: String) -> Result<(), Error> {
        let target = format!("{}/{}", *defaults::C_BOOTR_SYSDIR, &id);
        let sysroot = self.sysparts.get(target.as_str());

        if sysroot.is_none() {
            return Err(Error::new(std::io::ErrorKind::NotFound, format!("Sysroot by ID '{}' was not found", id)));
        }
        let sysroot = sysroot.unwrap();

        let curr_link = PathBuf::from(defaults::C_BOOTR_CURRENT_LNK.to_string());
        if curr_link.exists() {
            // Flip the symlink only if current sysroot is different than requested
            // This operation should be atomic. To achieve it, this code does the following:
            //
            // 1. creates a symlink to a new location, called "current.new"
            // 2. renames "current.new" into "current" (mv -T)
            if !sysroot.is_active {
                // Create a temporary new symlink
                symlinkat(sysroot.path.as_os_str(), None, defaults::C_BOOTR_CURRENT_LNK_TMP.as_str())?;

                // Rename the symlink
                renameat2(
                    None,
                    defaults::C_BOOTR_CURRENT_LNK_TMP.as_str(),
                    None,
                    defaults::C_BOOTR_CURRENT_LNK.as_str(),
                    RenameFlags::RENAME_EXCHANGE,
                )?;
            }
        } else {
            log::debug!("Creating new symlink to a current sysroot at {:?}", sysroot.path);
            // Create a new symlink to the current sysroot
            symlinkat(sysroot.path.as_os_str(), None, defaults::C_BOOTR_CURRENT_LNK.as_str())?;
        }

        // Reload everything
        self.init()?;

        Ok(())
    }

    /// Return metadata of the active sysroot.
    /// Returns None, if no active sysroot has been found
    pub fn get_sysroot_meta(&self) -> Option<StatusConfig> {
        if let Some(sysroot) = self.get_sysroot() {
            return Some(sysroot.statconf.to_owned());
        }
        None
    }

    /// Get currently active sysroot, if any.
    /// Returns None if no active sysroots has been found (in this case system cannot boot either)
    pub fn get_sysroot(&self) -> Option<&OCISysroot> {
        for (name, sysroot) in &self.sysparts {
            if sysroot.is_active {
                return Some(sysroot);
            }
        }

        None
    }

    /// Install a sysroot from an OCI container image onto existing system
    pub async fn install(&self) -> Result<(), Error> {
        log::debug!("Installing OCI container from {}", self.cfg.oci_registry.image);

        self.download().await?;
        installer::OCIInstaller::new().install()?;

        Ok(())
    }
}
