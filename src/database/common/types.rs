use std::borrow::Cow;

/// Alias for `Cow<'static, str>`.
/// Using `Cow` in updates context instead of the usual String helps avoid unnecessary copies of data.
pub type StringValue = Cow<'static, str>;

pub type JsonValue = serde_json::Value;

