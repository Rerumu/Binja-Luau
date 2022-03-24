#![feature(trait_alias)]

use binaryninja::{architecture::register_architecture, custombinaryview::register_view_type};

use file::view::ViewType;
use platform::architecture::Arch;

mod file;
mod instruction;
mod platform;

#[no_mangle]
pub extern "C" fn CorePluginInit() -> bool {
	register_architecture("luau", |handle, core| Arch { handle, core });
	register_view_type("Luau", "Roblox Luau", |typ| ViewType { typ });

	true
}
