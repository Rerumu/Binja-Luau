use binaryninja::string::BnString;

use crate::{
	decoder::{builtin::BuiltIn, import::Import, inst::Inst, opcode::Opcode},
	file::data::{Function, Module, Range, Value},
};

type TextToken = binaryninja::disassembly::InstructionTextToken;
type TextContent = binaryninja::disassembly::InstructionTextTokenContents;

macro_rules! surrounded {
	($lhs:literal, $infix:expr, $rhs:literal) => {{
		let begin = TextToken::new(BnString::new($lhs), TextContent::BeginMemoryOperand);
		let end = TextToken::new(BnString::new($rhs), TextContent::EndMemoryOperand);

		[begin, $infix, end]
	}};
}

macro_rules! bn_format {
	($($arg:tt)*) => {{
		let data = format!($($arg)*);

		BnString::new(data)
	}};
}

fn new_padding_for(name: &str) -> String {
	const MAX_PADDING: usize = Opcode::PrepVariadic.mnemonic().len() + 1;
	let len = name.len();

	" ".repeat(MAX_PADDING.saturating_sub(len).max(1))
}

pub struct TextBuilder {
	buffer: Vec<TextToken>,
}

impl TextBuilder {
	pub fn with_mnemonic(opcode: Opcode) -> Self {
		let name = opcode.mnemonic();
		let padding = new_padding_for(name);

		Self {
			buffer: vec![
				TextToken::new(BnString::new(name), TextContent::Instruction),
				TextToken::new(BnString::new(padding), TextContent::Text),
			],
		}
	}

	pub fn add_separator(&mut self) {
		self.buffer.push(TextToken::new(
			BnString::new(", "),
			TextContent::OperandSeparator,
		));
	}

	pub fn add_location(&mut self, addr: u64, offset: i64) {
		let target = Inst::get_jump_target(addr, offset);
		let token = TextToken::new(
			bn_format!("{offset:+}"),
			TextContent::PossibleAddress(target),
		);

		self.buffer.push(token);
		self.add_separator();
	}

	pub fn add_register(&mut self, register: u8) {
		let token = TextToken::new(bn_format!("r{register}"), TextContent::Register);

		self.buffer.push(token);
		self.add_separator();
	}

	pub fn add_upvalue(&mut self, upvalue: u8) {
		let token = TextToken::new(bn_format!("u{upvalue}"), TextContent::Register);

		self.buffer.push(token);
		self.add_separator();
	}

	fn add_named_integer(&mut self, name: &str) {
		let token = TextToken::new(BnString::new(name), TextContent::Integer(0));

		self.buffer.push(token);
		self.add_separator();
	}

	pub fn add_boolean(&mut self, value: bool) {
		let name = if value { "true" } else { "false" };

		self.add_named_integer(name);
	}

	pub fn add_integer(&mut self, value: i32) {
		let name = format!("{value}_i32");

		self.add_named_integer(&name);
	}

	fn add_number(&mut self, value: f64) {
		let token = TextToken::new(bn_format!("{value}_f64"), TextContent::FloatingPoint);

		self.buffer.push(token);
		self.add_separator();
	}

	fn add_string(&mut self, index: usize, str_list: &[Range]) -> Option<()> {
		if index == 0 {
			self.add_named_integer("no_string");

			return Some(());
		}

		let adjusted = index - 1;
		let address = str_list.get(adjusted)?.start;

		let list = surrounded!(
			"[",
			TextToken::new(
				bn_format!("str_{adjusted}"),
				TextContent::PossibleAddress(address as u64),
			),
			"]"
		);

		self.buffer.extend(list);
		self.add_separator();

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
			Value::Table => self.add_named_integer("any_table"),
		};

		Some(())
	}

	pub fn add_built_in(&mut self, index: u8) -> Option<()> {
		let name = BuiltIn::try_from(index).ok()?.name();
		let list = surrounded!(
			"\"",
			TextToken::new(BnString::new(name), TextContent::FloatingPoint,),
			"\""
		);

		self.buffer.extend(list);
		self.add_separator();

		Some(())
	}

	pub fn add_function(&mut self, index: usize, global: &[Function]) -> Option<()> {
		let target = global.get(index)?.code().start as u64;

		let list = surrounded!(
			"[",
			TextToken::new(
				bn_format!("func_{index}"),
				TextContent::PossibleAddress(target),
			),
			"]"
		);

		self.buffer.extend(list);
		self.add_separator();

		Some(())
	}

	pub fn add_import(&mut self, encoded: u32, func: &Function, parent: &Module) -> Option<()> {
		let list = &func.constant_list().data;

		for name in Import::from(encoded) {
			let value = list.get(name)?;

			self.add_constant(value, func, parent)?;
		}

		Some(())
	}
}

impl From<TextBuilder> for Vec<TextToken> {
	fn from(mut builder: TextBuilder) -> Self {
		builder.buffer.pop().unwrap();

		builder.buffer
	}
}
