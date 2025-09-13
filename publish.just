
bump:
    cargo workspaces version patch -y
patch:

  cargo publish -p sqlorm-core
  cargo publish -p sqlorm-macros
