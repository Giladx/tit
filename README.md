# TIT.RUN

> A fast, keyboard-first developer toolbox for the terminal—available as both an interactive TUI and a script-friendly CLI.

[![CI](https://github.com/Giladx/tit/actions/workflows/ci.yml/badge.svg)](https://github.com/Giladx/tit/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

TIT.RUN brings everyday developer utilities into one lightweight, offline application. It combines a responsive [Ratatui](https://ratatui.rs/) interface with headless commands that work in shell scripts, CI jobs, and agent workflows.

## Highlights

- Keyboard-first navigation with fuzzy tool search
- Responsive wide and compact terminal layouts
- Live conversion and validation as you type
- Headless CLI commands for automation
- Local processing—input is not sent to a remote service
- Cross-tool clipboard support with visible error reporting
- Tested conversion logic and strict linting in CI

## Included tools

| Category | Utilities |
| --- | --- |
| Converters | Date-Time, Base64, URL Encoding, HTML Entities, Number Base, Color |
| Crypto | MD5, SHA-256, and SHA-512 hashes |
| Text | Text Case Converter, Text Statistics, Lorem Ipsum |
| Network | URL Parser |
| Development | JSON Formatter, JWT Parser, URL Parser, Regex Tester, Cron Parser |
| Generators | UUID v4, Password Generator |

> [!WARNING]
> The JWT Parser decodes token contents but does not verify signatures. Never use decoded claims alone as proof of authenticity.

## Screenshots

The interface supports both a three-pane desktop view and a compact stacked layout. Project screenshots can be added under `docs/screenshots/` and referenced here without changing the application build.

## Installation

### Build from source

Install the current stable [Rust toolchain](https://rustup.rs/), then run:

```bash
git clone https://github.com/Giladx/tit.git
cd tit
cargo build --release
./target/release/tit
```

### Install locally with Cargo

```bash
git clone https://github.com/Giladx/tit.git
cd tit
cargo install --path .
tit
```

Clipboard integration requires a desktop clipboard provider. Copy operations may be unavailable on headless Linux hosts; TIT.RUN reports clipboard errors in the status bar.

## Interactive TUI

Run TIT.RUN without a subcommand:

```bash
cargo run --release
```

### Controls

| Key | Action |
| --- | --- |
| `↑` / `↓` or `j` / `k` | Select a tool |
| `←` / `→` or `h` / `l` | Change category |
| `Enter` | Open the selected tool |
| `/` | Fuzzy-search tools |
| `Esc` | Return to the tool list |
| `Ctrl+C` | Copy the current tool output |
| `q` | Quit while the tool list is focused |

Tool-specific shortcuts appear in the contextual help panel. Terminals narrower than 100 columns automatically switch to a compact stacked layout.

## Headless CLI

Use subcommands when output needs to be piped or consumed by another program:

```bash
tit uuid --count 3
tit base64 "hello world"
tit base64 --decode "aGVsbG8gd29ybGQ="
tit urlencode "hello world"
tit html-entities '<main class="app">'
tit hash "important input"
tit jwt "header.payload.signature"
tit stats "A short sentence."
```

Discover all commands with `tit --help` or `tit <command> --help`.

## Architecture

```text
src/
├── main.rs       # CLI/TUI entry point and terminal lifecycle
├── cli.rs        # Headless subcommands
├── app.rs        # Navigation, search, layout, and application state
├── theme.rs      # Shared terminal color system
├── tools/        # Individual tools and testable conversion logic
└── ui/           # Shared UI module boundary
```

Tools implement the shared `Tool` trait and are registered in `src/tools/mod.rs`. Parsing and conversion logic is kept separate from rendering so it can be tested without starting a terminal.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

GitHub Actions runs the same formatting, lint, and test gates for pushes and pull requests.

## Current limitations

- Cron parsing validates and explains standard five-field expressions but does not calculate future run times.
- Number-base conversion is limited to signed 128-bit integers.
- Color conversion currently accepts six-digit HEX input.
- Regex behavior follows Rust's `regex` crate and does not support look-around or backreferences.
- Clipboard availability depends on the host display environment.

## Project history

TIT.RUN preserves the original commits and authorship from [vdmo/tit](https://github.com/vdmo/tit), followed by the expanded responsive interface, reliability work, additional tools, tests, CI, and documentation maintained in this repository.

## License

TIT.RUN is a personal open-source project released under the [MIT License](LICENSE). You may use, modify, and distribute it under the license terms.
