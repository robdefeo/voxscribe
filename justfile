default:
    @just --list

install:
    #!/usr/bin/env bash
    set -euo pipefail
    mise install

build:
    #!/usr/bin/env bash
    set -euo pipefail
    mise exec -- cargo build

release:
    #!/usr/bin/env bash
    set -euo pipefail
    mise exec -- cargo build --release

run *args:
    #!/usr/bin/env bash
    set -euo pipefail
    mise exec -- cargo run -- {{args}}

dev:
    #!/usr/bin/env bash
    set -euo pipefail
    mise exec -- cargo watch -x run

test:
    #!/usr/bin/env bash
    set -euo pipefail
    mise exec -- cargo test

lint:
    #!/usr/bin/env bash
    set -euo pipefail
    mise exec -- cargo fmt -- --config-path .config --check
    mise exec -- cargo clippy -- -D warnings

fmt:
    #!/usr/bin/env bash
    set -euo pipefail
    mise exec -- cargo fmt -- --config-path .config

coverage:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p coverage
    mise exec -- cargo llvm-cov --no-report
    mise exec -- cargo llvm-cov report --lcov --output-path coverage/lcov --ignore-filename-regex 'src/main\.rs'
    mise exec -- cargo llvm-cov report --html --output-dir coverage --ignore-filename-regex 'src/main\.rs'
    lines=$(lcov --summary coverage/lcov --ignore-errors inconsistent,corrupt 2>&1 | grep "lines" | grep -oE '[0-9]+\.[0-9]+' | head -1)
    awk -v pct="$lines" 'BEGIN { if (pct+0 < 80) { print "Coverage " pct "% is below 80% threshold"; exit 1 } }'

changelog:
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(cargo pkgid | sed 's/.*[#@]//')
    mise exec -- git-cliff --tag "v${version}" --output CHANGELOG.md

clean:
    #!/usr/bin/env bash
    set -euo pipefail
    mise exec -- cargo clean
    rm -rf coverage
