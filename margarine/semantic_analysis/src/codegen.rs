use std::collections::HashMap;

use common::string_map::{StringIndex, StringMap};
use lexer::Literal;
use llvm_api::{builder::{Builder, Local, Loop}, tys::IsType, values::{IsValue, PointerValue, Value}, Context, Function, Module};
use parser::nodes::{decl::Decl, expr::{BinaryOperator, Expr, ExprId, UnaryOperator}, stmt::{Stmt, StmtId}, NodeId, AST};
use sti::{arena::Arena, format_in, vec::Vec};

use crate::{types::{containers::ContainerKind, func::FunctionKind, ty::{Type, TypeHash}, SymbolId, SymbolKind, SymbolMap}, TyChecker, TyInfo};

pub struct Codegen<'me, 'out, 'ast, 'str> {
    string_map: &'me StringMap<'str>,
    syms: &'me mut SymbolMap<'out>,
    ast: &'me AST<'ast>,
    module: Module,

    ty_info: &'me TyInfo<'out>,
    ty_mappings: HashMap<TypeHash, ConvertedTy>
}


pub struct Env<'me> {
    vars: Vec<(StringIndex, Local), &'me Arena>,
    loop_id: Option<Loop>,
}


impl<'me, 'out, 'ast, 'str> Codegen<'me, 'out, 'ast, 'str> {
    pub fn run(ty_checker: &mut TyChecker) -> (Context, Module) {
        let mut ctx = Context::new();
        let module = Module::new(&mut ctx, "margarine");

        let mut codegen = Codegen {
            string_map: ty_checker.string_map,
            syms: &mut ty_checker.syms,
            ast: ty_checker.ast,
            module,
            ty_info: &ty_checker.type_info,
            ty_mappings: HashMap::new(),
        };

        {
            macro_rules! register {
                ($enum: ident, $call: expr) => {
                    codegen.ty_mappings.insert(Type::$enum.hash(codegen.syms), ConvertedTy::Struct($call.ty()));
                };
            }


            register!(I8 , ctx.signed_int(8 ));
            register!(I16, ctx.signed_int(16));
            register!(I32, ctx.signed_int(32));
            register!(I64, ctx.signed_int(64));
            register!(U8 , ctx.unsigned_int(8 ));
            register!(U16, ctx.unsigned_int(16));
            register!(U32, ctx.unsigned_int(32));
            register!(U64, ctx.unsigned_int(64));
            register!(F32, ctx.f32());
            register!(F64, ctx.f64());
            register!(BOOL, ctx.bool());
            register!(UNIT, ctx.zst());
        }

        let void = ctx.void();
        let mut b = Function::new(&mut ctx, module, "__initStartupSystems__", void.ty(), &[]);

        for sym in &ty_checker.startups {
            let func = codegen.get_func(b.ctx(), *sym, &[]);
            b.call(func, &[]);
        }


        b.ret_void();
        b.build();

        (ctx, module)
    }


    fn ty_to_llvm(&mut self, ctx: &mut Context, ty: Type) -> ConvertedTy {
        let hash = ty.hash(self.syms);
        if let Some(ty) = self.ty_mappings.get(&hash) { return *ty }

        let gens = ty.gens(self.syms);

        let sym_id = ty.sym(self.syms).unwrap();
        let sym = self.syms.sym(sym_id);

        let name = ty.display(self.string_map, self.syms);

        if sym_id == SymbolId::PTR {
            let ty = ctx.pointer(llvm_api::tys::AddressSpace::Program);
            let ty = ConvertedTy::Struct(ty.ty());
            self.ty_mappings.insert(hash, ty);
            return ty;
        }

        match sym.kind {
            SymbolKind::Function(_) => todo!(),

            SymbolKind::Container(cont) => {
                match cont.kind {
                    ContainerKind::Struct => {
                        let strct = ctx.structure(name);
                        self.ty_mappings.insert(hash, ConvertedTy::Struct(strct.ty()));

                        let pool = Arena::tls_get_rec();
                        let fields = {
                            let mut vec = Vec::with_cap_in(&*pool, cont.fields.len());
                            for i in cont.fields {
                                let ty = i.1.to_ty(gens, self.syms).unwrap();
                                vec.push(self.ty_to_llvm(ctx, ty).base());
                            }

                            vec.leak()
                        };

                        strct.set_fields(ctx, fields);
                        ConvertedTy::Struct(strct.ty())

                    },


                    ContainerKind::Enum => {
                        let pool = Arena::tls_get_rec();
                        let union = ctx.union(&*format_in!(&*pool, "{name}_union"));
                        let strct = ctx.structure(name);

                        self.ty_mappings.insert(hash, ConvertedTy::Enum { base: strct.ty(), union: union.ty() });

                        let fields = {
                            let mut vec = Vec::with_cap_in(&*pool, cont.fields.len());
                            for i in cont.fields {
                                let ty = i.1.to_ty(gens, self.syms).unwrap();
                                vec.push(self.ty_to_llvm(ctx, ty).base());
                            }

                            vec.leak()
                        };

                        let index = (usize::BITS - fields.len().leading_zeros()).max(1);
                        let index = ctx.unsigned_int(index);

                        union.set_fields(ctx, self.module, fields);
                        strct.set_fields(ctx, &[index.ty(), union.ty()]);
                        ConvertedTy::Enum { base: strct.ty(), union: union.ty() }
                    },

                    ContainerKind::Tuple => todo!(),
                }
            },
        }
    }


    pub fn block(&mut self, builder: &mut Builder, env: &mut Env, block: &[NodeId]) -> Result<(Value, Type), ()> {
        for (i, n) in block.iter().enumerate() {
            let res = match *n {
                NodeId::Decl(_) => (builder.unit(), Type::UNIT),

                NodeId::Stmt(v) => {
                    self.stmt(builder, env, v);
                    (builder.unit(), Type::UNIT)
                },

                NodeId::Expr(v) => self.expr(builder, env, v)?,

                NodeId::Err(v) => {
                    self.error(builder, v);
                    return Err(())
                },
            };

            if i == block.len() - 1 {
                return Ok(res);
            }
        }

        Ok((builder.unit(), Type::UNIT))
    }


    pub fn stmt(&mut self, builder: &mut Builder, env: &mut Env, stmt: StmtId) {
        if let Some(e) = self.ty_info.stmt(stmt) {
            self.error(builder, e);
        };


        let _ = (|| {
        match self.ast.stmt(stmt) {
            Stmt::Variable { name, rhs, .. } => {
                let expr = self.expr(builder, env, rhs)?;
                let ty = self.ty_to_llvm(builder.ctx(), expr.1);
                let local = builder.local(ty.base());
                builder.local_set(local, expr.0);

                env.vars.push((name, local));
            },


            Stmt::VariableTuple { names, hint, rhs } => todo!(),


            Stmt::UpdateValue { lhs, rhs } => {
                let rhs = self.expr(builder, env, rhs)?;
                self.assign(builder, env, lhs, rhs.0)
            },


            Stmt::ForLoop { binding, expr, body } => todo!(),
        };
        Ok::<(), ()>(())
        })();
    }


    pub fn assign(&mut self, builder: &mut Builder, env: &mut Env, expr: ExprId, value: Value) {
        let ptr = self.assign_ptr(builder, env, expr);
        builder.ptr_store(value, ptr);
    }



    pub fn assign_ptr(&mut self, builder: &mut Builder, env: &mut Env, expr: ExprId) -> PointerValue {
        match self.ast.expr(expr) {
            Expr::Identifier(ident) => {
                let local = env.vars.iter().rev().find(|x| x.0 == ident).unwrap();

                builder.local_ptr(local.1)
            },


            Expr::Deref(v) => todo!(),


            Expr::AccessField { val, field_name } => todo!(),

            Expr::Unwrap(_) => todo!(),
            Expr::OrReturn(_) => todo!(),

            _ => unreachable!()
        }
    }


    pub fn expr(&mut self, builder: &mut Builder, env: &mut Env, expr: ExprId) -> Result<(Value, Type), ()> {
        let ty = match self.ty_info.expr(expr) {
            Ok(e) => e,
            Err(e) => {
                self.error(builder, e);
                return Err(());
            },
        };

        Ok((match self.ast.expr(expr) {
            Expr::Unit => builder.unit().value(),


            Expr::Literal(v) => {
                match v {
                    Literal::Integer(v) => {
                        let ty = builder.ctx().signed_int(64);
                        builder.constant(ty, v).value()
                    },


                    Literal::Float(v) => {
                        let ty = builder.ctx().f64();
                        builder.constant(ty, v.inner()).value()
                    },


                    Literal::String(v) => {
                        let ty = self.ty_to_llvm(builder.ctx(), Type::STR).base().as_struct();

                        let string = self.string_map.get(v);
                        let str = format!("\x01\x00\x00\x00\x00\x00\x00\x00{}", string);
                        let str = builder.string(&str, self.module);

                        let ptr = builder.str_to_ptr(str);

                        let len_ty = builder.ctx().unsigned_int(32);
                        let len = builder.constant(len_ty, string.len() as u64);

                        builder.constant(ty, &[ptr.value(), len.value()]).value()
                    },


                    Literal::Bool(v) => {
                        let ty = builder.ctx().bool();
                        builder.constant(ty, v).value()
                    },
                }
            },


            Expr::Identifier(v) => {
                let local = env.vars.iter().rev().find(|x| x.0 == v).unwrap().1;
                builder.local_get(local)
            },


            Expr::Deref(v) => {
                let (value, value_ty) = self.expr(builder, env, v)?;
                let value_ty = self.ty_to_llvm(builder.ctx(), value_ty).base();
                let value = value.as_ptr();

                let strct = builder.ptr_load(value_ty, value);
                let strct = strct.as_structure();

                let ret_ty = self.ty_to_llvm(builder.ctx(), ty).base();

                let ptr = builder.field_load(strct, ret_ty, 1);
                builder.ptr_load(ret_ty, ptr)
            },


            Expr::Range { lhs, rhs } => {
                let lhs = self.expr(builder, env, lhs)?;
                let rhs = self.expr(builder, env, rhs)?;

                let ty = builder.ctx().signed_int(64);
                let lhs = builder.cast_sint(lhs.0.as_signed_int(), ty);
                let rhs = builder.cast_sint(rhs.0.as_signed_int(), ty);

                let strct = self.ty_to_llvm(builder.ctx(), Type::RANGE).base().as_struct();
                builder.constant(strct, &[lhs.value(), rhs.value()]).value()
            },


            Expr::BinaryOp { operator, lhs, rhs } => {
                let lhs = self.expr(builder, env, lhs)?;
                let rhs = self.expr(builder, env, rhs)?;

                macro_rules! m {
                    ($l: expr, $r: expr, $($variant: ident, $func: ident);*) => {
                        match operator {
                            $(BinaryOperator::$variant => builder.$func($l, $r).value(),)*

                            #[allow(unreachable_patterns)]
                            _ => unreachable!(),
                        }
                    };
                }

                let sym = lhs.1.sym(self.syms).unwrap();
                if sym.is_sint() {
                    let l = lhs.0.as_signed_int();
                    let r = rhs.0.as_signed_int();

                    m!(l, r,
                        Add, add;
                        Sub, sub;
                        Mul, mul;
                        Div, div;
                        Rem, rem;
                        BitshiftLeft, shl;
                        BitshiftRight, shr;
                        BitwiseAnd, and;
                        BitwiseOr, or;
                        BitwiseXor, xor;
                        Eq, eq;
                        Ne, ne;
                        Gt, gt;
                        Ge, ge;
                        Lt, lt;
                        Le, le
                    )
                } else if sym.is_int() { 
                    let l = lhs.0.as_unsigned_int();
                    let r = rhs.0.as_unsigned_int();

                    m!(l, r,
                        Add, add;
                        Sub, sub;
                        Mul, mul;
                        Div, div;
                        Rem, rem;
                        BitshiftLeft, shl;
                        BitshiftRight, shr;
                        BitwiseAnd, and;
                        BitwiseOr, or;
                        BitwiseXor, xor;
                        Eq, eq;
                        Ne, ne;
                        Gt, gt;
                        Ge, ge;
                        Lt, lt;
                        Le, le
                    )
                } else if sym == SymbolId::F32 {
                    let l = lhs.0.as_f32();
                    let r = rhs.0.as_f32();

                    m!(l, r,
                        Add, add;
                        Sub, sub;
                        Mul, mul;
                        Div, div;
                        Rem, rem;
                        Eq, eq;
                        Ne, ne;
                        Gt, gt;
                        Ge, ge;
                        Lt, lt;
                        Le, le
                    )
                } else if sym == SymbolId::F64 {
                    let l = lhs.0.as_f64();
                    let r = rhs.0.as_f64();

                    m!(l, r,
                        Add, add;
                        Sub, sub;
                        Mul, mul;
                        Div, div;
                        Rem, rem;
                        Eq, eq;
                        Ne, ne;
                        Gt, gt;
                        Ge, ge;
                        Lt, lt;
                        Le, le
                    )
                } else if sym == SymbolId::BOOL {
                    let l = lhs.0.as_bool();
                    let r = rhs.0.as_bool();

                    m!(l, r,
                        Eq, eq;
                        Ne, ne
                    )
                } else if sym == SymbolId::UNIT {
                    let l = lhs.0.as_bool();
                    let r = rhs.0.as_bool();

                    m!(l, r,
                        Eq, eq;
                        Ne, ne
                    )
                } else { unreachable!() }
            },


            Expr::UnaryOp { operator, rhs } => {
                let rhs = self.expr(builder, env, rhs)?;
                
                match operator {
                    UnaryOperator::Not => builder.bool_not(rhs.0.as_bool()).value(),
                    UnaryOperator::Neg => {
                        let c = builder.constant(rhs.0.ty().as_signed_int(), -1);
                        builder.mul(rhs.0.as_signed_int(), c).value()
                    },
                }
            },


            Expr::If { condition, body, else_block } => {
                let cond = self.expr(builder, env, condition)?;

                let ty = self.ty_to_llvm(builder.ctx(), ty).base();
                let local = builder.local(ty);

                builder.ite(&mut (self, env), cond.0.as_bool(),
                |builder, (slf, env)| {
                    let Ok(value) = slf.expr(builder, env, body)
                    else { return; };

                    builder.local_set(local, value.0);
                },


                |builder, (slf, env)| {
                    let Some(body) = else_block
                    else { return; };

                    let Ok(value) = slf.expr(builder, env, body)
                    else { return; };

                    builder.local_set(local, value.0);                   
                },
                );

                builder.local_get(local)
            },


            Expr::Match { value, taken_as_inout, mappings } => todo!(),


            Expr::Block { block } => self.block(builder, env, &*block)?.0,


            Expr::CreateStruct { fields, .. } => {
                let pool = Arena::tls_get_rec();
                let mut vec = sti::vec::Vec::with_cap_in(&*pool, fields.len());

                let llvm_ty = self.ty_to_llvm(builder.ctx(), ty).base().as_struct();
                let sym = ty.sym(self.syms).unwrap();
                let SymbolKind::Container(cont) = self.syms.sym(sym).kind
                else { unreachable!() };

                for sf in cont.fields {
                    let f = fields.iter().find(|f| f.0 == sf.0.unwrap()).unwrap();

                    let value = self.expr(builder, env, f.2)?;
                    vec.push(value.0);
                }

                builder.constant(llvm_ty, &*vec).value()
            },


            Expr::AccessField { val, field_name } => {
                let value = self.expr(builder, env, val)?;
                let value_ty = value.1.sym(self.syms).unwrap();
                let SymbolKind::Container(cont) = self.syms.sym(value_ty).kind
                else { unreachable!() };

                let (i, _) = cont.fields.iter().enumerate().find(|x| x.1.0.unwrap() == field_name).unwrap();
                let ret_ty = self.ty_to_llvm(builder.ctx(), ty).base();
                
                let ptr = builder.field_load(value.0.as_structure(), ret_ty, i as u32);
                builder.ptr_load(ret_ty, ptr)
            },


            Expr::CallFunction { args, .. } => {
                let (sym, gens) = self.ty_info.funcs.get(&expr).unwrap();
                let func = self.get_func(builder.ctx(), *sym, gens);

                let pool = Arena::tls_get_rec();
                let mut inouts = sti::vec::Vec::with_cap_in(&*pool, args.len());
                let args = {
                    let mut vec = Vec::with_cap_in(&*pool, args.len());
                    for a in args { 
                        let (val, ty) = self.expr(builder, env, a.0)?;

                        if a.1 {
                            let ty = self.ty_to_llvm(builder.ctx(), ty);
                            let ptr = builder.as_ptr(val, ty.base());

                            inouts.push((ptr, a.0));
                            vec.push(ptr.value());
                        } else {
                            vec.push(val)
                        }
                    };

                    vec
                };

                let ret = builder.call(func, &*args);

                for i in inouts {
                    self.assign(builder, env, i.1, i.0.value())
                }

                ret
            },


            Expr::WithinNamespace { action, .. } => self.expr(builder, env, action)?.0,
            Expr::WithinTypeNamespace { action, .. } => self.expr(builder, env, action)?.0,


            Expr::Loop { body } => {
                let mut value = None;
                builder.loop_indefinitely(
                |builder, id| {
                    let prev = env.loop_id;
                    env.loop_id = Some(id);

                    let result = self.block(builder, env, &*body);

                    env.loop_id = prev;

                    if let Ok(e) = result { value = Some(e.0) }
                });

                match value {
                    Some(v) => v,
                    None => return Err(()),
                }
            },


            Expr::Return(v) => {
                let val = self.expr(builder, env, v)?;
                builder.ret(val.0);
                builder.unit()
            },


            Expr::Continue => {
                builder.loop_continue(env.loop_id.unwrap());
                builder.unit()
            },


            Expr::Break => {
                builder.loop_break(env.loop_id.unwrap());
                builder.unit()
            },


            Expr::Tuple(_) => todo!(),


            Expr::AsCast { lhs, .. } => {
                let lhs = self.expr(builder, env, lhs)?;
                let lsym = lhs.1.sym(self.syms).unwrap();
                let dest = self.ty_to_llvm(builder.ctx(), ty);


                if lsym.is_sint() {
                    builder.sint_cast(lhs.0.as_signed_int(), dest.base())
                } else if lsym.is_int() {
                    builder.uint_cast(lhs.0.as_unsigned_int(), dest.base())
                } else {
                    builder.float_cast(lhs.0, dest.base())
                }

            },


            Expr::Unwrap(_) => todo!(),


            Expr::OrReturn(_) => todo!(),
        }, ty))
    }

    fn error(&self, builder: &mut Builder<'_>, e: errors::ErrorId) {
        builder.unreachable();
    }


    fn get_func(&mut self, ctx: &mut Context, sym: SymbolId, gens: &[(StringIndex, Type)]) -> Function {
        let pool = Arena::tls_get_rec();
        let sym = self.syms.sym(sym);
        let SymbolKind::Function(sym_func) = sym.kind
        else { unreachable!() };

        let ret = sym_func.ret.to_ty(gens, self.syms).unwrap();
        let ret = self.ty_to_llvm(ctx, ret);


        match sym_func.kind {
            FunctionKind::Extern(path) => {
                let args = {
                    let mut vec = Vec::with_cap_in(&*pool, sym_func.args.len());
                    for i in sym_func.args {
                        let ty = i.symbol.to_ty(gens, self.syms).unwrap();
                        let ty = self.ty_to_llvm(ctx, ty);
                        vec.push(ty.base());
                    }

                    vec.leak()
                };


                return ctx.extern_function(self.module, self.string_map.get(path), &args, ret.base());
            },


            FunctionKind::UserDefined { decl } => {
                let args = {
                    let mut vec = Vec::with_cap_in(&*pool, sym_func.args.len());
                    for i in sym_func.args {
                        let ty = i.symbol.to_ty(gens, self.syms).unwrap();
                        let ty = self.ty_to_llvm(ctx, ty);
                        vec.push((self.string_map.get(i.name), ty.base(), i.inout));
                    }

                    vec.leak()
                };


                let mut func = Function::new(ctx, self.module, self.string_map.get(sym.name),
                                        ret.base(), args);

                let Decl::Function { body, .. } = self.ast.decl(decl)
                else { unreachable!() };

                let mut env = Env {
                    vars: Vec::new_in(&*pool),
                    loop_id: None,
                };

                for (i, a) in sym_func.args.iter().enumerate() {
                    env.vars.push((a.name, func.arg(i.try_into().unwrap())));
                }

                let result = self.block(&mut func, &mut env, &*body);

                if let Some(e) = self.ty_info.decl(decl) {
                    self.error(&mut func, e);
                } else if let Ok(val) = result {
                    func.ret(val.0);
                }

                return func.build()

            },

            FunctionKind::Enum { sym } => todo!(),
        }
    }
}


#[derive(Clone, Copy, Debug)]
enum ConvertedTy {
    Struct(llvm_api::tys::Type),
    Enum {
        base: llvm_api::tys::Type,
        union: llvm_api::tys::Type,
    }
}

impl ConvertedTy {
    fn base(self) -> llvm_api::tys::Type {
        match self {
            ConvertedTy::Struct(v) => v,
            ConvertedTy::Enum { base, .. } => base,
        }
    }

    fn union(self) -> llvm_api::tys::Type {
        match self {
            ConvertedTy::Struct(_) => unreachable!(),
            ConvertedTy::Enum { union, .. } => union,
        }
    }
}
