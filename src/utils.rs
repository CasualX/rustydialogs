use std::{env, fmt};
use std::path::{Path, PathBuf};
use std::borrow::Cow;

pub struct PrintJoin<'a> {
	pub parts: &'a [&'a str],
	pub separator: &'a str,
}
impl<'a> fmt::Display for PrintJoin<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut first = true;
		for part in self.parts {
			if !first {
				f.write_str(self.separator)?;
			}
			f.write_str(part)?;
			first = false;
		}
		Ok(())
	}
}

pub fn abspath(path: Option<&Path>) -> Cow<'_, Path> {
	match path {
		Some(path) if path.is_absolute() => Cow::Borrowed(path),
		Some(path) => {
			let directory = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
			Cow::Owned(directory.join(path))
		}
		None => {
			let directory = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
			Cow::Owned(directory)
		}
	}
}
