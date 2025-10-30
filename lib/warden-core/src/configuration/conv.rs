use crate::{
    configuration::typology::Operator,
    google::protobuf::{ListValue, NullValue, Struct, Value, value},
};

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
    use super::Operator;
    use serde::{self, Deserialize, Deserializer, Serializer};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::google::protobuf::{Value, value::Kind};
    use serde::{Deserialize, Serialize};
    use serde_json::{self, json};

    #[derive(Deserialize, Serialize)]
    struct OpWrap {
        #[serde(with = "super::operator_serde")]
        op: i32,
    }

    fn normalize_json_numbers(val: serde_json::Value) -> serde_json::Value {
        match val {
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.into_iter().map(normalize_json_numbers).collect())
            }
            serde_json::Value::Object(map) => serde_json::Value::Object(
                map.into_iter()
                    .map(|(k, v)| (k, normalize_json_numbers(v)))
                    .collect(),
            ),
            serde_json::Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(f)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    )
                } else {
                    serde_json::Value::Number(n)
                }
            }
            other => other,
        }
    }

    #[test]
    fn test_json_to_protobuf_value_null() {
        let json = serde_json::Value::Null;
        let protobuf_value = Value::try_from(json).unwrap();
        assert!(matches!(protobuf_value.kind.unwrap(), Kind::NullValue(_)));
    }

    #[test]
    fn test_json_to_protobuf_value_bool() {
        let json = serde_json::Value::Bool(true);
        let protobuf_value = Value::try_from(json).unwrap();
        assert_eq!(protobuf_value.kind.unwrap(), Kind::BoolValue(true));
    }

    #[test]
    fn test_json_to_protobuf_value_number() {
        let json = serde_json::Value::Number(serde_json::Number::from(42));
        let protobuf_value = Value::try_from(json).unwrap();
        assert_eq!(protobuf_value.kind.unwrap(), Kind::NumberValue(42.0));
    }

    #[test]
    fn test_json_to_protobuf_value_string() {
        let json = serde_json::Value::String("hello".to_string());
        let protobuf_value = Value::try_from(json).unwrap();
        assert_eq!(
            protobuf_value.kind.unwrap(),
            Kind::StringValue("hello".to_string())
        );
    }

    #[test]
    fn test_json_to_protobuf_value_array() {
        let json = json!([1, 2, "three"]);
        let protobuf_value = Value::try_from(json).unwrap();
        if let Kind::ListValue(list) = protobuf_value.kind.unwrap() {
            assert_eq!(list.values.len(), 3);
        } else {
            panic!("Expected ListValue");
        }
    }

    #[test]
    fn test_json_to_protobuf_value_object() {
        let json = json!({
            "a": 1,
            "b": true,
            "c": "hello"
        });
        let protobuf_value = Value::try_from(json).unwrap();
        if let Kind::StructValue(s) = protobuf_value.kind.unwrap() {
            assert_eq!(
                s.fields["a"].kind.as_ref().unwrap(),
                &Kind::NumberValue(1.0)
            );
            assert_eq!(s.fields["b"].kind.as_ref().unwrap(), &Kind::BoolValue(true));
            assert_eq!(
                s.fields["c"].kind.as_ref().unwrap(),
                &Kind::StringValue("hello".to_string())
            );
        } else {
            panic!("Expected StructValue");
        }
    }

    #[test]
    fn test_protobuf_to_json_roundtrip() {
        let original = json!({
            "x": 1,
            "y": [true, null, "str"],
            "z": {
                "nested": 3.90
            }
        });

        let protobuf_value = Value::try_from(original.clone()).unwrap();
        let json_value: serde_json::Value = protobuf_value.into();

        assert_eq!(
            normalize_json_numbers(original),
            normalize_json_numbers(json_value)
        );
    }

    #[test]
    fn test_operator_serialization() {
        let wrap = OpWrap {
            op: Operator::Add as i32, // Replace with actual enum variant
        };

        let s = serde_json::to_string(&wrap).unwrap();
        assert!(s.contains("ADD")); // Assuming .as_str_name() gives "ADD"
    }

    #[test]
    fn test_operator_deserialization() {
        let json_data = json!({ "op": "ADD" }).to_string();
        let wrap: OpWrap = serde_json::from_str(&json_data).unwrap();

        assert_eq!(wrap.op, Operator::Add as i32); // Replace with actual enum variant
    }

    #[test]
    fn test_operator_invalid_deserialization() {
        let json_data = json!({ "op": "UNKNOWN_OP" }).to_string();
        let result: Result<OpWrap, _> = serde_json::from_str(&json_data);
        assert!(result.is_err());
    }
}
