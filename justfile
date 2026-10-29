default: check

fmt:
    cargo fmt
    leptosfmt --quiet crates

check:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo fmt --check
    leptosfmt --check crates
    cargo clippy --workspace --all-targets -- --deny warnings
    cargo test --workspace
    cargo clippy -p component-gallery --no-default-features --features ssr \
        --all-targets -- --deny warnings
    cargo clippy -p component-gallery --lib --no-default-features --features hydrate \
        --target wasm32-unknown-unknown --target-dir target/front \
        -- --deny warnings
