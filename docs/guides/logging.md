---
title: Logging
---

You can change the verbosity of the logs by using either `TAILCALL_LOG_LEVEL` or `TC_LOG_LEVEL` environment variables. The available levels are:

- off
- error
- warn
- info
- debug
- trace

We accept both uppercase or lowercase, so both `TAILCALL_LOG_LEVEL=DEBUG` or `TAILCALL_LOG_LEVEL=debug` will work.

The default log level is `info`.

Example of starting the server with `debug` log level:
`TAILCALL_LOG_LEVEL=debug tailcall start ./examples/jsonplaceholder.graphql`
