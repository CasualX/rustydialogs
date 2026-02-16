use std::fmt;

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
