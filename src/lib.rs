#![feature(once_cell)]
#![feature(trait_alias)]

use binaryninja::{
	architecture::register_architecture, callingconvention::register_calling_convention,
	custombinaryview::register_view_type,
};

use backend::architecture::{Architecture, CallingConvention};
use file::view::Builder;

mod backend;
mod decoder;
mod file;

#[no_mangle]
pub extern "C" fn CorePluginInit() -> bool {
	let arch = register_architecture("luau", Architecture::new);

	register_calling_convention(arch, "luau", CallingConvention);
	register_view_type("Luau", "Roblox Luau", Builder::new);

	true
}
