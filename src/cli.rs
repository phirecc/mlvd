use crate::filter::Filter;
use argh::FromArgs;

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum Subcommand {
    Connect(ConnectArgs),
    Disconnect(DisconnectArgs),
    ListRelays(ListRelaysArgs),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "connect",
       example = "mlvd connect de-fra",
       example = "mlvd connect -p \"!(31173|M247|xtom)\" \"(de|nl|no)-\""
       )]
/// Connect to a relay
pub struct ConnectArgs {
    #[argh(positional)]
    /// the server locations/hostnames filter
    pub lh_filter: Filter,
    #[argh(option, short = 'p')]
    /// the provider filter
    pub provider_filter: Option<Filter>,
    #[argh(switch, short = 'n')]
    /// don't connect, just print the resulting configuration
    pub dont_act: bool,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "disconnect")]
/// Disconnect from the current relay
pub struct DisconnectArgs {}

#[derive(FromArgs)]
#[argh(subcommand, name = "list-relays",
       example = "mlvd list-relays de-fra",
       example = "mlvd list-relays -p \"!(31173|M247|xtom)\" \"(de|nl|no)-\""
       )]
/// List available relays
pub struct ListRelaysArgs {
    #[argh(positional)]
    /// the server locations/hostnames filter
    pub lh_filter: Option<Filter>,
    #[argh(option, short = 'p')]
    /// the provider filter
    pub provider_filter: Option<Filter>,
}

#[derive(FromArgs)]
#[argh(note = "mlvd's files are in /var/lib/mlvd, edit template.conf to change WireGuard options

HOW TO SETUP: Download a WireGuard config file from your account panel
(https://mullvad.net/en/account/#/wireguard-config/) and copy
its \"PrivateKey\" and \"Address\" values into /var/lib/mlvd/template.conf")]

/// A minimal Mullvad WireGuard client
pub struct Config {
    #[argh(subcommand)]
    pub subcommand: Subcommand,
}

pub fn get_config() -> Config {
    argh::from_env()
}
