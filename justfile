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

stress seed=`date +%s`:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo leptos build -p component-gallery
    mkdir -p target/stress
    server_log=target/stress/server.log
    LEPTOS_SITE_ROOT=target/site LEPTOS_SITE_PKG_DIR=pkg LEPTOS_OUTPUT_NAME=component-gallery \
        LEPTOS_SITE_ADDR=127.0.0.1:3099 LEPTOS_RELOAD_PORT=5099 \
        target/debug/component-gallery-server >"$server_log" 2>&1 &
    server=$!
    trap 'kill "$server" 2>/dev/null' EXIT
    until curl --silent --output /dev/null http://127.0.0.1:3099/; do sleep 1; done
    stress_result=0
    env --unset=LD_LIBRARY_PATH python3 stress/stress.py http://127.0.0.1:3099 "{{seed}}" || stress_result=$?
    if grep --quiet panicked "$server_log"; then
        grep --after-context=3 panicked "$server_log"
        echo "the server panicked under stress (seed {{seed}}), see $server_log"
        exit 1
    fi
    exit "$stress_result"
