default: postgres sqlite

full: postgres sqlite 


postgres:
    cargo test --workspace --features postgres,uuid,extra-traits -- --nocapture

sqlite:
    cargo test --workspace --features sqlite,uuid,extra-traits -- --nocapture


