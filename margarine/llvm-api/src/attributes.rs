use std::ptr::NonNull;

use llvm_sys::{
    core::{LLVMCreateEnumAttribute, LLVMCreateTypeAttribute, LLVMGetEnumAttributeKindForName},
    LLVMAttributeFunctionIndex, LLVMAttributeIndex, LLVMAttributeReturnIndex,
};

use crate::{ctx::ContextRef, tys::Type};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttributeLocation {
    Return,
    Parameter(usize),
    Function,
}

impl AttributeLocation {
    pub fn to_llvm_index(self) -> LLVMAttributeIndex {
        match self {
            AttributeLocation::Return => LLVMAttributeReturnIndex,
            AttributeLocation::Function => LLVMAttributeFunctionIndex,
            AttributeLocation::Parameter(index) => (index + 1) as LLVMAttributeIndex,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttributeKind<'ctx> {
    Type { name: &'static str, ty: Type<'ctx> },
    Enum { name: &'static str, value: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AbiAttribute<'ctx> {
    pub location: AttributeLocation,
    pub kind: AttributeKind<'ctx>,
}

impl<'ctx> AbiAttribute<'ctx> {
    pub fn new(location: AttributeLocation, kind: AttributeKind<'ctx>) -> Self {
        Self { location, kind }
    }

    /// Creates the LLVM attribute represented by this ABI attribute.
    ///
    /// Attribute names are static by construction, so the pointers passed to
    /// LLVM are valid for the duration of the call and LLVM copies the name
    /// into its context-owned attribute storage.
    pub(crate) fn to_llvm_attribute(self, ctx: ContextRef<'ctx>) -> NonNull<llvm_sys::LLVMOpaqueAttributeRef> {
        let attribute = match self.kind {
            AttributeKind::Type { name, ty } => {
                let kind = unsafe {
                    LLVMGetEnumAttributeKindForName(name.as_ptr().cast(), name.len())
                };
                assert_ne!(kind, 0, "unknown LLVM type attribute '{name}'");
                unsafe { LLVMCreateTypeAttribute(ctx.ptr.as_ptr(), kind, ty.llvm_ty().as_ptr()) }
            }
            AttributeKind::Enum { name, value } => {
                let kind = unsafe {
                    LLVMGetEnumAttributeKindForName(name.as_ptr().cast(), name.len())
                };
                assert_ne!(kind, 0, "unknown LLVM enum attribute '{name}'");
                unsafe { LLVMCreateEnumAttribute(ctx.ptr.as_ptr(), kind, value) }
            }
        };
        NonNull::new(attribute).expect("LLVM failed to create ABI attribute")
    }
}
