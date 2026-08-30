use owly_types::FrontmatterValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Eq,
    Ne,
    Lt,
    Lte,
    Gt,
    Gte,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub field_path: String,
    pub operator: Operator,
    pub value: FrontmatterValue,
}

impl Condition {
    pub fn new(field_path: impl Into<String>, operator: Operator, value: FrontmatterValue) -> Self {
        Self { field_path: field_path.into(), operator, value }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SearchQuery {
    pub conditions: Vec<Condition>,
}

impl SearchQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn and(mut self, field_path: impl Into<String>, operator: Operator, value: FrontmatterValue) -> Self {
        self.conditions.push(Condition::new(field_path, operator, value));
        self
    }

    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }
}
