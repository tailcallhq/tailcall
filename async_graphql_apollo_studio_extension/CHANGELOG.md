# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [3.2.11](https://github.com/async-graphql/async_graphql_apollo_studio_extension/compare/v3.2.10...v3.2.11) - 2024-02-05

### Other

- _(deps)_ bump tokio from 1.35.1 to 1.36.0

## [3.2.10](https://github.com/async-graphql/async_graphql_apollo_studio_extension/compare/v3.2.9...v3.2.10) - 2024-02-01

### Other

- _(deps)_ bump reqwest from 0.11.23 to 0.11.24

## [3.2.9](https://github.com/async-graphql/async_graphql_apollo_studio_extension/compare/v3.2.8...v3.2.9) - 2024-01-30

### Other

- _(deps)_ bump serde_json from 1.0.112 to 1.0.113

## [3.2.8](https://github.com/async-graphql/async_graphql_apollo_studio_extension/compare/v3.2.7...v3.2.8) - 2024-01-29

### Other

- _(deps)_ bump serde_json from 1.0.111 to 1.0.112
- _(deps)_ bump serde from 1.0.195 to 1.0.196

## [3.2.7](https://github.com/async-graphql/async_graphql_apollo_studio_extension/compare/v3.2.6...v3.2.7) - 2024-01-26

### Other

- _(deps)_ bump chrono from 0.4.32 to 0.4.33

## [3.2.6](https://github.com/async-graphql/async_graphql_apollo_studio_extension/compare/v3.2.5...v3.2.6) - 2024-01-23

### Other

- _(deps)_ bump chrono from 0.4.31 to 0.4.32
- _(deps)_ bump derive_builder from 0.12.0 to 0.13.0

## [3.2.5](https://github.com/async-graphql/async_graphql_apollo_studio_extension/compare/v3.2.4...v3.2.5) - 2024-01-22

### Other

- _(deps)_ bump async-graphql from 7.0.0 to 7.0.1

## [3.2.4](https://github.com/async-graphql/async_graphql_apollo_studio_extension/compare/v3.2.3...v3.2.4) - 2024-01-19

### Other

- _(deps)_ bump h2 from 0.3.22 to 0.3.24
- _(deps)_ bump h2 from 0.3.22 to 0.3.24 in /example/simple

## [3.2.3](https://github.com/async-graphql/async_graphql_apollo_studio_extension/compare/v3.2.2...v3.2.3) - 2024-01-19

### Other

- _(deps)_ bump uuid from 1.6.1 to 1.7.0

## [3.2.2](https://github.com/async-graphql/async_graphql_apollo_studio_extension/compare/v3.2.1...v3.2.2) - 2024-01-08

### Other

- _(deps)_ bump serde from 1.0.194 to 1.0.195
- _(deps)_ bump async-graphql from 6.0.11 to 7.0.0

## [3.2.1](https://github.com/async-graphql/async_graphql_apollo_studio_extension/compare/v3.2.0...v3.2.1) - 2024-01-05

### Other

- _(deps)_ bump async-trait from 0.1.76 to 0.1.77
- _(deps)_ bump anyhow from 1.0.78 to 1.0.79
- _(deps)_ bump serde_json from 1.0.109 to 1.0.111
- _(deps)_ bump serde_json from 1.0.108 to 1.0.109
- _(deps)_ bump async-trait from 0.1.75 to 0.1.76
- _(deps)_ bump anyhow from 1.0.77 to 1.0.78

## [3.2.0](https://github.com/async-graphql/async_graphql_apollo_studio_extension/compare/v3.1.0...v3.2.0) - 2023-12-28

### Added

- update Cargo.lock

## [3.1.0](https://github.com/async-graphql/async_graphql_apollo_studio_extension/compare/v3.0.1...v3.1.0) - 2023-12-28

### Added

- update to latest async-graphql version

### Fixed

- install protoc on github action

### Other

- fix doc
- remove useless targets
- allow doc build

## [3.0.1]

## [3.0.0]

### Feat

- Bump `async-graphql` to `4`

## [2.1.0]

### Feat

- Bump `async-graphql` to `3.0.7`
- Edition 2021

## [1.0.4]

### Fix

- Bump `async-graphql` to `2.10.*` to fix dependency resolution conflict while using latest async-graphql.

### Misc

- Add audit github action
- Issue with workflow token

## [1.0.3]

### Misc

- Commit Cargo.lock ... (idk why it was in the .gitignore)

## [1.0.2]

### Fix

- Do not send Traces over than 4mb

## [1.0.1]

### Fix

- Fix CI publish command, we can't build it with every features, we have to choose the runtime.

## [1.0.0]

### Feat

- Add async-std-comp & tokio-comp features
- Runtime agnostic

### Fix

- Fix auto-release workflow

## [0.4.2]

### Fix

- Fix auto-release

## [0.4.1]

### Misc

- Update documentation

## [0.4.0]

### Feat

- GZIP Compression

## [0.3.6]

### Misc

- Update workflows
- Update Cargo.toml Repository
- Clippy fix

## [0.3.5]

### Fix

- Fix tracing timings

## [0.3.4]

### Fix

- Use offset for start time

## [0.3.3]

### Misc

- Fix some README.md

## [0.3.2]

### Fix

- Do not show authorization_token

## [0.3.1]

### Fix

- Fix register schema

## [0.3.0]

### Feat

- Add schema export to apollo

## [0.2.4]

### Fix

- Make ApolloTracingDataExt fields public for hydratation

## [0.2.3]

### Fix

- On merge fix token github

## [0.2.2]

### Fix

- Merge deployment

## [0.2.1]

### Feat

- Opensource Brevz Apollo Extension
- Add on-merge workflows
- Fix on-merge workflows

## [0.1.0]

### Added

- Boot repo

[Unreleased]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v3.0.1...HEAD
[3.0.1]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v3.0.0...v3.0.1
[3.0.0]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v2.1.0...v3.0.0
[2.1.0]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v1.0.4...v2.1.0
[1.0.4]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v1.0.2...v1.0.3
[1.0.2]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v0.4.2...v1.0.0
[0.4.2]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v0.3.6...v0.4.0
[0.3.6]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v0.3.5...v0.3.6
[0.3.5]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v0.3.4...v0.3.5
[0.3.4]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v0.2.4...v0.3.0
[0.2.4]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/Miaxos/async_graphql_apollo_studio_extension/compare/v0.1.0...v0.2.1
