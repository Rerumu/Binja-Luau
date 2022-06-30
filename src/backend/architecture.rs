use binaryninja::{
	architecture::{
		Architecture as BaseArchitecture, BranchInfo, CoreArchitecture, CoreFlag, CoreFlagClass,
		CoreFlagGroup, CoreFlagWrite, CustomArchitectureHandle, InstructionInfo,
	},
	binaryninjacore_sys::BNLowLevelILFlagCondition,
	callingconvention::CallingConventionBase,
	disassembly::InstructionTextToken,
	llil::{LiftedExpr, Lifter},
	Endianness,
};

use crate::{
	decoder::{
		inst::Inst,
		opcode::{OpType, Opcode},
	},
	file::view::MODULE,
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

	fn get_opt_instruction_info(decoder: Inst, addr: u64) -> InstructionInfo {
		let op = decoder.op();
		let next = op.len() as i64 / 4 - 1;

		let mut info = InstructionInfo::new(op.len(), false);

		match op {
			Opcode::LoadBoolean => {
				let target = Inst::get_jump_target(addr, decoder.c());

				info.add_branch(BranchInfo::Unconditional(target), None);
			}
			Opcode::Return => {
				info.add_branch(BranchInfo::FunctionReturn, None);
			}
			Opcode::Jump | Opcode::JumpSafe => {
				let target = Inst::get_jump_target(addr, decoder.d());

				info.add_branch(BranchInfo::Unconditional(target), None);
			}
			Opcode::JumpEx => {
				let target = Inst::get_jump_target(addr, decoder.e());

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
				let on_false = Inst::get_jump_target(addr, next);
				let on_true = Inst::get_jump_target(addr, decoder.d());

				info.add_branch(BranchInfo::False(on_false), None);
				info.add_branch(BranchInfo::True(on_true), None);
			}
			Opcode::FastCall | Opcode::FastCall1 | Opcode::FastCall2 | Opcode::FastCall2K => {
				let on_false = Inst::get_jump_target(addr, next);
				let on_true = Inst::get_jump_target(addr, decoder.c() + 1);

				info.add_branch(BranchInfo::Indirect, None);
				info.add_branch(BranchInfo::False(on_false), None);
				info.add_branch(BranchInfo::True(on_true), None);
			}
			_ => {}
		}

		info
	}

	fn get_opt_instruction_text(decoder: Inst, addr: u64) -> Option<TextBuilder> {
		let opcode = decoder.op();

		let mut builder = TextBuilder::with_mnemonic(opcode);

		for (name, typ) in opcode.iter_operands() {
			let raw = decoder.with_name(name);

			match typ {
				OpType::Location => builder.add_location(addr, raw.into()),
				OpType::Register => builder.add_register(raw.try_into().ok()?),
				OpType::UpValue => builder.add_upvalue(raw.try_into().ok()?),
				OpType::Boolean => builder.add_boolean(raw != 0),
				OpType::Integer => builder.add_integer(raw),
				OpType::Constant => {
					let module = MODULE.read().unwrap();
					let func = module.by_address(addr)?;
					let value = func.constant_list().data.get(raw as usize)?;

					builder.add_constant(value, func, &module)?;
				}
				OpType::Function => {
					let module = MODULE.read().unwrap();
					let global = &module.function_list();
					let adjusted = module
						.by_address(addr)?
						.reference_list()
						.data
						.get(raw as usize)?;

					builder.add_function(*adjusted, &global.data)?;
				}
				OpType::Import => {
					let module = MODULE.read().unwrap();
					let func = module.by_address(addr)?;

					builder.add_import(raw as u32, func, &module)?;
				}
				OpType::BuiltIn => builder.add_built_in(raw.try_into().ok()?)?,
			}
		}

		Some(builder)
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
		let decoder = Inst::try_from(data).ok()?;
		let info = Self::get_opt_instruction_info(decoder, addr);

		Some(info)
	}

	fn instruction_text(
		&self,
		data: &[u8],
		addr: u64,
	) -> Option<(usize, Vec<InstructionTextToken>)> {
		let decoder = Inst::try_from(data).ok()?;
		let builder = Self::get_opt_instruction_text(decoder, addr)?;

		Some((decoder.op().len(), builder.into()))
	}

	fn instruction_llil(
		&self,
		_data: &[u8],
		_addr: u64,
		_il: &mut Lifter<Self>,
	) -> Option<(usize, bool)> {
		None
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
		vec![
			Register::Stack,  // lua stack pointer
			Register::Return, // lua return pointer
		]
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
		Some(Register::Stack)
	}

	fn link_reg(&self) -> Option<Self::Register> {
		Some(Register::Return)
	}

	fn register_from_id(&self, id: u32) -> Option<Self::Register> {
		u8::try_from(id)
			.ok()
			.and_then(|v| Register::try_from(v).ok())
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

pub struct CallingConvention;

impl CallingConventionBase for CallingConvention {
	type Arch = Architecture;

	fn caller_saved_registers(&self) -> Vec<Register> {
		Vec::new()
	}

	fn callee_saved_registers(&self) -> Vec<Register> {
		Vec::new()
	}

	fn int_arg_registers(&self) -> Vec<Register> {
		Vec::new()
	}

	fn float_arg_registers(&self) -> Vec<Register> {
		Vec::new()
	}

	fn arg_registers_shared_index(&self) -> bool {
		false
	}

	fn reserved_stack_space_for_arg_registers(&self) -> bool {
		false
	}

	fn stack_adjusted_on_return(&self) -> bool {
		false
	}

	fn is_eligible_for_heuristics(&self) -> bool {
		true
	}

	fn return_int_reg(&self) -> Option<Register> {
		None
	}

	fn return_hi_int_reg(&self) -> Option<Register> {
		None
	}

	fn return_float_reg(&self) -> Option<Register> {
		None
	}

	fn global_pointer_reg(&self) -> Option<Register> {
		None
	}

	fn implicitly_defined_registers(&self) -> Vec<Register> {
		Vec::new()
	}

	fn are_argument_registers_used_for_var_args(&self) -> bool {
		false
	}
}
