use std::{
	io::{Cursor, Error, ErrorKind, Read},
	ops::Range,
};

use binaryninja::binaryview::{BinaryView, BinaryViewBase, BinaryViewExt};
use num_enum::TryFromPrimitive;

use super::data::{Function, List, Module, Value};

type PResult<T> = std::io::Result<T>;
type Stream<'a> = Cursor<&'a [u8]>;

const LUAU_VERSION: u8 = 3;

#[repr(u8)]
#[derive(TryFromPrimitive)]
enum TypeConstant {
	Nil = 0,
	Boolean,
	Number,
	String,
	Import,
	Table,
	Closure,
}

fn position_of(stream: &Stream) -> usize {
	stream.position().try_into().expect("Position out of range")
}

fn parse_u8(s: &mut Stream) -> PResult<u8> {
	let mut buf = [0_u8; 1];

	s.read_exact(&mut buf)?;

	Ok(buf[0])
}

fn parse_u32(s: &mut Stream) -> PResult<u32> {
	let mut buf = [0_u8; 4];

	s.read_exact(&mut buf)?;

	Ok(u32::from_le_bytes(buf))
}

fn parse_u64(s: &mut Stream) -> PResult<u64> {
	let mut buf = [0_u8; 8];

	s.read_exact(&mut buf)?;

	Ok(u64::from_le_bytes(buf))
}

fn parse_any_size(s: &mut Stream) -> PResult<usize> {
	let mut result = 0;

	for i in 0.. {
		let v = parse_u8(s)? as usize;

		result |= (v & 0x7F) << (i * 7);

		if v & 0x80 == 0 {
			break;
		}
	}

	Ok(result)
}

fn parse_list_of<P, O>(s: &mut Stream, parse: P) -> PResult<List<O>>
where
	P: Fn(&mut Stream) -> PResult<O>,
{
	let start = position_of(s);
	let len = parse_any_size(s)?;
	let mut temp = Vec::with_capacity(len);

	for _ in 0..len {
		temp.push(parse(s)?);
	}

	let end = position_of(s);

	Ok(List {
		data: temp.into(),
		range: start..end,
	})
}

fn parse_list_ignored<P, O>(s: &mut Stream, parse: P) -> PResult<()>
where
	P: Fn(&mut Stream) -> PResult<O>,
{
	for _ in 0..parse_any_size(s)? {
		parse(s)?;
	}

	Ok(())
}

fn parse_string_data(s: &mut Stream) -> PResult<Range<usize>> {
	let len = parse_any_size(s)?;
	let start = position_of(s);

	for _ in 0..len {
		parse_u8(s)?;
	}

	Ok(start..position_of(s))
}

fn parse_func_meta_data(s: &mut Stream) -> PResult<()> {
	let _max_stack_size = parse_u8(s)?;
	let _num_param = parse_u8(s)?;
	let _num_upval = parse_u8(s)?;
	let _is_vararg = parse_u8(s)?;

	Ok(())
}

fn parse_code(s: &mut Stream) -> PResult<Range<usize>> {
	let len = parse_any_size(s)?;
	let start = position_of(s);

	for _ in 0..len {
		parse_u32(s)?;
	}

	Ok(start..position_of(s))
}

fn parse_constant(s: &mut Stream) -> PResult<Value> {
	let tag = parse_u8(s)?
		.try_into()
		.map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid constant tag"))?;

	let value = match tag {
		TypeConstant::Nil => Value::Nil,
		TypeConstant::Boolean => {
			if parse_u8(s)? == 0 {
				Value::False
			} else {
				Value::True
			}
		}
		TypeConstant::Number => {
			let temp = parse_u64(s)?;
			let data = f64::from_bits(temp);

			Value::Number(data)
		}
		TypeConstant::String => {
			let index = parse_any_size(s)?;

			Value::String(index)
		}
		TypeConstant::Import => {
			let data = parse_u32(s)?;

			Value::Import(data)
		}
		TypeConstant::Table => {
			parse_list_ignored(s, parse_any_size)?;

			Value::Table
		}
		TypeConstant::Closure => {
			let index = parse_any_size(s)?;

			Value::Closure(index)
		}
	};

	Ok(value)
}

fn parse_line_info(len: usize, s: &mut Stream) -> PResult<()> {
	let line_gap = parse_u8(s)?;
	let interval = ((len - 1) >> line_gap) + 1;

	for _ in 0..len {
		parse_u8(s)?;
	}

	for _ in 0..interval {
		parse_u32(s)?;
	}

	Ok(())
}

fn parse_local_info(s: &mut Stream) -> PResult<()> {
	let _name = parse_any_size(s)?;
	let _start_pc = parse_any_size(s)?;
	let _end_pc = parse_any_size(s)?;
	let _reg = parse_u8(s)?;

	Ok(())
}

fn parse_debug_info(len: usize, s: &mut Stream) -> PResult<()> {
	let has_line_info = parse_u8(s)? != 0;

	if has_line_info {
		parse_line_info(len, s)?;
	}

	let has_var_info = parse_u8(s)? != 0;

	if has_var_info {
		parse_list_ignored(s, parse_local_info)?;
		parse_list_ignored(s, parse_any_size)?;
	}

	Ok(())
}

fn parse_function(s: &mut Stream) -> PResult<Function> {
	let start = position_of(s);

	parse_func_meta_data(s)?;

	let code = parse_code(s)?;
	let constant_list = parse_list_of(s, parse_constant)?;
	let reference_list = parse_list_of(s, parse_any_size)?;
	let _line_defined = parse_any_size(s)?;
	let debug_name = parse_any_size(s)?;

	parse_debug_info(code.len() / 4, s)?;

	let end = position_of(s);

	Ok(Function::new(
		start..end,
		debug_name,
		code,
		constant_list,
		reference_list,
	))
}

fn parse_module(s: &mut Stream) -> PResult<Module> {
	let version = parse_u8(s)?;

	if version != LUAU_VERSION {
		return Err(Error::new(ErrorKind::InvalidData, "Invalid module version"));
	}

	let string_list = parse_list_of(s, parse_string_data)?;
	let function_list = parse_list_of(s, parse_function)?;
	let entry_point = parse_any_size(s)?;

	Ok(Module::new(function_list, string_list, entry_point))
}

pub fn parse(view: &BinaryView) -> Result<Module, ()> {
	let buffer = view
		.read_buffer(0, view.len())
		.expect("Failed to read buffer");

	let mut cursor = Cursor::new(buffer.get_data());

	parse_module(&mut cursor).map_err(drop)
}
