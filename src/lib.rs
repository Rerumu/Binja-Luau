#![feature(trait_alias)]

use binaryninja::{architecture::register_architecture, custombinaryview::register_view_type};

use backend::architecture::Architecture;
use file::view::ViewType;

mod backend;
mod file;
mod instruction;

#[no_mangle]
pub extern "C" fn CorePluginInit() -> bool {
	let arch = register_architecture("luau", Architecture::new);

	register_view_type("Luau", "Roblox Luau", |typ| ViewType::new(typ, arch));

	true
}
