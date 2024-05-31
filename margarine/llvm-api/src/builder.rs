use std::{marker::PhantomData, ops::Deref, ptr::NonNull};

use llvm_sys::{core::{LLVMAddCase, LLVMAppendBasicBlock, LLVMBuildAShr, LLVMBuildAdd, LLVMBuildAlloca, LLVMBuildAnd, LLVMBuildBr, LLVMBuildCall2, LLVMBuildCondBr, LLVMBuildFAdd, LLVMBuildFCmp, LLVMBuildFDiv, LLVMBuildFPCast, LLVMBuildFRem, LLVMBuildFSub, LLVMBuildICmp, LLVMBuildIntCast2, LLVMBuildLShr, LLVMBuildLoad2, LLVMBuildMul, LLVMBuildNot, LLVMBuildOr, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildSDiv, LLVMBuildSRem, LLVMBuildShl, LLVMBuildStore, LLVMBuildStructGEP2, LLVMBuildSub, LLVMBuildSwitch, LLVMBuildUDiv, LLVMBuildURem, LLVMBuildUnreachable, LLVMBuildXor, LLVMConstArray2, LLVMConstInt, LLVMConstReal, LLVMDeleteBasicBlock, LLVMDisposeBuilder, LLVMDoubleType, LLVMFloatType, LLVMGetFirstBasicBlock, LLVMGetInsertBlock, LLVMGetLastInstruction, LLVMGetParam, LLVMIntTypeInContext, LLVMIsATerminatorInst, LLVMPositionBuilderAtEnd}, LLVMBasicBlock, LLVMBuilder, LLVMIntPredicate, LLVMRealPredicate, LLVMValue};
use sti::{arena::Arena, define_key, keyed::KVec};

use crate::{cstr, ctx::ContextRef, tys::{func::FunctionType, integer::IntegerTy, strct::StructTy, Type, TypeKind}, values::{array::Array, bool::Bool, fp::FP, func::FunctionPtr, int::Integer, ptr::Ptr, strct::Struct, unit::Unit, Value}};


define_key!(u32, pub Local);


#[derive(Debug, Clone, Copy)]
pub struct Loop {
    body_bb: NonNull<LLVMBasicBlock>,
    cont_bb: NonNull<LLVMBasicBlock>,
}


pub struct Builder<'ctx> {
    // LLVM
    ptr: NonNull<LLVMBuilder>,
    phantom: PhantomData<&'ctx ()>,
    ctx: ContextRef<'ctx>,
    
    // API
    func   : FunctionPtr<'ctx>,
    prelude: NonNull<LLVMBasicBlock>,
    entry  : NonNull<LLVMBasicBlock>,
    argc   : usize,
    locals : KVec<Local, (Ptr<'ctx>, Type<'ctx>)>,
}


impl<'ctx> Builder<'ctx> {
    pub fn new(ptr: NonNull<LLVMBuilder>, ctx: ContextRef<'ctx>, func: FunctionPtr<'ctx>, ty: FunctionType<'ctx>) -> Self {
        let bb = unsafe { LLVMGetFirstBasicBlock(func.llvm_val().as_ptr()) };
        assert!(bb.is_null(), "this function already has a builder");

        let prelude = unsafe { LLVMAppendBasicBlock(func.llvm_val().as_ptr(), cstr!("prelude")) };
        let prelude = NonNull::new(prelude).expect("failed to initialise the prelude basic-block");

        let entry = unsafe { LLVMAppendBasicBlock(func.llvm_val().as_ptr(), cstr!("entry")) };
        let entry = NonNull::new(entry).unwrap();
        
        unsafe { LLVMPositionBuilderAtEnd(ptr.as_ptr(), entry.as_ptr()) };

        let mut builder = Builder {
            ptr,
            phantom: PhantomData,

            func, 
            locals: KVec::new(),
            prelude,
            entry,
            argc: ty.argument_count(),
            ctx,
        };


        // Convert arguments into locals
        for a in ty.args().iter().enumerate() {
            let arg = unsafe { LLVMGetParam(func.llvm_val().as_ptr(), a.0 as u32) };
            let ptr = builder.alloca(*a.1);

            unsafe { LLVMBuildStore(builder.ptr.as_ptr(),
                                    arg, ptr.llvm_val().as_ptr()) };

            builder.locals.push((ptr, *a.1));
        }

        builder
    }


    pub fn build(self) {
        // dropping should call build_internal anyway :,)
    }


    fn build_internal(&self) {
        // make sure the function is properly terminated
        // [?] All other blocks are terminated by the API and we don't 
        //     provide any way to switch blocks from the API so all other
        //     basic blocks should be properly terminated
        let curr_bb = unsafe { LLVMGetInsertBlock(self.ptr.as_ptr()) };
        let last_inst = unsafe { LLVMGetLastInstruction(curr_bb) };

        if last_inst.is_null()
            || unsafe { !LLVMIsATerminatorInst(last_inst).is_null() } {
            unsafe { LLVMDeleteBasicBlock(curr_bb) }
        }
       
        // make prelude jump to the entry
        unsafe { LLVMPositionBuilderAtEnd(self.ptr.as_ptr(), self.prelude.as_ptr()) };
        unsafe { LLVMBuildBr(self.ptr.as_ptr(), self.entry.as_ptr()) };

        // finalise
        // oh wait we don't need to do nothing
    }


    pub fn arg(&self, index: usize) -> Option<Local> {
        if index >= self.argc { return None }
        Some(Local(index as u32))
    }


    pub fn unreachable(&self) {
        unsafe { LLVMBuildUnreachable(self.ptr.as_ptr()) };
        let bb = unsafe { LLVMAppendBasicBlock(self.func.llvm_val().as_ptr(), "".as_ptr() as _) };
        unsafe { LLVMPositionBuilderAtEnd(self.ptr.as_ptr(), bb) };
    }
    

    pub fn loop_indefinitely(
        &mut self,
        body: impl FnOnce(&mut Self, Loop)
    ) {
        let body_bb = unsafe { LLVMAppendBasicBlock(self.func.llvm_val().as_ptr(), cstr!("loop_body")) };
        let body_bb = NonNull::new(body_bb).unwrap();

        let cont_bb = unsafe { LLVMAppendBasicBlock(self.func.llvm_val().as_ptr(), cstr!("loop_cont")) };
        let cont_bb = NonNull::new(cont_bb).unwrap();

        let data = Loop { body_bb, cont_bb };

        unsafe { LLVMBuildBr(self.ptr.as_ptr(), body_bb.as_ptr()) };
        unsafe { LLVMPositionBuilderAtEnd(self.ptr.as_ptr(), body_bb.as_ptr()) };

        body(self, data);

        unsafe { LLVMBuildBr(self.ptr.as_ptr(), body_bb.as_ptr()) }; // loop indefinitely
        unsafe { LLVMPositionBuilderAtEnd(self.ptr.as_ptr(), cont_bb.as_ptr()) }; // continue on
    }


    pub fn loop_continue(&mut self, l: Loop) {
        unsafe { LLVMBuildBr(self.ptr.as_ptr(), l.body_bb.as_ptr()) };
        let bb = unsafe { LLVMAppendBasicBlock(self.func.llvm_val().as_ptr(), "".as_ptr() as _) };
        unsafe { LLVMPositionBuilderAtEnd(self.ptr.as_ptr(), bb) };
    }


    pub fn loop_break(&mut self, l: Loop) {
        unsafe { LLVMBuildBr(self.ptr.as_ptr(), l.cont_bb.as_ptr()) };
        let bb = unsafe { LLVMAppendBasicBlock(self.func.llvm_val().as_ptr(), "".as_ptr() as _) };
        unsafe { LLVMPositionBuilderAtEnd(self.ptr.as_ptr(), bb) };
    }


    pub fn ite<T>(
        &mut self,
        data: &mut T,
        cond: Bool<'ctx>,
        then_body: impl FnOnce(&mut Self, &mut T),
        else_body: impl FnOnce(&mut Self, &mut T),
    ) {
        let then_bb = unsafe { LLVMAppendBasicBlock(self.func.llvm_val().as_ptr(), cstr!("then")) };
        let else_bb = unsafe { LLVMAppendBasicBlock(self.func.llvm_val().as_ptr(), cstr!("else")) };
        let cont_bb = unsafe { LLVMAppendBasicBlock(self.func.llvm_val().as_ptr(), cstr!("cont")) };

        unsafe { LLVMBuildCondBr(self.ptr.as_ptr(), cond.llvm_val().as_ptr(), then_bb, else_bb) };


        // then branch
        unsafe { LLVMPositionBuilderAtEnd(self.ptr.as_ptr(), then_bb) };
        then_body(self, data);
        unsafe { LLVMBuildBr(self.ptr.as_ptr(), cont_bb) };

        // else branch
        unsafe { LLVMPositionBuilderAtEnd(self.ptr.as_ptr(), else_bb) };
        else_body(self, data);
        unsafe { LLVMBuildBr(self.ptr.as_ptr(), cont_bb) };

        // continue
        unsafe { LLVMPositionBuilderAtEnd(self.ptr.as_ptr(), cont_bb) };
    }


    pub fn switch<T>(&mut self, value: Integer<'ctx>, datas: impl Iterator<Item=T>, mut f: impl FnMut(&mut Self, T)) {
        let end_bb = unsafe { LLVMAppendBasicBlock(self.func.llvm_val().as_ptr(), cstr!("switch_end")) };
        let switch = unsafe { LLVMBuildSwitch(self.ptr.as_ptr(), value.llvm_val().as_ptr(), end_bb, datas.size_hint().0 as u32) };

        for (i, d) in datas.into_iter().enumerate() {
            let bb = unsafe { LLVMAppendBasicBlock(self.func.llvm_val().as_ptr(), cstr!("switch_br")) };
            unsafe { LLVMPositionBuilderAtEnd(self.ptr.as_ptr(), bb) };

            f(self, d);

            unsafe { LLVMBuildBr(self.ptr.as_ptr(), end_bb) };

            let int = self.const_int(value.ty(), i as i64, true);
            unsafe { LLVMAddCase(switch, int.llvm_val().as_ptr(), bb) };
        }
        
        unsafe { LLVMPositionBuilderAtEnd(self.ptr.as_ptr(), end_bb) };
    }

    
    pub fn alloca(&self, ty: Type<'ctx>) -> Ptr<'ctx> {
        let bb = unsafe { LLVMGetInsertBlock(self.ptr.as_ptr()) };

        // having all the allocas in the starting 
        // BB is better for optimisation
        unsafe { LLVMPositionBuilderAtEnd(self.ptr.as_ptr(), self.prelude.as_ptr()) };
        let ptr = unsafe { LLVMBuildAlloca(self.ptr.as_ptr(),
                                            ty.llvm_ty().as_ptr(),
                                            "".as_ptr().cast()) };

        // switch back to the original position
        unsafe { LLVMPositionBuilderAtEnd(self.ptr.as_ptr(), bb) };

        let ptr = NonNull::new(ptr).expect("failed to build alloca");
        let ptr = Value::new(ptr);
        unsafe { Ptr::new(ptr) }
    }
    

    pub fn store(&self, ptr: Ptr<'ctx>, val: Value<'ctx>) {
        unsafe { LLVMBuildStore(self.ptr.as_ptr(), val.llvm_val().as_ptr(), ptr.llvm_val().as_ptr()) };
    }


    pub fn load(&self, ptr: Ptr<'ctx>, ty: Type<'ctx>) -> Value<'ctx> {
        let ptr = unsafe { LLVMBuildLoad2(self.ptr.as_ptr(), ty.llvm_ty().as_ptr(),
                                ptr.llvm_val().as_ptr(), cstr!("load")) };

        Value::new(NonNull::new(ptr).unwrap())
    }


    pub fn alloca_store(&self, val: Value<'ctx>) -> Ptr<'ctx> {
        let ptr = self.alloca(val.ty());
        self.store(ptr, val);
        ptr
    }


    pub fn local(&mut self, ty: Type<'ctx>) -> Local {
        let alloc = self.alloca(ty);
        self.locals.push((alloc, ty))
    }


    pub fn local_ptr(&self, local: Local) -> Ptr<'ctx> {
        self.locals[local].0
    }


    pub fn local_ty(&self, local: Local) -> Type<'ctx> {
        self.locals[local].1
    }


    pub fn local_set(&mut self, local: Local, val: Value<'ctx>) {
        let local = self.local_ptr(local);
        self.store(local, val)
    }


    pub fn local_get(&mut self, local: Local) -> Value<'ctx> {
        let local_ptr = self.local_ptr(local);
        let local_ty  = self.local_ty (local);

        assert!(local_ty.is_sized());

        self.load(local_ptr, local_ty)
    }


    pub fn const_unit(&self) -> Unit<'ctx> {
        let ptr = unsafe { LLVMConstInt(LLVMIntTypeInContext(self.ctx.ptr.as_ptr(), 1), 0, 0) };
        let ptr = NonNull::new(ptr).unwrap();
        let ptr = Value::new(ptr);
        unsafe { Unit::new(ptr) }
    }


    pub fn const_array(&self, ty: Type<'ctx>, vals: &[Value<'ctx>]) -> Array<'ctx> {
        let pool = Arena::tls_get_rec();
        let mut vec = sti::vec::Vec::with_cap_in(&*pool, vals.len());
        for v in vals { vec.push(unsafe { v.llvm_val().as_ptr() }) }

        let ptr = unsafe { LLVMConstArray2(ty.llvm_ty().as_ptr(), vec.as_mut_ptr(), vec.len() as u64) };
        let ptr = NonNull::new(ptr).unwrap();
        let ptr = Value::new(ptr);
        unsafe { Array::new(ptr) }
    }


    pub fn const_int(&self, ty: IntegerTy<'ctx>, val: i64, sign_extended: bool) -> Integer<'ctx> {
        if val as u64 > 2u64.saturating_pow(ty.bit_size() as u32) {
            panic!("the constant ({val}) is out of bounds of the integer size ({})", ty.bit_size());
        }

        let ptr = unsafe { LLVMConstInt(ty.llvm_ty().as_ptr(), val as u64, sign_extended as i32) };
        let ptr = NonNull::new(ptr).expect("failed to build a const int");
        let ptr = Value::new(ptr);
        unsafe { Integer::new(ptr) }
    }


    pub fn const_f32(&self, val: f32) -> FP<'ctx> {
        let ptr = unsafe { LLVMConstReal(LLVMFloatType(), val as f64) };
        let ptr = NonNull::new(ptr).expect("failed to build a const f32");
        let ptr = Value::new(ptr);
        unsafe { FP::new(ptr) }
    }


    pub fn const_f64(&self, val: f64) -> FP<'ctx> {
        let ptr = unsafe { LLVMConstReal(LLVMDoubleType(), val) };
        let ptr = NonNull::new(ptr).expect("failed to build a const f64");
        let ptr = Value::new(ptr);
        unsafe { FP::new(ptr) }
    }


    pub fn const_bool(&self, val: bool) -> Bool<'ctx> {
        let ptr = unsafe { LLVMConstInt(LLVMIntTypeInContext(self.ctx.ptr.as_ptr(), 1), val as u64, 0) };
        let ptr = NonNull::new(ptr).expect("failed to build a const bool");
        let ptr = Value::new(ptr);
        unsafe { Bool::new(ptr) }
    }


    pub fn struct_instance(&self, ty: StructTy<'ctx>, fields: &[Value<'ctx>]) -> Struct<'ctx> {
        assert!(!ty.is_opaque(), "can't create a non-opaque type");

        assert_eq!(ty.fields_count(), fields.len());
        let arena = Arena::tls_get_temp();
        let ptr = self.alloca(*ty);
        for (i, (sf, ff)) in ty.fields(&*arena).iter().zip(fields.iter()).enumerate() {
            assert_eq!(*sf, ff.ty());
            unsafe { dbg!(sf.llvm_ty(), ff.ty().llvm_ty()) };

            let ptr = self.field_ptr(ptr, ty, i);
            self.store(ptr, *ff);
        }

        self.load(ptr, *ty).as_struct()
    }


    pub fn bitcast(&self, val: Value<'ctx>, to: Type<'ctx>) -> Value<'ctx> {
        let alloca = self.alloca_store(val);
        self.load(alloca, to)
    }


    pub fn add_int(&self, lhs: Integer<'ctx>, rhs: Integer<'ctx>) -> Integer<'ctx> {
        unsafe { Integer::new(self.internal_call(LLVMBuildAdd, lhs, rhs, cstr!("addi"))) }
    }


    pub fn sub_int(&self, lhs: Integer<'ctx>, rhs: Integer<'ctx>) -> Integer<'ctx> {
        unsafe { Integer::new(self.internal_call(LLVMBuildSub, lhs, rhs, cstr!("subi"))) }
    }


    pub fn mul_int(&self, lhs: Integer<'ctx>, rhs: Integer<'ctx>) -> Integer<'ctx> {
        unsafe { Integer::new(self.internal_call(LLVMBuildMul, lhs, rhs, cstr!("muli"))) }
    }


    pub fn div_int(&self, lhs: Integer<'ctx>, rhs: Integer<'ctx>, is_signed: bool) -> Integer<'ctx> {
        let ptr  = 
            if is_signed { self.internal_call(LLVMBuildSDiv, lhs, rhs, cstr!("divs")) }
            else { self.internal_call(LLVMBuildUDiv, lhs, rhs, cstr!("divu")) };

        unsafe { Integer::new(ptr) }
    }
    

    pub fn rem_int(&self, lhs: Integer<'ctx>, rhs: Integer<'ctx>, is_signed: bool) -> Integer<'ctx> {
        let ptr  = 
            if is_signed { self.internal_call(LLVMBuildSRem, lhs, rhs, cstr!("rems")) }
            else { self.internal_call(LLVMBuildURem, lhs, rhs, cstr!("remu")) };

        unsafe { Integer::new(ptr) }
    }


    pub fn add_fp(&self, lhs: FP<'ctx>, rhs: FP<'ctx>) -> FP<'ctx> {
        unsafe { FP::new(self.internal_call(LLVMBuildFAdd, lhs, rhs, cstr!("addfp"))) }
    }


    pub fn sub_fp(&self, lhs: FP<'ctx>, rhs: FP<'ctx>) -> FP<'ctx> {
        unsafe { FP::new(self.internal_call(LLVMBuildFSub, lhs, rhs, cstr!("subfp"))) }
    }


    pub fn mul_fp(&self, lhs: FP<'ctx>, rhs: FP<'ctx>) -> FP<'ctx> {
        unsafe { FP::new(self.internal_call(LLVMBuildMul, lhs, rhs, cstr!("mulfp"))) }
    }


    pub fn div_fp(&self, lhs: FP<'ctx>, rhs: FP<'ctx>) -> FP<'ctx> {
        unsafe { FP::new(self.internal_call(LLVMBuildFDiv, lhs, rhs, cstr!("divfp"))) }
    }


    pub fn rem_fp(&self, lhs: FP<'ctx>, rhs: FP<'ctx>) -> FP<'ctx> {
        unsafe { FP::new(self.internal_call(LLVMBuildFRem, lhs, rhs, cstr!("remfp"))) }
    }


    pub fn cmp_int(&self, lhs: Integer<'ctx>, rhs: Integer<'ctx>, cmp: IntCmp) -> Bool<'ctx> {
        assert_eq!(lhs.ty().bit_size(), rhs.ty().bit_size(),
                    "the two integers can't be compared as their bit-sizes are different");

        let pred = match cmp {
            IntCmp::Eq => LLVMIntPredicate::LLVMIntEQ,
            IntCmp::Ne => LLVMIntPredicate::LLVMIntNE,
            IntCmp::SignedGt => LLVMIntPredicate::LLVMIntSGT,
            IntCmp::SignedGe => LLVMIntPredicate::LLVMIntSGE,
            IntCmp::SignedLt => LLVMIntPredicate::LLVMIntSLT,
            IntCmp::SignedLe => LLVMIntPredicate::LLVMIntSLE,
            IntCmp::UnsignedGt => LLVMIntPredicate::LLVMIntUGT,
            IntCmp::UnsignedGe => LLVMIntPredicate::LLVMIntUGE,
            IntCmp::UnsignedLt => LLVMIntPredicate::LLVMIntULT,
            IntCmp::UnsignedLe => LLVMIntPredicate::LLVMIntULE,
        };

        let ptr = unsafe { LLVMBuildICmp(self.ptr.as_ptr(), pred, lhs.llvm_val().as_ptr(),
                                        rhs.llvm_val().as_ptr(), cstr!("icmp")) };

        unsafe { Bool::new(Value::new(NonNull::new(ptr).unwrap())) }
    }


    pub fn cmp_fp(&self, lhs: FP<'ctx>, rhs: FP<'ctx>, cmp: FPCmp) -> Bool<'ctx> {
        let pred = match cmp {
            FPCmp::Eq => LLVMRealPredicate::LLVMRealUEQ,
            FPCmp::Ne => LLVMRealPredicate::LLVMRealUNE,
            FPCmp::Gt => LLVMRealPredicate::LLVMRealUGT,
            FPCmp::Ge => LLVMRealPredicate::LLVMRealUGE,
            FPCmp::Lt => LLVMRealPredicate::LLVMRealULT,
            FPCmp::Le => LLVMRealPredicate::LLVMRealULE,
        };

        let ptr = unsafe { LLVMBuildFCmp(self.ptr.as_ptr(), pred, lhs.llvm_val().as_ptr(),
                                        rhs.llvm_val().as_ptr(), cstr!("fcmp")) };

        unsafe { Bool::new(Value::new(NonNull::new(ptr).unwrap())) }
    }


    pub fn bool_eq(&self, lhs: Bool<'ctx>, rhs: Bool<'ctx>) -> Bool<'ctx> {
        self.cmp_int(lhs.as_integer(), rhs.as_integer(), IntCmp::Eq)
    }


    pub fn bool_ne(&self, lhs: Bool<'ctx>, rhs: Bool<'ctx>) -> Bool<'ctx> {
        self.cmp_int(lhs.as_integer(), rhs.as_integer(), IntCmp::Ne)
    }


    pub fn and(&self, lhs: Integer<'ctx>, rhs: Integer<'ctx>) -> Integer<'ctx> {
        unsafe { Integer::new(self.internal_call(LLVMBuildAnd, lhs, rhs, cstr!("and"))) }
    }


    pub fn or(&self, lhs: Integer<'ctx>, rhs: Integer<'ctx>) -> Integer<'ctx> {
        unsafe { Integer::new(self.internal_call(LLVMBuildOr, lhs, rhs, cstr!("or"))) }
    }


    pub fn xor(&self, lhs: Integer<'ctx>, rhs: Integer<'ctx>) -> Integer<'ctx> {
        unsafe { Integer::new(self.internal_call(LLVMBuildXor, lhs, rhs, cstr!("xor"))) }
    }


    pub fn int_not(&self, lhs: Integer<'ctx>) -> Integer<'ctx> {
        let ptr = unsafe { LLVMBuildNot(self.ptr.as_ptr(), lhs.llvm_val().as_ptr(), cstr!("not")) };
        unsafe { Integer::new(Value::new(NonNull::new(ptr).unwrap())) }
    }


    pub fn shl(&self, lhs: Integer<'ctx>, rhs: Integer<'ctx>) -> Integer<'ctx> {
        unsafe { Integer::new(self.internal_call(LLVMBuildShl, lhs, rhs, cstr!("shl"))) }
    }


    pub fn shr(&self, lhs: Integer<'ctx>, rhs: Integer<'ctx>, is_signed: bool) -> Integer<'ctx> {
        let ptr  = 
            if is_signed { self.internal_call(LLVMBuildAShr, lhs, rhs, cstr!("ashr")) }
            else { self.internal_call(LLVMBuildLShr, lhs, rhs, cstr!("lshr")) };

        unsafe { Integer::new(ptr) }
    }


    pub fn int_cast(&self, from: Integer<'ctx>, to: Type<'ctx>, is_signed: bool) -> Value<'ctx> {
        let ptr = unsafe { LLVMBuildIntCast2(self.ptr.as_ptr(), from.llvm_val().as_ptr(),
                                             to.llvm_ty().as_ptr(), is_signed as i32,
                                             cstr!("icast")) };

        Value::new(NonNull::new(ptr).unwrap())
    }


    pub fn fp_cast(&self, from: FP<'ctx>, to: Type<'ctx>) -> Value<'ctx> {
        let ptr = unsafe { LLVMBuildFPCast(self.ptr.as_ptr(), from.llvm_val().as_ptr(),
                                             to.llvm_ty().as_ptr(), cstr!("fcast")) };

        Value::new(NonNull::new(ptr).unwrap())
    }


    pub fn bool_not(&self, lhs: Bool<'ctx>) -> Bool<'ctx> {
        let ptr = unsafe { LLVMBuildNot(self.ptr.as_ptr(), lhs.llvm_val().as_ptr(), cstr!("bnot")) };
        unsafe { Bool::new(Value::new(NonNull::new(ptr).unwrap())) }
    }


    pub fn field_load(&self, lhs: Struct<'ctx>, index: usize) -> Value<'ctx> {
        assert!(index < lhs.ty().fields_count());

        let alloca = self.alloca_store(*lhs);
        let ptr = unsafe { LLVMBuildStructGEP2(self.ptr.as_ptr(),
                                               lhs.ty().llvm_ty().as_ptr(),
                                               alloca.llvm_val().as_ptr(),
                                               index as u32,
                                               cstr!("field_load")) };

        let ptr = unsafe { Ptr::new(Value::new(NonNull::new(ptr).unwrap())) };
        let arena = Arena::tls_get_rec();
        let arr = lhs.ty().fields(&*arena);
        self.load(ptr, arr[index])
    }


    pub fn field_ptr(&self, lhs: Ptr<'ctx>, ty: StructTy<'ctx>, index: usize) -> Ptr<'ctx> {
        let ptr = unsafe { LLVMBuildStructGEP2(self.ptr.as_ptr(),
                                               ty.llvm_ty().as_ptr(),
                                               lhs.llvm_val().as_ptr(),
                                               index as u32,
                                               cstr!("field_load")) };

        unsafe { Ptr::new(Value::new(NonNull::new(ptr).unwrap())) }
    }


    pub fn ret(&self, val: Value<'ctx>) {
        unsafe { LLVMBuildRet(self.ptr.as_ptr(), val.llvm_val().as_ptr()) };
        let bb = unsafe { LLVMAppendBasicBlock(self.func.llvm_val().as_ptr(), "".as_ptr() as _) };
        unsafe { LLVMPositionBuilderAtEnd(self.ptr.as_ptr(), bb) };
    }


    pub fn ret_void(&self) {
        unsafe { LLVMBuildRetVoid(self.ptr.as_ptr()) };
        let bb = unsafe { LLVMAppendBasicBlock(self.func.llvm_val().as_ptr(), "".as_ptr() as _) };
        unsafe { LLVMPositionBuilderAtEnd(self.ptr.as_ptr(), bb) };
    }


    fn internal_call(&self,
                func: unsafe extern "C" fn(*mut LLVMBuilder, *mut LLVMValue, *mut LLVMValue, *const i8) -> *mut LLVMValue,
                v1  : impl Deref<Target=Value<'ctx>>,
                v2  : impl Deref<Target=Value<'ctx>>,
                name: *const i8,
                ) -> Value<'ctx> {
        let ptr = unsafe { func(self.ptr.as_ptr(), v1.llvm_val().as_ptr(),
                                        v2.deref().llvm_val().as_ptr(), name) };
        Value::new(NonNull::new(ptr).unwrap())
    }

    pub fn call(&self, func: FunctionPtr<'ctx>, func_ty: FunctionType<'ctx>, args: &[Value<'ctx>]) -> Value<'ctx> {
        for (af, aa) in func_ty.args().iter().zip(args.iter()) {
            assert_eq!(*af, aa.ty());
        }

        let is_void = func_ty.ret().kind() == TypeKind::Void;
        let name = if is_void { "".as_ptr() as *const i8 }
                   else { cstr!("name") };

        let ptr = unsafe { LLVMBuildCall2(self.ptr.as_ptr(), func_ty.llvm_ty().as_ptr(),
                                          func.llvm_val().as_ptr(), args.as_ptr().cast_mut().cast(),
                                          args.len() as u32, name) };

        Value::new(NonNull::new(ptr).unwrap())
    }
}


#[derive(Clone, Copy, Debug)]
pub enum IntCmp {
    Eq,
    Ne,
    SignedGt,
    SignedGe,
    SignedLt,
    SignedLe,
    UnsignedGt,
    UnsignedGe,
    UnsignedLt,
    UnsignedLe,
}


#[derive(Clone, Copy, Debug)]
pub enum FPCmp {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
}

impl<'ctx> Drop for Builder<'ctx> {
    fn drop(&mut self) {
        self.build_internal();
        unsafe { LLVMDisposeBuilder(self.ptr.as_ptr()) };
    }
}


