use super::opcode::{OpName, Opcode};

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
}

impl<'a> TryFrom<&'a [u8]> for Inst<'a> {
	type Error = ();

	fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
		let len = Opcode::try_from(data[0])?.len();

		if data.len() < len {
			Err(())
		} else {
			Ok(Self(data))
		}
	}
}

pub fn get_jump_target(addr: u64, offset: i64) -> u64 {
	let new = addr as i64 + offset * 4 + 4;

	new as u64
}
