<p align="center">
  <h1 align="center">巻紙 makigami</h1>
  <p align="center"><strong>High-Performance Log Search for Low-Cost Storage</strong></p>
  <p align="center">Search terabytes of logs on cheap HDDs at SSD speeds</p>
</p>

<p align="center">
  <a href="https://github.com/nt-riken/makigami/blob/main/LICENSE"><img src="https://img.shields.io/github/license/nt-riken/makigami" alt="License"></a>
  <a href="https://github.com/nt-riken/makigami/releases"><img src="https://img.shields.io/github/v/release/nt-riken/makigami" alt="Release"></a>
  <a href="https://github.com/nt-riken/makigami/stargazers"><img src="https://img.shields.io/github/stars/nt-riken/makigami" alt="Stars"></a>
</p>

---

## Why makigami?

**Problem:** Fast storage (SSD) is expensive. Cheap storage (HDD/Object Storage) is slow.
**Solution:** Makigami makes cheap storage fast enough for interactive search.

Traditional log search tools like `grep` and `zcat | grep` read files sequentially from start to finish. On large log files stored on HDDs, this means **minutes of waiting**.

**makigami** creates a search index (approx. **1/3** of original file size) that enables intelligent block-skipping, dramatically reducing read time.

| Tool | Storage | Log Size | Search Time | Speedup |
|------|---------|----------|-------------|---------|
| **makigami + grep** | HDD (500MB/s seq.) | 200GB | **5.5 seconds** | **Up to 40x faster*** |
| zstd -d -c \| grep | HDD (500MB/s seq.) | 200GB | 3m 37s | baseline |

> *40x speedup achieved in cold-cache scenarios where the target data is near the end of the file.

> Named after the Japanese word for "scroll paper" (巻紙), makigami processes logs sequentially like unrolling a scroll—but intelligently skips irrelevant sections.

---

## Quick Start

### Installation

**Linux (Prebuilt Binary):**

```bash
curl -LO https://github.com/nt-riken/makigami/releases/download/v0.1.0/mg-linux-x64-musl
chmod +x mg-linux-x64-musl
sudo mv mg-linux-x64-musl /usr/local/bin/mg
```

**macOS / Other (Build from Source):**

```bash
# Requires Rust installed (https://rustup.rs/)
git clone https://github.com/nt-riken/makigami.git
cd makigami
cargo build --release
sudo mv target/release/mg /usr/local/bin/
```

### Basic Usage

**Step 1: Build index** — Creates compressed `.zstd` file and tiny `.mg` index

```bash
mg build access.log
# Output: access.log.zstd (compressed) + access.log.mg (index, ~33% size)
```

**Step 2: Search** — Lightning-fast search using the index

```bash
mg search -z access.log.zstd "404 NOT FOUND" | grep "404 NOT FOUND"
```

**Step 3: Pipe to your tools** — Full UNIX philosophy compatibility

```bash
mg search -z access.log.zstd "ERROR" | grep "database" | awk '{print $1, $2}'
```

---

## Features

### ⚡ Optimized for Slow Storage
Sequential read optimization designed specifically for HDD performance characteristics. No random seeks means maximum throughput.

### 📦 Search Index
Index files are approx. **1/3** of the original log size. A 200GB log produces a ~66GB index.

### 🔧 UNIX Philosophy
Works seamlessly with `grep`, `awk`, `sed`, `sort`, and any other command-line tool. No lock-in.

### 🗜️ Built-in Compression
Uses Zstandard compression for storage efficiency while maintaining search speed.

---

## Use Cases

- **Historical log analysis** — Search years of archived logs without expensive storage
- **Cost-effective log management** — Use cheap HDDs instead of SSDs for cold log storage
- **Compliance & audit** — Quick searches across massive audit log archives
- **Personal to enterprise** — Scales from single-machine to distributed storage

---

## How It Works

```
┌─────────────┐     mg build      ┌─────────────────┐
│  access.log │ ───────────────►  │ access.log.zstd │  (compressed data)
│   (200GB)   │                   │ access.log.mg   │  (index, ~66GB)
└─────────────┘                   └─────────────────┘
                                           │
                                           │ mg search "pattern"
                                           ▼
                                  ┌─────────────────┐
                                  │ Skip irrelevant │
                                  │ blocks using    │──► Only read matching blocks
                                  │ index           │    (sequential, fast on HDD)
                                  └─────────────────┘
```

---

## FAQ

**Q: Does makigami work on SSDs?**

A: Yes, but the performance advantage is smaller. makigami's sequential read optimization is designed to maximize HDD throughput where random access is slow.

**Q: What about Windows?**

A: Currently tested on Linux and macOS. Windows support is untested but may work.

**Q: How does it compare to Elasticsearch/Splunk?**

A: makigami is a lightweight CLI tool for searching compressed log files, not a full SIEM. It's ideal for cold storage search where you don't need real-time indexing or complex queries.

**Q: Can I search without building an index first?**

A: No, the index is required for the performance benefits. Without it, use standard `zcat | grep`.

---

## Roadmap

- [ ] Windows support
- [ ] macOS ARM64 binary
- [ ] Parallel search across multiple files
- [ ] Regex pattern support in index
- [ ] crates.io publication

---

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

[Apache-2.0](LICENSE)

---

<p align="center">
  <a href="https://github.com/nt-riken/makigami/issues">🐛 Report Bug</a> •
  <a href="https://github.com/nt-riken/makigami/issues">✨ Request Feature</a>
</p>