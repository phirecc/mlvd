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
#[argh(subcommand, name = "connect")]
/// Connect to a relay
pub struct ConnectArgs {
    #[argh(positional)]
    /// the server locations/hostnames filter
    pub lh_filter: Filter,
    #[argh(option, short = 'p')]
    /// the provider filter
    pub provider_filter: Option<Filter>,
}

#[derive(FromArgs)]
#[argh(subcommand, name = "disconnect")]
/// Disconnect from the current relay
pub struct DisconnectArgs {}

#[derive(FromArgs)]
#[argh(subcommand, name = "list-relays")]
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
/// A minimal Mullvad WireGuard client
pub struct Config {
    #[argh(subcommand)]
    pub subcommand: Subcommand,
}

pub fn get_config() -> Config {
    argh::from_env()
}
