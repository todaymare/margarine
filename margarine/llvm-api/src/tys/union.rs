use std::ops::Deref;

use llvm_sys::{core::{LLVMGetStructName, LLVMIsOpaqueStruct, LLVMStructSetBody}, LLVMType};
use sti::{arena::Arena, format_in};

use crate::{ctx::{Context, ContextRef}, info::Message, module::Module, tys::TypeKind};

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

        let pool = Arena::tls_get_temp();

        let index_ty = ctx.integer(fields.len().count_ones().next_power_of_two());
        let name = self.name();

        let mut max = 0;

        for ty in fields {
            let name = format_in!(&*pool, "{}<{}>", name, ty.name());
            let strct = ctx.structure(&*name);

            strct.set_fields(&[*index_ty, *ty]);

            let size = strct.size_of(module).unwrap();
            max = max.max(size);
        }

        let unit = ctx.integer(8);
        let array = ctx.array(*unit, max.next_power_of_two().div_ceil(8));

        unsafe { LLVMStructSetBody(self.llvm_ty().as_ptr(),
                                   [index_ty.llvm_ty().as_ptr(), array.llvm_ty().as_ptr()].as_mut_ptr(),
                                   2, 0) };
    }
}


impl<'ctx> Deref for UnionTy<'ctx> {
    type Target = Type<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
