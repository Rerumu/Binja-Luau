use binaryninja::architecture::{InstructionTextToken, InstructionTextTokenContents};

use crate::instruction::{builtin::BuiltIn, decoder::get_jump_target, opcode::Opcode};

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

	pub fn add_built_in(&mut self, index: i32) {
		let name = match u8::try_from(index)
			.ok()
			.and_then(|v| BuiltIn::try_from(v).ok())
		{
			Some(v) => v.to_string(),
			None => "unknown".to_string(),
		};

		let token = InstructionTextToken::new(InstructionTextTokenContents::Instruction, name);

		self.add_space();
		self.buffer.push(token);
	}
}

impl From<TextBuilder> for Vec<InstructionTextToken> {
	fn from(builder: TextBuilder) -> Self {
		builder.buffer
	}
}