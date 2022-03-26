use binaryninja::{
	architecture::{
		Architecture as BaseArchitecture, BranchInfo, CoreArchitecture, CoreFlag, CoreFlagClass,
		CoreFlagGroup, CoreFlagWrite, CustomArchitectureHandle, InstructionInfo,
	},
	binaryninjacore_sys::BNLowLevelILFlagCondition,
	llil::{LiftedExpr, Lifter},
	Endianness,
};

use crate::instruction::{
	decoder::{get_jump_target, Decoder},
	opcode::{OpType, Opcode},
};

use super::{
	associated::{Register, RegisterInfo},
	text_builder::TextBuilder,
};

pub struct Architecture {
	pub handle: CustomArchitectureHandle<Self>,
	pub core: CoreArchitecture,
}

impl Architecture {
	pub fn new(handle: CustomArchitectureHandle<Self>, core: CoreArchitecture) -> Self {
		Self { handle, core }
	}
}

impl BaseArchitecture for Architecture {
	type Handle = CustomArchitectureHandle<Self>;

	type RegisterInfo = RegisterInfo;

	type Register = Register;

	type Flag = CoreFlag;

	type FlagWrite = CoreFlagWrite;

	type FlagClass = CoreFlagClass;

	type FlagGroup = CoreFlagGroup;

	type InstructionTextContainer = TextBuilder;

	fn endianness(&self) -> Endianness {
		Endianness::LittleEndian
	}

	fn address_size(&self) -> usize {
		8
	}

	fn default_integer_size(&self) -> usize {
		8
	}

	fn instruction_alignment(&self) -> usize {
		1
	}

	fn max_instr_len(&self) -> usize {
		8
	}

	fn opcode_display_len(&self) -> usize {
		self.max_instr_len()
	}

	fn associated_arch_by_addr(&self, _: &mut u64) -> CoreArchitecture {
		self.core
	}

	fn instruction_info(&self, data: &[u8], addr: u64) -> Option<InstructionInfo> {
		let decoder = Decoder::try_from(data).ok()?;
		let mut info = InstructionInfo::new(decoder.op().len(), false);

		match decoder.op() {
			Opcode::LoadBoolean => {
				let target = get_jump_target(addr, decoder.c().into());

				info.add_branch(BranchInfo::Unconditional(target), None);
			}
			Opcode::Return => {
				info.add_branch(BranchInfo::FunctionReturn, None);
			}
			Opcode::Jump | Opcode::JumpSafe => {
				let target = get_jump_target(addr, decoder.d().into());

				info.add_branch(BranchInfo::Unconditional(target), None);
			}
			Opcode::JumpEx => {
				let target = get_jump_target(addr, decoder.e().into());

				info.add_branch(BranchInfo::Unconditional(target), None);
			}
			Opcode::JumpIfTruthy
			| Opcode::JumpIfFalsy
			| Opcode::JumpIfEqual
			| Opcode::JumpIfNotEqual
			| Opcode::JumpIfLessThan
			| Opcode::JumpIfLessEqual
			| Opcode::JumpIfMoreThan
			| Opcode::JumpIfMoreEqual
			| Opcode::ForNumericPrep
			| Opcode::ForNumericLoop
			| Opcode::ForGenericLoop
			| Opcode::ForGenericPrepINext
			| Opcode::ForGenericLoopINext
			| Opcode::ForGenericPrepNext
			| Opcode::ForGenericLoopNext
			| Opcode::JumpIfConstant
			| Opcode::JumpIfNotConstant => {
				let on_false = get_jump_target(addr, 0);
				let on_true = get_jump_target(addr, decoder.d().into());

				info.add_branch(BranchInfo::False(on_false), None);
				info.add_branch(BranchInfo::True(on_true), None);
			}
			Opcode::FastCall | Opcode::FastCall1 | Opcode::FastCall2 | Opcode::FastCall2K => {
				let on_false = get_jump_target(addr, 0);
				let on_true = get_jump_target(addr, (decoder.c() + 1).into());

				info.add_branch(BranchInfo::Indirect, None);
				info.add_branch(BranchInfo::False(on_false), None);
				info.add_branch(BranchInfo::True(on_true), None);
			}
			_ => {}
		}

		Some(info)
	}

	fn instruction_text(
		&self,
		data: &[u8],
		addr: u64,
	) -> Option<(usize, Self::InstructionTextContainer)> {
		let decoder = Decoder::try_from(data).ok()?;
		let opcode = decoder.op();

		let mut builder = TextBuilder::new();

		builder.add_mnemonic(opcode);

		for (name, typ) in opcode.iter_operands() {
			let raw = decoder.with_name(name);

			match typ {
				OpType::Location => builder.add_location(addr, raw.into()),
				OpType::Register => builder.add_register(raw),
				// OpType::UpValue => todo!(),
				// OpType::Global => todo!(),
				OpType::Boolean => builder.add_boolean(raw != 0),
				OpType::Integer => builder.add_integer(raw),
				// OpType::Constant => todo!(),
				// OpType::Function => todo!(),
				// OpType::Import => todo!(),
				OpType::BuiltIn => builder.add_built_in(raw),
				_ => builder.add_failure(),
			}
		}

		Some((opcode.len(), builder))
	}

	fn instruction_llil(
		&self,
		data: &[u8],
		_addr: u64,
		il: &mut Lifter<Self>,
	) -> Option<(usize, bool)> {
		let decoder = Decoder::try_from(data).ok()?;
		let opcode = decoder.op();

		il.unimplemented();

		Some((opcode.len(), false))
	}

	fn flags_required_for_flag_condition(
		&self,
		_: BNLowLevelILFlagCondition,
		_: Option<Self::FlagClass>,
	) -> Vec<Self::Flag> {
		Vec::new()
	}

	fn flag_group_llil<'a>(
		&self,
		_: Self::FlagGroup,
		_: &'a mut Lifter<Self>,
	) -> Option<LiftedExpr<'a, Self>> {
		None
	}

	fn registers_all(&self) -> Vec<Self::Register> {
		(0..255).map(Register::new).collect()
	}

	fn registers_full_width(&self) -> Vec<Self::Register> {
		self.registers_all()
	}

	fn registers_global(&self) -> Vec<Self::Register> {
		Vec::new()
	}

	fn registers_system(&self) -> Vec<Self::Register> {
		Vec::new()
	}

	fn flags(&self) -> Vec<Self::Flag> {
		Vec::new()
	}

	fn flag_write_types(&self) -> Vec<Self::FlagWrite> {
		Vec::new()
	}

	fn flag_classes(&self) -> Vec<Self::FlagClass> {
		Vec::new()
	}

	fn flag_groups(&self) -> Vec<Self::FlagGroup> {
		Vec::new()
	}

	fn stack_pointer_reg(&self) -> Option<Self::Register> {
		None
	}

	fn link_reg(&self) -> Option<Self::Register> {
		None
	}

	fn register_from_id(&self, id: u32) -> Option<Self::Register> {
		(id < 0x100).then(|| Register::new(id))
	}

	fn flag_from_id(&self, _: u32) -> Option<Self::Flag> {
		None
	}

	fn flag_write_from_id(&self, _: u32) -> Option<Self::FlagWrite> {
		None
	}

	fn flag_class_from_id(&self, _: u32) -> Option<Self::FlagClass> {
		None
	}

	fn flag_group_from_id(&self, _: u32) -> Option<Self::FlagGroup> {
		None
	}

	fn handle(&self) -> Self::Handle {
		self.handle
	}
}

impl AsRef<CoreArchitecture> for Architecture {
	fn as_ref(&self) -> &CoreArchitecture {
		&self.core
	}
}
