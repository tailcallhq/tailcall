---
title: Logging
---
Logging serves as a crucial tool for gaining insights into code execution and troubleshooting challenges in software development The verbosity of the logs can be configured using different log levels. The log level of the application can be configured using `TAILCALL_LOG_LEVEL` or `TC_LOG_LEVEL` environment variables. The available log levels are:

### error
The highest severity level. Indicates a critical issue that may lead to the failure of the program or a part of it.

```bash
TAILCALL_LOG_LEVEL=error tailcall <COMMAND>
# or 
TC_LOG_LEVEL=error tailcall <COMMAND>
```

### warn
Signifies potential issues or warnings that don't necessarily result in immediate failure but may require attention.

```bash
TAILCALL_LOG_LEVEL=warn tailcall <COMMAND>
# or 
TC_LOG_LEVEL=warn tailcall <COMMAND>
```

### info
Provides general information about the program's execution, giving insights into its state and activities.

```bash
TAILCALL_LOG_LEVEL=info tailcall <COMMAND>
# or 
TC_LOG_LEVEL=info tailcall <COMMAND>
```

### debug
Useful for developers during the debugging process. Outputs detailed information about the program's internal workings.

```bash
TAILCALL_LOG_LEVEL=debug tailcall <COMMAND>
# or 
TC_LOG_LEVEL=debug tailcall <COMMAND>
```

### trace
The most detailed logging level, used for fine-grained debugging. Provides exhaustive details about the program's execution flow.

```bash
TAILCALL_LOG_LEVEL=trace tailcall <COMMAND>
# or 
TC_LOG_LEVEL=trace tailcall <COMMAND>
```

### off
Special level indicating that no logs should be generated. It can be used to disable logging entirely.

```bash
TAILCALL_LOG_LEVEL=off tailcall <COMMAND>
# or 
TC_LOG_LEVEL=off tailcall <COMMAND>
```

Log levels are hierarchical, meaning if you set the log level to a specific level, it includes all the levels above it. For example, setting the log level to `info` will include logs at the `info`, `warn`, and `error` levels, but exclude `debug` and `trace` logs.

![Hierarchy of Log Levels](../../assets/logging.png)

:::info
Log levels are flexible and can be provided in either uppercase or lowercase. For instance, setting TAILCALL_LOG_LEVEL=DEBUG or TAILCALL_LOG_LEVEL=debug will yield the same result.
By default, the log level is configured to `info`.
:::

