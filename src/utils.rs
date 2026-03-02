use std::{env, fmt};
use std::path::Path;
use std::borrow::Cow;

#[allow(dead_code)]
pub struct FromFn<F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result>(pub F);

impl<F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result> fmt::Display for FromFn<F> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		(self.0)(f)
	}
}

#[allow(dead_code)]
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

pub fn abspath(path: Option<&Path>) -> Option<Cow<'_, Path>> {
	path.and_then(|p| {
		if p.is_absolute() {
			Some(Cow::Borrowed(p))
		}
		else {
			let directory = env::current_dir().ok()?;
			Some(Cow::Owned(directory.join(p)))
		}
	})
}
