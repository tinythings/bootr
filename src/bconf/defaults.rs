use lazy_static::lazy_static;

pub static C_BOOTR_ROOT: &str = "/bootr";
lazy_static! {
    // Central configuration file
    pub static ref C_BOOTR_CFG: String = format!("{}{}", C_BOOTR_ROOT, "/config");

    // Root directory for system blob stores
    pub static ref C_BOOTR_SYSDIR: String = format!("{}{}", C_BOOTR_ROOT, "/system");
}
