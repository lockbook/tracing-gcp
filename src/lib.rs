mod google_cloud_logging;

use std::collections::HashMap;

use google_cloud_logging::{GCLogSeverity, GoogleCloudStructLog};
use tracing::{field::Visit, span::{Attributes, Id}, Level, Subscriber};
use tracing_subscriber::{
    fmt::{self, format::Writer, FormatEvent, FormatFields}, layer::Context, registry::LookupSpan, Layer
};

// pub fn layer<S>() -> Layer<S>
// {
//         fmt::layer()
//             .json()
//             .event_format(GcpEventFormatter::default()),
// }

#[derive(Default)]
pub struct GcpEventFormatter {}

impl Subscriber for GcpEventFormatter {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        todo!()
    }

    fn new_span(&self, span: &span::Attributes<'_>) -> span::Id {
        todo!()
    }

    fn record(&self, span: &span::Id, values: &span::Record<'_>) {
        todo!()
    }

    fn record_follows_from(&self, span: &span::Id, follows: &span::Id) {
        todo!()
    }

    fn event(&self, event: &tracing::Event<'_>) {
        todo!()
    }

    fn enter(&self, span: &span::Id) {
        todo!()
    }

    fn exit(&self, span: &span::Id) {
        todo!()
    }
}

impl<S> Layer<S> for GcpEventFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            let mut extensions = span.extensions_mut();
            let mut field_collector = FieldCollector::new();
            attrs.record(&mut field_collector);
            extensions.insert(field_collector.fields);
        }
    }
}

impl<S, N> FormatEvent<S, N> for GcpEventFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &fmt::FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let mut entry = GoogleCloudStructLog::default();

        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                let a = S::span_data(span.id());
            }
        }

        entry.severity = severity(event);
        event.record(&mut entry);

        let entry = serde_json::to_string(&entry).unwrap_or_else(fatal_log);
        writeln!(writer, "{entry}")?;
        Ok(())
    }
}

fn severity(e: &tracing::Event<'_>) -> Option<GCLogSeverity> {
    match *e.metadata().level() {
        Level::TRACE => None,
        Level::DEBUG => Some(GCLogSeverity::Debug),
        Level::INFO => Some(GCLogSeverity::Info),
        Level::WARN => Some(GCLogSeverity::Warning),
        Level::ERROR => Some(GCLogSeverity::Error),
    }
}

impl Visit for GoogleCloudStructLog<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
        } else {
            self.labels
                .insert(field.name().to_string(), format!("{:?}", value));
        }
    }
}

fn fatal_log(_e: serde_json::Error) -> String {
    "todo".to_string()
}
