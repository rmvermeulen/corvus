# https://just.systems

watch:
    # watch changes in src/
    cargo watch -w src -x r

fix:
    cargo clippy --fix --allow-staged
    cargo fmt

build:
    cargo b
