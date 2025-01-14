![License](https://img.shields.io/github/license/nt-riken/makigami)
![GitHub release](https://img.shields.io/github/v/release/nt-riken/makigami)

# makigami
Indexed quick search solution for large log data and slow HDD

## Installation
Clone the repository and build the tool:
```bash
git clone https://github.com/nt-riken/makigami.git
cd makigami
cargo build --release
mv target/release/mg /usr/local/bin/
```
## Downloads

Prebuilt binaries are available for the following platforms:

- **Linux (x64)**: [Download](https://github.com/nt-riken/makigami/releases/download/v0.1.0/mg-linux-x64)

## Quick Example
Build an index and search:
```bash
mg build access.log
mg search -z access.log.zstd "404 NOT FOUND" | grep "404 NOT FOUND"
```

## Features
- Optimized for low-speed storage like HDDs
- Index size is only % of the original log file size
- Fully integration with any UNIX tools like `grep`, `awk`, and `sed`.

## Contributing
Contributions are welcome! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Use Cases
- Searching for specific events in large archived historical logs
- Lightweight and low cost log management for personal to enterprise

- 

