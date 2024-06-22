mod bconf;
mod cli;
mod logger;
mod ociman;

use bconf::mcfg;
use clap::ArgMatches;
use ociman::{ocidata, ocisys::OCISysMgr};
use std::{env, io::Error, path::PathBuf};

static VERSION: &str = "0.0.1";
static APPNAME: &str = "bootr";
static LOGGER: logger::STDOUTLogger = logger::STDOUTLogger;

/// Wrapper to handle all the errors in main() :-)
async fn run() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let mut cliarg = cli::clidef(VERSION, APPNAME);

    if args.len() == 1 {
        return {
            cliarg.print_help().unwrap();
            Ok(())
        };
    }

    let p = cliarg.get_matches();

    // Setup logger
    log::set_logger(&LOGGER)
        .map(|()| {
            log::set_max_level(match p.get_one::<String>("log").unwrap().as_str() {
                "info" => log::LevelFilter::Info,
                "verbose" => log::LevelFilter::Trace,
                "quiet" => log::LevelFilter::Off,
                _ => log::LevelFilter::Error,
            })
        })
        .unwrap();

    let oci_mgr = OCISysMgr::new(mcfg::get_bootr_config(PathBuf::from(p.get_one::<String>("config").unwrap()))?)?;
    let oci_cnt = ocidata::OciClient::new(None);

    if let Some(subarg) = p.subcommand_matches("install") {
        // System installation
        log::info!("Installing system");
    } else if let Some(subarg) = p.subcommand_matches("update") {
        // System Update
        log::info!("Updating the system");
        if subarg.get_flag("check") {
            todo!("Check for available updates is not implemented yet");
        } else {
            match oci_cnt.pull("registry.suse.com/bci/bci-busybox:15.6").await {
                Ok(img) => {
                    println!("Manifest: {}", img.manifest.unwrap());
                    println!("{} layers found:", &img.layers.len());
                    for layer in &img.layers {
                        println!("   Type: {}, size: {}", layer.media_type, layer.data.len());
                    }
                }
                Err(x) => println!("Error: {}", x),
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    if let Err(err) = run().await {
        log::error!("{}", err);
    }

    Ok(())
}
