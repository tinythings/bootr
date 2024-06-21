use lazy_static::lazy_static;

pub static C_BOOTR_ROOT: &str = "/bootr";
lazy_static! {
    /// Central configuration file
    pub static ref C_BOOTR_CFG: String = format!("{}{}", C_BOOTR_ROOT, "/config");

    /// Root directory for system blob stores
    pub static ref C_BOOTR_SYSDIR: String = format!("{}{}", C_BOOTR_ROOT, "/system");

    /// The "A" section among A/B sections
    pub static ref C_BOOTR_SECT_A: String = format!("{}{}", C_BOOTR_SYSDIR.to_string(), "/A");

    /// The "B" section among A/B sections
    pub static ref C_BOOTR_SECT_B: String = format!("{}{}", C_BOOTR_SYSDIR.to_string(), "/B");

    /// The ".temp" section among A/B sections
    pub static ref C_BOOTR_SECT_TMP: String = format!("{}{}", C_BOOTR_SYSDIR.to_string(), "/.temp");
}
