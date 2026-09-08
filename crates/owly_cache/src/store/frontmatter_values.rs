use owly_types::{FrontmatterValue, IndexedFrontmatterValue};
use rusqlite::{Connection, Result, Row, Transaction, params, params_from_iter, types::Value};

use crate::query::{Condition, Operator, SearchQuery};

pub fn delete_for_document(transaction: &Transaction<'_>, path: &str) -> Result<()> {
    transaction.execute("DELETE FROM frontmatter_values WHERE document_path = ?1", [path])?;
    Ok(())
}

pub fn insert(transaction: &Transaction<'_>, values: &[IndexedFrontmatterValue]) -> Result<()> {
    for value in values {
        transaction.execute(
            "INSERT INTO frontmatter_values(document_path, field_path, value_type, value_text, value_number,
             value_boolean, value_date, value_datetime, ordinal) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                value.document_path,
                value.field_path,
                sql_value_type(&value.value),
                sql_value_text(&value.value),
                sql_value_number(&value.value),
                sql_value_boolean(&value.value),
                sql_value_date(&value.value),
                sql_value_datetime(&value.value),
                value.ordinal
            ],
        )?;
    }
    Ok(())
}

pub fn for_document(connection: &Connection, path: &str) -> Result<Vec<IndexedFrontmatterValue>> {
    let mut statement = connection.prepare(
        "SELECT document_path, field_path, value_type, value_text, value_number, value_boolean,
         value_date, value_datetime, ordinal FROM frontmatter_values
         WHERE document_path = ?1 ORDER BY field_path, ordinal",
    )?;
    statement.query_map([path], from_row)?.collect()
}

pub fn search(connection: &Connection, query: &SearchQuery) -> Result<Vec<String>> {
    if query.is_empty() {
        return Err(rusqlite::Error::InvalidParameterName("search query must contain a condition".to_string()));
    }
    let mut sql = String::from("SELECT d.path FROM documents AS d WHERE ");
    let mut values = Vec::new();
    for (index, condition) in query.conditions.iter().enumerate() {
        if index > 0 {
            sql.push_str(" AND ");
        }
        let compiled = CompiledCondition::new(condition)?;
        sql.push_str(&format!("EXISTS (SELECT 1 FROM frontmatter_values AS v{index} WHERE v{index}.document_path = d.path AND v{index}.field_path = ? AND v{index}.value_type = ? AND v{index}.{column} {operator} {comparison})", column = compiled.column, operator = compiled.operator, comparison = compiled.comparison));
        values.push(Value::Text(condition.field_path.clone()));
        values.push(Value::Text(compiled.value_type.to_string()));
        if let Some(value) = compiled.value {
            values.push(value);
        }
    }
    sql.push_str(" ORDER BY d.path");
    let mut statement = connection.prepare(&sql)?;
    statement.query_map(params_from_iter(values.iter()), |row| row.get(0))?.collect()
}

struct CompiledCondition {
    value_type: &'static str,
    column: &'static str,
    operator: &'static str,
    comparison: &'static str,
    value: Option<Value>,
}

impl CompiledCondition {
    fn new(condition: &Condition) -> Result<Self> {
        let (value_type, column, value) = match &condition.value {
            FrontmatterValue::Text(value) => ("string", "value_text", Some(Value::Text(value.clone()))),
            FrontmatterValue::Number(value) => ("number", "value_number", Some(Value::Real(*value))),
            FrontmatterValue::Boolean(value) => ("boolean", "value_boolean", Some(Value::Integer(i64::from(*value)))),
            FrontmatterValue::Date(value) => ("date", "value_date", Some(Value::Text(value.clone()))),
            FrontmatterValue::Datetime(value) => ("datetime", "value_datetime", Some(Value::Text(value.clone()))),
            FrontmatterValue::Null => ("null", "value_text", None),
        };
        let (operator, comparison) = match (&condition.operator, &condition.value) {
            (Operator::Eq, FrontmatterValue::Null) => ("IS", "NULL"),
            (Operator::Ne, FrontmatterValue::Null) => ("IS NOT", "NULL"),
            (Operator::Eq, _) => ("=", "?"),
            (Operator::Ne, _) => ("<>", "?"),
            (Operator::Lt, _) => ("<", "?"),
            (Operator::Lte, _) => ("<=", "?"),
            (Operator::Gt, _) => (">", "?"),
            (Operator::Gte, _) => (">=", "?"),
        };
        if matches!(condition.value, FrontmatterValue::Null)
            && !matches!(condition.operator, Operator::Eq | Operator::Ne)
        {
            return Err(rusqlite::Error::InvalidParameterName("null only supports Eq or Ne".to_string()));
        }
        Ok(Self { value_type, column, operator, comparison, value })
    }
}

fn from_row(row: &Row<'_>) -> Result<IndexedFrontmatterValue> {
    let value_type: String = row.get(2)?;
    let value = match value_type.as_str() {
        "string" => FrontmatterValue::Text(row.get::<_, Option<String>>(3)?.unwrap_or_default()),
        "number" => FrontmatterValue::Number(row.get::<_, Option<f64>>(4)?.unwrap_or_default()),
        "boolean" => FrontmatterValue::Boolean(row.get::<_, Option<i64>>(5)?.unwrap_or_default() != 0),
        "date" => FrontmatterValue::Date(row.get::<_, Option<String>>(6)?.unwrap_or_default()),
        "datetime" => FrontmatterValue::Datetime(row.get::<_, Option<String>>(7)?.unwrap_or_default()),
        "null" => FrontmatterValue::Null,
        _ => {
            return Err(rusqlite::Error::InvalidParameterName(format!(
                "unknown frontmatter value type: {value_type}"
            )));
        }
    };
    Ok(IndexedFrontmatterValue { document_path: row.get(0)?, field_path: row.get(1)?, value, ordinal: row.get(8)? })
}

fn sql_value_type(value: &FrontmatterValue) -> &'static str {
    match value {
        FrontmatterValue::Text(_) => "string",
        FrontmatterValue::Number(_) => "number",
        FrontmatterValue::Boolean(_) => "boolean",
        FrontmatterValue::Date(_) => "date",
        FrontmatterValue::Datetime(_) => "datetime",
        FrontmatterValue::Null => "null",
    }
}
fn sql_value_text(value: &FrontmatterValue) -> Option<&str> {
    if let FrontmatterValue::Text(value) = value { Some(value) } else { None }
}
fn sql_value_number(value: &FrontmatterValue) -> Option<f64> {
    if let FrontmatterValue::Number(value) = value { Some(*value) } else { None }
}
fn sql_value_boolean(value: &FrontmatterValue) -> Option<i64> {
    if let FrontmatterValue::Boolean(value) = value { Some(i64::from(*value)) } else { None }
}
fn sql_value_date(value: &FrontmatterValue) -> Option<&str> {
    if let FrontmatterValue::Date(value) = value { Some(value) } else { None }
}
fn sql_value_datetime(value: &FrontmatterValue) -> Option<&str> {
    if let FrontmatterValue::Datetime(value) = value { Some(value) } else { None }
}
