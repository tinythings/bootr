// OCI meta
// Stores system status and hash sums of used layers

use super::installer;
use crate::{
    bconf::{
        defaults,
        mcfg::BootrConfig,
        scfg::{self, StatusConfig},
    },
    ociman::{
        ocidata::OciClient,
        ocistate::{OCIMeta, OCIMetaTryFrom},
    },
};
use log::{debug, error, info, warn};
use nix::{
    fcntl::{renameat2, RenameFlags},
    unistd::symlinkat,
    NixPath,
};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Error, ErrorKind, Write},
    path::PathBuf,
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
                debug!("Directory {:?} was not found, creating", p);
                fs::create_dir_all(p)?;
            }
        }

        // Load sysroots idempotently
        self.sysparts.clear();
        for p in [defaults::C_BOOTR_SECT_A.as_str(), defaults::C_BOOTR_SECT_B.as_str()] {
            debug!("Loading sysroot at {}", p);
            self.load_sysroot(PathBuf::from(p))?;
        }

        Ok(())
    }

    /// Scans all sysroots
    fn load_sysroot(&mut self, pth: PathBuf) -> Result<(), Error> {
        if !pth.exists() {
            return Err(Error::new(ErrorKind::NotFound, format!("Path at {:?} not found", pth.to_str())));
        }

        let sr = OCISysroot::new(pth.to_owned());
        if let Ok(sr) = sr {
            self.sysparts.insert(pth.to_str().unwrap().to_owned(), sr);
        } else {
            warn!("Skipping sysroot: {}", sr.err().unwrap());
        }

        Ok(())
    }

    /// Download latest update for the known image, taken from the Bootr Config.
    /// This can be used by installer and updater, in case a whole new refresh
    /// is required. It will download all layers all over again, discarding the
    /// previous state.
    ///
    /// Current implementation assumes if a destination (dst) directory is empty,
    /// then this is a new installation. Otherwise update operation and OCI state
    /// file is required.
    async fn download(&self, dst: &PathBuf) -> Result<(), Error> {
        debug!("Checking the environment mode");
        if !dst.exists() {
            return Err(Error::new(ErrorKind::NotFound, format!("Path {:?} does not exist!", dst)));
        }

        // Get meta, if any
        let mut oci_meta: Option<OCIMeta> = None;
        if let Ok(meta) = <OCIMeta as OCIMetaTryFrom<PathBuf>>::try_from(dst.join(defaults::C_BOOTR_SECT_OCI_META)) {
            oci_meta = Some(meta);
        }
        debug!("{} mode", if oci_meta.is_some() { "Update" } else { "Install" });

        info!("Downloading OCI data to {:?}", dst);
        let oci_cnt = OciClient::new(None);

        match oci_cnt
            .pull(
                &self.cfg.oci_registry.image,
                if let Some(oci_meta) = oci_meta { oci_meta.get_layers_as_digests() } else { vec![] },
            )
            .await
        {
            Ok(img) => {
                for layer in &img.layers {
                    let dst_layer = dst.join(layer.sha256_digest().trim_start_matches("sha256:"));
                    info!("Layer: {:?}, size: {}", dst_layer.file_name().unwrap_or_default(), layer.data.len(),);
                    let mut f = File::create(dst_layer)?;
                    f.write_all(&layer.data)?;
                }

                info!("Importing data from the OCI manifest");
                OCIMeta::save(&OCIMeta::from(img.manifest.to_owned().unwrap()), dst)?;
            }
            Err(err) => error!("Error: {}", err),
        }

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
            return Err(Error::new(ErrorKind::NotFound, format!("Sysroot by ID '{}' was not found", id)));
        }
        let sysroot = sysroot.unwrap();

        let curr_link = PathBuf::from(defaults::C_BOOTR_CURRENT_LNK.to_string());
        if curr_link.exists() {
            // Flip the symlink only if current sysroot is different than requested
            // This operation should be atomic. To achieve it, this code does the following:
            //
            // 1. creates a symlink to a new location, called "current.temp"
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
            debug!("Creating new symlink to a current sysroot at {:?}", sysroot.path);
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

    /// Update an existing sysroot from an OCI container image.
    ///
    /// Update process finds the oldest, non-active slot in the /bootr/system directory
    /// and copies it entirely into a .temp directory and then adding layers that
    /// weren't yet applied. Once done that, it renames .temp into the slot name
    pub async fn update(&self) -> Result<(), Error> {
        todo!("Not implemented yet");
    }

    /// Install a sysroot from an OCI container image onto existing system.
    ///
    /// Installation process initiates an empty .temp in the /bootr/system directory,
    /// because it meant to start a new system "from scratch". This is contrary to
    /// the update session, where an older slot is taken as a base and then moved
    /// to the same .temp directory in the /bootr/system.
    pub async fn install(&self) -> Result<(), Error> {
        // Check if installation can be performed at all
        let mut can_install = true;
        if PathBuf::from(defaults::C_BOOTR_CURRENT_LNK.to_string()).exists() {
            debug!("Current slot pointer found");
            can_install = false;
        }

        if can_install {
            for sect in [defaults::C_BOOTR_SECT_A.as_str(), defaults::C_BOOTR_SECT_B.as_str()] {
                let s_pth = PathBuf::from(sect);
                if s_pth.exists() && !s_pth.read_dir()?.next().is_none() {
                    debug!("{:?} is not empty", s_pth);
                    can_install = false;
                    break;
                }
            }
        }

        if !can_install {
            return Err(Error::new(ErrorKind::AlreadyExists, "System seems already installed"));
        }

        // Perform the installation
        info!("Installing system from OCI container at {}", self.cfg.oci_registry.image);

        // Prepare slot
        let slot_path = PathBuf::from(defaults::C_BOOTR_SECT_TMP.to_string());
        if slot_path.exists() {
            fs::remove_dir_all(&slot_path)?;
        }
        fs::create_dir_all(&slot_path)?;

        // Download data to the slot.
        self.download(&slot_path).await?;

        // Install a new OCI content into a temporary slot
        let slot_path = installer::OCIInstaller::new(slot_path.join(defaults::C_BOOTR_SECT_RFS_DIR)).install()?;

        // Atomically flip temporary slot to "A" (at this point A and B are empty)
        renameat2(
            None,
            slot_path.to_str().unwrap(),
            None,
            slot_path.parent().unwrap().join(defaults::C_BOOTR_SECT_A.to_string()).to_str().unwrap(),
            RenameFlags::RENAME_EXCHANGE,
        )?;

        // Remove empty temporary slot
        fs::remove_dir(slot_path)?;

        // Symlink "A" slot as current
        symlinkat(defaults::C_BOOTR_SECT_A.as_str(), None, defaults::C_BOOTR_CURRENT_LNK.as_str())?;

        info!("OCI image {} installed successfully. Probably... :-)", self.cfg.oci_registry.image);

        Ok(())
    }
}
