use binaryninja::{
	architecture::{
		Architecture as BaseArchitecture, BranchInfo, CoreArchitecture, CoreFlag, CoreFlagClass,
		CoreFlagGroup, CoreFlagWrite, CustomArchitectureHandle, InstructionInfo,
	},
	binaryninjacore_sys::BNLowLevelILFlagCondition,
	callingconvention::CallingConventionBase,
	disassembly::InstructionTextToken,
	llil::{Label, LiftedExpr, Lifter},
	Endianness,
};

use crate::{
	decoder::{
		inst::Inst,
		opcode::{OpType, Opcode},
	},
	file::{
		data::{Module, Value},
		view::MODULE,
	},
};

use super::{
	associated::{Register, RegisterInfo},
	text_builder::TextBuilder,
};

const NUM_SIZE: usize = 8;

type Expression<'func> = binaryninja::llil::Expression<
	'func,
	Architecture,
	binaryninja::llil::Mutable,
	binaryninja::llil::NonSSA<binaryninja::llil::LiftedNonSSA>,
	binaryninja::llil::ValueExpr,
>;

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
			| Opcode::JumpIfNotConstant
			| Opcode::ForGenericPrep => {
				let on_false = Inst::get_jump_target(addr, next);
				let on_true = Inst::get_jump_target(addr, decoder.d());

				info.add_branch(BranchInfo::False(on_false), None);
				info.add_branch(BranchInfo::True(on_true), None);
			}
			Opcode::JumpIfNil
			| Opcode::JumpIfBoolean
			| Opcode::JumpIfNumber
			| Opcode::JumpIfString => {
				let mut on_false = Inst::get_jump_target(addr, next);
				let mut on_true = Inst::get_jump_target(addr, decoder.d());

				if decoder.adjacent() < 0 {
					std::mem::swap(&mut on_false, &mut on_true);
				}

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

	fn get_stack_offset(il: &Lifter<Self>, index: u8) -> Expression {
		let offset = index as usize * NUM_SIZE;

		il.add(
			NUM_SIZE,
			il.reg(NUM_SIZE, Register::Stack),
			il.const_int(NUM_SIZE, offset as u64),
		)
		.into()
	}

	fn get_variable(il: &Lifter<Self>, index: u8) -> Expression {
		let offset = Self::get_stack_offset(il, index);

		il.load(NUM_SIZE, offset).into()
	}

	fn set_variable<'a, V>(il: &Lifter<Self>, index: u8, value: V)
	where
		V: Into<Expression<'a>>,
	{
		let offset = Self::get_stack_offset(il, index);

		il.store(NUM_SIZE, offset, value.into()).append();
	}

	fn set_to_register(il: &Lifter<Self>, index: u8, value: Register) {
		Self::set_variable(il, index, il.reg(NUM_SIZE, value));
	}

	fn get_two_var_operands<A, B>(
		il: &Lifter<Self>,
		op_1: A,
		op_2: B,
	) -> Option<(Expression, Expression)>
	where
		A: TryInto<u8>,
		B: TryInto<u8>,
	{
		let var_1 = Self::get_variable(il, op_1.try_into().ok()?);
		let var_2 = Self::get_variable(il, op_2.try_into().ok()?);

		Some((var_1, var_2))
	}

	fn add_if_condition<'a, O, C>(
		il: &Lifter<Self>,
		addr: u64,
		offset: O,
		condition: C,
	) -> Option<()>
	where
		O: Into<i64>,
		C: Into<Expression<'a>>,
	{
		let target = Inst::get_jump_target(addr, offset);
		let on_true = il.label_for_address(target)?;
		let mut on_false = Label::new();

		il.if_expr(condition.into(), on_true, &on_false).append();
		il.mark_label(&mut on_false);

		Some(())
	}

	fn get_value(addr: u64, index: usize, parent: &Module) -> Option<&Value> {
		parent.by_address(addr)?.constant_list().data.get(index)
	}

	fn get_as_constant<'a>(
		il: &'a Lifter<Self>,
		value: &Value,
		parent: &Module,
	) -> Option<Expression<'a>> {
		let result = match value {
			Value::Nil => il.reg(NUM_SIZE, Register::Nil),
			Value::False => il.reg(NUM_SIZE, Register::False),
			Value::True => il.reg(NUM_SIZE, Register::True),
			// TODO: Need float support
			Value::Number(f) => il.const_int(NUM_SIZE, f.to_bits()),
			Value::String(i) => {
				let list = &parent.string_list().data;
				let ptr = match i.checked_sub(1) {
					Some(i) => list.get(i)?.start as u64,
					None => 0,
				};

				il.const_ptr(ptr)
			}
			Value::Closure(i) => {
				let ptr = parent.function_list().data.get(*i)?.position().start as u64;

				il.const_ptr(ptr)
			}
			Value::Import(_) => il.unimplemented(),
			Value::Table => il.unimplemented(),
		};

		Some(result)
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
		NUM_SIZE
	}

	fn default_integer_size(&self) -> usize {
		NUM_SIZE
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
		data: &[u8],
		addr: u64,
		il: &mut Lifter<Self>,
	) -> Option<(usize, bool)> {
		let decoder = Inst::try_from(data).ok()?;
		let op = decoder.op();

		match op {
			Opcode::Nop => il.nop().append(),
			Opcode::Break => il.bp().append(),
			Opcode::LoadNil => Self::set_to_register(il, decoder.a(), Register::Nil),
			Opcode::LoadBoolean => {
				let target = Inst::get_jump_target(addr, decoder.c());
				let value = if decoder.b() == 0 {
					Register::False
				} else {
					Register::True
				};

				Self::set_to_register(il, decoder.a(), value);
				il.goto(il.label_for_address(target)?).append();
			}
			Opcode::LoadInteger => {
				let value = decoder.d() as u64;

				Self::set_variable(il, decoder.a(), il.const_int(NUM_SIZE, value));
			}
			Opcode::LoadConstant => {
				let module = MODULE.read().unwrap();
				let reffed = &module.by_address(addr)?.constant_list().data[decoder.d() as usize];
				let value = Self::get_as_constant(il, reffed, &module)?;

				Self::set_variable(il, decoder.a(), value);
			}
			Opcode::Move => {
				let rhs = Self::get_variable(il, decoder.b());

				Self::set_variable(il, decoder.a(), rhs);
			}
			// Opcode::GetGlobal => todo!(),
			// Opcode::SetGlobal => todo!(),
			// Opcode::GetUpValue => todo!(),
			// Opcode::SetUpValue => todo!(),
			// Opcode::CloseUpValues => todo!(),
			// Opcode::GetImport => todo!(),
			// Opcode::GetTable => todo!(),
			// Opcode::SetTable => todo!(),
			// Opcode::GetTableKey => todo!(),
			// Opcode::SetTableKey => todo!(),
			// Opcode::GetTableIndex => todo!(),
			// Opcode::SetTableIndex => todo!(),
			// Opcode::NewClosure => todo!(),
			// Opcode::NameCall => todo!(),
			// Opcode::Call => todo!(),
			Opcode::Return if decoder.b() == 0 => {
				il.unimplemented().append();
			}
			Opcode::Return => {
				let start = Self::get_stack_offset(il, decoder.a());
				let count = decoder.b() as usize - 1;

				il.ret(il.load(count * NUM_SIZE, start)).append();
			}
			Opcode::Jump | Opcode::JumpSafe => {
				let target = Inst::get_jump_target(addr, decoder.d());

				il.goto(il.label_for_address(target)?).append();
			}
			Opcode::JumpIfTruthy => {
				let cond = Self::get_variable(il, decoder.a());

				Self::add_if_condition(il, addr, decoder.d(), cond);
			}
			Opcode::JumpIfFalsy => {
				let variable = Self::get_variable(il, decoder.a());
				let cond = il.not(NUM_SIZE, variable);

				Self::add_if_condition(il, addr, decoder.d(), cond);
			}
			Opcode::JumpIfEqual => {
				let (lhs, rhs) = Self::get_two_var_operands(il, decoder.a(), decoder.adjacent())?;
				let cond = il.cmp_e(NUM_SIZE, lhs, rhs);

				Self::add_if_condition(il, addr, decoder.d(), cond);
			}
			Opcode::JumpIfLessEqual => {
				let (lhs, rhs) = Self::get_two_var_operands(il, decoder.a(), decoder.adjacent())?;
				let cond = il.cmp_sle(NUM_SIZE, lhs, rhs);

				Self::add_if_condition(il, addr, decoder.d(), cond);
			}
			Opcode::JumpIfLessThan => {
				let (lhs, rhs) = Self::get_two_var_operands(il, decoder.a(), decoder.adjacent())?;
				let cond = il.cmp_slt(NUM_SIZE, lhs, rhs);

				Self::add_if_condition(il, addr, decoder.d(), cond);
			}
			Opcode::JumpIfNotEqual => {
				let (lhs, rhs) = Self::get_two_var_operands(il, decoder.a(), decoder.adjacent())?;
				let cond = il.cmp_ne(NUM_SIZE, lhs, rhs);

				Self::add_if_condition(il, addr, decoder.d(), cond);
			}
			Opcode::JumpIfMoreThan => {
				let (lhs, rhs) = Self::get_two_var_operands(il, decoder.a(), decoder.adjacent())?;
				let cond = il.cmp_sgt(NUM_SIZE, lhs, rhs);

				Self::add_if_condition(il, addr, decoder.d(), cond);
			}
			Opcode::JumpIfMoreEqual => {
				let (lhs, rhs) = Self::get_two_var_operands(il, decoder.a(), decoder.adjacent())?;
				let cond = il.cmp_sge(NUM_SIZE, lhs, rhs);

				Self::add_if_condition(il, addr, decoder.d(), cond);
			}
			Opcode::Add => {
				let (lhs, rhs) = Self::get_two_var_operands(il, decoder.b(), decoder.c())?;
				let result = il.add(NUM_SIZE, lhs, rhs);

				Self::set_variable(il, decoder.a(), result);
			}
			Opcode::Sub => {
				let (lhs, rhs) = Self::get_two_var_operands(il, decoder.b(), decoder.c())?;
				let result = il.sub(NUM_SIZE, lhs, rhs);

				Self::set_variable(il, decoder.a(), result);
			}
			Opcode::Mul => {
				let (lhs, rhs) = Self::get_two_var_operands(il, decoder.b(), decoder.c())?;
				let result = il.mul(NUM_SIZE, lhs, rhs);

				Self::set_variable(il, decoder.a(), result);
			}
			Opcode::Div => {
				let (lhs, rhs) = Self::get_two_var_operands(il, decoder.b(), decoder.c())?;
				let result = il.divs(NUM_SIZE, lhs, rhs);

				Self::set_variable(il, decoder.a(), result);
			}
			Opcode::Mod => {
				let (lhs, rhs) = Self::get_two_var_operands(il, decoder.b(), decoder.c())?;
				let result = il.mods(NUM_SIZE, lhs, rhs);

				Self::set_variable(il, decoder.a(), result);
			}
			// Opcode::Pow => todo!(),
			Opcode::AddConstant => {
				let module = MODULE.read().unwrap();
				let reffed = &module.by_address(addr)?.constant_list().data[decoder.c() as usize];

				let lhs = Self::get_variable(il, decoder.b());
				let rhs = Self::get_as_constant(il, reffed, &module)?;
				let result = il.add(NUM_SIZE, lhs, rhs);

				Self::set_variable(il, decoder.a(), result);
			}
			Opcode::SubConstant => {
				let module = MODULE.read().unwrap();
				let reffed = &module.by_address(addr)?.constant_list().data[decoder.c() as usize];

				let lhs = Self::get_variable(il, decoder.b());
				let rhs = Self::get_as_constant(il, reffed, &module)?;
				let result = il.sub(NUM_SIZE, lhs, rhs);

				Self::set_variable(il, decoder.a(), result);
			}
			Opcode::MulConstant => {
				let module = MODULE.read().unwrap();
				let reffed = &module.by_address(addr)?.constant_list().data[decoder.c() as usize];

				let lhs = Self::get_variable(il, decoder.b());
				let rhs = Self::get_as_constant(il, reffed, &module)?;
				let result = il.mul(NUM_SIZE, lhs, rhs);

				Self::set_variable(il, decoder.a(), result);
			}
			Opcode::DivConstant => {
				let module = MODULE.read().unwrap();
				let reffed = &module.by_address(addr)?.constant_list().data[decoder.c() as usize];

				let lhs = Self::get_variable(il, decoder.b());
				let rhs = Self::get_as_constant(il, reffed, &module)?;
				let result = il.divs(NUM_SIZE, lhs, rhs);

				Self::set_variable(il, decoder.a(), result);
			}
			Opcode::ModConstant => {
				let module = MODULE.read().unwrap();
				let reffed = &module.by_address(addr)?.constant_list().data[decoder.c() as usize];

				let lhs = Self::get_variable(il, decoder.b());
				let rhs = Self::get_as_constant(il, reffed, &module)?;
				let result = il.mods(NUM_SIZE, lhs, rhs);

				Self::set_variable(il, decoder.a(), result);
			}
			// Opcode::PowConstant => todo!(),
			// Opcode::And => todo!(),
			// Opcode::Or => todo!(),
			// Opcode::AndConstant => todo!(),
			// Opcode::OrConstant => todo!(),
			// Opcode::Concat => todo!(),
			Opcode::Not => {
				let rhs = Self::get_variable(il, decoder.b());
				let result = il.not(NUM_SIZE, rhs);

				Self::set_variable(il, decoder.a(), result);
			}
			Opcode::Minus => {
				let rhs = Self::get_variable(il, decoder.b());
				let result = il.neg(NUM_SIZE, rhs);

				Self::set_variable(il, decoder.a(), result);
			}
			// Opcode::Length => todo!(),
			// Opcode::NewTable => todo!(),
			// Opcode::DupTable => todo!(),
			// Opcode::SetList => todo!(),
			// Opcode::ForNumericPrep => todo!(),
			// Opcode::ForNumericLoop => todo!(),
			// Opcode::ForGenericLoop => todo!(),
			// Opcode::ForGenericPrepINext => todo!(),
			// Opcode::ForGenericLoopINext => todo!(),
			// Opcode::ForGenericPrepNext => todo!(),
			// Opcode::ForGenericLoopNext => todo!(),
			// Opcode::GetVariadic => todo!(),
			// Opcode::DupClosure => todo!(),
			// Opcode::PrepVariadic => todo!(),
			Opcode::LoadConstantEx => {
				let module = MODULE.read().unwrap();
				let reffed = Self::get_value(addr, decoder.adjacent() as usize, &module)?;
				let value = Self::get_as_constant(il, reffed, &module)?;

				Self::set_variable(il, decoder.a(), value);
			}
			Opcode::JumpEx => {
				let target = Inst::get_jump_target(addr, decoder.e());

				il.goto(il.label_for_address(target)?).append();
			}
			// Opcode::FastCall => todo!(),
			// Opcode::Coverage => todo!(),
			// Opcode::Capture => todo!(),
			Opcode::JumpIfConstant => {
				let module = MODULE.read().unwrap();
				let reffed = Self::get_value(addr, decoder.adjacent() as usize, &module)?;
				let lhs = Self::get_variable(il, decoder.a());
				let rhs = Self::get_as_constant(il, reffed, &module)?;
				let cond = il.cmp_e(NUM_SIZE, lhs, rhs);

				Self::add_if_condition(il, addr, decoder.d(), cond);
			}
			Opcode::JumpIfNotConstant => {
				let module = MODULE.read().unwrap();
				let reffed = Self::get_value(addr, decoder.adjacent() as usize, &module)?;
				let lhs = Self::get_variable(il, decoder.a());
				let rhs = Self::get_as_constant(il, reffed, &module)?;
				let cond = il.cmp_ne(NUM_SIZE, lhs, rhs);

				Self::add_if_condition(il, addr, decoder.d(), cond);
			}
			// Opcode::FastCall1 => todo!(),
			// Opcode::FastCall2 => todo!(),
			// Opcode::FastCall2K => todo!(),
			_ => il.unimplemented().append(),
		}

		Some((op.len(), true))
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
