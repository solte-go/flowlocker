use serde::Serialize;
use serde_json::{json, Map, Number, Value};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Argument {
    StringArg(String),
    BoolArg(bool),
    IntArg(i32),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Comparison {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

#[derive(Debug)]
pub struct Condition {
    field: String,
    value: Value,
    comparison: Comparison,
}

impl From<String> for Argument {
    fn from(s: String) -> Self {
        Argument::StringArg(s)
    }
}

impl From<&str> for Argument {
    fn from(s: &str) -> Self {
        Argument::StringArg(s.to_string())
    }
}

impl From<bool> for Argument {
    fn from(b: bool) -> Self {
        Argument::BoolArg(b)
    }
}

impl From<i32> for Argument {
    fn from(i: i32) -> Self {
        Argument::IntArg(i)
    }
}

impl From<Argument> for String {
    fn from(arg: Argument) -> Self {
        match arg {
            Argument::StringArg(s) => s,
            _ => panic!("Cannot convert Argument to String"),
        }
    }
}

impl From<Argument> for bool {
    fn from(arg: Argument) -> Self {
        match arg {
            Argument::BoolArg(b) => b,
            _ => panic!("Cannot convert Argument to bool"),
        }
    }
}

impl From<Argument> for i32 {
    fn from(arg: Argument) -> Self {
        match arg {
            Argument::IntArg(i) => i,
            _ => panic!("Cannot convert Argument to i32"),
        }
    }
}

// impl&'a From<Argument> for <'a>Value {
impl<'a> Into<Value> for &'a Argument {
    fn into(self) -> Value {
        match self {
            Argument::StringArg(s) => Value::String(s.clone()),
            Argument::BoolArg(b) => Value::Bool(*b),
            Argument::IntArg(i) => Value::Number(Number::from(*i)),
        }
    }
}

impl From<Argument> for Value {
    fn from(arg: Argument) -> Self {
        match arg {
            Argument::StringArg(s) => Value::String(s),
            Argument::BoolArg(b) => Value::Bool(b),
            Argument::IntArg(i) => Value::Number(i.into()),
        }
    }
}

#[derive(Debug)]
pub struct SurellDBQueryBuilder {
    // operation: String,
    select_fields: Option<Vec<String>>,
    where_conditions: Option<Vec<Condition>>,
}

impl SurellDBQueryBuilder {
    pub fn new() -> Self {
        SurellDBQueryBuilder {
            select_fields: None,
            where_conditions: None,
        }
    }

    pub fn select(&mut self, fields: Vec<&str>) -> &mut Self {
        self.select_fields = Some(fields.iter().map(|f| f.to_string()).collect());
        self
    }

    pub fn where_condition(&mut self, field: &str, value: Value, comparison: Comparison) -> &mut Self {
        let condition = Condition {
            field: field.to_string(),
            value,
            comparison,
        };

        if self.where_conditions.is_none() {
            self.where_conditions = Some(Vec::new());
        }

        self.where_conditions.as_mut().unwrap().push(condition);
        self
    }

    pub fn build(&self) -> String {
        let mut query = json!({
            // "op": self.operation
        });

        if let Some(select_fields) = &self.select_fields {
            query["select"] = Value::Array(
                select_fields
                    .iter()
                    .map(|f| Value::String(f.to_string()))
                    .collect(),
            );
        }

        if let Some(where_conditions) = &self.where_conditions {
            let mut conditions_map = Map::new();
            for condition in where_conditions.iter() {
                let operator = match condition.comparison {
                    Comparison::Equal => "$eq",
                    Comparison::NotEqual => "$ne",
                    Comparison::GreaterThan => "$gt",
                    Comparison::LessThan => "$lt",
                    Comparison::GreaterThanOrEqual => "$gte",
                    Comparison::LessThanOrEqual => "$lte",
                };

                conditions_map.insert(condition.field.clone(), json!({ operator: condition.value }));
            }

            query["where"] = Value::Object(conditions_map);
        }
        serde_json::to_string(&query).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_select_query_with_multiple_conditions() {
        let mut query_builder = SurellDBQueryBuilder::new();
        query_builder.select(vec!["name", "department"]);
        query_builder.where_condition("age", Value::Number(serde_json::Number::from(30)), Comparison::GreaterThan);
        query_builder.where_condition("salary", Value::Number(serde_json::Number::from(50000)), Comparison::GreaterThanOrEqual);
        query_builder.where_condition("rating", Value::Number(serde_json::Number::from(4)), Comparison::Equal);

        let expected_result = r#"{"select":["name","department"],"where":{"age":{"$gt":30},"rating":{"$eq":4},"salary":{"$gte":50000}}}"#;

        assert_eq!(query_builder.build(), expected_result);
    }
}
