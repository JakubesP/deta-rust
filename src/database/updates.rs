//! Tools for defining updates to be performed on an item in the database.

use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::Into;

/// Alias for `Cow<'static, str>`.
/// Using `Cow` in updates context instead of the usual String helps avoid unnecessary copies of data.
pub type Key = Cow<'static, str>;

type JsonValue = serde_json::Value;

pub(crate) type UpdatesSchemaSet = HashMap<Key, JsonValue>;
pub(crate) type UpdatesSchemaIncrement = HashMap<Key, i32>;
pub(crate) type UpdatesSchemaAppend = HashMap<Key, Vec<JsonValue>>;
pub(crate) type UpdatesSchemaPrepend = HashMap<Key, Vec<JsonValue>>;
pub(crate) type UpdatesSchemaDelete = Vec<Key>;

// An intermediate structure in building the final JSON value, based on updates to be made.
#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct UpdatesSchema {
    set: Option<UpdatesSchemaSet>,
    increment: Option<UpdatesSchemaIncrement>,
    append: Option<UpdatesSchemaAppend>,
    prepend: Option<UpdatesSchemaPrepend>,
    delete: Option<UpdatesSchemaDelete>,
}

impl<'a> UpdatesSchema {
    fn new() -> Self {
        Self {
            set: None,
            increment: None,
            append: None,
            prepend: None,
            delete: None,
        }
    }
}

/// Enum specifying the variants of actions to be performed when updating the item.
/// The type contains factory methods to facilitate the construction of variants.
/// Check [deta docs](https://docs.deta.sh/docs/base/http#update-item) for more information.
#[derive(Debug, Clone)]
pub enum Action {
    /// The attribute to be updated or created.
    Set(JsonValue),

    /// The attribute to be incremented. Increment value can be negative.
    Increment(i32),

    /// The attribute to append a values to.
    Append(Vec<JsonValue>),

    /// The attribute to prepend a values to.
    Prepend(Vec<JsonValue>),

    /// The attribute to be deleted.
    Delete,
}

/// Factory methods.
impl Action {
    pub fn set<T>(value: T) -> serde_json::Result<Self>
    where
        T: Serialize,
    {
        let serde_value = serde_json::to_value(value)?;
        Ok(Self::Set(serde_value))
    }

    pub fn increment(value: i32) -> Self {
        Self::Increment(value)
    }

    pub fn append<T>(value: T) -> serde_json::Result<Self>
    where
        T: Serialize,
    {
        let serde_value = serde_json::to_value(value)?;
        Ok(Self::Append(vec![serde_value]))
    }

    pub fn append_many<T>(value: &[T]) -> serde_json::Result<Self>
    where
        T: Serialize,
    {
        let mut serde_value = serde_json::to_value(value)?;
        let serde_value_array: Vec<JsonValue> = serde_value
            .as_array_mut()
            .unwrap() // It will never panic because the `serde_value` variable is always an array here.
            .iter_mut()
            .map(|item| item.take())
            .collect();
        Ok(Self::Append(serde_value_array))
    }

    pub fn prepend<T>(value: T) -> serde_json::Result<Self>
    where
        T: Serialize,
    {
        let serde_value = serde_json::to_value(value)?;
        Ok(Self::Prepend(vec![serde_value]))
    }

    pub fn prepend_many<T>(value: &[T]) -> serde_json::Result<Self>
    where
        T: Serialize,
    {
        let mut serde_value = serde_json::to_value(value)?;
        let serde_value_array: Vec<JsonValue> = serde_value
            .as_array_mut()
            .unwrap() // It will never panic because the `serde_value` variable is always an array here.
            .iter_mut()
            .map(|item| item.take())
            .collect();
        Ok(Self::Prepend(serde_value_array))
    }

    pub fn delete() -> Self {
        Self::Delete
    }

    // Consumes the specified action variant and inserts this value of type `UpdatesSchema`.
    pub(crate) fn render<'a>(
        self,
        key: Key,
        mut target: UpdatesSchema,
    ) -> serde_json::Result<UpdatesSchema> {
        match self {
            Self::Set(set_value) => {
                if let None = target.set {
                    target.set = Some(HashMap::new());
                }
                if let Some(value) = &mut target.set {
                    value.insert(key, set_value);
                }
            }
            Self::Increment(increment_value) => {
                if let None = target.increment {
                    target.increment = Some(HashMap::new());
                }
                if let Some(value) = &mut target.increment {
                    value.insert(key, increment_value);
                }
            }
            Self::Append(append_value) => {
                if let None = target.append {
                    target.append = Some(HashMap::new());
                }
                if let Some(value) = &mut target.append {
                    value.insert(key, append_value);
                }
            }
            Self::Prepend(prepend_value) => {
                if let None = target.prepend {
                    target.prepend = Some(HashMap::new());
                }
                if let Some(value) = &mut target.prepend {
                    value.insert(key, prepend_value);
                }
            }
            Self::Delete => {
                if let None = target.delete {
                    target.delete = Some(vec![]);
                }
                if let Some(value) = &mut target.delete {
                    value.push(key);
                }
            }
        };

        Ok(target)
    }
}

/// Useful conversion to wrap an Action type value to [`serde_json::Result`](serde_json::Result)
/// for standardization purposes inside the `Updates` type.
impl From<Action> for serde_json::Result<Action> {
    fn from(action: Action) -> serde_json::Result<Action> {
        Ok(action)
    }
}

type PartialActions = Vec<(Key, serde_json::Result<Action>)>;

/// Builder type to build a list of updates to perform.
pub struct Updates {
    actions: PartialActions,
}

impl Updates {
    /// Initializes the builder.
    pub fn init() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    /// Adds a new action to be performed during an update.
    /// Both `Action` and `serde_json::Result<Action>` types can be specified as `action` parameters.
    /// This allows the deserialisation error handling to be postponed.
    ///
    /// **NOTE:** If you multiple add the same action types to execute for the same key,
    /// the new action will overwrite the old one.
    /// 
    /// Remember that the [`Action::append`](Action::append),
    /// [`Action::append_many`](ction::append_many) and
    /// [`Action::prepend`](Action::prepend),
    /// [`Action::prepend_many`](Action::prepend_many)
    /// methods generate a common action variants: [`Action::Append`](Action::Append) and [`Action::Prepend`](Action::Prepend).
    pub fn add<T, D>(mut self, attr: T, action: D) -> Self
    where
        T: Into<Key>,
        D: Into<serde_json::Result<Action>>,
    {
        self.actions.push((attr.into(), action.into()));
        self
    }

    pub(crate) fn render(self) -> serde_json::Result<JsonValue> {
        let mut target = UpdatesSchema::new();
        for (k, v) in self.actions {
            target = v?.render(k, target)?;
        }

        let target_json = serde_json::to_value(target)?;
        Ok(target_json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_for_general_updates() {
        // Specify actions to perform on item
        let target = Updates::init()
            .add("profile.age", Action::set(33))
            .add("profile.active", Action::set(true))
            .add("profile.email", Action::set("jimmy@deta.sh"))
            .add("purchases", Action::increment(2))
            .add("likes", Action::append_many(&["ramen", "jimmy"]))
            .add("watchers", Action::prepend_many(&["mark"]))
            .add("profile.hometown", Action::delete())
            .add("count", Action::increment(1))
            .add("age", Action::delete())
            .add("clients", Action::append("jacob"))
            .add("fans", Action::prepend("alex"))
            .render()
            .expect("Render failed");

        // Construct expected_target
        let mut set_section = HashMap::<Key, JsonValue>::new();
        set_section.insert("profile.active".into(), true.into());
        set_section.insert("profile.age".into(), 33.into());
        set_section.insert("profile.email".into(), "jimmy@deta.sh".into());

        let mut increment_section = HashMap::<Key, i32>::new();
        increment_section.insert("count".into(), 1);
        increment_section.insert("purchases".into(), 2);

        let mut append_section = HashMap::<Key, Vec<JsonValue>>::new();
        append_section.insert("likes".into(), vec!["ramen".into(), "jimmy".into()]);
        append_section.insert("clients".into(), vec!["jacob".into()]);

        let mut prepend_section = HashMap::<Key, Vec<JsonValue>>::new();
        prepend_section.insert("watchers".into(), vec!["mark".into()]);
        prepend_section.insert("fans".into(), vec!["alex".into()]);

        let mut delete_section = Vec::<Key>::new();
        delete_section.push("profile.hometown".into());
        delete_section.push("age".into()); // ???

        let expected_target = serde_json::to_value(UpdatesSchema {
            set: Some(set_section),
            increment: Some(increment_section),
            append: Some(append_section),
            prepend: Some(prepend_section),
            delete: Some(delete_section),
        })
        .expect("Convert UpdatesSchema to JSON failed");

        // Compare
        assert_eq!(target, expected_target);
    }

    #[test]
    fn render_for_overrides() {
        // Specify actions to perform on item
        let target = Updates::init()
            .add("profile.age", Action::set(33))
            .add("count", Action::set(7))
            .add("likes", Action::prepend_many(&["tom", "adam"]))
            .add("likes", Action::prepend("julie"))
            .add("profile.age", Action::set(57))
            .add("count", Action::increment(8))
            .render()
            .expect("Render failed");

        // Construct expected_target
        let mut set_section = HashMap::<Key, JsonValue>::new();
        set_section.insert("count".into(), 7.into());
        set_section.insert("profile.age".into(), 57.into());

        let mut increment_section = HashMap::<Key, i32>::new();
        increment_section.insert("count".into(), 8);

        let mut prepend_section = HashMap::<Key, Vec<JsonValue>>::new();
        prepend_section.insert("likes".into(), vec!["julie".into()]);

        let expected_target = serde_json::to_value(UpdatesSchema {
            set: Some(set_section),
            increment: Some(increment_section),
            append: None,
            prepend: Some(prepend_section),
            delete: None,
        })
        .expect("Convert UpdatesSchema to JSON failed");

        // Compare
        assert_eq!(target, expected_target);
    }
}
