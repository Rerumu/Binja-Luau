#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum RefKnown {
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

impl RefKnown {
	fn name(self) -> &'static str {
		match self {
			RefKnown::Assert => "assert",
			RefKnown::Abs => "math.abs",
			RefKnown::Acos => "math.acos",
			RefKnown::Asin => "math.asin",
			RefKnown::Atan2 => "math.atan2",
			RefKnown::Atan => "math.atan",
			RefKnown::Ceil => "math.ceil",
			RefKnown::Cosh => "math.cosh",
			RefKnown::Cos => "math.cos",
			RefKnown::Deg => "math.deg",
			RefKnown::Exp => "math.exp",
			RefKnown::Floor => "math.floor",
			RefKnown::Fmod => "math.fmod",
			RefKnown::Frexp => "math.frexp",
			RefKnown::Ldexp => "math.ldexp",
			RefKnown::Log10 => "math.log10",
			RefKnown::Log => "math.log",
			RefKnown::Max => "math.max",
			RefKnown::Min => "math.min",
			RefKnown::Modf => "math.modf",
			RefKnown::Pow => "math.pow",
			RefKnown::Rad => "math.rad",
			RefKnown::Sinh => "math.sinh",
			RefKnown::Sin => "math.sin",
			RefKnown::Sqrt => "math.sqrt",
			RefKnown::Tanh => "math.tanh",
			RefKnown::Tan => "math.tan",
			RefKnown::Arshift => "bit32.arshift",
			RefKnown::Band => "bit32.band",
			RefKnown::Bnot => "bit32.bnot",
			RefKnown::Bor => "bit32.bor",
			RefKnown::Bxor => "bit32.bxor",
			RefKnown::Btest => "bit32.btest",
			RefKnown::Extract => "bit32.extract",
			RefKnown::Lrotate => "bit32.lrotate",
			RefKnown::Lshift => "bit32.lshift",
			RefKnown::Replace => "bit32.replace",
			RefKnown::Rrotate => "bit32.rrotate",
			RefKnown::Rshift => "bit32.rshift",
			RefKnown::Type => "type",
			RefKnown::Byte => "string.byte",
			RefKnown::Char => "string.char",
			RefKnown::Len => "string.len",
			RefKnown::Typeof => "typeof",
			RefKnown::Sub => "string.sub",
			RefKnown::Clamp => "math.clamp",
			RefKnown::Sign => "math.sign",
			RefKnown::Round => "math.round",
			RefKnown::Rawset => "rawset",
			RefKnown::Rawget => "rawget",
			RefKnown::Rawequal => "rawequal",
			RefKnown::Tinsert => "table.insert",
			RefKnown::Tunpack => "table.unpack",
			RefKnown::Vector => "vector",
			RefKnown::Countlz => "bit32.countlz",
			RefKnown::Countrz => "bit32.countrz",
			RefKnown::Select => "select",
		}
	}
}

impl TryFrom<u8> for RefKnown {
	type Error = ();

	fn try_from(other: u8) -> Result<Self, Self::Error> {
		let ok = other <= Self::Select as u8;

		ok.then(|| unsafe { std::mem::transmute(other) }).ok_or(())
	}
}

impl std::fmt::Display for RefKnown {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		f.write_str(self.name())
	}
}
