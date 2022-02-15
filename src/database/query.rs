//! Tools for defining the query to be used when fetching items from the database.

use super::common::{JsonValue, StringValue};
use serde::Serialize;
use std::borrow::Borrow;
use std::convert::Into;

/// Enum specifying the variants of conditions to be useed when querying (fetching) the items.
/// The type contains factory methods to facilitate the construction of variants.
/// Check [deta docs](https://docs.deta.sh/docs/base/sdk#queries) for more information.
pub enum Condition {
    Equal(JsonValue),
    NotEqual(JsonValue),
    LessThan(f64),
    GreaterThan(f64),
    LessThanOrEqual(f64),
    GreaterThatOrEqual(f64),
    Prefix(StringValue),
    Range(f64, f64),
    Contains(StringValue),
    NotContains(StringValue),
}

fn set_postfix(key: StringValue, postfix: &str) -> StringValue {
    format!("{}?{}", key, postfix).into()
}

impl Condition {
    fn gen_pair(self, key: StringValue) -> (StringValue, JsonValue) {
        match self {
            Self::Equal(val) => (key, val),
            Self::NotEqual(val) => (set_postfix(key, "ne"), val),
            Self::LessThan(val) => (set_postfix(key, "lt"), val.into()),
            Self::GreaterThan(val) => (set_postfix(key, "gt"), val.into()),
            Self::LessThanOrEqual(val) => (set_postfix(key, "lte"), val.into()),
            Self::GreaterThatOrEqual(val) => (set_postfix(key, "gte"), val.into()),
            Self::Prefix(val) => (set_postfix(key, "pfx"), val.into()),
            Self::Range(val1, val2) => (set_postfix(key, "r"), serde_json::json!([val1, val2])),
            Self::Contains(val) => (set_postfix(key, "contains"), val.into()),
            Self::NotContains(val) => (set_postfix(key, "not_contains"), val.into()),
        }
    }
}

/// Factory methods.
impl Condition {
    pub fn equal<T>(value: T) -> serde_json::Result<Condition>
    where
        T: Serialize,
    {
        let json_val = serde_json::to_value(value)?;
        Ok(Self::Equal(json_val))
    }

    pub fn not_equal<T>(value: T) -> serde_json::Result<Condition>
    where
        T: Serialize,
    {
        let json_val = serde_json::to_value(value)?;
        Ok(Self::NotEqual(json_val))
    }

    pub fn less_than<T>(value: T) -> Condition
    where
        T: Into<f64>,
    {
        Self::LessThan(value.into())
    }

    pub fn greater_than<T>(value: T) -> Condition
    where
        T: Into<f64>,
    {
        Self::GreaterThan(value.into())
    }

    pub fn less_than_or_equal<T>(value: T) -> Condition
    where
        T: Into<f64>,
    {
        Self::LessThanOrEqual(value.into())
    }

    pub fn greater_than_or_equal<T>(value: T) -> Condition
    where
        T: Into<f64>,
    {
        Self::GreaterThatOrEqual(value.into())
    }

    pub fn prefix<T>(value: T) -> Condition
    where
        T: Into<StringValue>,
    {
        Self::Prefix(value.into())
    }

    pub fn range<T>(start: T, end: T) -> Condition
    where
        T: Into<f64>,
    {
        Self::Range(start.into(), end.into())
    }

    pub fn contains<T>(value: T) -> Condition
    where
        T: Into<StringValue>,
    {
        Self::Contains(value.into())
    }

    pub fn not_contains<T>(value: T) -> Condition
    where
        T: Into<StringValue>,
    {
        Self::NotContains(value.into())
    }
}

/// Useful conversion to wrap an Condition type value to [`serde_json::Result`](serde_json::Result)
/// for standardization purposes inside the `Query` type.
impl From<Condition> for serde_json::Result<Condition> {
    fn from(condition: Condition) -> serde_json::Result<Condition> {
        Ok(condition)
    }
}

/// Builder type to build a query to perform.
pub struct Query {
    // Each element in the list makes up an OR.
    // A single element represents an AND expression.
    conditions: Vec<Vec<(StringValue, serde_json::Result<Condition>)>>,
}

impl Query {
    /// Initializes the builder.
    pub fn init() -> Self {
        Self { conditions: vec![] }
    }

    /// Adds a new condition that the item must satisfy.
    pub fn on<K, V>(mut self, key: K, condition: V) -> Self
    where
        K: Into<StringValue>,
        V: Into<serde_json::Result<Condition>>,
    {
        if let None = self.conditions.last() {
            self.conditions.push(vec![]);
        }
        if let Some(and) = self.conditions.last_mut() {
            and.push((key.into(), condition.into()));
        }
        self
    }

    /// Separates alternative conditions (or statement).
    pub fn either(mut self) -> Self {
        if let Some(and) = self.conditions.last_mut() {
            if and.len() > 0 {
                self.conditions.push(vec![]);
            }
        }
        self
    }

    pub(crate) fn render(self) -> serde_json::Result<JsonValue> {
        let mut target = vec![];
        for condition in self.conditions {
            let mut target_obj = serde_json::json!({});
            for and in condition {
                let (key, val_result) = and;
                let val = val_result?;
                let (key, val) = val.gen_pair(key);
                let key: &str = key.borrow();
                target_obj[key] = val;
            }
            target.push(target_obj);
        }
        serde_json::to_value(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[test]
    fn render_for_all_condition_types() {
        let query = Query::init()
            .on("name", Condition::equal("Anna"))
            .on("surname", Condition::not_equal("Kowal"))
            .on("count", Condition::less_than(10))
            .on("likes", Condition::greater_than(10))
            .on("watchers", Condition::greater_than_or_equal(78))
            .on("customers", Condition::less_than_or_equal(4))
            .on("homepage", Condition::prefix("https"))
            .on("age", Condition::range(23, 78))
            .on("title", Condition::not_contains("car"))
            .on("description", Condition::contains("Tom"))
            .render()
            .unwrap();

        let target_query = serde_json::json!([
            {
                "name": "Anna",
                "surname?ne": "Kowal",
                "count?lt": 10.,
                "likes?gt": 10.,
                "watchers?gte": 78.,
                "customers?lte": 4.,
                "homepage?pfx": "https",
                "age?r": [23., 78.],
                "title?not_contains": "car",
                "description?contains": "Tom"
            },
        ]);

        assert_eq!(query, target_query);
    }

    #[test]
    fn render_with_either_statements() {
        let query = Query::init()
            .on("age", Condition::greater_than(50))
            .either()
            .on("hometown", Condition::equal("Greenville"))
            .render()
            .unwrap();

        let target_query = serde_json::json!([
            {
                "age?gt": 50.,
            },
            {
                "hometown": "Greenville",
            }
        ]);

        assert_eq!(query, target_query);
    }

    #[test]
    fn render_with_redundant_either_statements() {
        let query = Query::init()
            .either()
            .on("age", Condition::equal(15))
            .either()
            .either()
            .on("name", Condition::not_contains("om"))
            .either()
            .either()
            .either();

        assert_eq!(query.conditions.len(), 3);

        let query = query.render().unwrap();

        let target_query = serde_json::json!([
            {
                "age": 15,
            },
            {
                "name?not_contains": "om",
            },
            {}
        ]);

        assert_eq!(query, target_query);
    }

    #[test]
    fn render_with_complex_obects() {
        #[derive(Serialize)]
        struct PersonalData {
            name: &'static str,
            age: u8,
        }

        let query = Query::init()
            .on(
                "personal_data",
                Condition::equal(PersonalData {
                    name: "Jan",
                    age: 43,
                }),
            )
            .either()
            .on("personal_data.name", Condition::equal("Janina"))
            .on("personal_data.age", Condition::equal(51))
            .render()
            .unwrap();

        let target_query = serde_json::json!([
            {
                "personal_data": {
                    "name": "Jan",
                    "age": 43
                },
            },
            {
                "personal_data.name": "Janina",
                "personal_data.age": 51,

            },
        ]);

        assert_eq!(query, target_query);
    }
}
