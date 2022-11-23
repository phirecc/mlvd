# mlvd
A simple Mullvad WireGuard client written in Rust. This supersedes my old [POSIX shell
implementation](https://github.com/phirecc/mlvd.sh).

```
$ mlvd
Usage: mlvd <command> [<args>]

A minimal Mullvad WireGuard client

Options:
  --help            display usage information

Commands:
  connect           Connect to a relay
  disconnect        Disconnect from the current relay
  list-relays       List available relays

Notes:
  mlvd's files are in /var/lib/mlvd, edit template.conf to change WireGuard options
  
  HOW TO SETUP: Download a WireGuard config file from your account panel
  (https://mullvad.net/en/account/#/wireguard-config/) and copy
  its "PrivateKey" and "Address" values into /var/lib/mlvd/template.conf
```

## Installation
Dependencies: 

- wg-quick (wireguard-tools)
- openresolv (for dns)

To install them on arch:
```
sudo pacman -S --needed wireguard-tools openresolv
```

You can install `mlvd` manually like so:
```
git clone https://github.com/phirecc/mlvd
cd mlvd
cargo build --release
sudo install -Dm600 template.conf /var/lib/mlvd/template.conf
sudo install -Dm755 target/release/mlvd /usr/bin/mlvd
```

Or through an AUR package:
```
paru -S mlvd
```

## Configuration
To configure your user account, download a wireguard config from [your account
panel](https://mullvad.net/en/account/#/wireguard-config/) and copy its `PrivateKey` and `Address`
values into `/var/lib/mlvd/template.conf`

`mlvd` will replace the `SERVER_IP` and `SERVER_PUBKEY` placeholders with the respective values for
the server you want to connect to. Other than that the template can be modified like any other
Wireguard config.

## Usage
Once the configuration is done you should be able to connect to a server:
```
sudo mlvd connect de-fra
```

To get a list of available servers run:
```
sudo mlvd list-relays
```

`connect` and `list-relays` take regular expressions to filter relays by hostname/location. They
also support a `-p` flag to filter providers.

For example, to connect to a server in Germany, Netherlands or Norway that is not hosted by either
of the 3 listed providers (Note the `!` extension, which inverts the rule):
```
sudo mlvd connect -p "!(31173|M247|xtom)" "(de|nl|no)-"
```

Multiple regex rules are separated by a comma.

## Relay selection
`mlvd` chooses from relay candidates using a weighted random selection, which is the same method
the official client uses.

## Note
This third-party project is in no way affiliated with Mullvad.
