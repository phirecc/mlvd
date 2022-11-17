use crate::filter::Filter;
use crate::MLVD_BASE_PATH;
use anyhow::{Context, Result};
use log::{debug, info};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::net;
use std::time::{Duration, SystemTime};

/// A Mullvad WireGuard VPN Server
#[derive(Deserialize, Serialize, Debug)]
pub struct Relay {
    pub hostname: String,
    pub location: String,
    pub active: bool,
    pub provider: String,
    pub weight: usize,
    #[serde(rename = "ipv4_addr_in")]
    pub ip: net::IpAddr,
    pub public_key: String,
}

/// A list of relays
#[derive(Deserialize, Serialize, Debug)]
pub struct Relays(pub Vec<Relay>);

#[derive(Deserialize)]
struct Wireguard {
    relays: Relays,
}

#[derive(Deserialize)]
struct Wrapper {
    wireguard: Wireguard,
}

fn read_relays_from_cache(path: &str) -> Result<Relays> {
    Ok(serde_json::from_str(
        &fs::read_to_string(&path).with_context(|| format!("Failed reading {}", path))?,
    )?)
}

/// Fetches a list of relays from Mullvad's API and returns them. Respects the API's `ETag` header.
pub fn get_relays() -> Result<Relays> {
    let relays_path = MLVD_BASE_PATH.to_string() + "/relays.json";
    let metadata = fs::metadata(&relays_path)
        .with_context(|| format!("Failed to read metadata from {}", relays_path))?;
    if let Ok(time) = metadata.modified() {
        if SystemTime::now().duration_since(time).unwrap() < Duration::from_secs(900) {
            info!("Using cached relay list");
            return read_relays_from_cache(&relays_path);
        }
    }
    info!("Requesting relay list...");
    let resp = ureq::get("https://api.mullvad.net/app/v1/relays")
        .call()
        .context("Failed requesting relays from Mullvad API")?;
    if let Some(etag) = resp.header("etag") {
        let etag_path = MLVD_BASE_PATH.to_string() + "/relays.etag";
        let stored_etag = fs::read_to_string(&etag_path).unwrap_or("".into());
        // TODO On second run this would be true if an ETag was missing. It would then never update
        // the relays list.
        debug!("Response ETag: {:?}, stored ETag: {:?}", etag, stored_etag);
        if etag == stored_etag {
            info!("List hasn't changed");
            // Update modification date for cache
            File::open(&relays_path)
                .with_context(|| format!("Failed to open {}", relays_path))?
                .set_modified(SystemTime::now())
                .with_context(|| format!("Failed to set modification date of {}", relays_path))?;
            return read_relays_from_cache(&relays_path);
        } else {
            fs::write(&etag_path, etag)
                .with_context(|| format!("Failed to write {}", etag_path))?;
        }
    }
    info!("Updating relay list");
    let relays = resp
        .into_json::<Wrapper>()
        .context("Failed to parse API response into JSON")?
        .wireguard
        .relays;
    // We messed up if `Relays` fails to serialize
    fs::write(&relays_path, serde_json::to_string(&relays).unwrap())
        .with_context(|| format!("Failed to write {}", relays_path))?;
    Ok(relays)
}
impl Relays {
    /// Exclude providers which don't match `filter`
    pub fn filter_providers(mut self, filter: &Filter) -> Self {
        self.0 = self
            .0
            .into_iter()
            .filter(|x| filter.is_match(&x.provider))
            .collect();
        self
    }
    /// Exclude relays whose location and hostname doesn't match `filter`
    pub fn filter_location_hostname(mut self, filter: &Filter) -> Self {
        self.0 = self
            .0
            .into_iter()
            .filter(|x| filter.is_match(&x.location) || filter.is_match(&x.hostname))
            .collect();
        self
    }
    /// Pick a random relay with hostname/location, taking into account the relay weights
    pub fn pick(&self) -> Result<&Relay> {
        let mut rng = rand::thread_rng();
        let dist = WeightedIndex::new(self.0.iter().map(|x| x.weight))?;
        Ok(&self.0[dist.sample(&mut rng)])
    }
    /// Filter out inactive relays
    pub fn active(mut self) -> Relays {
        self.0 = self.0.into_iter().filter(|x| x.active).collect();
        self
    }
}
