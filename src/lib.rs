use std::{collections::HashMap, io::Write, sync::Mutex};

use google_cloud_logging::{GCHttpMethod, GCHttpRequest, GCLogSeverity, GoogleCloudStructLog};
use tracing::{
    field::Visit,
    span::{self, Id},
    Level, Subscriber,
};
use tracing_subscriber::Layer;

pub struct GcpLayer<W: Write> {
    state: Mutex<State>,
    write: Mutex<W>,
}

impl<W: Write> GcpLayer<W> {
    pub fn init_with_writer(w: W) -> Self {
        Self {
            state: Default::default(),
            write: Mutex::new(w),
        }
    }
}

#[derive(Default)]
struct State {
    spans: HashMap<Id, IR>,
}

#[derive(Default)]
struct IR {
    pub severity: Option<GCLogSeverity>,
    pub method: Option<GCHttpMethod>,
    pub parent: Option<Id>,
    pub message: Option<String>,
    pub unknown_fields: Vec<(String, String)>,
}

impl IR {
    fn apply(&self, log_entry: &mut GoogleCloudStructLog) {
        if let Some(sev) = self.severity {
            log_entry.severity = Some(sev);
        }

        if let Some(meth) = self.method {
            match &mut log_entry.http_request {
                Some(req) => {
                    req.request_method = Some(meth);
                }
                None => {
                    let mut req = GCHttpRequest::default();
                    req.request_method = Some(meth);
                    log_entry.http_request = Some(req);
                }
            }
        }

        if let Some(ref message) = self.message {
            log_entry.message = Some(message.clone());
        }

        for (key, val) in &self.unknown_fields {
            log_entry.labels.insert(key.clone(), val.clone());
        }
    }
}

impl<S: Subscriber, W: Write + 'static> Layer<S> for GcpLayer<W> {
    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut state = self.state.lock().unwrap();

        let mut ir = IR::default();
        ir.parent = attrs.parent().cloned();
        attrs.values().record(&mut ir);

        state.spans.insert(id.clone(), ir);
    }

    fn on_record(
        &self,
        id: &Id,
        values: &span::Record<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut state = self.state.lock().unwrap();

        if let Some(ir) = state.spans.get_mut(id) {
            values.record(ir);
        }
    }

    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // this is the only fn that would benefit from RwLock
        let state = self.state.lock().unwrap();

        let mut ir = IR::default();
        ir.parent = event.parent().cloned();
        ir.severity = severity(event);
        event.record(&mut ir);

        let mut log_entry = GoogleCloudStructLog::default();
        ir.apply(&mut log_entry);

        // process parents
        let mut par_ir = &ir;
        loop {
            let Some(ref parent) = par_ir.parent else {
                break;
            };

            let Some(parent) = state.spans.get(parent) else {
                break;
            };

            parent.apply(&mut log_entry);
            par_ir = parent;
        }

        // log_entry ready to be used
        // may make sense to use a queue and do this somewhere else in the future
        let log_entry = serde_json::to_string(&log_entry).unwrap();
        writeln!(self.write.lock().unwrap(), "{log_entry}").unwrap();
    }

    fn on_close(&self, id: span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        self.state.lock().unwrap().spans.remove(&id);
    }
}

impl Visit for IR {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        match field.name() {
            "requestMethod" => {
                let method = format!("{:?}", value).to_lowercase();
                self.method = match method.as_ref() {
                    "\"post\"" => Some(GCHttpMethod::Post),
                    "\"get\"" => Some(GCHttpMethod::Get),
                    "\"put\"" => Some(GCHttpMethod::Put),
                    "\"head\"" => Some(GCHttpMethod::Head),
                    _ => None,
                }
            }
            "message" => {
                self.message = Some(format!("{:?}", value));
            }
            name => self
                .unknown_fields
                .push((name.to_string(), format!("{:?}", value))),
        }
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
