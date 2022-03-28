#![feature(once_cell)]
#![feature(trait_alias)]

use binaryninja::{architecture::register_architecture, custombinaryview::register_view_type};

use backend::architecture::Architecture;
use file::view::ViewType;

mod backend;
mod decoder;
mod file;

#[no_mangle]
pub extern "C" fn CorePluginInit() -> bool {
	register_architecture("luau", Architecture::new);
	register_view_type("Luau", "Roblox Luau", ViewType::new);

	true
}
