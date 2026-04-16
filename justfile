default:
    @just --list

install:
    #!/usr/bin/env bash
    set -euo pipefail
    mise install

build:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build

release:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build --release

run *args:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo run -- {{args}}

dev:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo watch -x run

test:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo test

lint:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo fmt -- --config-path .config --check
    cargo clippy -- -D warnings

fmt:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo fmt -- --config-path .config

coverage:
    #!/usr/bin/env bash
    set -euo pipefail
    CARGO_INCREMENTAL=0 \
    RUSTFLAGS='-Cinstrument-coverage' \
    LLVM_PROFILE_FILE='coverage-%p-%m.profraw' \
    cargo test
    mkdir -p coverage
    grcov . --binary-path ./target/debug/ -s . -t html,lcov \
      --branch --ignore-not-existing \
      --excl-line '^\s*$|^\s*//|#\[derive\(|grcov-excl-line' \
      --excl-start 'grcov-excl-start' \
      --excl-stop 'grcov-excl-stop' \
      -o coverage
    lines=$(lcov --summary coverage/lcov 2>&1 | grep "lines" | grep -oE '[0-9]+\.[0-9]+' | head -1)
    awk -v pct="$lines" 'BEGIN { if (pct+0 < 60) { print "Coverage " pct "% is below 60% threshold"; exit 1 } }'

clean:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo clean
    rm -rf coverage
