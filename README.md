# Pice Chess (WIP)

[![Build Status](https://github.com/ivan-r-sigaev/pico_chess/actions/workflows/rust.yml/badge.svg)](https://github.com/ivan-r-sigaev/pico_chess/actions)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

Another fast chess engine written in Rust (UCI support).

## Overview
- **Position crate**: FEN position (see crate [README](./position/README.md))
- **Game crate**: PGN game (see crate [README](./game/README.md))
- **TODO**: UCI support, search, evaluation

## Getting Started
```bash
git clone https://github.com/ivan-r-sgiaev/pico_chess.git
cd pico_chess

# Build all crates
cargo build

# Run tests
cargo test
```
