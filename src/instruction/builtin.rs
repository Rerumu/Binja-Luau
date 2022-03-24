#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum BuiltIn {
	Assert = 1,

	Abs,
	Acos,
	Asin,
	Atan2,
	Atan,
	Ceil,
	Cosh,
	Cos,
	Deg,
	Exp,
	Floor,
	Fmod,
	Frexp,
	Ldexp,
	Log10,
	Log,
	Max,
	Min,
	Modf,
	Pow,
	Rad,
	Sinh,
	Sin,
	Sqrt,
	Tanh,
	Tan,

	Arshift,
	Band,
	Bnot,
	Bor,
	Bxor,
	Btest,
	Extract,
	Lrotate,
	Lshift,
	Replace,
	Rrotate,
	Rshift,

	Type,

	Byte,
	Char,
	Len,

	Typeof,

	Sub,

	Clamp,
	Sign,
	Round,

	Rawset,
	Rawget,
	Rawequal,

	Tinsert,
	Tunpack,

	Vector,

	Countlz,
	Countrz,

	Select,
}

impl BuiltIn {
	fn name(self) -> &'static str {
		match self {
			BuiltIn::Assert => "assert",
			BuiltIn::Abs => "math.abs",
			BuiltIn::Acos => "math.acos",
			BuiltIn::Asin => "math.asin",
			BuiltIn::Atan2 => "math.atan2",
			BuiltIn::Atan => "math.atan",
			BuiltIn::Ceil => "math.ceil",
			BuiltIn::Cosh => "math.cosh",
			BuiltIn::Cos => "math.cos",
			BuiltIn::Deg => "math.deg",
			BuiltIn::Exp => "math.exp",
			BuiltIn::Floor => "math.floor",
			BuiltIn::Fmod => "math.fmod",
			BuiltIn::Frexp => "math.frexp",
			BuiltIn::Ldexp => "math.ldexp",
			BuiltIn::Log10 => "math.log10",
			BuiltIn::Log => "math.log",
			BuiltIn::Max => "math.max",
			BuiltIn::Min => "math.min",
			BuiltIn::Modf => "math.modf",
			BuiltIn::Pow => "math.pow",
			BuiltIn::Rad => "math.rad",
			BuiltIn::Sinh => "math.sinh",
			BuiltIn::Sin => "math.sin",
			BuiltIn::Sqrt => "math.sqrt",
			BuiltIn::Tanh => "math.tanh",
			BuiltIn::Tan => "math.tan",
			BuiltIn::Arshift => "bit32.arshift",
			BuiltIn::Band => "bit32.band",
			BuiltIn::Bnot => "bit32.bnot",
			BuiltIn::Bor => "bit32.bor",
			BuiltIn::Bxor => "bit32.bxor",
			BuiltIn::Btest => "bit32.btest",
			BuiltIn::Extract => "bit32.extract",
			BuiltIn::Lrotate => "bit32.lrotate",
			BuiltIn::Lshift => "bit32.lshift",
			BuiltIn::Replace => "bit32.replace",
			BuiltIn::Rrotate => "bit32.rrotate",
			BuiltIn::Rshift => "bit32.rshift",
			BuiltIn::Type => "type",
			BuiltIn::Byte => "string.byte",
			BuiltIn::Char => "string.char",
			BuiltIn::Len => "string.len",
			BuiltIn::Typeof => "typeof",
			BuiltIn::Sub => "string.sub",
			BuiltIn::Clamp => "math.clamp",
			BuiltIn::Sign => "math.sign",
			BuiltIn::Round => "math.round",
			BuiltIn::Rawset => "rawset",
			BuiltIn::Rawget => "rawget",
			BuiltIn::Rawequal => "rawequal",
			BuiltIn::Tinsert => "table.insert",
			BuiltIn::Tunpack => "table.unpack",
			BuiltIn::Vector => "vector",
			BuiltIn::Countlz => "bit32.countlz",
			BuiltIn::Countrz => "bit32.countrz",
			BuiltIn::Select => "select",
		}
	}
}

impl TryFrom<u8> for BuiltIn {
	type Error = ();

	fn try_from(other: u8) -> Result<Self, Self::Error> {
		let ok = other <= Self::Select as u8;

		ok.then(|| unsafe { std::mem::transmute(other) }).ok_or(())
	}
}

impl std::fmt::Display for BuiltIn {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.write_str(self.name())
	}
}
