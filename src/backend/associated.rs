use std::borrow::Cow;

use binaryninja::{
	architecture::{Register as IRegister, RegisterInfo as IRegisterInfo},
	binaryninjacore_sys::BNImplicitRegisterExtend,
	llil::Register as LRegister,
};
use num_enum::TryFromPrimitive;

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

#[repr(u8)]
#[derive(TryFromPrimitive, Clone, Copy)]
pub enum Register {
	Stack,
	Return,
}

impl IRegister for Register {
	type InfoType = RegisterInfo;

	fn name(&self) -> Cow<str> {
		match self {
			Register::Stack => "stack_pointer".into(),
			Register::Return => "return_pointer".into(),
		}
	}

	fn info(&self) -> Self::InfoType {
		RegisterInfo
	}

	fn id(&self) -> u32 {
		*self as u32
	}
}

impl From<Register> for LRegister<Register> {
	fn from(reg: Register) -> Self {
		LRegister::ArchReg(reg)
	}
}
