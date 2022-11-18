use anyhow::{bail, Context, Result};
use log::info;
use std::fs;
use std::net;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

/// Connects to a WireGuard server using `wg-quick` and a pre-defined template
pub fn connect(ip: net::IpAddr, public_key: &str) -> Result<()> {
    let template = fs::read_to_string(super::MLVD_BASE_PATH.to_owned() + "/template.conf")
        .context("Failed to read template.conf")?;
    let wg_conf = template
        .replace("SERVER_IP", &ip.to_string())
        .replace("SERVER_PUBKEY", public_key);
    fs::write("/etc/wireguard/mlvd.conf", wg_conf).context("Failed to write mlvd.conf")?;
    if Path::new("/sys/class/net/mlvd").exists() {
        info!("Reusing mlvd interface");
        // Reuse the interface
        let prod = Command::new("wg-quick")
            .arg("strip")
            .arg("mlvd")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()
            .context("Failed to run wg-quick")?;
        let sink_ec = Command::new("wg")
            .arg("setconf")
            .arg("mlvd")
            .arg("/dev/stdin")
            .stdin(prod.stdout.unwrap())
            .status()
            .context("Failed to run wg")?;
        if !sink_ec.success() {
            bail!("wg-quick failed with {}", sink_ec);
        }
        let fw_ec = Command::new("wg")
            .arg("set")
            .arg("mlvd")
            .arg("fwmark")
            .arg("0xca6c")
            .stdin(Stdio::null())
            .status()
            .context("Failed to run wg")?;
        if !fw_ec.success() {
            bail!("wg failed with {}", fw_ec);
        }
    } else {
        let exit_code = Command::new("wg-quick")
            .arg("up")
            .arg("mlvd")
            .status()
            .context("Failed to run wg-quick")?;
        if !exit_code.success() {
            bail!("wg-quick failed with {}", exit_code);
        }
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
