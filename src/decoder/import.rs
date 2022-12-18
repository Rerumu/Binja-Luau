#[derive(Clone)]
pub struct Import {
	data: u32,
}

impl Iterator for Import {
	type Item = usize;

	fn next(&mut self) -> Option<Self::Item> {
		let len = u32::try_from(self.len()).ok()?;

		if len == 0 {
			return None;
		}

		let value = self.data & 0x3FF;

		self.data = self.data >> 10 | (len - 1) << 30;

		Some(value.try_into().unwrap())
	}
}

impl ExactSizeIterator for Import {
	fn len(&self) -> usize {
		let len = self.data >> 30;

		len.try_into().unwrap()
	}
}

impl From<u32> for Import {
	fn from(data: u32) -> Self {
		Self { data }
	}
}
