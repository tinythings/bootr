use clap::{builder::styling, Arg, Command};

use crate::bconf::defaults;

/// CLI definition
pub fn clidef(version: &'static str, appname: &'static str) -> Command {
    let styles = styling::Styles::styled()
        .header(styling::AnsiColor::Yellow.on_default() | styling::Effects::BOLD)
        .usage(styling::AnsiColor::Yellow.on_default() | styling::Effects::BOLD)
        .literal(styling::AnsiColor::BrightGreen.on_default())
        .placeholder(styling::AnsiColor::BrightRed.on_default());

    Command::new(appname)
        .version(version)
        .about(format!("{} - transactional, in-place system updates using OCI containers paradigm", appname))
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Specify the configuration")
                .default_value(defaults::C_BOOTR_CFG.to_string()),
        )
        .arg(
            Arg::new("log")
                .short('l')
                .long("log")
                .help("Set logging verbosity")
                .default_value("info")
                .value_parser(["quiet", "info", "verbose"]),
        )
        .subcommand(Command::new("install").about(
            "Install a specified OCI image onto current rootfs, \
            replacing current content with the OCI image data",
        ))
        .subcommand(
            Command::new("update")
                .about("Update the system from available OCI images")
                .arg(Arg::new("check").short('c').long("check").help("Check for updates").action(clap::ArgAction::SetTrue)),
        )
        .disable_version_flag(true)
        .disable_colored_help(false)
        .styles(styles)
}
