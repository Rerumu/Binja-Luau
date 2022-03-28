#[derive(Clone)]
pub struct RefUnknown(u32);

impl Iterator for RefUnknown {
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

impl ExactSizeIterator for RefUnknown {
	fn len(&self) -> usize {
		let high = self.0 >> 30;

		high.try_into().unwrap()
	}
}

impl From<u32> for RefUnknown {
	fn from(value: u32) -> Self {
		Self(value)
	}
}
