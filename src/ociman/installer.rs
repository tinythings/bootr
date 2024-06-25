use crate::bconf::defaults;
use dircpy::CopyBuilder;
use flate2::read::GzDecoder;
use log::{debug, error, info, warn};
use nix::NixPath;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Error,
    path::{Path, PathBuf},
};
use tar::Archive;
use walkdir::WalkDir;

use super::ocistate::{OCIMeta, OCIMetaTryFrom};

/// OCI container installer.
///
/// The installer supposed to pick already downloaded blobs,
/// configs etc and just follow the instructions. It doesn't
/// meant to download or check anything, but only go over
/// already prepared and verified environment.
///
/// Installation process and current principles are as follows:
///
/// 1. Latest and newest update data supposed to be already placed
///    in the /bootr/system/.temp directory.
///
/// 2. Installer also doesn't know what slot to pick, so this
///    should be the part of the installation information/data.
///
/// 3. The slot supposed to be already cleaned up, checked for
///    mounted volumes and prepared for the installation. Installer
///    only installs and if something is there missing, it will be
///    basically destroyed. ¯\_(ツ)_/¯
///
/// 4. Installer is used only once at the beginning after whatever
///    image provisioning. Essentially, the installer wiping out
///    whatever current rootfs is, and installs container rootfs.
///
/// 5. After slot is successfully provisioned/installed, the
///    temporary directory is removed by the installer at the end
///    of the process. However, activation is done by the manager
///    in atomic way.
pub struct OCIInstaller {
    /// If set to true, installer will keep current kernel.
    ///
    /// This means:
    /// - initramfs is not regenerated
    /// - /lib/modules/* is not touched and just skipped
    /// - no changes to /boot whatsoever
    keep_kernel: bool,

    /// The main buildroot. By default it is /bootr/system/.temp/build
    buildroot: PathBuf,
}

/// Implementation of the OCI container installer.
impl OCIInstaller {
    pub const WHITEOUT: &'static str = ".wh.";

    /// Constructor
    pub fn new<P: AsRef<Path>>(pth: P) -> Self {
        OCIInstaller {
            // TODO: Kernel handling (update/install) is not implemented yet
            keep_kernel: true,
            buildroot: pth.as_ref().to_path_buf(),
        }
    }

    /// Apply a layer to the root filesystem.
    ///
    /// This works the following way:
    /// - If this is a very first layer (installation mode, directory is empty),
    ///   it is simply unpacked "as is"
    ///
    /// - Every other layer (diff) is applied into its own sub-directory,
    ///   named after its hash, and then copied over the final target.
    ///   White-outs are resolved to the artifacts on the target and deleted,
    ///   and then markers are removed from the layer. Once this done, layer
    ///   is simply copied over.
    ///
    fn apply_layer(&self, is_base: bool, layer_arc_pth: &PathBuf, digest: String) -> Result<(), Error> {
        debug!("Unpacking archive to {:?}", self.buildroot);
        if is_base {
            // First time install, just unpack without any post-processing
            Archive::new(GzDecoder::new(File::open(layer_arc_pth)?)).unpack(&self.buildroot)?;
            fs::remove_file(layer_arc_pth)?;
            info!("Base layer {} applied", digest);
        } else {
            let dst = self.buildroot.join("tmp"); // This is a /tmp on the target, but might not exist
            let dst_dirty = !dst.exists();
            if dst_dirty {
                debug!("Creating temp directory on target at {:?}", dst);
                fs::create_dir_all(&dst)?;
            }
            let dst = dst.join(digest.as_str()); // This contains /tmp/<SHA256> with the diff in it
            fs::create_dir_all(&dst)?;

            Archive::new(GzDecoder::new(File::open(layer_arc_pth)?)).unpack(&dst)?;
            fs::remove_file(layer_arc_pth)?;

            for e in WalkDir::new(&dst).into_iter().flatten() {
                if e.file_name().to_str().unwrap().starts_with(Self::WHITEOUT) {
                    // Trim the temporary path to diff-related path only
                    let wh_p = self.buildroot.join(
                        e.path()
                            .to_str()
                            .unwrap()
                            .split_once(&digest)
                            .unwrap_or_default()
                            .1
                            .trim_start_matches('/')
                            .replace(Self::WHITEOUT, ""),
                    );

                    // If diff went bonkers, we just stop everything, preventing permanent system damage
                    if !wh_p.exists() {
                        return Err(Error::new(
                            std::io::ErrorKind::NotFound,
                            format!("Requested object does not exists at {:?}", wh_p),
                        ));
                    }

                    // First, remove the target
                    if wh_p.is_dir() {
                        fs::remove_dir_all(&wh_p)?;
                    } else {
                        fs::remove_file(&wh_p)?;
                    }

                    // Now remove the whiteout marker
                    fs::remove_file(e.path())?;
                }
            }

            // Copy over the temporary diff and remove the temp
            CopyBuilder::new(&dst, &self.buildroot).overwrite(true).run()?;

            // Remove temporary directory for this layer
            fs::remove_dir_all(&dst)?;
            if dst_dirty {
                fs::remove_dir_all(dst.parent().unwrap())?;
            }

            info!("Diff layer {} applied", digest);
        }
        Ok(())
    }

    /// Unpack required layers and remove them from the disk, once done.
    fn populate_oci_data(&self) -> Result<(), Error> {
        info!("Applying OCI data");
        let br_parent = self.buildroot.parent().unwrap().to_path_buf();
        let meta = <OCIMeta as OCIMetaTryFrom<PathBuf>>::try_from(br_parent.join(defaults::C_BOOTR_SECT_OCI_META).to_owned())?;
        for (idx, layer) in meta.layers.iter().enumerate() {
            // Handle "vnd.docker.image" and "vnd.oci.image". What a mess... :-(
            if layer.media_type.ends_with("tar.gzip") || layer.media_type.ends_with(".tar+gzip") {
                let digest = layer.digest.trim_start_matches("sha256:").to_owned();
                let arcp = &br_parent.join(&digest);
                debug!("Reading layer at {:?}", arcp);
                if !arcp.exists() {
                    return Err(Error::new(std::io::ErrorKind::NotFound, format!("Layer file at {:?} was not found", arcp)));
                }

                // XXX: applied layers needs to be skipped when updating!
                self.apply_layer(idx == 0, arcp, digest)?;
            }
        }
        Ok(())
    }

    /// Populate system directories (mountpoints) inside the slot.
    fn populate_dirtree(&self) -> Result<(), Error> {
        // Flush build dir if any and re-empty it again
        debug!("Preparing build dir");
        if self.buildroot.exists() {
            fs::remove_dir_all(&self.buildroot)?;
            fs::create_dir_all(&self.buildroot)?;
        }

        debug!("Populating system directories into {:?}", self.buildroot);
        for sd in defaults::C_BOOTR_SYSDIRS {
            let p = self.buildroot.join(sd.trim_start_matches('/'));
            if p.exists() {
                warn!("Directory {} already exists! Removing, including its content...", p.as_os_str().to_str().unwrap());
                fs::remove_dir_all(&p)?;
            }

            debug!("Creating {} directory", p.as_os_str().to_str().unwrap());
            fs::create_dir_all(p)?;
        }
        Ok(())
    }

    /// Preserves existing kernel only if this is requested.
    /// If not requested, the routine just bails out.
    /// This also keeps initramfs and boot options with all
    /// the existing kernel modules untouched.
    fn maybe_keep_kernel(&self) -> Result<(), Error> {
        if !self.keep_kernel {
            return Ok(());
        }

        info!("Keeping kernel, boot options and initramfs from the current image");
        for sd in ["/boot", "/lib/modules"] {
            let tgt = self.buildroot.join(sd.trim_start_matches('/'));
            debug!("Copying {} to {:?}", sd, tgt);
            if !tgt.exists() {
                fs::create_dir_all(&tgt)?;
            }

            if let Err(err) = CopyBuilder::new(sd, &tgt).overwrite(true).run() {
                error!("Cannot copy {:?} to {:?}", sd, tgt.parent().unwrap());
                return Err(Error::new(std::io::ErrorKind::InvalidData, err.to_string()));
            }
        }

        Ok(())
    }

    /// Main method for the installation to begin
    pub fn install(&self) -> Result<(), Error> {
        // Flush the buildroot, if any and [re]create it.
        self.populate_dirtree()?;

        // Unpack downloaded OCI artefacts and remove the archives from the disk
        self.populate_oci_data()?;

        // If kernel requested to be preserved, keep it.
        // This then include /boot as well, because the initramfs
        // won't be regenerated at this point.
        //self.maybe_keep_kernel()?;

        Ok(())
    }
}
