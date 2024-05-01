# Sample gRPC Rust News Server

## Overview

This folder contains a gRPC-based Rust server implementing CRUD operations for a news list. It features a batched `GetNews` API, allowing efficient retrieval of multiple news items.

### Features

- **CRUD Operations**: Create, Read, Update, and Delete news items.
- **Batched News Retrieval**: Fetch multiple news items in a single request.
- **gRPC Interface**: Efficient and modern protocol for inter-service communication.

## Running the Server

Start the server with:

```bash
cargo run
```

## Reflection api

The server supports reflection api by default

### example

`grpcurl -plaintext localhost:50051 list`
