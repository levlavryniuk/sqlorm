default: postgres sqlite

postgres:
    cargo test --workspace --features postgres -- --nocapture

sqlite:
    cargo test --workspace --features sqlite -- --nocapture

