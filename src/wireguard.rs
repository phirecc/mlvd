use anyhow::{bail, Context, Result};
use std::fs;
use std::net;
use std::process::Command;

/// Connects to a WireGuard server using `wg-quick` and a pre-defined template
pub fn connect(ip: net::IpAddr, public_key: &str) -> Result<()> {
    let template = fs::read_to_string(super::MLVD_BASE_PATH.to_owned() + "/template.conf")
        .context("Failed to read template.conf")?;
    let wg_conf = template
        .replace("SERVER_IP", &ip.to_string())
        .replace("SERVER_PUBKEY", &public_key);
    fs::write("/etc/wireguard/mlvd.conf", wg_conf).context("Failed to write mlvd.conf")?;
    let exit_code = Command::new("wg-quick")
        .arg("up")
        .arg("mlvd")
        .status()
        .context("Failed to run wg-quick")?;
    if !exit_code.success() {
        bail!("wg-quick failed with {}", exit_code);
    }
    Ok(())
}
/// Disconnects from WireGuard server using `wg-quick`
pub fn disconnect() -> Result<()> {
    let exit_code = Command::new("wg-quick").arg("down").arg("mlvd").status()?;
    if !exit_code.success() {
        bail!("wg-quick failed with {}", exit_code);
    }
    Ok(())
}
