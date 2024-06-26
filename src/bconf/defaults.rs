use lazy_static::lazy_static;

/// Main Bootr directory where "everything" is located
pub static C_BOOTR_ROOT: &str = "/bootr";

/// Section status file
pub static C_BOOTR_SECT_STATUS: &str = "status";

/// Section OCI metadata file
pub static C_BOOTR_SECT_OCI_META: &str = "oci-meta";

/// Marker that this system is just freshly installed.
/// It is merely used as another step to prevent disasters
/// such as replacing current rootfs with a wrong one. :-)
///
/// This file is placed right after the fresh installation (happens once)
/// and then removed right after the filesystem is replaced (symlinked)
pub static C_BOOTR_SECT_INSTALLED_MARKER: &str = ".installed";

/// Name of the rootfs entry within the slot (any)
pub static C_BOOTR_SECT_RFS_DIR: &str = "rootfs";

/// List of forever untouchable directories
/// NOTE: subject to change, especially with SELinux :)
pub static C_BOOTR_SYSDIRS: [&str; 4] = ["/dev", "/proc", "/sys", "/run"];

lazy_static! {
    /// Central configuration file
    pub static ref C_BOOTR_CFG: String = format!("{}{}", C_BOOTR_ROOT, "/config");

    /// Root directory for system blob stores
    pub static ref C_BOOTR_SYSDIR: String = format!("{}{}", C_BOOTR_ROOT, "/system");

    /// Symlink path, which should point to the current running section.
    /// The symlink should be always present. Symlink might be absent if the system is not yet provisioned,
    /// but if both sections are present and symlink is absent (removed accidentally etc), the latest
    /// section is chosen as current. Section is chosen by MTIME of status file
    pub static ref C_BOOTR_CURRENT_LNK: String = format!("{}{}", *C_BOOTR_SYSDIR, "/current");

    /// Temporary symlink for atomic flip
    pub static ref C_BOOTR_CURRENT_LNK_TMP: String = format!("{}{}", *C_BOOTR_SYSDIR, "/current.temp");

    /// The "A" section among A/B sections
    pub static ref C_BOOTR_SECT_A: String = format!("{}{}", *C_BOOTR_SYSDIR, "/A");

    /// The "B" section among A/B sections
    pub static ref C_BOOTR_SECT_B: String = format!("{}{}", *C_BOOTR_SYSDIR, "/B");

    /// The ".temp" section among A/B sections
    pub static ref C_BOOTR_SECT_TMP: String = format!("{}{}", *C_BOOTR_SYSDIR, "/.temp");
}
