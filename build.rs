use std::{fs, io, path::Path};

fn main() {
	export_dir("src/uri", "src/iri").unwrap()
}

fn export_dir(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<(), io::Error> {
	let dir = fs::read_dir(input)?;

	for entry in dir {
		let entry = entry?;
		let file_type = entry.file_type()?;

		if file_type.is_file() {
			let output_file = output.as_ref().join(entry.file_name());
			if output_file.extension().is_some_and(|ext| ext == "rs") {
				export_file(entry.path(), output_file)?;
			}
		} else {
			let output = output.as_ref().join(entry.file_name());
			export_dir(entry.path(), output)?;
		}
	}

	Ok(())
}

fn export_file(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<(), io::Error> {
	println!("cargo::rerun-if-changed={}", input.as_ref().display());

	let content = fs::read_to_string(input)?;

	if let Some(parent) = output.as_ref().parent() {
		fs::create_dir_all(parent)?;
	}

	fs::write(output, replace(content))
}

fn replace(s: impl AsRef<str>) -> String {
	s.as_ref()
		.replace("URI", "IRI")
		.replace("Uri", "Iri")
		.replace("uri", "iri")
		.replace(
			r#""authority", "host", "userinfo" as UserInfo, "path", "segment", "query", "fragment""#,
			r#""iauthority", "ihost", "iuserinfo" as UserInfo, "ipath", "isegment", "iquery", "ifragment""#,
		)
		.replace("macro_rules! ", "macro_rules! i")
		.replace("macro_rules! iiri", "macro_rules! iri")
}
