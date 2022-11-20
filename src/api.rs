use crate::MLVD_BASE_PATH;
use anyhow::{bail, Context, Error, Result};
use log::{debug, info};
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

#[derive(Deserialize)]
struct Wireguard {
    relays: Vec<Relay>,
}

#[derive(Deserialize)]
struct Wrapper {
    wireguard: Wireguard,
}

fn read_relays_from_cache(path: &str) -> Result<Vec<Relay>> {
    Ok(serde_json::from_str(
        &fs::read_to_string(path).with_context(|| format!("Failed reading {}", path))?,
    )?)
}

/// Fetches a list of relays from the cache or Mullvad's API and returns them
pub fn get_relays() -> Result<Vec<Relay>> {
    let relays_path = MLVD_BASE_PATH.to_owned() + "/relays.json";
    match fs::metadata(&relays_path) {
        Ok(m) => {
            if let Ok(time) = m.modified() {
                if SystemTime::now().duration_since(time).unwrap() < Duration::from_secs(900) {
                    info!("Using cached relay list");
                    return read_relays_from_cache(&relays_path);
                }
            }
        }
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(
                    Error::new(e).context(format!("Failed to read metadata from {}", relays_path))
                );
            }
        }
    }
    let etag_path = MLVD_BASE_PATH.to_string() + "/relays.etag";
    let stored_etag = fs::read_to_string(&etag_path).unwrap_or_else(|_| "".into());
    info!("Requesting relay list...");
    let resp = ureq::get("https://api.mullvad.net/app/v1/relays")
        .set("If-None-Match", &stored_etag)
        .call()
        .context("Failed requesting relays from Mullvad API")?;
    if resp.status() == 304 {
        info!("List hasn't changed");
        // Update modification date for cache
        File::open(&relays_path)
            .with_context(|| format!("Failed to open {}", relays_path))?
            .set_modified(SystemTime::now())
            .with_context(|| format!("Failed to set modification date of {}", relays_path))?;
        read_relays_from_cache(&relays_path)
    } else if resp.status() == 200 {
        let etag = resp.header("etag").unwrap_or("");
        debug!("Response ETag: {:?}, stored ETag: {:?}", etag, stored_etag);
        fs::write(&etag_path, etag).with_context(|| format!("Failed to write {}", etag_path))?;
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
    } else {
        bail!("Unexpected server response: {:?}", resp);
    }
}
