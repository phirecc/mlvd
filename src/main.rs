#![feature(file_set_times)]
mod api;
mod cli;
mod filter;
mod wireguard;
use anyhow::{bail, Result};
use api::Relay;
use colored::*;
use filter::Filter;
use log::{debug, info};
use rand::distributions::WeightedIndex;
use rand::prelude::*;

pub const MLVD_BASE_PATH: &str = "/var/lib/mlvd";

/// Returns relays filtered by a location/hostname filter and a provider filter
fn get_filtered_relays(
    lh_filter: Option<Filter>,
    provider_filter: Option<Filter>,
) -> Result<Box<dyn Iterator<Item = Relay>>> {
    let relays = api::get_relays()?;
    debug!("Relays: {:#?}", relays);
    let mut iter: Box<dyn Iterator<Item = Relay>> = Box::new(relays.into_iter());
    if let Some(f) = lh_filter {
        iter = Box::new(iter.filter(move |x| f.is_match(&x.location) || f.is_match(&x.hostname)));
    }
    if let Some(f) = provider_filter {
        iter = Box::new(iter.filter(move |x| f.is_match(&x.provider)));
    }
    Ok(iter)
}

/// Pick a random relay with hostname/location, taking into account the relay weights
fn pick(relays: &[Relay]) -> Result<&Relay> {
    let mut rng = rand::thread_rng();
    let dist = WeightedIndex::new(relays.iter().map(|x| x.weight))?;
    Ok(&relays[dist.sample(&mut rng)])
}

fn main() -> Result<()> {
    env_logger::Builder::new()
        .format_timestamp(None)
        .format_target(false)
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();
    let config = cli::get_config();
    match config.subcommand {
        cli::Subcommand::Connect(args) => {
            let relays: Vec<Relay> =
                get_filtered_relays(Some(args.lh_filter), args.provider_filter)?
                    .filter(|r| r.active)
                    .collect();
            let l = relays.len();
            let relay = if l > 1 {
                info!("Found {} matching relays, picking one", relays.len());
                pick(&relays)?
            } else if l == 0 {
                bail!("No matching active relays found");
            } else {
                &relays[0]
            };
            debug!("Chosen relay: {:#?}", relay);
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
                    "{:8} {:15} {:12} {:6} {:8}",
                    "Location", "Hostname", "Provider", "Weight", "Inactive"
                )
                .bold()
            );
            for relay in relays {
                let mut inactive = "";
                if !relay.active {
                    inactive = "INACTIVE";
                }
                println!(
                    "{:8} {:15} {:12} {:6} {:8}",
                    relay.location, relay.hostname, relay.provider, relay.weight, inactive
                );
            }
        }
    }
    Ok(())
}
