# VaultCLI 🔐
 
A secure, offline-first password manager that runs entirely on your own machine. Built with Rust — fast, safe, and encrypted.



## What It Does
 
VaultCLI stores your passwords in a local PostgreSQL database, encrypted with **ChaCha20-Poly1305**. Your master password never touches the disk — it lives in memory only while the vault is unlocked, and is wiped when you lock it.
 
It is split into two binaries:
 
**`daemon`** — runs in the background. Handles everything: database, encryption, key management. Also has two setup commands:
- `init` — creates tables, stores DB URL and salt in OS keyring (run once)
- `start` — starts the background server, listens on a Unix socket
**`cli`** — the tool you actually type commands into. Sends requests to the daemon over a Unix socket and prints the response. All the commands you care about live here:
- `user add / remove / rename` — manage users
- `default` — set which user is active
- `unlock / lock` — load or wipe the master key from memory
- `vault add / get` — store and retrieve passwords



## Architecture
 
```
┌─────────────────────────────────────────────────────┐
│                      User                           │
└─────────────────────┬───────────────────────────────┘
                      │  runs commands
                      ▼
┌─────────────────────────────────────────────────────┐
│                   CLI Binary                        │
│                                                     │
│  • Parses commands (clap)                           │
│  • Prompts for passwords (rpassword)                │
│  • Sends Request over Unix Socket                   │
│  • Prints Response                                  │
└─────────────────────┬───────────────────────────────┘
                      │  Unix Socket (/tmp/vaultcli.sock)
                      │  [length-prefixed JSON]
                      ▼
┌─────────────────────────────────────────────────────┐
│                 Daemon Binary                       │
│                                                     │
│  • Listens on Unix Socket                           │
│  • Holds master key in memory (Arc<Mutex<[u8;32]>>) │
│  • Routes requests to workers                       │
│  • Returns encrypted/decrypted responses            │
└─────────────────────┬───────────────────────────────┘
                      │  sqlx async queries
                      ▼
┌─────────────────────────────────────────────────────┐
│               PostgreSQL Database                   │
│                                                     │
│  users table  →  master usernames                   │
│  data table   →  encrypted passwords + nonces       │
└─────────────────────────────────────────────────────┘

```


### How a Request Flows
 
```

CLI                          Daemon
 │                              │
 │── parse command ──▶          │
 │── prompt password ──▶        │
 │                              │
 │── [4 bytes: length] ────────▶│
 │── [N bytes: JSON] ──────────▶│
 │                              │── deserialize request
 │                              │── handle (encrypt/decrypt/DB)
 │                              │── serialize response
 │                              │
 │◀── [4 bytes: length] ────────│
 │◀── [N bytes: JSON] ──────────│
 │                              │
 │── print result ──▶           │
```
 
---
 

## Prerequisites
 
- Rust (stable)
- PostgreSQL running locally

## Features

- End-to-end local encryption (ChaCha20-Poly1305)
- Master key stored only in memory (zeroized on lock)
- PostgreSQL-backed secure storage
- OS keyring integration for DB credentials + salt
- CLI + daemon architecture
- Unix socket IPC for fast local communication
- Multi-user support with default user switching
- Secure unlock/lock lifecycle


## 🗄️ Database Schema

### users
- master (username)
- is_default (boolean)

### data
- master (user reference)
- username (login username)
- password (encrypted BYTEA)
- nonce (encryption nonce)
- app (service name)
- hint (optional hint)
- created_at (timestamp)




## Setup
 
**1. Clone and build**
```bash
git clone https://github.com/MrArmanDev/vault-cli.git
cd vaultcli
cargo build
```
 
**2. Create a database**
```bash
psql -U postgres -c "CREATE DATABASE vaultcli;"
```

**3. build**
```bash

cd deamon
cargo build --release
sudo cp target/release/daemon /usr/local/bin/vaultcli-deamon

cd cli
cargo build --release
sudo cp target/release/cli /usr/local/bin/vaultcli
```


**4. Initialize (run once)**
```bash
vaultcli-deamon init --url "postgresql://postgres:password@localhost/vaultcli"
```

This creates the tables and stores the DB URL + salt in your OS keyring.
 
**5. Start the daemon (keep this running)**
```bash
vaultcli-daemon start
```
 
---


## Usage
 
```bash
# Add a user
vaultcli user add --name rahul
 
# Set as default user
vaultcli default --name rahul
 
# Unlock the vault (enter your master password)
vaultcli unlock
 
# Store a password
vaultcli vault add --username me@gmail.com --app gmail --hint "personal account"
 
# Retrieve passwords
vaultcli vault get --app gmail
 
# Lock the vault
vaultcli lock
 
# Remove a user
vaultcli user remove --name rahul
```
 
---


## Security Model

- Master password is never stored
- Argon2 used for key derivation
- ChaCha20-Poly1305 for encryption
- Each entry uses unique nonce
- Key is zeroized on lock



## License
This project is licensed under the MIT License.
