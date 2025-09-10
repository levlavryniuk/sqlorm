default: postgres sqlite

examples: postgres sqlite

postgres:
    cargo test --workspace --features postgres,uuid,extra-traits -- --nocapture

sqlite:
    cargo test --workspace --features sqlite,uuid,extra-traits -- --nocapture

