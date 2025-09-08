default: postgres sqlite

postgres:
    cargo test --workspace --features postgres,uuid -- --nocapture

sqlite:
    cargo test --workspace --features sqlite,uuid -- --nocapture

