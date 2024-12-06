# tracing-gcp

If you're writing a webserver in rust and running it on GCP you'll likely want to use this crate.

At [lockbook](https://lockbook.net) our production server runs in GCP and we want to fully cloud monitoring's [structured logging fields](https://cloud.google.com/logging/docs/structured-logging#structured_logging_special_fields).

Tracing gives us the ability to attach context to each log line using [Spans](https://docs.rs/tracing/latest/tracing/#spans). Gcp's [ops-agent](https://cloud.google.com/logging/docs/agent/ops-agent/configuration#logging-processor-parse-json) supports a JSON formatted log lines which allow us to engage with cloud logging in a more sophisticated manner.

This crate provides a [FormatEvent impl](https://docs.rs/tracing-subscriber/0.3.18/tracing_subscriber/fmt/trait.FormatEvent.html) which formats events and their span's context into the format gcp expects. Fields are populated using the following strategies.

- severity is populated using tracing's severity
- 
