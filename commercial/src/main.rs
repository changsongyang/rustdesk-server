use clap::{Arg, Command};
use flexi_logger::Logger;
use log::info;

use rustdesk_server_pro::AppState;

fn main() -> anyhow::Result<()> {
    let matches = Command::new("rustdesk-pro")
        .version("1.0.0")
        .about("RustDesk Server Commercial Edition")
        .arg(
            Arg::new("log_level")
                .short('l')
                .long("log-level")
                .value_name("LEVEL")
                .default_value("info"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .default_value("8080"),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE"),
        )
        .subcommand(
            Command::new("generate-license")
                .about("Generate a license key")
                .arg(
                    Arg::new("type")
                        .short('t')
                        .long("type")
                        .value_name("TYPE")
                        .required(true),
                )
                .arg(
                    Arg::new("duration_days")
                        .short('d')
                        .long("duration-days")
                        .value_name("DAYS")
                        .required(true),
                )
                .arg(
                    Arg::new("max_devices")
                        .short('m')
                        .long("max-devices")
                        .value_name("NUM"),
                ),
        )
        .subcommand(
            Command::new("validate-license")
                .about("Validate a license key")
                .arg(
                    Arg::new("key")
                        .short('k')
                        .long("key")
                        .value_name("KEY")
                        .required(true),
                ),
        )
        .subcommand(Command::new("serve").about("Start the server"))
        .get_matches();

    let log_level = matches.get_one::<String>("log_level")
        .cloned()
        .unwrap_or_else(|| "info".to_string());
    let port: u16 = matches.get_one::<String>("port")
        .cloned()
        .unwrap_or_else(|| "8080".to_string())
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid port number: {}", e))?;

    Logger::try_with_env_or_str(&log_level)?
        .log_to_stdout()
        .start()?;

    info!("RustDesk Server Commercial Edition v1.0.0 starting...");

    tokio::runtime::Runtime::new()?.block_on(async {
        match matches.subcommand() {
            Some(("generate-license", sub_m)) => {
                let r#type = sub_m.get_one::<String>("type")
                    .ok_or_else(|| anyhow::anyhow!("type argument is required"))?;
                let duration_days: i64 = sub_m.get_one::<String>("duration_days")
                    .ok_or_else(|| anyhow::anyhow!("duration_days argument is required"))?
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid duration_days: {}", e))?;
                let max_devices = sub_m
                    .get_one::<String>("max_devices")
                    .map(|s| s.parse())
                    .transpose()?;

                let state = AppState::new().await;
                let key = state
                    .license_manager
                    .generate_license(r#type, duration_days, max_devices)
                    .await?;
                println!("Generated license key: {}", key);
            }
            Some(("validate-license", sub_m)) => {
                let key = sub_m.get_one::<String>("key")
                    .ok_or_else(|| anyhow::anyhow!("key argument is required"))?;

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
            Some(("serve", _)) | None => {
                let state = AppState::new().await;
                rustdesk_server_pro::web::start_server(state, port).await?;
            }
            _ => {}
        }
        Ok(())
    })
}
