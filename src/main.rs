#![feature(file_set_times)]
mod api;
mod cli;
mod filter;
mod wireguard;
use anyhow::Result;
use api::Relays;
use colored::*;
use filter::Filter;
use log::{debug, info};

pub const MLVD_BASE_PATH: &str = "/var/lib/mlvd";

fn get_filtered_relays(
    lh_filter: Option<Filter>,
    provider_filter: Option<Filter>,
) -> Result<Relays> {
    let mut relays = api::get_relays()?;
    debug!("Relays: {:?}", relays);
    if let Some(f) = lh_filter {
        relays = relays.filter_location_hostname(&f);
    }
    if let Some(f) = provider_filter {
        relays = relays.filter_providers(&f);
    }
    Ok(relays)
}

fn main() -> Result<()> {
    env_logger::Builder::new()
        .format_timestamp(None)
        .format_target(false)
        .filter_level(log::LevelFilter::Info)
        .init();
    let config = cli::get_config();
    match config.subcommand {
        cli::Subcommand::Connect(args) => {
            let relays = get_filtered_relays(Some(args.lh_filter), args.provider_filter)?.active();
            info!("Found {} matching relays, picking one", relays.0.len());
            let relay = relays.pick()?;
            debug!("Chosen relay: {:?}", relay);
            info!(
                "Connecting to {} ({}), hosted by {}",
                relay.hostname.purple().bold(),
                relay.location.purple().bold(),
                relay.provider.purple().bold()
            );
            wireguard::connect(relay.ip, &relay.public_key)?;
            info!("Connected successfully!");
        }
        cli::Subcommand::Disconnect(_) => {
            info!("Disconnecting...");
            wireguard::disconnect()?;
            info!("Disconnected successfully!");
        }
        cli::Subcommand::ListRelays(args) => {
            let relays = get_filtered_relays(args.lh_filter, args.provider_filter)?;
            println!(
                "{}",
                format!(
                    "{:8} {:15} {:12} {:3} {:8}",
                    "Location", "Hostname", "Provider", "Weight", "Inactive"
                )
                .bold()
            );
            for relay in relays.0 {
                let mut inactive = "";
                if !relay.active {
                    inactive = "INACTIVE";
                }
                println!(
                    "{:8} {:15} {:12} {:3} {:8}",
                    relay.location, relay.hostname, relay.provider, relay.weight, inactive
                );
            }
        }
    }
    Ok(())
}
