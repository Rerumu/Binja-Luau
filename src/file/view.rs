use std::{lazy::SyncLazy, ops::Range, sync::RwLock};

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
	Endianness,
};

use super::{
	data::{Function, Module},
	parser::parse,
};

pub static MODULE: SyncLazy<RwLock<Module>> = SyncLazy::new(RwLock::default);

pub struct ViewType {
	pub typ: BinaryViewType,
}

impl ViewType {
	pub fn new(typ: BinaryViewType) -> Self {
		Self { typ }
	}
}

impl AsRef<BinaryViewType> for ViewType {
	fn as_ref(&self) -> &BinaryViewType {
		&self.typ
	}
}

impl BinaryViewTypeBase for ViewType {
	fn is_valid_for(&self, data: &BinaryView) -> bool {
		parse(data).is_ok()
	}
}

impl CustomBinaryViewType for ViewType {
	fn create_custom_view<'builder>(
		&self,
		data: &BinaryView,
		builder: CustomViewBuilder<'builder, Self>,
	) -> BResult<CustomView<'builder>> {
		let module = parse(data)?;

		builder.create::<View>(data, module)
	}
}

pub struct View {
	view: Ref<BinaryView>,
}

impl View {
	fn add_string_section(&self, string_list: &[Range<usize>]) {
		if string_list.is_empty() {
			return;
		}

		let first = string_list.first().unwrap().start;
		let last = string_list.last().unwrap().end;
		let range = to_range(first..last);

		self.add_segment(
			Segment::new(range.clone())
				.parent_backing(range.clone())
				.contains_data(true)
				.readable(true),
		);

		self.add_section(Section::new("string_list", range).semantics(Semantics::ReadOnlyData));
	}

	fn add_function_segment(&self, index: usize, func: &Function) {
		let position = to_range(func.position());

		self.add_segment(
			Segment::new(position.clone())
				.parent_backing(position)
				.contains_code(true)
				.contains_data(true)
				.readable(true)
				.executable(true),
		);

		self.add_code_section(index, func.code());

		self.add_constant_section(index, func.constant_list());
	}

	fn add_code_section(&self, index: usize, code: Range<usize>) {
		if code.is_empty() {
			return;
		}

		let range = to_range(code);

		self.add_section(
			Section::new(format!("code_{}", index), range).semantics(Semantics::ReadOnlyCode),
		);
	}

	fn add_constant_section(&self, index: usize, constant_list: &[Range<usize>]) {
		if constant_list.is_empty() {
			return;
		}

		let first = constant_list.first().unwrap().start;
		let last = constant_list.last().unwrap().end;
		let name = format!("data_{}", index);

		self.add_section(
			Section::new(name, to_range(first..last)).semantics(Semantics::ReadOnlyData),
		);
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

fn to_range(old: Range<usize>) -> Range<u64> {
	old.start as u64..old.end as u64
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

		self.add_string_section(args.string_list());

		for (i, func) in args.function_list().iter().enumerate() {
			let code = to_range(func.code());

			self.add_function_segment(i, func);

			self.add_auto_function(&plat, code.start);
		}

		self.add_entry_point(&plat, args.entry_point());

		*MODULE.write().unwrap() = args;

		Ok(())
	}
}
