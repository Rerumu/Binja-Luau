use std::cmp::Ordering;

pub type Range = std::ops::Range<usize>;

#[derive(Clone)]
pub struct Function {
	position: Range,
	code: Range,
	constant_list: Box<[Range]>,
	reference_list: Box<[usize]>,
}

impl Function {
	pub fn new(
		position: Range,
		code: Range,
		constant_list: Box<[Range]>,
		reference_list: Box<[usize]>,
	) -> Self {
		Self {
			position,
			code,
			constant_list,
			reference_list,
		}
	}

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

#[derive(Default)]
pub struct Module {
	function_list: Box<[Function]>,
	string_list: Box<[Range]>,
	start_id: usize,
}

fn cmp_range_to_usize(range: Range, value: usize) -> Ordering {
	if range.start > value {
		Ordering::Greater
	} else if range.end <= value {
		Ordering::Less
	} else {
		Ordering::Equal
	}
}

impl Module {
	pub fn new(function_list: Box<[Function]>, string_list: Box<[Range]>, start_id: usize) -> Self {
		Self {
			function_list,
			string_list,
			start_id,
		}
	}

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

	pub fn by_address(&self, addr: u64) -> Option<&Function> {
		let func_list = self.function_list();
		let addr = addr as usize;

		func_list
			.binary_search_by(|v| cmp_range_to_usize(v.position(), addr))
			.map(|v| &func_list[v])
			.ok()
	}
}
