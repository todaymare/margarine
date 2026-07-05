use std::ops::Deref;

use llvm_sys::core::{LLVMIsOpaqueStruct, LLVMStructSetBody};

use crate::{ctx::ContextRef, module::Module, tys::TypeKind};

use super::Type;

#[derive(Debug, Clone, Copy)]
pub struct UnionTy<'ctx>(Type<'ctx>);


impl<'ctx> UnionTy<'ctx> {

    /// # Safety
    /// Undefined behaviour if `ty` isn't a union
    pub unsafe fn new(ty: Type<'ctx>) -> Self {
        debug_assert!(matches!(ty.kind(), TypeKind::Struct));

        Self(ty)
    }


    pub fn is_opaque(self) -> bool {
        unsafe { LLVMIsOpaqueStruct(self.llvm_ty().as_ptr()) == 1 } 
    }


    pub fn set_fields(self, ctx: ContextRef<'ctx>, module: Module<'ctx>, fields: &[Type<'ctx>]) {
        assert!(self.is_opaque());

        let mut max_size = 1usize;
        let mut max_align = 1usize;

        for ty in fields {
            max_size = max_size.max(ty.size_of(module).unwrap());
            max_align = max_align.max(ty.align_of(module).unwrap());
        }

        // round size up to the max alignment so every payload fits with correct alignment
        let rem = max_size % max_align;
        if rem != 0 {
            max_size += max_align - rem;
        }

        let buffer =
            if max_align >= 16 {
                let count = max_size / 16;
                ctx.array(*ctx.integer(128), count)
            } else if max_align >= 8 {
                let count = max_size / 8;
                ctx.array(*ctx.integer(64), count)
            } else {
                let count = max_size;
                ctx.array(*ctx.integer(8), count)
            };

        unsafe { LLVMStructSetBody(self.llvm_ty().as_ptr(),
                                   [buffer.llvm_ty().as_ptr()].as_mut_ptr(),
                                   1, 0) };
    }
}


impl<'ctx> Deref for UnionTy<'ctx> {
    type Target = Type<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
