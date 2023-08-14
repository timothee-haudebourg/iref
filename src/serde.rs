#[cfg(feature = "serde")]
impl<'de: 'a, 'a> serde::Deserialize<'de> for &'a Iri {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor;

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = Iri<'de>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(formatter, "an IRI")
			}

			fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Iri::new(v).map_err(|_| E::invalid_value(serde::de::Unexpected::Str(v), &self))
			}

			fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Iri::new(v).map_err(|_| E::invalid_value(serde::de::Unexpected::Bytes(v), &self))
			}
		}

		deserializer.deserialize_str(Visitor)
	}
}

#[cfg(feature = "serde")]
impl<'a> serde::Serialize for &'a Iri {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(self.as_str())
	}
}
