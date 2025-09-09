patch: bump core macros sqlorm

bump:
    cargo workspaces version patch -y && git add . && git commit -m "bump version"

macros:
    cargo publish --manifest-path ./sqlorm-macros/Cargo.toml

core:
    cargo publish --manifest-path ./sqlorm-core/Cargo.toml

sqlorm:
    cargo publish
