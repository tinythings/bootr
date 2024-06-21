// OCI meta
// Stores system status and hash sums of used layers

use crate::bconf::{self, defaults, mcfg::BootrConfig};
use nix::fcntl::{renameat2, RenameFlags};
use std::{fs, io::Error, path::PathBuf};

/// OCISysroot is an object that contains all the structure of
/// the container-related metadata and an actual sysroot.
pub struct OCISysroot {
    /// Name of the sysroot. It is a directory name after "/system".
    /// This yields to the following schema:
    ///
    ///   /bootr/system/<NAME>
    name: String,

    /// True|False if the sysroot is currently active or not.
    is_active: bool,
}

/// OCI Manager manages known sysroots:
///   - returns their information
///   - sets which one is active and ready to boot
///   - physical path to the sysroot etc
pub struct OCISysMgr<'a> {
    cfg: &'a BootrConfig,
}

impl<'a> OCISysMgr<'a> {
    /// Manager constructor
    pub fn new(cfg: &'a BootrConfig) -> Result<Self, Error> {
        OCISysMgr::init()?;
        Ok(OCISysMgr { cfg })
    }

    /// Init always checks the directory structure,
    /// creates missing, cleaning up junk if any, reports it etc.
    fn init() -> Result<(), Error> {
        // pre-create all required directories in the rootfs
        for s in [defaults::C_BOOTR_SYSDIR.as_str(), defaults::C_BOOTR_SECT_A.as_str(), defaults::C_BOOTR_SECT_B.as_str()] {
            let p = PathBuf::from(s);
            if !p.exists() {
                fs::create_dir_all(p)?;
            }
        }

        Ok(())
    }

    /// Scans all sysroots
    fn scan_sysroots() {}

    /// Scans all sysroots and sets active pointer to the latest
    pub fn set_active_latest() {}

    /// Sets active pointer to the sysroot by the ID
    pub fn set_active_by_id(id: String) {}

    /// Return metadata of known sysroots
    pub fn get_sysroots_meta() {}
}

/*
fn foo() {
    renameat2(old_dirfd, old_path, new_dirfd, new_path, RenameFlags::RENAME_EXCHANGE);
}
*/
