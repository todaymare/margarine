use std::{ops::Deref, ptr::NonNull};

use llvm_sys::{core::{LLVMAddAttributeAtIndex, LLVMCreateBuilderInContext, LLVMCreateEnumAttribute, LLVMCreateStringAttribute, LLVMCreateTypeAttribute, LLVMGetEnumAttributeKindForName, LLVMSetLinkage}, LLVMAttributeFunctionIndex, LLVMAttributeReturnIndex, LLVMLinkage};

use crate::{builder::Builder, cstr, ctx::ContextRef, tys::{func::FunctionType, ptr::PtrTy, Type, TypeKind}};

use super::Value;

#[derive(Clone, Copy, Debug)]
pub struct FunctionPtr<'ctx>(Value<'ctx>);


impl<'ctx> FunctionPtr<'ctx> {
    /// # Safety
    /// Undefined behaviour if the value isn't a function
    pub unsafe fn new(val: Value<'ctx>) -> Self {
        debug_assert_eq!(val.ty().kind(), TypeKind::Ptr);

        Self(val)
    }


    pub fn builder(self, ctx: ContextRef<'ctx>, ty: FunctionType<'ctx>) -> Builder<'ctx> {
        let ptr = unsafe { LLVMCreateBuilderInContext(ctx.ptr.as_ptr()) };
        Builder::new(NonNull::new(ptr).unwrap(), ctx, self, ty)
    }


    pub fn ty(self) -> PtrTy<'ctx> { unsafe { PtrTy::new(self.deref().ty()) } }


    pub fn set_linkage(self, linkage: Linkage) {
        unsafe { LLVMSetLinkage(self.llvm_val().as_ptr(), linkage.llvm_linkage()); }
    }


    pub fn set_noreturn(self, ctx: ContextRef<'ctx>) {
        let attr_kind = unsafe { LLVMGetEnumAttributeKindForName(cstr!("noreturn"), 8) };
        let attr = unsafe { LLVMCreateEnumAttribute(ctx.ptr.as_ptr(), attr_kind, 0) };
        unsafe { LLVMAddAttributeAtIndex(self.llvm_val().as_ptr(), LLVMAttributeFunctionIndex, attr) };
    }


    pub fn set_cold(self, ctx: ContextRef<'ctx>) {
        let attr_kind = unsafe { LLVMGetEnumAttributeKindForName(cstr!("cold"), 4) };
        let attr = unsafe { LLVMCreateEnumAttribute(ctx.ptr.as_ptr(), attr_kind, 0) };
        unsafe { LLVMAddAttributeAtIndex(self.llvm_val().as_ptr(), LLVMAttributeFunctionIndex, attr) };
    }




    /// Marks the result as a fresh allocation that cannot alias existing pointers.
    pub fn set_noalias_return(self, ctx: ContextRef<'ctx>) {
        let attr_kind = unsafe { LLVMGetEnumAttributeKindForName(cstr!("noalias"), 7) };
        let attr = unsafe { LLVMCreateEnumAttribute(ctx.ptr.as_ptr(), attr_kind, 0) };
        unsafe { LLVMAddAttributeAtIndex(self.llvm_val().as_ptr(), LLVMAttributeReturnIndex, attr) };
    }


    /// Records the argument containing the allocation size.
    pub fn set_alloc_size(self, ctx: ContextRef<'ctx>, size_arg: u32) {
        let attr_kind = unsafe { LLVMGetEnumAttributeKindForName(cstr!("allocsize"), 9) };
        // The low 32 bits hold the zero-based element-size argument; UINT32_MAX omits a count argument.
        let raw_value = u64::from(size_arg) << 32 | u64::from(u32::MAX);
        let attr = unsafe { LLVMCreateEnumAttribute(ctx.ptr.as_ptr(), attr_kind, raw_value) };
        unsafe { LLVMAddAttributeAtIndex(self.llvm_val().as_ptr(), LLVMAttributeFunctionIndex, attr) };
    }


    /// Gives LLVM the allocation/deallocation operation encoded by this function.
    pub fn set_alloc_kind(self, ctx: ContextRef<'ctx>, kind: AllocKind) {
        let attr_kind = unsafe { LLVMGetEnumAttributeKindForName(cstr!("allockind"), 9) };
        let attr = unsafe { LLVMCreateEnumAttribute(ctx.ptr.as_ptr(), attr_kind, kind as u64) };
        unsafe { LLVMAddAttributeAtIndex(self.llvm_val().as_ptr(), LLVMAttributeFunctionIndex, attr) };
    }


    /// Associates this function with LLVM's malloc allocation family.
    pub fn set_malloc_family(self, ctx: ContextRef<'ctx>) {
        let attr = unsafe {
            LLVMCreateStringAttribute(
                ctx.ptr.as_ptr(),
                cstr!("alloc-family"),
                12,
                cstr!("malloc"),
                6,
            )
        };
        unsafe { LLVMAddAttributeAtIndex(self.llvm_val().as_ptr(), LLVMAttributeFunctionIndex, attr) };
    }


    pub fn set_sret(self, ctx: ContextRef<'ctx>, ty: Type<'ctx>) {
        let attr_kind = unsafe { LLVMGetEnumAttributeKindForName(cstr!("sret"), 4) };
        let attr = unsafe { LLVMCreateTypeAttribute(ctx.ptr.as_ptr(), attr_kind, ty.llvm_ty().as_ptr()) };
        unsafe { LLVMAddAttributeAtIndex(self.llvm_val().as_ptr(), 1, attr) };
    }
}


impl<'ctx> Deref for FunctionPtr<'ctx> {
    type Target = Value<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}


pub enum FunctionAttribute {
    NoReturn,
}


#[repr(u64)]
pub enum AllocKind {
    AllocUninitialized = 1 | 8,
    Free = 4,
}


#[derive(Clone, Copy, Debug)]
pub enum Linkage {
    External,
    AvailableExternally,
    LinkOnceAny,
    LinkOnceODR,
    LinkONceODRAutoHide,
    WeakAny,
    WeakODR,
    Appending,
    Internal,
    Private,
    LLImport,
    LLExport,
    ExternalWeak,
    Ghost,
    Common,
    LinkerPrivate,
    LinkerPrivateWeak,
}
impl Linkage {
    pub fn llvm_linkage(self) -> LLVMLinkage {
        match self {
            Linkage::External => LLVMLinkage::LLVMExternalLinkage,
            Linkage::AvailableExternally => LLVMLinkage::LLVMAvailableExternallyLinkage,
            Linkage::LinkOnceAny => LLVMLinkage::LLVMLinkOnceAnyLinkage,
            Linkage::LinkOnceODR => LLVMLinkage::LLVMLinkOnceODRLinkage,
            Linkage::LinkONceODRAutoHide => LLVMLinkage::LLVMLinkOnceODRAutoHideLinkage,
            Linkage::WeakAny => LLVMLinkage::LLVMWeakAnyLinkage,
            Linkage::WeakODR => LLVMLinkage::LLVMWeakODRLinkage,
            Linkage::Appending => LLVMLinkage::LLVMAppendingLinkage,
            Linkage::Internal => LLVMLinkage::LLVMInternalLinkage,
            Linkage::Private => LLVMLinkage::LLVMLinkerPrivateLinkage,
            Linkage::LLImport => LLVMLinkage::LLVMDLLImportLinkage,
            Linkage::LLExport => LLVMLinkage::LLVMDLLExportLinkage,
            Linkage::ExternalWeak => LLVMLinkage::LLVMExternalWeakLinkage,
            Linkage::Ghost => LLVMLinkage::LLVMGhostLinkage,
            Linkage::Common => LLVMLinkage::LLVMCommonLinkage,
            Linkage::LinkerPrivate => LLVMLinkage::LLVMLinkerPrivateLinkage,
            Linkage::LinkerPrivateWeak => LLVMLinkage::LLVMLinkerPrivateWeakLinkage,
        }
    }
}
