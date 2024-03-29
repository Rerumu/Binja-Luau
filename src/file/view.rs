use std::{ops::Range, sync::RwLock};

use binaryninja::{
	architecture::{ArchitectureExt, CoreArchitecture},
	binaryview::{BinaryView, BinaryViewBase, BinaryViewExt, Result as BResult},
	custombinaryview::{
		BinaryViewType, BinaryViewTypeBase, CustomBinaryView, CustomBinaryViewType, CustomView,
		CustomViewBuilder,
	},
	rc::Ref,
	section::{Section, Semantics},
	segment::Segment,
	symbol::{Symbol, SymbolType},
	types::Type,
	Endianness,
};
use once_cell::sync::Lazy;

use super::{data::Module, parser::parse};

pub static MODULE: Lazy<RwLock<Module>> = Lazy::new(RwLock::default);

pub struct Builder {
	pub typ: BinaryViewType,
}

impl Builder {
	pub fn new(typ: BinaryViewType) -> Self {
		Self { typ }
	}
}

impl AsRef<BinaryViewType> for Builder {
	fn as_ref(&self) -> &BinaryViewType {
		&self.typ
	}
}

impl BinaryViewTypeBase for Builder {
	fn is_deprecated(&self) -> bool {
		false
	}

	fn is_valid_for(&self, data: &BinaryView) -> bool {
		parse(data).is_ok()
	}
}

impl CustomBinaryViewType for Builder {
	fn create_custom_view<'builder>(
		&self,
		data: &BinaryView,
		builder: CustomViewBuilder<'builder, Self>,
	) -> BResult<CustomView<'builder>> {
		let module = parse(data)?;

		builder.create::<View>(data, module)
	}
}

fn to_range_u64(old: Range<usize>) -> Range<u64> {
	old.start as u64..old.end as u64
}

pub struct View {
	view: Ref<BinaryView>,
}

impl View {
	fn add_string_section(&self, range: Range<usize>) {
		if range.is_empty() {
			return;
		}

		let range = to_range_u64(range);

		self.add_segment(
			Segment::new(range.clone())
				.parent_backing(range.clone())
				.contains_data(true)
				.readable(true)
				.is_auto(true),
		);

		self.add_section(Section::new("string_list", range).semantics(Semantics::ReadOnlyData));
	}

	fn add_string_data(&self, data: &[Range<usize>]) {
		if data.is_empty() {
			return;
		}

		let plat = self.default_platform().unwrap();
		let byte = &*Type::char();

		for (i, range) in data.iter().enumerate() {
			let name = format!("str_{i}");
			let sym = Symbol::new(SymbolType::Data, name, range.start as u64).create();

			let typ = &*Type::array(byte, range.len() as u64);

			self.define_auto_symbol_with_type(&sym, &plat, typ)
				.expect("Failed to define symbol");
		}
	}

	fn add_function_segment(&self, position: Range<usize>) {
		let position = to_range_u64(position);

		self.add_segment(
			Segment::new(position.clone())
				.parent_backing(position)
				.contains_code(true)
				.contains_data(true)
				.readable(true)
				.executable(true)
				.is_auto(true),
		);
	}

	fn add_code_section(&self, index: usize, code: Range<usize>) {
		if code.is_empty() {
			return;
		}

		let range = to_range_u64(code);

		self.add_section(
			Section::new(format!("code_{index}"), range)
				.semantics(Semantics::ReadOnlyCode)
				.is_auto(true),
		);
	}

	fn add_constant_section(&self, index: usize, constant: Range<usize>) {
		if constant.is_empty() {
			return;
		}

		let range = to_range_u64(constant);

		self.add_section(
			Section::new(format!("data_{index}"), range)
				.semantics(Semantics::ReadOnlyData)
				.is_auto(true),
		);
	}

	fn add_alias_for_function(&self, name: usize, data: &[Range<usize>], start: u64) {
		let range = match name.checked_sub(1).and_then(|i| data.get(i)) {
			Some(range) => range,
			None => return,
		};

		let raw = self.view.read_vec(range.start as u64, range.len());
		let name = String::from_utf8_lossy(&raw);
		let symbol = Symbol::new(SymbolType::Function, name.as_ref(), start).create();

		self.define_auto_symbol(&symbol);
	}
}

impl AsRef<BinaryView> for View {
	fn as_ref(&self) -> &BinaryView {
		&self.view
	}
}

impl BinaryViewBase for View {
	fn entry_point(&self) -> u64 {
		0
	}

	fn default_endianness(&self) -> Endianness {
		Endianness::LittleEndian
	}

	fn address_size(&self) -> usize {
		8
	}
}

unsafe impl CustomBinaryView for View {
	type Args = Module;

	fn new(handle: &BinaryView, _args: &Self::Args) -> BResult<Self> {
		let view = handle.to_owned();

		Ok(Self { view })
	}

	fn init(&self, args: Self::Args) -> BResult<()> {
		let arch = CoreArchitecture::by_name("luau").ok_or(())?;
		let plat = arch.standalone_platform().ok_or(())?;

		self.set_default_arch(&arch);
		self.set_default_platform(&plat);

		let str_list = args.string_list();

		self.add_string_section(str_list.range.clone());
		self.add_string_data(&str_list.data);

		for (i, func) in args.function_list().data.iter().enumerate() {
			let constant = func.constant_list().range.clone();
			let inst = func.code().start as u64;

			self.add_function_segment(func.position());

			self.add_code_section(i, func.code());
			self.add_constant_section(i, constant);

			self.add_auto_function(&plat, inst);
			self.add_alias_for_function(func.name(), &str_list.data, inst);
		}

		self.add_entry_point(&plat, args.entry_point());

		*MODULE.write().unwrap() = args;

		Ok(())
	}
}
