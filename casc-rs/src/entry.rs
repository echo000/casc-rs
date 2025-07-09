use crate::span_info::SpanInfo;

#[derive(Debug)]
pub struct Entry {
    pub name: String,
    pub spans: Vec<SpanInfo>,
}

impl Entry {
    pub fn new_with_spans(name: String, spans: Vec<SpanInfo>) -> Self {
        Self { name, spans }
    }
}
