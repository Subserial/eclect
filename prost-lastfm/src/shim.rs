struct BoolVisitor;

impl serde::de::Visitor<'_> for BoolVisitor {
    type Value = bool;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("(\"0\" or \"false\") or (\"1\" or \"true\")")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "0" | "false" => Ok(false),
            "1" | "true" => Ok(true),
            _ => Err(E::invalid_value(serde::de::Unexpected::Str(v), &self)),
        }
    }
}

pub fn parse_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    deserializer.deserialize_str(BoolVisitor)
}

pub fn parse_option_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    Ok(Some(parse_bool(deserializer)?))
}

struct ParseVisitor<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: std::str::FromStr> serde::de::Visitor<'_> for ParseVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(&format!("{} as a string", std::any::type_name::<T>()))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse::<T>()
            .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(v), &self))
    }
}

pub fn parse_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: std::str::FromStr,
    D: serde::de::Deserializer<'de>,
{
    deserializer.deserialize_str(ParseVisitor {
        _marker: std::marker::PhantomData,
    })
}

pub fn parse_option_from_string<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: std::str::FromStr,
    D: serde::de::Deserializer<'de>,
{
    Ok(Some(parse_from_string(deserializer)?))
}
