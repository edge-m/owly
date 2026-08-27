use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub path: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
}

impl Diagnostic {
    pub fn error(code: &str, path: Option<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            severity: Severity::Error,
            path,
            line: None,
            column: None,
            message: message.into(),
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self.severity, Severity::Error)
    }

    pub fn warning(code: &str, path: Option<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            severity: Severity::Warning,
            path,
            line: None,
            column: None,
            message: message.into(),
        }
    }
}
