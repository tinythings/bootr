mod cli;
mod ociman;

use ociman::ocidata;
use std::{env, io::Error};

static VERSION: &str = "0.0.1";
static APPNAME: &str = "bootr";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let mut cliarg = cli::clidef(VERSION, APPNAME);

    if args.len() == 1 {
        return {
            cliarg.print_help().unwrap();
            Ok(())
        };
    }

    let p = cliarg.get_matches();

    // System Update
    if let Some(subarg) = p.subcommand_matches("update") {
        let c = ocidata::OciClient::new(None);
        println!("updating the system");
        if subarg.get_flag("check") {
            todo!("Check for available updates is not implemented yet");
        } else {
            match c.pull("registry.suse.com/bci/bci-busybox:15.6").await {
                Ok(img) => {
                    println!("Manifest: {}", img.manifest.unwrap().to_string());
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
