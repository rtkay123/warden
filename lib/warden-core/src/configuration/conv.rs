use crate::{configuration::typology::Operator, google::protobuf::{value, ListValue, NullValue, Struct, Value}};

#[derive(Debug)]
/// Generic JSON value
pub struct GenericParameter(pub(crate) serde_json::Value);

impl From<Value> for GenericParameter {
    fn from(value: Value) -> Self {
        Self(value.into())
    }
}

impl From<value::Kind> for GenericParameter {
    fn from(value: value::Kind) -> Self {
        Self(value.into())
    }
}

impl TryFrom<serde_json::Value> for value::Kind {
    type Error = String;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        match value {
            serde_json::Value::Null => Ok(value::Kind::NullValue(NullValue::NullValue as i32)),
            serde_json::Value::Bool(b) => Ok(value::Kind::BoolValue(b)),
            serde_json::Value::Number(n) => n
                .as_f64()
                .map(value::Kind::NumberValue)
                .ok_or_else(|| "Invalid number".to_string()),
            serde_json::Value::String(s) => Ok(value::Kind::StringValue(s)),
            serde_json::Value::Array(arr) => {
                let values = arr
                    .into_iter()
                    .map(Value::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(value::Kind::ListValue(ListValue { values }))
            }
            serde_json::Value::Object(map) => {
                let mut fields = std::collections::HashMap::new();
                for (k, v) in map {
                    let v = Value::try_from(v)?;
                    fields.insert(k, v);
                }
                Ok(value::Kind::StructValue(Struct { fields }))
            }
        }
    }
}

impl TryFrom<serde_json::Value> for Value {
    type Error = String;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        let kind = Some(value::Kind::try_from(value)?);
        Ok(Value { kind })
    }
}

impl From<value::Kind> for serde_json::Value {
    fn from(kind: value::Kind) -> Self {
        match kind {
            value::Kind::NullValue(_) => serde_json::Value::Null,
            value::Kind::BoolValue(b) => serde_json::Value::Bool(b),
            value::Kind::NumberValue(n) => serde_json::Value::Number(
                serde_json::Number::from_f64(n).unwrap_or_else(|| serde_json::Number::from(0)),
            ),
            value::Kind::StringValue(s) => serde_json::Value::String(s),
            value::Kind::StructValue(s) => serde_json::Value::from(s),
            value::Kind::ListValue(l) => serde_json::Value::from(l),
        }
    }
}

impl From<Value> for serde_json::Value {
    fn from(value: Value) -> Self {
        match value.kind {
            Some(kind) => kind.into(),
            None => serde_json::Value::Null,
        }
    }
}

impl From<Struct> for serde_json::Value {
    fn from(s: Struct) -> Self {
        let map = s
            .fields
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::from(v)))
            .collect();
        serde_json::Value::Object(map)
    }
}

impl From<ListValue> for serde_json::Value {
    fn from(l: ListValue) -> Self {
        let list = l.values.into_iter().map(serde_json::Value::from).collect();
        serde_json::Value::Array(list)
    }
}

impl serde::Serialize for GenericParameter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let json = self.0.clone();
        json.serialize(serializer)
    }
}

pub(crate) mod operator_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use super::Operator;

    pub fn serialize<S>(operator: &i32, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let operator = Operator::try_from(*operator);
        if let Ok(d) = operator {
            return s.serialize_str(d.as_str_name());
        }
        s.serialize_none()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;

        if let Some(s) = s {
            let op = Operator::from_str_name(&s)
                .ok_or_else(|| serde::de::Error::custom("unsupported"))?
                as i32;
            return Ok(op);
        }

        Err(serde::de::Error::custom("deserialise error for operator"))
    }
}

impl From<Operator> for String {
    fn from(value: Operator) -> Self {
        value.as_str_name().to_owned()
    }
}

impl TryFrom<String> for Operator {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.to_uppercase();
        Operator::from_str_name(&value).ok_or_else(|| format!("unsupported operator: {}", value))
    }
}
