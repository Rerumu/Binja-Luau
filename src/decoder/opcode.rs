use num_enum::TryFromPrimitive;

#[derive(Clone, Copy)]
pub enum OpName {
	A,
	B,
	C,
	D,
	E,
	X,
}

#[derive(Clone, Copy)]
pub enum OpType {
	Location,
	Register,
	UpValue,
	Boolean,
	Integer,
	Constant,
	Function,
	Import,
	BuiltIn,
}

#[repr(u8)]
#[derive(TryFromPrimitive, Clone, Copy)]
pub enum Opcode {
	Nop = 0,
	Break,

	LoadNil,
	LoadBoolean,
	LoadInteger,
	LoadConstant,

	Move,

	GetGlobal,
	SetGlobal,

	GetUpValue,
	SetUpValue,
	CloseUpValues,

	GetImport,

	GetTable,
	SetTable,
	GetTableKey,
	SetTableKey,
	GetTableIndex,
	SetTableIndex,

	NewClosure,

	NameCall,
	Call,
	Return,

	Jump,
	JumpSafe,

	JumpIfTruthy,
	JumpIfFalsy,
	JumpIfEqual,
	JumpIfLessEqual,
	JumpIfLessThan,
	JumpIfNotEqual,
	JumpIfMoreThan,
	JumpIfMoreEqual,

	Add,
	Sub,
	Mul,
	Div,
	Mod,
	Pow,

	AddConstant,
	SubConstant,
	MulConstant,
	DivConstant,
	ModConstant,
	PowConstant,

	And,
	Or,

	AndConstant,
	OrConstant,

	Concat,

	Not,
	Minus,
	Length,

	NewTable,
	DupTable,

	SetList,

	ForNumericPrep,
	ForNumericLoop,
	ForGenericLoop,

	ForGenericPrepINext,
	// DEPRECATED
	ForGenericLoopINext,

	ForGenericPrepNext,
	// DEPRECATED
	ForGenericLoopNext,

	GetVariadic,

	DupClosure,

	PrepVariadic,

	LoadConstantEx,

	JumpEx,

	FastCall,

	Coverage,
	Capture,

	// DEPRECATED
	JumpIfConstant,
	// DEPRECATED
	JumpIfNotConstant,

	FastCall1,
	FastCall2,
	FastCall2K,

	ForGenericPrep,

	JumpIfNil,
	JumpIfBoolean,
	JumpIfNumber,
	JumpIfString,
}

impl Opcode {
	pub const fn len(self) -> usize {
		match self {
			Self::GetGlobal
			| Self::SetGlobal
			| Self::GetImport
			| Self::GetTableKey
			| Self::SetTableKey
			| Self::NameCall
			| Self::JumpIfEqual
			| Self::JumpIfLessEqual
			| Self::JumpIfLessThan
			| Self::JumpIfNotEqual
			| Self::JumpIfMoreThan
			| Self::JumpIfMoreEqual
			| Self::NewTable
			| Self::SetList
			| Self::ForGenericLoop
			| Self::LoadConstantEx
			| Self::JumpIfConstant
			| Self::JumpIfNotConstant
			| Self::FastCall2
			| Self::FastCall2K
			| Self::JumpIfNil
			| Self::JumpIfBoolean
			| Self::JumpIfNumber
			| Self::JumpIfString => 8,
			_ => 4,
		}
	}

	pub const fn mnemonic(self) -> &'static str {
		match self {
			Self::Nop => "nop",
			Self::Break => "break",
			Self::LoadNil => "load_nil",
			Self::LoadBoolean => "load_boolean",
			Self::LoadInteger => "load_integer",
			Self::LoadConstant => "load_constant",
			Self::Move => "move",
			Self::GetGlobal => "get_global",
			Self::SetGlobal => "set_global",
			Self::GetUpValue => "get_upvalue",
			Self::SetUpValue => "set_upvalue",
			Self::CloseUpValues => "close_upvalues",
			Self::GetImport => "get_import",
			Self::GetTable => "get_table",
			Self::SetTable => "set_table",
			Self::GetTableKey => "get_table_key",
			Self::SetTableKey => "set_table_key",
			Self::GetTableIndex => "get_table_index",
			Self::SetTableIndex => "set_table_index",
			Self::NewClosure => "new_closure",
			Self::NameCall => "name_call",
			Self::Call => "call",
			Self::Return => "return",
			Self::Jump => "jump",
			Self::JumpSafe => "jump_safe",
			Self::JumpIfTruthy => "jump_if_truthy",
			Self::JumpIfFalsy => "jump_if_falsy",
			Self::JumpIfEqual => "jump_if_equal",
			Self::JumpIfLessEqual => "jump_if_less_equal",
			Self::JumpIfLessThan => "jump_if_less_than",
			Self::JumpIfNotEqual => "jump_if_not_equal",
			Self::JumpIfMoreThan => "jump_if_more_than",
			Self::JumpIfMoreEqual => "jump_if_more_equal",
			Self::Add => "add",
			Self::Sub => "sub",
			Self::Mul => "mul",
			Self::Div => "div",
			Self::Mod => "mod",
			Self::Pow => "pow",
			Self::AddConstant => "add_constant",
			Self::SubConstant => "sub_constant",
			Self::MulConstant => "mul_constant",
			Self::DivConstant => "div_constant",
			Self::ModConstant => "mod_constant",
			Self::PowConstant => "pow_constant",
			Self::And => "and",
			Self::Or => "or",
			Self::AndConstant => "and_constant",
			Self::OrConstant => "or_constant",
			Self::Concat => "concat",
			Self::Not => "not",
			Self::Minus => "minus",
			Self::Length => "length",
			Self::NewTable => "new_table",
			Self::DupTable => "dup_table",
			Self::SetList => "set_list",
			Self::ForNumericPrep => "for_numeric_prep",
			Self::ForNumericLoop => "for_numeric_loop",
			Self::ForGenericLoop => "for_generic_loop",
			Self::ForGenericPrepINext => "for_generic_prep_i_next",
			Self::ForGenericLoopINext => "for_generic_loop_i_next",
			Self::ForGenericPrepNext => "for_generic_prep_next",
			Self::ForGenericLoopNext => "for_generic_loop_next",
			Self::GetVariadic => "get_variadic",
			Self::DupClosure => "dup_closure",
			Self::PrepVariadic => "prep_variadic",
			Self::LoadConstantEx => "load_constant_ex",
			Self::JumpEx => "jump_ex",
			Self::FastCall => "fast_call",
			Self::Coverage => "coverage",
			Self::Capture => "capture",
			Self::JumpIfConstant => "jump_if_constant",
			Self::JumpIfNotConstant => "jump_if_not_constant",
			Self::FastCall1 => "fast_call1",
			Self::FastCall2 => "fast_call2",
			Self::FastCall2K => "fast_call2_k",
			Self::ForGenericPrep => "for_generic_prep",
			Self::JumpIfNil => "jump_if_nil",
			Self::JumpIfBoolean => "jump_if_boolean",
			Self::JumpIfNumber => "jump_if_number",
			Self::JumpIfString => "jump_if_string",
		}
	}

	#[allow(clippy::match_same_arms)]
	const fn name_list(self) -> &'static [OpName] {
		use OpName::{A, B, C, D, E, X};

		match self {
			Self::Nop => &[],
			Self::Break => &[],
			Self::LoadNil => &[A],
			Self::LoadBoolean => &[A, B, C],
			Self::LoadInteger => &[A, D],
			Self::LoadConstant => &[A, B],
			Self::Move => &[A, B],
			Self::GetGlobal => &[A, X],
			Self::SetGlobal => &[A, X],
			Self::GetUpValue => &[A, B],
			Self::SetUpValue => &[A, B],
			Self::CloseUpValues => &[A],
			Self::GetImport => &[A, D, X],
			Self::GetTable => &[A, B, C],
			Self::SetTable => &[A, B, C],
			Self::GetTableKey => &[A, B, X],
			Self::SetTableKey => &[A, B, X],
			Self::GetTableIndex => &[A, B, C],
			Self::SetTableIndex => &[A, B, C],
			Self::NewClosure => &[A, D],
			Self::NameCall => &[A, B, X],
			Self::Call => &[A, B, C],
			Self::Return => &[A, B],
			Self::Jump => &[D],
			Self::JumpSafe => &[D],
			Self::JumpIfTruthy => &[A, D],
			Self::JumpIfFalsy => &[A, D],
			Self::JumpIfEqual => &[A, X, D],
			Self::JumpIfLessEqual => &[A, X, D],
			Self::JumpIfLessThan => &[A, X, D],
			Self::JumpIfNotEqual => &[A, X, D],
			Self::JumpIfMoreThan => &[A, X, D],
			Self::JumpIfMoreEqual => &[A, X, D],
			Self::Add => &[A, B, C],
			Self::Sub => &[A, B, C],
			Self::Mul => &[A, B, C],
			Self::Div => &[A, B, C],
			Self::Mod => &[A, B, C],
			Self::Pow => &[A, B, C],
			Self::AddConstant => &[A, B, C],
			Self::SubConstant => &[A, B, C],
			Self::MulConstant => &[A, B, C],
			Self::DivConstant => &[A, B, C],
			Self::ModConstant => &[A, B, C],
			Self::PowConstant => &[A, B, C],
			Self::And => &[A, B, C],
			Self::Or => &[A, B, C],
			Self::AndConstant => &[A, B, C],
			Self::OrConstant => &[A, B, C],
			Self::Concat => &[A, B, C],
			Self::Not => &[A, B],
			Self::Minus => &[A, B],
			Self::Length => &[A, B],
			Self::NewTable => &[A, B, X],
			Self::DupTable => &[A, D],
			Self::SetList => &[A, B, C, X],
			Self::ForNumericPrep => &[A, D],
			Self::ForNumericLoop => &[A, D],
			Self::ForGenericLoop => &[A, X, D],
			Self::ForGenericPrepINext => &[A, D],
			Self::ForGenericLoopINext => &[A, D],
			Self::ForGenericPrepNext => &[A, D],
			Self::ForGenericLoopNext => &[A, D],
			Self::GetVariadic => &[A, B],
			Self::DupClosure => &[A, D],
			Self::PrepVariadic => &[A],
			Self::LoadConstantEx => &[A, X],
			Self::JumpEx => &[E],
			Self::FastCall => &[A, C],
			Self::Coverage => &[E],
			Self::Capture => &[],
			Self::JumpIfConstant => &[A, X, D],
			Self::JumpIfNotConstant => &[A, X, D],
			Self::FastCall1 => &[A, B, C],
			Self::FastCall2 => &[A, B, X, C],
			Self::FastCall2K => &[A, B, X, C],
			Self::ForGenericPrep => &[A, D],
			Self::JumpIfNil => &[A, D, X],
			Self::JumpIfBoolean => &[A, D, X],
			Self::JumpIfNumber => &[A, D, X],
			Self::JumpIfString => &[A, D, X],
		}
	}

	#[allow(clippy::match_same_arms)]
	const fn type_list(self) -> &'static [OpType] {
		use OpType::{
			Boolean, BuiltIn, Constant, Function, Import, Integer, Location, Register, UpValue,
		};

		match self {
			Self::Nop => &[],
			Self::Break => &[],
			Self::LoadNil => &[Register],
			Self::LoadBoolean => &[Register, Boolean, Location],
			Self::LoadInteger => &[Register, Integer],
			Self::LoadConstant => &[Register, Constant],
			Self::Move => &[Register, Register],
			Self::GetGlobal => &[Register, Constant],
			Self::SetGlobal => &[Register, Constant],
			Self::GetUpValue => &[Register, UpValue],
			Self::SetUpValue => &[Register, UpValue],
			Self::CloseUpValues => &[Register],
			Self::GetImport => &[Register, Constant, Import],
			Self::GetTable => &[Register, Register, Register],
			Self::SetTable => &[Register, Register, Register],
			Self::GetTableKey => &[Register, Register, Constant],
			Self::SetTableKey => &[Register, Register, Constant],
			Self::GetTableIndex => &[Register, Register, Integer],
			Self::SetTableIndex => &[Register, Register, Integer],
			Self::NewClosure => &[Register, Function],
			Self::NameCall => &[Register, Register, Constant],
			Self::Call => &[Register, Integer, Integer],
			Self::Return => &[Register, Integer],
			Self::Jump => &[Location],
			Self::JumpSafe => &[Location],
			Self::JumpIfTruthy => &[Register, Location],
			Self::JumpIfFalsy => &[Register, Location],
			Self::JumpIfEqual => &[Register, Register, Location],
			Self::JumpIfLessEqual => &[Register, Register, Location],
			Self::JumpIfLessThan => &[Register, Register, Location],
			Self::JumpIfNotEqual => &[Register, Register, Location],
			Self::JumpIfMoreThan => &[Register, Register, Location],
			Self::JumpIfMoreEqual => &[Register, Register, Location],
			Self::Add => &[Register, Register, Register],
			Self::Sub => &[Register, Register, Register],
			Self::Mul => &[Register, Register, Register],
			Self::Div => &[Register, Register, Register],
			Self::Mod => &[Register, Register, Register],
			Self::Pow => &[Register, Register, Register],
			Self::AddConstant => &[Register, Register, Constant],
			Self::SubConstant => &[Register, Register, Constant],
			Self::MulConstant => &[Register, Register, Constant],
			Self::DivConstant => &[Register, Register, Constant],
			Self::ModConstant => &[Register, Register, Constant],
			Self::PowConstant => &[Register, Register, Constant],
			Self::And => &[Register, Register, Register],
			Self::Or => &[Register, Register, Register],
			Self::AndConstant => &[Register, Register, Constant],
			Self::OrConstant => &[Register, Register, Constant],
			Self::Concat => &[Register, Register, Register],
			Self::Not => &[Register, Register],
			Self::Minus => &[Register, Register],
			Self::Length => &[Register, Register],
			Self::NewTable => &[Register, Integer, Integer],
			Self::DupTable => &[Register, Constant],
			Self::SetList => &[Register, Register, Integer, Integer],
			Self::ForNumericPrep => &[Register, Location],
			Self::ForNumericLoop => &[Register, Location],
			Self::ForGenericLoop => &[Register, Integer, Location],
			Self::ForGenericPrepINext => &[Register, Location],
			Self::ForGenericLoopINext => &[Register, Location],
			Self::ForGenericPrepNext => &[Register, Location],
			Self::ForGenericLoopNext => &[Register, Location],
			Self::GetVariadic => &[Register, Integer],
			Self::DupClosure => &[Register, Constant],
			Self::PrepVariadic => &[Integer],
			Self::LoadConstantEx => &[Register, Constant],
			Self::JumpEx => &[Location],
			Self::FastCall => &[BuiltIn, Location],
			Self::Coverage => &[Integer],
			Self::Capture => &[],
			Self::JumpIfConstant => &[Register, Constant, Location],
			Self::JumpIfNotConstant => &[Register, Constant, Location],
			Self::FastCall1 => &[BuiltIn, Register, Location],
			Self::FastCall2 => &[BuiltIn, Register, Register, Location],
			Self::FastCall2K => &[BuiltIn, Register, Constant, Location],
			Self::ForGenericPrep => &[Register, Location],
			Self::JumpIfNil => &[Register, Location, Integer],
			Self::JumpIfBoolean => &[Register, Location, Integer],
			Self::JumpIfNumber => &[Register, Location, Integer],
			Self::JumpIfString => &[Register, Location, Integer],
		}
	}

	pub fn iter_operands(self) -> impl Iterator<Item = (OpName, OpType)> {
		self.name_list()
			.iter()
			.copied()
			.zip(self.type_list().iter().copied())
	}
}
