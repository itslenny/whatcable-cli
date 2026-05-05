# WhatCable CLI

> **What can this USB-C cable actually do?** — Now from your terminal.

A command-line tool that shows you, in plain English, what each USB-C cable plugged into your Mac can actually do.

```bash
$ whatcable
━━━ Port-USB-C@1 ━━━
📍 Thunderbolt / USB4 · 96W charger
   Supports high-speed data, video, smart cable.

   • Thunderbolt / USB4 link active
   • Carrying DisplayPort video
   • Cable has an e-marker chip (advertises its capabilities)
   • Charger advertises up to 96W
   • Currently negotiated: 20.0V @ 4.80A (96W)
   • Cable speed: USB4 Gen 3 (20 / 40 Gbps)
   • Cable rated for 5 A at up to 20V (~100W)
   • Cable made by Apple (0x05AC)
```

## Features

- **Plain-English output** — No jargon. Just tells you what the cable can do.
- **Cable capabilities** — Speed (USB 2.0 / 3.x / 4), power rating (3A / 5A), active vs passive
- **Power delivery info** — Shows charger wattage and negotiated voltage/current
- **Vendor identification** — Recognizes common cable manufacturers
- **Technical mode** — Show raw IOKit properties with `--technical`
- **JSON output** — Machine-readable format with `--json`

## Installation

### Option 1: Build from source (Rust required)

```bash
git clone https://github.com/itslenny/whatcable-cli
cd whatcable-cli
cargo build --release
sudo cp target/release/whatcable /usr/local/bin/
```

### Option 2: Download pre-built binary

Pre-built universal binaries (Apple Silicon + Intel) are available from the [Releases page](https://github.com/itslenny/whatcable-cli/releases).

## Usage

```bash
# Show only connected cables
whatcable

# Show all ports, including empty ones
whatcable --all

# Show technical details (raw IOKit properties)
whatcable --technical

# Output in JSON format for scripting
whatcable --json

# Combine flags
whatcable --all --technical
```

## How it works

WhatCable CLI reads three families of IOKit services via the `ioreg` command:

| Service | What it gives us |
| --- | --- |
| `AppleHPMInterfaceType10/11/12` (M3+)<br>`AppleTCControllerType10` (M1/M2) | Per-port state: connection, transports, plug orientation, e-marker presence |
| `IOPortFeaturePowerSource` | Full PDO list from the connected source, with the live "winning" PDO |
| `IOPortTransportComponentCCUSBPDSOP` | PD Discover Identity VDOs for SOP (port partner) and SOP' (cable e-marker) |

Cable speed and power decoding follow the USB Power Delivery 3.x spec.

**No entitlements, no private APIs, no sudo required.**

## Requirements

- macOS 14+ (Sonoma or later)
- Apple Silicon or Intel Mac with USB-C / Thunderbolt ports

## Comparison with menu bar app

| Feature | CLI | Menu Bar App |
| --- | --- | --- |
| Plain-English cable info | ✅ | ✅ |
| Cable e-marker decoding | ✅ | ✅ |
| Power delivery info | ✅ | ✅ |
| Charging diagnostics | ❌ | ✅ |
| Live updates / notifications | ❌ | ✅ |
| Scriptable / automation | ✅ | ❌ |
| Visual interface | ❌ | ✅ |
| Run at login | ❌ | ✅ |

**Use the CLI for:** automation, quick checks, scripting, SSH sessions
**Use the menu bar app for:** continuous monitoring, visual feedback, charging diagnostics

## Examples

### Check if a cable supports Thunderbolt
```bash
whatcable | grep -i thunderbolt
```

### Monitor cable connections in a script
```bash
#!/bin/bash
while true; do
    clear
    whatcable
    sleep 2
done
```

### Export all port data to JSON
```bash
whatcable --all --json > ports.json
```

## Caveats

- **Cable e-marker info only appears for cables that carry one.** Most USB-C cables under 60W are unmarked.
- **WhatCable trusts the e-marker.** Counterfeit cables can advertise capabilities they don't deliver.
- **macOS only.** This tool relies on IOKit, which is macOS-specific.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on commit message format (which controls automatic version bumping), development setup, and pull request guidelines.

## Credits

CLI version built by [Lenny Urbanowski](https://github.com/itslenny), mostly by vibes and [Claude Code](https://claude.com/claude-code).

Based on [WhatCable](https://github.com/darrylmorley/whatcable) by [Darryl Morley](https://github.com/darrylmorley) — all cable analysis logic, VDO decoding, and vendor database ported from the original Swift implementation.

Inspired by every terminal user who wanted cable info without leaving the shell.

## License

MIT — see [LICENSE](LICENSE) for details.
