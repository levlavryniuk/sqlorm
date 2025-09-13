
bump:
    cargo workspaces version patch -y
patch:
  cargo ws version patch --yes
  cargo publish -p sqlorm-core
  cargo publish -p sqlorm-macros
  cargo publish
