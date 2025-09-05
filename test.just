
default:
  cargo test --workspace -- --nocapture

p:
  cargo nextest run --workspace --cargo-quiet --cargo-quiet

pp PACKAGE:
  cargo nextest run --cargo-quiet --cargo-quiet --package {{PACKAGE}}
