pub use crate::uri::{InvalidPort, Port, PortBuf};

mod host;
mod r#mut;
mod userinfo;

pub use host::*;
pub use r#mut::*;
pub use userinfo::*;

crate::common::authority!();

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parts() {
		let vectors = [
			("host", (None, "host", None)),
			("user@host", (Some("user"), "host", None)),
			("host:123", (None, "host", Some("123"))),
			("user@host:123", (Some("user"), "host", Some("123"))),
			("a:b@host", (Some("a:b"), "host", None)),
			("a:b@host:123", (Some("a:b"), "host", Some("123"))),
		];

		for (input, expected) in vectors {
			// eprintln!("{input} => {expected:?}");
			let input = Authority::new(input).unwrap();
			let parts = input.parts();

			assert_eq!(parts.user_info.map(UserInfo::as_str), expected.0);
			assert_eq!(parts.host.as_str(), expected.1);
			assert_eq!(parts.port.map(Port::as_str), expected.2)
		}
	}
}
