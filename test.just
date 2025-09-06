
default: p s

postgres:
    cargo test --workspace --features postgres -- --nocapture

sqlite:
    cargo test --workspace --features sqlite -- --nocapture

all: postgres sqlite

p:
    cargo nextest run --workspace --features postgres --cargo-quiet --cargo-quiet

s:
    cargo nextest run --workspace --features sqlite --cargo-quiet --cargo-quiet

pp PACKAGE:
    cargo nextest run --cargo-quiet --cargo-quiet --package {{PACKAGE}}
