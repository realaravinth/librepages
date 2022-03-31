# Bare metal:

The process is tedious, most of this will be automated with a script in
the future.

## 1. Create new user for running `pages`:

```bash
sudo useradd -b /srv -m -s /usr/bin/zsh pages
```

## 2. Install Runtime dependencies

1. [Nginx](https://packages.debian.org/bullseye/nginx)

On Debian-based systems, run:

```bash
sudo apt install nginx
```

## 3. Build `Pages`

### i. Install Build Dependencies

To build `pages`, you need the following dependencies:

1. [Git](https://packages.debian.org/bullseye/git)
2. [pkg-config](https://packages.debian.org/bullseye/pkg-config)
3. [GNU make](https://packages.debian.org/bullseye/make)
4. [libssl-dev](https://packages.debian.org/bullseye/libssl-dev)
5. Rust

To install all dependencies **except Rust** on Debian boxes, run:

```bash
sudo apt-get install -y git pkg-config libssl-dev
```

### ii. Install Rust

Install Rust using [rustup](https://rustup.rs/).

rustup is the official Rust installation tool. It enables installation
of multiple versions of `rustc` for different architectures across
multiple release channels(stable, nightly, etc.).

Rust undergoes [six-week release
cycles](https://doc.rust-lang.org/book/appendix-05-editions.html#appendix-e---editions)
and some of the dependencies that are used in this program have often
relied on cutting edge features of the Rust compiler. OS Distribution
packaging teams don't often track the latest releases. For this reason,
we encourage managing your Rust installation with `rustup`.

**rustup is the officially supported Rust installation method of this
project.**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### iii. Build

```bash
$ make release
```

## 5. Install

Install binary and copy demo configuration file into default configuration
lookup path(`/etc/static-pages/config.toml`)

```bash
sudo cp ./target/release/pages /usr/local/bin/ && \
	sudo mkdir /etc/static-pages && \
	sudo cp config/default.toml /etc/static-pages/config.toml
```

## 4. Systemd service configuration:

### i. Copy the following to `/etc/systemd/system/pages.service`:

```systemd
[Unit]
Description=pages: Auto-deploy static websites from git repositories

[Service]
Type=simple
User=pages
ExecStart=/usr/local/bin/pages
Restart=on-failure
RestartSec=1
MemoryDenyWriteExecute=true
NoNewPrivileges=true
Environment="RUST_LOG=info"

[Unit]
Wants=network-online.target
Wants=network-online.target
After=syslog.target

[Install]
WantedBy=multi-user.target
```

### ii. Enable and start service:

```bash
sudo systemctl daemon-reload && \
	sudo systemctl enable pages && \ # Auto startup during boot
	sudo systemctl start pages
```

## 5. Optionally configure Nginx to reverse proxy requests to Pages

**NOTE: This sections includes instructions to reverse proxy requests to
Pages API, not the websites managed by Pages.**

See [here](../../config/pages-nginx-config) for sample Nginx configuration.
