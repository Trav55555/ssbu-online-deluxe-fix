use std::fmt::Write;

pub enum Value<'a> {
    Text(&'a str),
    Number(u64),
    Signed(i32),
    Bool(bool),
    Null,
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(value: &'a str) -> Self {
        Self::Text(value)
    }
}
impl From<bool> for Value<'_> {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
impl From<i32> for Value<'_> {
    fn from(value: i32) -> Self {
        Self::Signed(value)
    }
}
impl From<u32> for Value<'_> {
    fn from(value: u32) -> Self {
        Self::Number(value as u64)
    }
}
impl From<u16> for Value<'_> {
    fn from(value: u16) -> Self {
        Self::Number(value as u64)
    }
}
impl From<usize> for Value<'_> {
    fn from(value: usize) -> Self {
        Self::Number(value as u64)
    }
}
impl From<Option<i32>> for Value<'_> {
    fn from(value: Option<i32>) -> Self {
        value.map_or(Self::Null, Self::from)
    }
}
impl From<Option<u32>> for Value<'_> {
    fn from(value: Option<u32>) -> Self {
        value.map_or(Self::Null, Self::from)
    }
}
impl From<Option<bool>> for Value<'_> {
    fn from(value: Option<bool>) -> Self {
        value.map_or(Self::Null, Self::from)
    }
}

fn quote(output: &mut String, value: &str) {
    output.push('"');
    for c in value.chars() {
        match c {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            c if c < '\u{20}' => {
                let _ = write!(output, "\\u{:04x}", c as u32);
            }
            c => output.push(c),
        }
    }
    output.push('"');
}

pub fn format_event(sequence: u64, event: &str, fields: &[(&str, Value<'_>)]) -> String {
    let mut output = format!("{{\"schema\":1,\"sequence\":{sequence},\"event\":");
    quote(&mut output, event);
    output.push_str(",\"data\":{");
    for (index, (key, value)) in fields.iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        quote(&mut output, key);
        output.push(':');
        match value {
            Value::Text(value) => quote(&mut output, value),
            Value::Number(value) => {
                let _ = write!(output, "{value}");
            }
            Value::Signed(value) => {
                let _ = write!(output, "{value}");
            }
            Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
            Value::Null => output.push_str("null"),
        }
    }
    output.push_str("}}");
    output
}

#[cfg(test)]
mod tests {
    use super::{format_event, Value};

    #[test]
    fn unavailable_is_null_not_false_or_zero() {
        assert_eq!(
            format_event(
                1,
                "apis",
                &[
                    ("missing", Value::from(None::<u32>)),
                    ("rejected", Some(false).into()),
                    ("version", Some(2u32).into()),
                ]
            ),
            r#"{"schema":1,"sequence":1,"event":"apis","data":{"missing":null,"rejected":false,"version":2}}"#
        );
    }

    #[test]
    fn strings_escape_control_characters_and_preserve_unicode() {
        assert_eq!(format_event(9, "boot", &[("version", "\"\\\n\r\t\0é".into())]),
            "{\"schema\":1,\"sequence\":9,\"event\":\"boot\",\"data\":{\"version\":\"\\\"\\\\\\n\\r\\t\\u0000é\"}}");
    }

    #[test]
    fn unqueried_and_negative_fighter_counts_are_not_zero() {
        assert_eq!(
            format_event(
                2,
                "stage",
                &[
                    ("unqueried", None::<i32>.into()),
                    ("count", Some(-1i32).into()),
                ]
            ),
            r#"{"schema":1,"sequence":2,"event":"stage","data":{"unqueried":null,"count":-1}}"#
        );
    }

    #[test]
    fn empty_payload_is_a_json_object() {
        assert_eq!(
            format_event(0, "nro_loaded", &[]),
            r#"{"schema":1,"sequence":0,"event":"nro_loaded","data":{}}"#
        );
    }
}
