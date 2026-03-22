# proxynexus-rs

Reinvention of https://proxynexus.net/ as a CLI, desktop and web app.
Focuses on collection management and local processing.

## Prerequisites

*I'm still working out these details...*

- [Rust](https://rustup.rs/)
- [dioxus](https://github.com/DioxusLabs/dioxus/releases/) (required for building the gui)
- [nasm](https://www.nasm.us/pub/nasm/releasebuilds/3.01/) (only required for desktop builds)
- [cmake](https://cmake.org/download/)

If you're trying to build the cli, webapp or desktop app on any platform, and you're having trouble, please reach out 
or create a new issue. I would love to iron our the details.

## Build CLI

```bash
cargo build --bin proxynexus-cli --release
```

## Run CLI

```bash
cargo run --bin proxynexus-cli -- --help
# Or run the binary directly:
./target/release/proxynexus-cli --help
```

## Essential Usage

```bash
# Build a collection from a folder of card images
# Images must be named by NetrunnerDB card code (e.g., 01050.jpg or 01050.png)
# Variants use an underscore (e.g., 01050_alt1.jpg, 01050_promo.png)
./target/release/proxynexus-cli collection build --images ./core_set_scans --output core_set.pnx

# Add the built collection to your local library
./target/release/proxynexus-cli collection add core_set.pnx

# Query command showing all known sets and available printings for each set across all collections
./target/release/proxynexus-cli query --list-sets

# Query a specific set to check what printings are available for each card
# The output format is directly compatible with the --cardlist option in generate commands.
./target/release/proxynexus-cli query --set-name "Core Set"

# Generate a printable PDF from a specific set or cardlist
./target/release/proxynexus-cli generate pdf --set-name "Core Set" --output-path core-set.pdf
```

## GUI is in progress