# Lithium

A decentralized, anonymous messenger built on I2P garlic routing.
No servers. No accounts. No phone numbers. Just encrypted messages over I2P.

## Features

- **Anonymous** — all traffic routed through I2P, your IP is never exposed
- **Decentralized** — no central server, pure P2P
- **End-to-end encrypted** — Noise XX protocol with forward secrecy
- **Keypair identity** — your Ed25519 public key is your address
- **Encrypted storage** — local message history encrypted with SQLCipher
- **Cross-platform** — Linux, BSD, macOS (iOS coming soon)

## Prerequisites

- Rust 1.75+
- i2pd running with SAM bridge enabled on `127.0.0.1:7656`

### Enable SAM in i2pd

Add to `/etc/i2pd/i2pd.conf`:

```ini
[sam]
enabled = true
address = 127.0.0.1
port = 7656
```

Then restart i2pd:

```bash
sudo systemctl restart i2pd
```

## Build

```bash
git clone https://github.com/NyAncQt/lithium
cd lithium
cargo build --release
```

## Run

```bash
cargo run -p lithium-tui
```

## Controls

| Key | Action |
|-----|--------|
| `Ctrl+N` | Add new contact |
| `Up/Down` | Navigate contacts |
| `Enter` | Select contact / send message |
| `Tab` | Switch between panels |
| `Esc` | Cancel / go back |
| `Ctrl+Q` | Quit |

## Architecture

lithium/
├── core/      # Rust core — I2P, crypto, storage
├── tui/       # Ratatui terminal UI
├── ios/       # SwiftUI iOS app (WIP)
├── bindings/  # UniFFI bindings for Swift
└── docs/      # Protocol spec, threat model


## Security Model

- Identity = Ed25519 keypair, never registered anywhere
- Messages encrypted with Noise XX (forward secrecy)
- Local storage encrypted with SQLCipher
- No IP ever exposed — all traffic through I2P tunnels
- Key verification via fingerprint comparison

## Roadmap

- [x] I2P SAM bridge integration
- [x] Noise XX end-to-end encryption
- [x] Ratatui TUI with contact list
- [x] Encrypted local storage
- [ ] Group chats
- [ ] File transfer
- [ ] iOS SwiftUI app
- [ ] Disappearing messages

## License

BSD

