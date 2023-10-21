# Logging Specification Design and Architecture

## Description

Logging is an essential aspect of system maintenance and debugging. The existing logging infrastructure requires an upgrade to provide a richer and more configurable experience. This document outlines the design and architecture of a comprehensive logging system that will follow industry standards and conventions, be configurable and flexible, work with spans, and provide time duration for upstream requests. Inspiration for this design is taken from popular proxy servers such as Nginx, HaProxy, and Envoy.

## Objectives

1. **Configurability:** The logging system should be highly configurable. Users should be able to define what information is logged, the level of granularity, and the format in which it is logged.
2. **Flexibility:** The system should support logging to multiple outputs such as console, files, and external systems.
3. **Span Support:** The system should be able to work with spans to provide distributed tracing.
4. **Upstream Request Timing:** The system should be able to record and report the time duration of upstream requests.

## Design

### Components

#### Logger

The primary component of the logging system. It will be responsible for creating and formatting log messages based on configuration settings.

#### Appender

Appenders are responsible for writing log messages to their designated destinations, such as files, databases, or external systems.

#### Filter

Filters will be used to determine whether a log event should be passed to a particular appender based on rules defined in the configuration.

### Configuration

The logging system should be configurable using a configuration file. This file will define the loggers, appenders, and filters, as well as their properties.

Example configuration:

```yaml
loggers:
  - name: "root"
    level: "INFO"
    appenders:
      - "console"
      - "file"

appenders:
  - type: "console"
    layout: "simple"

  - type: "file"
    file: "logs/app.log"
    layout: "json"

filters:
  - type: "threshold"
    level: "ERROR"
```

## Log Events
Log events will include the following information:

+ Timestamp
+ Logger name
+ Log level (e.g., DEBUG, INFO, WARN, ERROR)
+ Message
+ Additional context (optional)
+ Span information (optional)
+ Upstream request duration (optional)


## Levels of Granularity
The system should support different levels of granularity for tracing. This could be configured as a separate setting in the configuration file. The available levels could be:

+ NONE: No tracing information is logged.
+ BASIC: Basic tracing information is logged, such as entry and exit points of methods.
+ DETAILED: Detailed tracing information is logged, including method arguments and return values.

## Formats
The system should support different log message formats, such as:

+ Simple: Human-readable format with basic information.
+ JSON: Structured format that can be easily parsed by other tools.
+ Custom: User-defined format.

## Architecture
The logging system should follow a modular architecture, with different components responsible for different aspects of the logging process.

1. **Logger:** The logger component will be responsible for creating log events based on the log message and level. It will pass the log event to the appropriate appenders based on the configuration.

2. **Appender:** The appender component will be responsible for writing the log event to its designated destination. There could be multiple appenders configured, each with its own properties and destination.

3. **Filter:** The filter component will be responsible for filtering log events based on rules defined in the configuration. It will determine whether a log event should be passed to a particular appender.

4. **Formatter:** The formatter component will be responsible for formatting the log event based on the layout specified in the configuration. It could be a simple layout, JSON layout, or a custom layout defined by the user.


## Conclusion
The proposed logging system design and architecture aim to provide a comprehensive, configurable, and flexible logging experience. By following industry standards and conventions, the system will be compatible with various tools and systems. The modular architecture will make it easy to extend and customize the logging system to meet specific requirements.