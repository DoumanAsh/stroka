use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

use crate::String;

impl Serialize for String {
    fn serialize<SER: Serializer>(&self, ser: SER) -> Result<SER::Ok, SER::Error> {
        ser.serialize_str(self.as_str())
    }
}

struct StringVisitor;

impl<'de> serde::de::Visitor<'de> for StringVisitor {
    type Value = String;

    #[inline(always)]
    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a string")
    }

    #[inline]
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(String::new_str(v))
    }
}

impl<'de> Deserialize<'de> for String {
    #[inline]
    fn deserialize<D: Deserializer<'de>>(des: D) -> Result<Self, D::Error> {
        des.deserialize_str(StringVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::String;

    use serde::de::Deserialize;
    use serde::de::value::{BorrowedStrDeserializer, Error as ValueError};

    #[test]
    fn should_deserialize_within_sso_cap() {
        let des = BorrowedStrDeserializer::<ValueError>::new("lolka");
        let res = String::deserialize(des).expect("Unexpected fail");
        assert_eq!(res, "lolka");
        assert!(!res.is_alloc());
    }

    #[test]
    fn should_deserialize_outside_sso_cap() {
        const TEXT: &str = "lolka lol lolid by loli";
        let des = BorrowedStrDeserializer::<ValueError>::new(TEXT);
        let res = String::deserialize(des).expect("Unexpected fail");
        assert_eq!(res.as_str(), TEXT);
        assert!(res.is_alloc());
    }
}
