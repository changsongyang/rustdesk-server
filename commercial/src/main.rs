use clap::{App, Arg, SubCommand};
use flexi_logger::Logger;
use log::info;

use rustdesk_server_pro::AppState;

fn main() -> anyhow::Result<()> {
    let matches = App::new("rustdesk-pro")
        .version("1.0.0")
        .about("RustDesk Server Commercial Edition")
        .arg(Arg::with_name("log_level")
            .short("l")
            .long("log-level")
            .takes_value(true)
            .default_value("info"))
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .takes_value(true)
            .default_value("8080"))
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .takes_value(true))
        .subcommand(SubCommand::with_name("generate-license")
            .about("Generate a license key")
            .arg(Arg::with_name("type")
                .short("t")
                .long("type")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("duration_days")
                .short("d")
                .long("duration-days")
                .takes_value(true)
                .required(true))
            .arg(Arg::with_name("max_devices")
                .short("m")
                .long("max-devices")
                .takes_value(true)))
        .subcommand(SubCommand::with_name("validate-license")
            .about("Validate a license key")
            .arg(Arg::with_name("key")
                .short("k")
                .long("key")
                .takes_value(true)
                .required(true)))
        .subcommand(SubCommand::with_name("serve")
            .about("Start the server"))
        .get_matches();

    let log_level = matches.value_of("log_level").unwrap_or("info");
    let port = matches.value_of("port").unwrap_or("8080").parse()?;

    Logger::try_with_env_or_str(log_level)?
        .log_to_stdout()
        .start()?;

    info!("RustDesk Server Commercial Edition v1.0.0 starting...");

    tokio::runtime::Runtime::new()?.block_on(async {
        match matches.subcommand() {
            ("generate-license", Some(sub_m)) => {
                let r#type = sub_m.value_of("type").unwrap();
                let duration_days = sub_m.value_of("duration_days")
                    .unwrap()
                    .parse()?;
                let max_devices = sub_m.value_of("max_devices")
                    .map(|s| s.parse())
                    .transpose()?;
                
                let state = AppState::new().await;
                let key = state.license_manager.generate_license(r#type, duration_days, max_devices).await?;
                println!("Generated license key: {}", key);
            }
            ("validate-license", Some(sub_m)) => {
                let key = sub_m.value_of("key").unwrap();
                
                let state = AppState::new().await;
                match state.license_manager.validate_license(key).await {
                    Ok(info) => {
                        println!("License is valid: {:?}", info);
                    }
                    Err(e) => {
                        eprintln!("License validation failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            ("serve", _) | (_, _) => {
                let state = AppState::new().await;
                rustdesk_server_pro::web::start_server(state, port).await?;
            }
        }
        Ok(())
    })
}
