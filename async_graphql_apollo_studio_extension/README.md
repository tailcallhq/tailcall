# async-graphql-extension-apollo-tracing

<div align="center">
  <!-- CI -->
  <img src="https://github.com/Miaxos/async_graphql_apollo_studio_extension/actions/workflows/ci.yml/badge.svg" />
  <!-- Crates version -->
  <a href="https://crates.io/crates/async-graphql-extension-apollo-tracing">
    <img src="https://img.shields.io/crates/v/async-graphql-extension-apollo-tracing.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Documentation -->
  <a href="https://docs.rs/async-graphql-extension-apollo-tracing/">
    <img src="https://docs.rs/async-graphql-extension-apollo-tracing/badge.svg?style=flat-square"
      alt="Documentation" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/async-graphql-extension-apollo-tracing">
    <img src="https://img.shields.io/crates/d/async-graphql-extension-apollo-tracing.svg?style=flat-square"
      alt="Download" />
  </a>
</div>
<br />
<br />

async-graphql-extension-apollo-tracing is an open-source extension for the crates [async_graphql](https://github.com/async-graphql/async-graphql). The purpose of this extension is to provide a simple way to create & send your graphql metrics to [Apollo Studio](https://studio.apollographql.com/).

- [Documentation](https://docs.rs/async-graphql-extension-apollo-tracing/)

_Tested at Rust version: `rustc 1.75.0`_

![Apollo Studio with async_graphql](apollo-studio.png?raw=true "Apollo Studio with async_graphql")

## Features

- Runtime agnostic (tokio / async-std)
- Fully support traces & errors
- Batched Protobuf transfer
- Client segmentation
- Additional data to segment your queries by visitors
- Tracing
- Schema export to studio
- Error traces
- Gzip compression

## Crate features

This crate offers the following features, all of which are not activated by default:

- `compression`: Enable the GZIP Compression when sending traces.
- `tokio-comp`: Enable the Tokio compatibility when you have a tokio-runtime

## Example

Check the example from `example` directory.

## References

- [GraphQL](https://graphql.org)
- [Async Graphql Crates](https://github.com/async-graphql/async-graphql)
- [Apollo Tracing](https://github.com/apollographql/apollo-tracing)
- [Apollo Server](https://github.com/apollographql/apollo-server)
