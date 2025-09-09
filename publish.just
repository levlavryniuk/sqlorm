patch:
  cargo bump patch && git add Cargo.toml && git commit -m "bump version" && git push && cargo publish
