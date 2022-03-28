use std::ops::Range;

use binaryninja::architecture::{InstructionTextToken, InstructionTextTokenContents};

use crate::instruction::{
	builtin::BuiltIn, decoder::get_jump_target, import::Import, opcode::Opcode,
};

#[derive(Default)]
pub struct TextBuilder {
	buffer: Vec<InstructionTextToken>,
}

impl TextBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	fn add_space(&mut self) {
		let token = InstructionTextToken::new(InstructionTextTokenContents::OperandSeparator, " ");

		self.buffer.push(token);
	}

	pub fn add_failure(&mut self) {
		let token = InstructionTextToken::new(InstructionTextTokenContents::Text, "?");

		self.add_space();
		self.buffer.push(token);
	}

	pub fn add_mnemonic(&mut self, opcode: Opcode) {
		let name = opcode.mnemonic();
		let token = InstructionTextToken::new(InstructionTextTokenContents::Instruction, name);

		self.buffer.push(token);
	}

	pub fn add_location(&mut self, addr: u64, offset: i64) {
		let target = get_jump_target(addr, offset);
		let token = InstructionTextToken::new(
			InstructionTextTokenContents::PossibleAddress(target),
			format!("{offset:+}"),
		);

		self.add_space();
		self.buffer.push(token);
	}

	pub fn add_register(&mut self, reg: i32) {
		let name = if (0..0x100).contains(&reg) {
			format!("r{}", reg)
		} else {
			"r?".to_string()
		};

		let token = InstructionTextToken::new(InstructionTextTokenContents::Register, name);

		self.add_space();
		self.buffer.push(token);
	}

	pub fn add_boolean(&mut self, value: bool) {
		let token = InstructionTextToken::new(
			InstructionTextTokenContents::Integer(value.into()),
			value.to_string(),
		);

		self.add_space();
		self.buffer.push(token);
	}

	pub fn add_integer(&mut self, value: i32) {
		let token =
			InstructionTextToken::new(InstructionTextTokenContents::Integer(0), value.to_string());

		self.add_space();
		self.buffer.push(token);
	}

	pub fn add_constant(&mut self, index: usize, list: &[Range<usize>]) {
		let target = list.get(index).map_or(0, |v| v.start as u64);
		let token = InstructionTextToken::new(
			InstructionTextTokenContents::PossibleAddress(target),
			target.to_string(),
		);

		self.add_space();
		self.buffer.push(token);
	}

	pub fn add_built_in(&mut self, index: u8) {
		let name = match BuiltIn::try_from(index).ok() {
			Some(v) => v.to_string(),
			None => "unknown".to_string(),
		};

		let token = InstructionTextToken::new(InstructionTextTokenContents::FloatingPoint, name);
		let wrap =
			InstructionTextToken::new(InstructionTextTokenContents::BeginMemoryOperand, "\"");

		self.add_space();
		self.buffer.push(wrap.clone());
		self.buffer.push(token);
		self.buffer.push(wrap);
	}

	pub fn add_import(&mut self, encoded: u32, list: &[Range<usize>]) {
		for name in Import::new(encoded) {
			self.add_constant(name, list);
		}
	}
}

impl From<TextBuilder> for Vec<InstructionTextToken> {
	fn from(builder: TextBuilder) -> Self {
		builder.buffer
	}
}
