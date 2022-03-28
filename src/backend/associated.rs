use std::borrow::Cow;

use binaryninja::{
	architecture::{Register as IRegister, RegisterInfo as IRegisterInfo},
	binaryninjacore_sys::BNImplicitRegisterExtend,
	llil::Register as LRegister,
};

pub struct RegisterInfo;

impl IRegisterInfo for RegisterInfo {
	type RegType = Register;

	fn parent(&self) -> Option<Self::RegType> {
		None
	}

	fn size(&self) -> usize {
		8
	}

	fn offset(&self) -> usize {
		0
	}

	fn implicit_extend(&self) -> BNImplicitRegisterExtend {
		BNImplicitRegisterExtend::ZeroExtendToFullWidth
	}
}

#[derive(Clone, Copy)]
pub struct Register {
	id: u8,
}

impl Register {
	pub fn new(id: u8) -> Self {
		Register { id }
	}
}

impl IRegister for Register {
	type InfoType = RegisterInfo;

	fn name(&self) -> Cow<str> {
		format!("r{}", self.id).into()
	}

	fn info(&self) -> Self::InfoType {
		RegisterInfo
	}

	fn id(&self) -> u32 {
		self.id.into()
	}
}

impl From<Register> for LRegister<Register> {
	fn from(reg: Register) -> Self {
		LRegister::ArchReg(reg)
	}
}
