use super::opcode::{OpName, Opcode};

#[derive(Clone, Copy)]
pub struct Inst<'a>(&'a [u8]);

impl<'a> Inst<'a> {
	pub fn op(&self) -> Opcode {
		Opcode::try_from(self.0[0]).unwrap()
	}

	pub fn a(&self) -> u8 {
		self.0[1]
	}

	pub fn b(&self) -> u8 {
		self.0[2]
	}

	pub fn c(&self) -> u8 {
		self.0[3]
	}

	pub fn d(&self) -> i16 {
		let b = self.b();
		let c = self.c();

		i16::from_le_bytes([b, c])
	}

	pub fn e(&self) -> i32 {
		let a = self.a();
		let b = self.b();
		let c = self.c();

		i32::from_le_bytes([0, a, b, c])
	}

	pub fn adjacent(&self) -> i32 {
		let data = self.0[4..8].try_into().unwrap();

		i32::from_le_bytes(data)
	}

	pub fn with_name(&self, name: OpName) -> i32 {
		match name {
			OpName::A => self.a().into(),
			OpName::B => self.b().into(),
			OpName::C => self.c().into(),
			OpName::D => self.d().into(),
			OpName::E => self.e(),
			OpName::X => self.adjacent(),
		}
	}

	pub fn get_jump_target<T>(start: u64, offset: T) -> u64
	where
		T: Into<i64>,
	{
		start.wrapping_add_signed(offset.into() * 4) + 4
	}
}

impl<'a> TryFrom<&'a [u8]> for Inst<'a> {
	type Error = ();

	fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
		let first = *data.get(0).ok_or(())?;
		let len = Opcode::try_from(first).map_err(drop)?.len();

		if data.len() < len {
			Err(())
		} else {
			Ok(Self(data))
		}
	}
}
