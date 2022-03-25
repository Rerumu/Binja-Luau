use binaryninja::binaryview::{BinaryView, BinaryViewBase, BinaryViewExt};
use chumsky::prelude::{choice, filter, just};

const LUAU_VERSION: u8 = 2;

trait Parser<O> = chumsky::Parser<u8, O, Error = chumsky::error::Cheap<u8>> + Clone;

type Range = std::ops::Range<usize>;

#[repr(u8)]
enum TypeConstant {
	Nil = 0,
	Boolean,
	Number,
	String,
	Import,
	Table,
	Closure,
}

#[derive(Clone)]
pub struct Function {
	position: Range,
	code: Range,
	constant_list: Box<[Range]>,
	reference_list: Box<[usize]>,
}

impl Function {
	pub fn position(&self) -> Range {
		self.position.clone()
	}

	pub fn code(&self) -> Range {
		self.code.clone()
	}

	pub fn constant_list(&self) -> &[Range] {
		&self.constant_list
	}

	pub fn reference_list(&self) -> &[usize] {
		&self.reference_list
	}
}

pub struct Module {
	function_list: Box<[Function]>,
	string_list: Box<[Range]>,
	start_id: usize,
}

impl Module {
	pub fn function_list(&self) -> &[Function] {
		&self.function_list
	}

	pub fn string_list(&self) -> &[Range] {
		&self.string_list
	}

	pub fn entry_point(&self) -> u64 {
		let func = &self.function_list()[self.start_id];

		func.code().start as u64
	}
}

fn u8_parser() -> impl Parser<u8> {
	chumsky::prelude::any()
}

fn u32_parser() -> impl Parser<()> {
	u8_parser().ignored().repeated().exactly(4).ignored()
}

fn u64_parser() -> impl Parser<()> {
	u8_parser().ignored().repeated().exactly(8).ignored()
}

fn any_size_parser() -> impl Parser<usize> {
	fn to_integer(data: (Vec<u8>, u8)) -> usize {
		let head = data.0.into_iter();
		let tail = std::iter::once(data.1);

		head.into_iter()
			.chain(tail)
			.enumerate()
			.fold(0, |acc, (i, v)| acc | (v as usize & 0xFF) << (i * 7))
	}

	filter(|v| v & 0x80 != 0)
		.repeated()
		.then(u8_parser())
		.map(to_integer)
}

fn list_of_parser<P, O>(parser: P) -> impl Parser<Vec<O>>
where
	P: Parser<O>,
{
	let repeated = parser.repeated();

	any_size_parser().then_with(move |n| repeated.clone().exactly(n))
}

fn string_parser() -> impl Parser<Range> {
	list_of_parser(u8_parser().ignored()).map_with_span(|_, s| s)
}

fn meta_parser() -> impl Parser<()> {
	let max_stack_size = u8_parser();
	let num_param = u8_parser();
	let num_upval = u8_parser();
	let is_vararg = u8_parser();

	max_stack_size
		.then(num_param)
		.then(num_upval)
		.then(is_vararg)
		.ignored()
}

fn code_parser() -> impl Parser<Range> {
	any_size_parser().then_with(|n| {
		u32_parser()
			.ignored()
			.repeated()
			.exactly(n)
			.map_with_span(|_, s| s)
	})
}

fn constant_parser() -> impl Parser<Range> {
	let nil = just(TypeConstant::Nil as u8).ignored();

	let boolean = just(TypeConstant::Boolean as u8)
		.then(u8_parser())
		.ignored();

	let number = just(TypeConstant::Number as u8)
		.then(u64_parser())
		.ignored();

	let string = just(TypeConstant::String as u8)
		.then(any_size_parser())
		.ignored();

	let import = just(TypeConstant::Import as u8)
		.then(u32_parser())
		.ignored();

	let table = just(TypeConstant::Table as u8)
		.then(list_of_parser(any_size_parser().ignored()))
		.ignored();

	let closure = just(TypeConstant::Closure as u8)
		.then(any_size_parser())
		.ignored();

	choice((nil, boolean, number, string, import, table, closure)).map_with_span(|_, s| s)
}

fn line_info_parser(len: usize) -> impl Parser<()> {
	let line_gap = u8_parser();

	line_gap
		.then_with(move |g| {
			let interval = ((len - 1) >> g) + 1;
			let line_info = u8_parser().repeated().exactly(len);
			let abs_line_info = u32_parser().repeated().exactly(interval);

			line_info.then(abs_line_info)
		})
		.ignored()
}

fn loc_info_parser() -> impl Parser<()> {
	let name = any_size_parser();
	let start_pc = any_size_parser();
	let end_pc = any_size_parser();
	let reg = u8_parser();

	name.then(start_pc).then(end_pc).then(reg).ignored()
}

fn debug_info_parser(len: usize) -> impl Parser<()> {
	let line_info = line_info_parser(len);
	let loc_info = list_of_parser(loc_info_parser());
	let upv_info = list_of_parser(any_size_parser());

	let line_present = just(0).ignored().or(u8_parser().then(line_info).ignored());
	let var_present = just(0).ignored().or(loc_info.then(upv_info).ignored());

	line_present.then(var_present).ignored()
}

fn function_parser() -> impl Parser<Function> {
	type Data = ((Range, Vec<Range>), Vec<usize>);

	fn to_function(data: Data, position: Range) -> Function {
		Function {
			position,
			code: data.0 .0,
			constant_list: data.0 .1.into(),
			reference_list: data.1.into(),
		}
	}

	fn read_remaining(func: Function) -> impl Parser<Function> {
		debug_info_parser(func.code.len() / 4).to(func)
	}

	let meta = meta_parser();

	let code_list = code_parser();
	let constant_list = list_of_parser(constant_parser());
	let reference_list = list_of_parser(any_size_parser());

	let debug_name = any_size_parser();
	let line_defined = any_size_parser();

	meta.ignore_then(code_list)
		.then(constant_list)
		.then(reference_list)
		.then_ignore(debug_name)
		.then_ignore(line_defined)
		.map_with_span(to_function)
		.then_with(read_remaining)
}

fn module_parser() -> impl Parser<Module> {
	type Data = ((Vec<Range>, Vec<Function>), usize);

	fn to_module(data: Data) -> Module {
		Module {
			function_list: data.0 .1.into(),
			string_list: data.0 .0.into(),
			start_id: data.1,
		}
	}

	let version = just(LUAU_VERSION);
	let string_list = list_of_parser(string_parser());
	let proto_list = list_of_parser(function_parser());
	let entry_point = any_size_parser();

	version
		.ignore_then(string_list)
		.then(proto_list)
		.then(entry_point)
		.map(to_module)
}

pub fn parse(view: &BinaryView) -> Result<Module, ()> {
	let buffer = view
		.read_buffer(0, view.len())
		.expect("Failed to read buffer");

	module_parser().parse(buffer.get_data()).map_err(|_| ())
}
