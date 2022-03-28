#[derive(Clone)]
pub struct Import(u32);

impl Import {
	pub fn new(id: u32) -> Self {
		Import(id)
	}
}

impl Iterator for Import {
	type Item = usize;

	fn next(&mut self) -> Option<Self::Item> {
		let len = self.len();

		if len == 0 {
			return None;
		}

		let value = self.0 & 0x3FF;

		self.0 = self.0 >> 10 | (len as u32 - 1) << 30;

		Some(value.try_into().unwrap())
	}
}

impl ExactSizeIterator for Import {
	fn len(&self) -> usize {
		let high = self.0 >> 30;

		high.try_into().unwrap()
	}
}
