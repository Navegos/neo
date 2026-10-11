use std::collections::HashMap;
use std::io::{Cursor, Read};
use serde_json::{Map, Value};

pub type NamespaceMap = HashMap<String, [String; 2]>;

/// Rewrites the tools array for chat-only upstreams:
/// namespace tools become flat functions named "<namespace>__<name>" and
/// unsupported tool types are dropped. Reports whether a tools array existed.
pub fn flatten_responses_tools(raw: &[u8]) -> (Vec<u8>, NamespaceMap, bool) {
    let Ok(Value::Object(mut payload)) = serde_json::from_slice::<Value>(raw) else {
        return (Vec::new(), HashMap::new(), false);
    };
    let Some(Value::Array(tools)) = payload.get("tools").cloned() else {
        return (Vec::new(), HashMap::new(), false);
    };
    let mut ns_map: NamespaceMap = HashMap::new();
    let mut flat: Vec<Value> = Vec::with_capacity(tools.len());

    for entry in tools {
        let Value::Object(tool) = entry else {
            continue;
        };
        let tool_type = tool.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match tool_type {
            "function" => {
                flat.push(Value::Object(tool));
            }
            "namespace" => {
                let name = tool
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let Some(Value::Array(sub)) = tool.get("tools") else {
                    continue;
                };
                for s in sub {
                    let Value::Object(st) = s else {
                        continue;
                    };
                    if st.get("type").and_then(|v| v.as_str()) != Some("function") {
                        continue;
                    }
                    let orig = st
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if name.is_empty() || orig.is_empty() {
                        continue;
                    }
                    let mut clone = st.clone();
                    let flat_name = format!("{}__{}", name, orig);
                    clone.insert("name".to_string(), Value::String(flat_name.clone()));
                    ns_map.insert(flat_name, [name.clone(), orig]);
                    flat.push(Value::Object(clone));
                }
            }
            _ => {}
        }
    }

    payload.insert("tools".to_string(), Value::Array(flat));
    match serde_json::to_vec(&Value::Object(payload)) {
        Ok(out) => (out, ns_map, true),
        Err(_) => (Vec::new(), HashMap::new(), false),
    }
}

/// Repairs sparse Responses API SSE streams while they stream:
/// item/content part events get injected around text deltas, namespace fields
/// are restored on flattened function calls, and the stream ends with a
/// completed output item.
pub struct SseFixupBody<R> {
    inner: R,
    pub ns_map: NamespaceMap,
    out: Vec<u8>,
    out_pos: usize,
    acc: Vec<u8>,
    event: Vec<String>,
    eof: bool,
    read_err: Option<std::io::Error>,
    saw_item: bool,
    saw_item_done: bool,
    injected: bool,
    acc_text: String,
    item_id: String,
}

impl<R> SseFixupBody<R> {
    pub fn new(inner: R, ns_map: NamespaceMap) -> Self {
        Self {
            inner,
            ns_map,
            out: Vec::new(),
            out_pos: 0,
            acc: Vec::new(),
            event: Vec::new(),
            eof: false,
            read_err: None,
            saw_item: false,
            saw_item_done: false,
            injected: false,
            acc_text: String::new(),
            item_id: String::new(),
        }
    }

    fn buffered_len(&self) -> usize {
        self.out.len().saturating_sub(self.out_pos)
    }

    pub fn feed(&mut self, chunk: &[u8]) {
        self.acc.extend_from_slice(chunk);
        while let Some(i) = self.acc.iter().position(|&b| b == b'\n') {
            let mut line_bytes = self.acc[..i].to_vec();
            while line_bytes.last() == Some(&b'\r') {
                line_bytes.pop();
            }
            let line = String::from_utf8_lossy(&line_bytes).to_string();
            self.acc.drain(..=i);
            self.feed_line(&line);
        }
    }

    pub fn feed_line(&mut self, line: &str) {
        if line.trim().is_empty() {
            if !self.event.is_empty() {
                let ev = std::mem::take(&mut self.event);
                self.transform(&ev);
            }
            return;
        }
        self.event.push(line.to_string());
    }

    pub fn transform(&mut self, ev: &[String]) {
        let mut data_idx: Option<usize> = None;
        let mut payload = "";
        for (i, ln) in ev.iter().enumerate() {
            if let Some(rest) = ln.strip_prefix("data:") {
                data_idx = Some(i);
                payload = rest.trim();
                break;
            }
        }
        let Some(idx) = data_idx else {
            self.emit(ev);
            return;
        };
        let Ok(Value::Object(mut data)) = serde_json::from_str::<Value>(payload) else {
            self.emit(ev);
            return;
        };

        let typ = string_field(&data, "type");
        match typ.as_str() {
            "response.output_text.delta" => {
                if !self.saw_item && !self.injected {
                    self.injected = true;
                    self.item_id = string_field(&data, "item_id");
                    if self.item_id.is_empty() {
                        self.item_id = "msg_compat".to_string();
                    }
                    let out_idx = int_field(&data, "output_index");
                    let item_id = self.item_id.clone();
                    self.emit_item_added(out_idx, &item_id);
                    self.emit_content_part_added(out_idx, &item_id);
                }
                if let Some(d) = data.get("delta").and_then(|v| v.as_str()) {
                    self.acc_text.push_str(d);
                }
                self.emit_data(ev, idx, &data);
            }
            "response.output_item.added"
            | "response.output_item.done"
            | "response.content_part.added" => {
                self.saw_item = true;
                if typ == "response.output_item.done" {
                    self.saw_item_done = true;
                }
                if let Some(Value::Object(mut item)) = data.get("item").cloned() {
                    if let Some(iid) = item.get("id").and_then(|v| v.as_str()) {
                        if !iid.is_empty() && self.item_id.is_empty() {
                            self.item_id = iid.to_string();
                        }
                    }
                    self.restore_function_call(&mut item);
                    data.insert("item".to_string(), Value::Object(item));
                    self.emit_data(ev, idx, &data);
                } else {
                    if let Some(iid) = data.get("item_id").and_then(|v| v.as_str()) {
                        if !iid.is_empty() && self.item_id.is_empty() {
                            self.item_id = iid.to_string();
                        }
                    }
                    self.emit(ev);
                }
            }
            "response.completed" => {
                let mut output_len = 0usize;
                let mut has_resp = false;
                if let Some(Value::Object(mut resp)) = data.get("response").cloned() {
                    has_resp = true;
                    if let Some(Value::Array(mut output)) = resp.get("output").cloned() {
                        output_len = output.len();
                        for it in &mut output {
                            if let Value::Object(m) = it {
                                self.restore_function_call(m);
                            }
                        }
                        resp.insert("output".to_string(), Value::Array(output));
                    }
                    data.insert("response".to_string(), Value::Object(resp));
                }
                if !self.saw_item_done && (!self.acc_text.is_empty() || output_len == 0) {
                    let text = self.acc_text.clone();
                    self.emit_item_done(&text);
                }
                if has_resp && output_len == 0 {
                    let msg_item = self.message_item("completed", &self.acc_text);
                    if let Some(Value::Object(resp)) = data.get_mut("response") {
                        resp.insert("output".to_string(), Value::Array(vec![msg_item]));
                    }
                }
                self.emit_data(ev, idx, &data);
            }
            _ => {
                self.emit(ev);
            }
        }
    }

    pub fn restore_function_call(&self, item: &mut Map<String, Value>) {
        if item.get("type").and_then(|v| v.as_str()) != Some("function_call") {
            return;
        }
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if let Some(pair) = self.ns_map.get(&name) {
            item.insert("namespace".to_string(), Value::String(pair[0].clone()));
            item.insert("name".to_string(), Value::String(pair[1].clone()));
        }
    }

    pub fn message_item(&self, status: &str, text: &str) -> Value {
        let id = if self.item_id.is_empty() {
            "msg_compat"
        } else {
            &self.item_id
        };
        let mut content_part = Map::new();
        content_part.insert("annotations".to_string(), Value::Array(Vec::new()));
        content_part.insert("text".to_string(), Value::String(text.to_string()));
        content_part.insert("type".to_string(), Value::String("output_text".to_string()));

        let mut item = Map::new();
        item.insert(
            "content".to_string(),
            Value::Array(vec![Value::Object(content_part)]),
        );
        item.insert("id".to_string(), Value::String(id.to_string()));
        item.insert("role".to_string(), Value::String("assistant".to_string()));
        item.insert("status".to_string(), Value::String(status.to_string()));
        item.insert("type".to_string(), Value::String("message".to_string()));
        Value::Object(item)
    }

    pub fn emit_item_added(&mut self, idx: i64, id: &str) {
        let mut item = Map::new();
        item.insert("content".to_string(), Value::Array(Vec::new()));
        item.insert("id".to_string(), Value::String(id.to_string()));
        item.insert("role".to_string(), Value::String("assistant".to_string()));
        item.insert(
            "status".to_string(),
            Value::String("in_progress".to_string()),
        );
        item.insert("type".to_string(), Value::String("message".to_string()));

        let mut ev = Map::new();
        ev.insert("item".to_string(), Value::Object(item));
        ev.insert("output_index".to_string(), Value::from(idx));
        ev.insert(
            "type".to_string(),
            Value::String("response.output_item.added".to_string()),
        );
        self.emit_typed(&Value::Object(ev));
    }

    pub fn emit_content_part_added(&mut self, idx: i64, id: &str) {
        let mut part = Map::new();
        part.insert("annotations".to_string(), Value::Array(Vec::new()));
        part.insert("text".to_string(), Value::String(String::new()));
        part.insert("type".to_string(), Value::String("output_text".to_string()));

        let mut ev = Map::new();
        ev.insert("content_index".to_string(), Value::from(0));
        ev.insert("item_id".to_string(), Value::String(id.to_string()));
        ev.insert("output_index".to_string(), Value::from(idx));
        ev.insert("part".to_string(), Value::Object(part));
        ev.insert(
            "type".to_string(),
            Value::String("response.content_part.added".to_string()),
        );
        self.emit_typed(&Value::Object(ev));
    }

    pub fn emit_item_done(&mut self, text: &str) {
        let msg = self.message_item("completed", text);
        let mut ev = Map::new();
        ev.insert("item".to_string(), msg);
        ev.insert("output_index".to_string(), Value::from(0));
        ev.insert(
            "type".to_string(),
            Value::String("response.output_item.done".to_string()),
        );
        self.emit_typed(&Value::Object(ev));
    }

    pub fn emit_data(&mut self, ev: &[String], idx: usize, data: &Map<String, Value>) {
        let Ok(payload) = serde_json::to_string(&Value::Object(data.clone())) else {
            self.emit(ev);
            return;
        };
        let mut cp = ev.to_vec();
        cp[idx] = format!("data: {}", payload);
        self.emit(&cp);
    }

    pub fn emit(&mut self, lines: &[String]) {
        for ln in lines {
            self.out.extend_from_slice(ln.as_bytes());
            self.out.push(b'\n');
        }
        self.out.push(b'\n');
    }

    pub fn emit_typed(&mut self, data: &Value) {
        let Ok(payload) = serde_json::to_string(data) else {
            return;
        };
        let typ = data.get("type").and_then(|v| v.as_str()).unwrap_or("");
        self.emit(&[format!("event: {}", typ), format!("data: {}", payload)]);
    }
}

impl<R: Read> Read for SseFixupBody<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        while self.buffered_len() == 0 {
            if self.eof || self.read_err.is_some() {
                if !self.event.is_empty() {
                    let ev = std::mem::take(&mut self.event);
                    self.transform(&ev);
                }
                if self.buffered_len() == 0 {
                    if let Some(err) = self.read_err.take() {
                        return Err(err);
                    }
                    return Ok(0);
                }
                break;
            }
            let mut chunk = [0u8; 32 * 1024];
            match self.inner.read(&mut chunk) {
                Ok(0) => {
                    if !self.acc.is_empty() {
                        let rest = String::from_utf8_lossy(&self.acc).to_string();
                        self.acc.clear();
                        self.feed_line(&rest);
                    }
                    self.eof = true;
                }
                Ok(n) => {
                    self.feed(&chunk[..n]);
                }
                Err(err) => {
                    if !self.acc.is_empty() {
                        let rest = String::from_utf8_lossy(&self.acc).to_string();
                        self.acc.clear();
                        self.feed_line(&rest);
                    }
                    self.read_err = Some(err);
                }
            }
        }
        let avail = &self.out[self.out_pos..];
        let to_copy = std::cmp::min(buf.len(), avail.len());
        buf[..to_copy].copy_from_slice(&avail[..to_copy]);
        self.out_pos += to_copy;
        if self.out_pos >= self.out.len() {
            self.out.clear();
            self.out_pos = 0;
        }
        Ok(to_copy)
    }
}

pub fn transform_sse_bytes(raw: &[u8], ns_map: NamespaceMap) -> Vec<u8> {
    let mut fixup = SseFixupBody::new(Cursor::new(raw), ns_map);
    let mut out = Vec::new();
    let _ = fixup.read_to_end(&mut out);
    out
}

fn string_field(data: &Map<String, Value>, key: &str) -> String {
    data.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn int_field(data: &Map<String, Value>, key: &str) -> i64 {
    if let Some(i) = data.get(key).and_then(|v| v.as_i64()) {
        return i;
    }
    if let Some(f) = data.get(key).and_then(|v| v.as_f64()) {
        return f as i64;
    }
    0
}
