# tracing-gcp

If you're writing a webserver in rust and running it on GCP you'll likely want to use this crate.

At [lockbook](https://lockbook.net) our production server runs in GCP and we want to fully utilize cloud monitoring's [structured logging fields](https://cloud.google.com/logging/docs/structured-logging#structured_logging_special_fields).

Tracing gives us the ability to attach context to each log line using [Spans](https://docs.rs/tracing/latest/tracing/#spans). Gcp's [ops-agent](https://cloud.google.com/logging/docs/agent/ops-agent/configuration#logging-processor-parse-json) supports a JSON formatted log lines which allow us to engage with cloud logging in a more sophisticated manner.

This crate provides an opinionated subscriber which extracts metadata from fields in spans or events in the following manner:
- log entries are written, in json, to the specified file.
- only events emit log entries
- the following special fields are extracted from any parent spans or events

In the future we may provide a way to send data directly to cloud logging using a client API (reducing the need for the ops agent).
