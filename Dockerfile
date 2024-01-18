# Builder stage
FROM rust:slim-buster AS builder

WORKDIR /prod
# Copy manifests and the graphql file
COPY Cargo.lock Cargo.toml examples/jsonplaceholder.graphql ./
COPY autogen ./autogen
RUN sed -i 's/"cloudflare",\s*//;s/, "cloudflare"//g' Cargo.toml
# This is the trick to speed up the building process.
RUN mkdir .cargo \
    && cargo vendor > .cargo/config

# Install required system dependencies and cleanup in the same layer
RUN apt-get update && apt-get install -y pkg-config libssl-dev python g++ git make && apt-get clean && rm -rf /var/lib/apt/lists/*

# Copy the rest of the source code
COPY . .
RUN sed -i 's/"cloudflare",\s*//;s/, "cloudflare"//g' Cargo.toml

# Compile the project
RUN RUST_BACKTRACE=1 cargo build --release

# Runner stage
FROM fedora:34 AS runner

# Copy necessary files from the builder stage
COPY --from=builder /prod/target/release/tailcall /bin
COPY --from=builder /prod/jsonplaceholder.graphql /jsonplaceholder.graphql

ENV TAILCALL_LOG_LEVEL=error
CMD ["/bin/tailcall", "start", "jsonplaceholder.graphql"]

