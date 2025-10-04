use crate::{
    check, ir, lexer,
    lower::{self, HItem},
    parser, resolve,
};

#[derive(Debug, Clone)]
pub struct PipelineSnapshot {
    module_path: Vec<String>,
    stages: Vec<PipelineStage>,
}

#[derive(Debug, Clone)]
pub struct PipelineStage {
    name: &'static str,
    status: StageStatus,
    metrics: Vec<StageMetric>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StageStatus {
    Success,
    Failed { message: String },
}

#[derive(Debug, Clone)]
pub struct StageMetric {
    key: &'static str,
    value: MetricValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    Integer(usize),
    Text(String),
    Bool(bool),
    List(Vec<String>),
}

impl PipelineSnapshot {
    pub fn capture(source: &str) -> Self {
        let mut stages = Vec::new();
        let mut module_path = Vec::new();

        let tokens = match lexer::lex(source) {
            Ok(tokens) => {
                stages.push(
                    PipelineStage::success("lexer")
                        .with_metric("tokens", MetricValue::Integer(tokens.len())),
                );
                tokens
            }
            Err(err) => {
                stages.push(PipelineStage::failed("lexer", err.to_string()));
                return PipelineSnapshot {
                    module_path,
                    stages,
                };
            }
        };

        let module = match parser::parse_module(source) {
            Ok(module) => {
                module_path = module.name.clone();
                stages.push(
                    PipelineStage::success("parser")
                        .with_metric("items", MetricValue::Integer(module.items.len()))
                        .with_metric("tokens", MetricValue::Integer(tokens.len())),
                );
                module
            }
            Err(err) => {
                stages.push(PipelineStage::failed("parser", err.to_string()));
                return PipelineSnapshot {
                    module_path,
                    stages,
                };
            }
        };

        let resolved = resolve::resolve_module(&module);
        let mut capability_names = resolved
            .capabilities
            .iter()
            .map(|cap| cap.name.clone())
            .collect::<Vec<_>>();
        capability_names.sort();
        stages.push(
            PipelineStage::success("resolve")
                .with_metric("imports", MetricValue::Integer(resolved.imports.len()))
                .with_metric("capabilities", MetricValue::List(capability_names))
                .with_metric(
                    "diagnostics",
                    MetricValue::Integer(resolved.diagnostics.len()),
                ),
        );

        let check_result = check::check_module(&module);
        stages.push(
            PipelineStage::success("check")
                .with_metric(
                    "diagnostics",
                    MetricValue::Integer(check_result.diagnostics.len()),
                )
                .with_metric("ok", MetricValue::Bool(check_result.diagnostics.is_empty())),
        );

        let lowered = lower::lower_module(&module);
        let function_count = lowered
            .items
            .iter()
            .filter(|item| matches!(item, HItem::Function(_)))
            .count();
        stages.push(
            PipelineStage::success("lower")
                .with_metric("functions", MetricValue::Integer(function_count))
                .with_metric("items", MetricValue::Integer(lowered.items.len())),
        );

        let ir_module = ir::lower_module(&lowered);
        let block_count: usize = ir_module
            .functions
            .iter()
            .map(|func| func.blocks.len())
            .sum();
        let effect_count = ir_module.effects.entries().count();
        stages.push(
            PipelineStage::success("ir")
                .with_metric("functions", MetricValue::Integer(ir_module.functions.len()))
                .with_metric("blocks", MetricValue::Integer(block_count))
                .with_metric("effects", MetricValue::Integer(effect_count)),
        );

        PipelineSnapshot {
            module_path,
            stages,
        }
    }

    pub fn module_path(&self) -> &[String] {
        &self.module_path
    }

    pub fn stages(&self) -> &[PipelineStage] {
        &self.stages
    }

    pub fn to_json_string(&self) -> String {
        let mut json = String::new();
        json.push('{');
        json.push_str("\"module_path\":[");
        for (index, segment) in self.module_path.iter().enumerate() {
            if index > 0 {
                json.push(',');
            }
            json.push('"');
            json.push_str(&escape_json_string(segment));
            json.push('"');
        }
        json.push(']');
        json.push(',');
        json.push_str("\"stages\":[");
        for (index, stage) in self.stages.iter().enumerate() {
            if index > 0 {
                json.push(',');
            }
            json.push_str(&stage.to_json());
        }
        json.push(']');
        json.push('}');
        json
    }
}

impl PipelineStage {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn status(&self) -> &StageStatus {
        &self.status
    }

    pub fn metrics(&self) -> &[StageMetric] {
        &self.metrics
    }

    fn success(name: &'static str) -> Self {
        PipelineStage {
            name,
            status: StageStatus::Success,
            metrics: Vec::new(),
        }
    }

    fn failed(name: &'static str, message: String) -> Self {
        PipelineStage {
            name,
            status: StageStatus::Failed { message },
            metrics: Vec::new(),
        }
    }

    fn with_metric(mut self, key: &'static str, value: MetricValue) -> Self {
        self.metrics.push(StageMetric { key, value });
        self
    }

    fn to_json(&self) -> String {
        let mut json = String::new();
        json.push('{');
        json.push_str("\"name\":\"");
        json.push_str(&escape_json_string(self.name));
        json.push_str("\",");
        match &self.status {
            StageStatus::Success => {
                json.push_str("\"status\":\"ok\"");
            }
            StageStatus::Failed { message } => {
                json.push_str("\"status\":\"error\",");
                json.push_str("\"message\":\"");
                json.push_str(&escape_json_string(message));
                json.push('"');
            }
        }
        json.push(',');
        json.push_str("\"metrics\":");
        json.push_str(&metrics_to_json(&self.metrics));
        json.push('}');
        json
    }
}

impl StageMetric {
    pub fn key(&self) -> &str {
        self.key
    }

    pub fn value(&self) -> &MetricValue {
        &self.value
    }
}

fn metrics_to_json(metrics: &[StageMetric]) -> String {
    if metrics.is_empty() {
        return "{}".to_string();
    }
    let mut entries = metrics
        .iter()
        .map(|metric| (metric.key, &metric.value))
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut json = String::from("{");
    for (index, (key, value)) in entries.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push('"');
        json.push_str(&escape_json_string(key));
        json.push('"');
        json.push(':');
        json.push_str(&metric_value_to_json(value));
    }
    json.push('}');
    json
}

fn metric_value_to_json(value: &MetricValue) -> String {
    match value {
        MetricValue::Integer(value) => value.to_string(),
        MetricValue::Text(text) => format!("\"{}\"", escape_json_string(text)),
        MetricValue::Bool(flag) => flag.to_string(),
        MetricValue::List(values) => {
            let mut entries = values.clone();
            entries.sort();
            let mut json = String::from("[");
            for (index, entry) in entries.iter().enumerate() {
                if index > 0 {
                    json.push(',');
                }
                json.push('"');
                json.push_str(&escape_json_string(entry));
                json.push('"');
            }
            json.push(']');
            json
        }
    }
}

fn escape_json_string(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0C}' => escaped.push_str("\\f"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if (ch as u32) < 0x20 => {
                use std::fmt::Write;
                write!(&mut escaped, "\\u{:04X}", ch as u32).expect("json escape write");
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}
