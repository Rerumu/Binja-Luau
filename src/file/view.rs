use std::{ops::Range, sync::Arc};

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

use super::parser::{parse, Module};

pub struct ViewType {
	pub typ: BinaryViewType,
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
		let module = Arc::new(parse(data)?);

		builder.create::<View>(data, module)
	}
}

pub struct View {
	view: Ref<BinaryView>,
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
	type Args = Arc<Module>;

	fn new(handle: &BinaryView, _args: &Self::Args) -> BResult<Self> {
		let view = handle.to_owned();

		Ok(Self { view })
	}

	fn init(&self, args: Self::Args) -> BResult<()> {
		let arch = CoreArchitecture::by_name("luau").ok_or(())?;
		let plat = arch.standalone_platform().ok_or(())?;

		self.set_default_arch(&arch);
		self.set_default_platform(&plat);

		let string_list = args.string_list();

		if !string_list.is_empty() {
			let range =
				to_range(string_list.first().unwrap().start..string_list.last().unwrap().end);

			self.add_segment(
				Segment::new(range.clone())
					.parent_backing(range.clone())
					.contains_data(true)
					.readable(true),
			);

			self.add_section(Section::new("string_list", range).semantics(Semantics::ReadOnlyData));
		}

		for (i, func) in args.function_list().iter().enumerate() {
			let code = to_range(func.code());
			let position = to_range(func.position());

			self.add_segment(
				Segment::new(position.clone())
					.parent_backing(position.clone())
					.contains_code(true)
					.contains_data(true)
					.readable(true)
					.executable(true),
			);

			self.add_section(
				Section::new(format!("code_{}", i), code.clone())
					.semantics(Semantics::ReadOnlyCode),
			);

			self.add_section(
				Section::new(format!("data_{}", i), position.start..code.start)
					.semantics(Semantics::ReadOnlyData),
			);

			self.add_auto_function(&plat, code.start);
		}

		self.add_entry_point(&plat, args.entry_point());

		Ok(())
	}
}
