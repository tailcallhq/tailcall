# Convert Cargo.toml to YAML
yj -t < Cargo.toml > Cargo.yaml

# Modify Cargo.yaml
yq e 'del(.workspace.members[] | select(. == "cloudflare" or . == "autogen" or . == "testconv"))' -i Cargo.yaml

# Convert Cargo.yaml back to TOML
yj -yt < Cargo.yaml > Cargo.toml