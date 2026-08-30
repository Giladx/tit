# TIT.RUN v0.2.0

The v0.2.0 release of TIT.RUN brings every TUI tool to the headless CLI, adds tests across the entire tool suite, extends core capabilities, and includes shell completion support.

## Highlights

- All 20 tools now have headless CLI subcommands
- Most text-processing commands read from stdin when no argument is given
- Full unit-test coverage for every tool module
- Cron parser previews the next 5 UTC run times
- Color converter accepts HEX (3/6 digit), RGB, and HSL inputs
- Number-base converter supports unsigned 128-bit values
- DateTime converter supports IANA timezone formatting
- JWT parser supports optional HMAC signature verification (HS256/HS384/HS512)
- Shell completion scripts via `tit completions <bash|zsh|fish|powershell>`

## Included tools

Date/time, Base64, URL encoding, HTML entities, number bases, colors, hashes, text cases, text statistics, Lorem Ipsum, JSON formatting, JSON/YAML, JWT parsing, regular expressions, cron expressions, URL parsing, IPv4 subnets, UUIDs, passwords, and MAC addresses.

## Verification

Each archive has a corresponding entry in `SHA256SUMS.txt`. Download the file for your operating system and architecture, extract it, and run `tit` (`tit.exe` on Windows).
