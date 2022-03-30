use binaryninja::architecture::{InstructionTextToken, InstructionTextTokenContents};

use crate::{
	decoder::{
		inst::get_jump_target, opcode::Opcode, ref_known::RefKnown, ref_unknown::RefUnknown,
	},
	file::data::{Function, Module, Range, Value},
};

const MAX_PADDING: usize = 8;

#[derive(Default)]
pub struct TextBuilder {
	buffer: Vec<InstructionTextToken>,
}

impl TextBuilder {
	pub fn new() -> Self {
		Self::default()
	}

	fn set_adjustment(&mut self) {
		let mut iter = self.buffer.iter().enumerate();
		let mut adjustment = Vec::new();
		let mut accumulated = 0;

		while let Some((i, token)) = iter.next() {
			accumulated += token.text().to_bytes().len();

			if token.contents() == InstructionTextTokenContents::BeginMemoryOperand {
				let after = iter.next().unwrap().1;

				accumulated += after.text().to_bytes().len();

				continue;
			}

			let remain = MAX_PADDING - accumulated % MAX_PADDING;

			adjustment.push((i, remain));
			accumulated = 0;
		}

		for (i, remain) in adjustment.into_iter().rev() {
			let pad = " ".repeat(remain);
			let token =
				InstructionTextToken::new(InstructionTextTokenContents::OperandSeparator, pad);

			self.buffer.insert(i + 1, token);
		}
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

		self.buffer.push(token);
	}

	pub fn add_register(&mut self, reg: u8) {
		let token =
			InstructionTextToken::new(InstructionTextTokenContents::Register, format!("r{reg}"));

		self.buffer.push(token);
	}

	pub fn add_upvalue(&mut self, upv: u8) {
		let token =
			InstructionTextToken::new(InstructionTextTokenContents::Register, format!("u{upv}"));

		self.buffer.push(token);
	}

	fn add_memory_begin(&mut self, what: &str) {
		let token =
			InstructionTextToken::new(InstructionTextTokenContents::BeginMemoryOperand, what);

		self.buffer.push(token);
	}

	fn add_memory_end(&mut self, what: &str) {
		let token = InstructionTextToken::new(InstructionTextTokenContents::EndMemoryOperand, what);

		self.buffer.push(token);
	}

	fn add_named_integer(&mut self, name: &str) {
		let token = InstructionTextToken::new(InstructionTextTokenContents::Integer(0), name);

		self.add_memory_begin("(");
		self.buffer.push(token);
		self.add_memory_end(")");
	}

	pub fn add_boolean(&mut self, value: bool) {
		let name = if value { "true" } else { "false" };

		self.add_named_integer(name);
	}

	pub fn add_integer(&mut self, value: i32) {
		let name = value.to_string();

		self.add_named_integer(&name);
	}

	fn add_number(&mut self, value: f64) {
		let token = InstructionTextToken::new(
			InstructionTextTokenContents::FloatingPoint,
			value.to_string(),
		);

		self.add_memory_begin("(");
		self.buffer.push(token);
		self.add_memory_end(")");
	}

	fn add_string(&mut self, index: usize, str_list: &[Range]) -> Option<()> {
		if index == 0 {
			self.add_named_integer("no_string");

			return Some(());
		}

		let adjusted = index - 1;
		let address = str_list.get(adjusted)?.start;

		let token = InstructionTextToken::new(
			InstructionTextTokenContents::PossibleAddress(address as u64),
			format!("str_{adjusted}"),
		);

		self.add_memory_begin("[");
		self.buffer.push(token);
		self.add_memory_end("]");

		Some(())
	}

	pub fn add_constant(&mut self, value: &Value, func: &Function, parent: &Module) -> Option<()> {
		match value {
			Value::Nil => self.add_named_integer("nil"),
			Value::False => self.add_boolean(false),
			Value::True => self.add_boolean(true),
			Value::Number(n) => self.add_number(*n),
			Value::String(index) => self.add_string(*index, &parent.string_list().data)?,
			Value::Closure(index) => {
				let global = &parent.function_list().data;

				self.add_function(*index, global)?;
			}
			Value::Import(data) => self.add_import(*data, func, parent)?,
			Value::Table => self.add_named_integer("table"),
		};

		Some(())
	}

	pub fn add_built_in(&mut self, index: u8) -> Option<()> {
		let name = RefKnown::try_from(index).ok()?.name();
		let token = InstructionTextToken::new(InstructionTextTokenContents::FloatingPoint, name);

		self.add_memory_begin("\"");
		self.buffer.push(token);
		self.add_memory_end("\"");

		Some(())
	}

	pub fn add_function(&mut self, index: usize, global: &[Function]) -> Option<()> {
		let target = global.get(index)?.code().start as u64;

		let token = InstructionTextToken::new(
			InstructionTextTokenContents::PossibleAddress(target),
			format!("func_{index}"),
		);

		self.buffer.push(token);

		Some(())
	}

	pub fn add_import(&mut self, encoded: u32, func: &Function, parent: &Module) -> Option<()> {
		let list = &func.constant_list().data;

		for name in RefUnknown::from(encoded) {
			let value = list.get(name)?;

			self.add_constant(value, func, parent)?;
		}

		Some(())
	}
}

impl From<TextBuilder> for Vec<InstructionTextToken> {
	fn from(mut builder: TextBuilder) -> Self {
		builder.set_adjustment();

		builder.buffer
	}
}
