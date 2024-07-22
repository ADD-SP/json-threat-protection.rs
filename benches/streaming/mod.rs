use serde::de::{DeserializeSeed, Deserializer, Error, MapAccess, SeqAccess, Visitor};
use std::collections::BTreeSet;
use std::fmt;

#[derive(Copy, Clone)]
pub struct StreamingValidator {
    depth: u16,
}

impl StreamingValidator {
    pub fn new() -> Self {
        StreamingValidator { depth: 0 }
    }
}

impl<'de> DeserializeSeed<'de> for StreamingValidator {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for StreamingValidator {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("JSON value that conforms to the constraints of json-threat-protection")
    }

    fn visit_bool<E: Error>(self, _v: bool) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_i64<E: Error>(self, _v: i64) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_u64<E: Error>(self, _v: u64) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_f64<E: Error>(self, _v: f64) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        if v.len() > isize::MAX as usize {
            return Err(E::custom("string too long"));
        }
        Ok(())
    }

    // null
    fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if self.depth == 1024 {
            return Err(A::Error::custom("too deep"));
        }
        let mut entries = 0;
        let validator = StreamingValidator {
            depth: self.depth + 1,
        };
        while seq.next_element_seed(validator)?.is_some() {
            entries += 1;
            if entries > isize::MAX as usize {
                return Err(A::Error::custom("too many entries"));
            }
        }
        Ok(())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        if self.depth == 1024 {
            return Err(A::Error::custom("too deep"));
        }
        let mut keys = BTreeSet::<String>::new();
        let validator = StreamingValidator {
            depth: self.depth + 1,
        };
        while let Some(k) = map.next_key::<String>()? {
            self.visit_str(&k)?;
            if !keys.insert(k) {
                return Err(A::Error::custom("duplicate key"));
            }
            if keys.len() > isize::MAX as usize {
                return Err(A::Error::custom("too many entries"));
            }
            map.next_value_seed(validator)?;
        }
        Ok(())
    }
}
