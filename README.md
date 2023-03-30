
# Hublot Esl Backend

A small service broker between a rest api and some electronic shelf labels api. 

Right now only the Pricer API is supported, but Hanshow might be supported soon.

We only provide a cli interface for now.


## Installation

After cloning the repo make sure that rustup is installed and up to date.

`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` for unix, refer to https://rustup.rs/ for other platforms.

Then install rust-nightly
```bash
rustup install nightly
# or 
rustup update nightly
```
(Optional) set nightly as your default version 
```bash
rustup default nightly
```

To make sure that everything is ok and to install the project dependencies : 
```bash
cargo build
```
    
## Features

This crate exposes the `rustls-tls` feature that enables reqwest's `rustls-tls` feature.

## Deployment

To deploy this project run

```bash
 cargo build --release
```
or with rustls-tls 
```bash
 cargo build --release --features rustls-tls
```

Windows support:

```bash
cargo build --release --target x86_64-pc-windows-gnu --features rustls-tls
```