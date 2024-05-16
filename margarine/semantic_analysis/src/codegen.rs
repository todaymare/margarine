use std::collections::HashMap;

use common::string_map::{StringIndex, StringMap};
use lexer::Literal;
use llvm_api::{builder::{Builder, FPCmp, IntCmp, Local, Loop}, ctx::{Context, ContextRef}, module::Module, tys::{func::FunctionType, Type as LLVMType}, values::{func::{FunctionPtr, Linkage}, int::Integer, ptr::Ptr, Value}};
use parser::nodes::{decl::Decl, expr::{BinaryOperator, Expr, ExprId, UnaryOperator}, stmt::{Stmt, StmtId}, NodeId, AST};
use sti::{arena::Arena, format_in, vec::Vec};

use crate::{types::{containers::ContainerKind, func::FunctionKind, ty::{Type, TypeHash}, SymbolId, SymbolKind, SymbolMap}, TyChecker, TyInfo};

pub struct Codegen<'me, 'out, 'ast, 'str, 'ctx> {
    string_map: &'me StringMap<'str>,
    syms: &'me mut SymbolMap<'out>,
    ast: &'me AST<'ast>,

    ctx: ContextRef<'ctx>,
    module: Module<'ctx>,

    ty_info: &'me TyInfo<'out>,
    ty_mappings: HashMap<TypeHash, LLVMType<'ctx>>
}


pub struct Env<'me> {
    vars: Vec<(StringIndex, Local), &'me Arena>,
    loop_id: Option<Loop>,
}


impl<'me, 'out, 'ast, 'str, 'ctx> Codegen<'me, 'out, 'ast, 'str, 'ctx> {
    pub fn run(ty_checker: &mut TyChecker) -> (Context<'me>, Module<'me>) {
        let ctx = Context::new();
        let module = ctx.module("margarine");

        let mut codegen = Codegen {
            string_map: ty_checker.string_map,
            syms: &mut ty_checker.syms,
            ast: ty_checker.ast,
            module,
            ty_info: &ty_checker.type_info,
            ty_mappings: HashMap::new(),
            ctx: ctx.as_ctx_ref(),
        };

        {
            macro_rules! register {
                ($enum: ident, $call: expr) => {
                    codegen.ty_mappings.insert(Type::$enum.hash(codegen.syms), *$call);
                };
            }


            register!(I8 , ctx.integer(8 ));
            register!(I16, ctx.integer(16));
            register!(I32, ctx.integer(32));
            register!(I64, ctx.integer(64));
            register!(U8 , ctx.integer(8 ));
            register!(U16, ctx.integer(16));
            register!(U32, ctx.integer(32));
            register!(U64, ctx.integer(64));
            register!(F32, ctx.f32());
            register!(F64, ctx.f64());
            register!(BOOL, ctx.bool());
            register!(UNIT, ctx.unit());
        }

        let void = ctx.void();
        let func_ty = void.fn_ty(&[], false);
        let func = module.function("__initStartupSystems__", func_ty);
        func.set_linkage(Linkage::External);
        let b = func.builder(codegen.ctx.as_ctx_ref(), func_ty);

        for sym in &ty_checker.startups {
            let (func, func_ty) = codegen.get_func(*sym, &[]);
            b.call(func, func_ty, &[]);
        }


        b.ret_void();
        b.build();

        (ctx, module)
    }


    fn ty_to_llvm(&mut self, ty: Type) -> LLVMType<'ctx> {
        let hash = ty.hash(self.syms);
        if let Some(ty) = self.ty_mappings.get(&hash) { return *ty }

        let gens = ty.gens(self.syms);

        let sym_id = ty.sym(self.syms).unwrap();
        debug_assert_ne!(sym_id, SymbolId::ERROR);

        let sym = self.syms.sym(sym_id);

        let name = ty.display(self.string_map, self.syms);

        if sym_id == SymbolId::PTR {
            let ty = self.ctx.ptr();
            self.ty_mappings.insert(hash, *ty);
            return *ty;
        }

        match sym.kind {
            SymbolKind::Function(_) => todo!(),

            SymbolKind::Container(cont) => {
                match cont.kind {
                    ContainerKind::Struct => {
                        let strct = self.ctx.structure(name);
                        self.ty_mappings.insert(hash, *strct);

                        let pool = Arena::tls_get_rec();
                        let fields = {
                            let mut vec = Vec::with_cap_in(&*pool, cont.fields.len());
                            for i in cont.fields {
                                let ty = i.1.to_ty(gens, self.syms).unwrap();
                                vec.push(self.ty_to_llvm(ty));
                            }

                            vec.leak()
                        };

                        strct.set_fields(fields);
                        *strct

                    },


                    ContainerKind::Enum => {
                        let pool = Arena::tls_get_rec();
                        let union = self.ctx.union(name);

                        self.ty_mappings.insert(hash, *union);

                        let fields = {
                            let mut vec = Vec::with_cap_in(&*pool, cont.fields.len());
                            for i in cont.fields {
                                let ty = i.1.to_ty(gens, self.syms).unwrap();
                                vec.push(self.ty_to_llvm(ty));
                            }

                            vec.leak()
                        };

                        union.set_fields(self.ctx, self.module, fields);
                        *union
                    },

                    ContainerKind::Tuple => todo!(),
                }
            },
        }
    }


    pub fn block(&mut self, builder: &mut Builder<'ctx>, env: &mut Env, block: &[NodeId]) -> Result<(Value<'ctx>, Type), ()> {
        for (i, n) in block.iter().enumerate() {
            let res = match *n {
                NodeId::Decl(_) => (*builder.const_unit(), Type::UNIT),

                NodeId::Stmt(v) => {
                    self.stmt(builder, env, v);
                    (*builder.const_unit(), Type::UNIT)
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

        Ok((*builder.const_unit(), Type::UNIT))
    }


    pub fn stmt(&mut self, builder: &mut Builder<'ctx>, env: &mut Env, stmt: StmtId) {
        if let Some(e) = self.ty_info.stmt(stmt) {
            self.error(builder, e);
        };


        let _ = (|| {
        match self.ast.stmt(stmt) {
            Stmt::Variable { name, rhs, .. } => {
                let expr = self.expr(builder, env, rhs)?;
                let ty = self.ty_to_llvm(expr.1);
                let local = builder.local(ty);
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


    pub fn assign(&mut self, builder: &mut Builder<'ctx>, env: &mut Env, expr: ExprId, value: Value) {
        let ptr = self.assign_ptr(builder, env, expr);
        builder.store(ptr, value);
    }



    pub fn assign_ptr(&mut self, builder: &mut Builder<'ctx>, env: &mut Env, expr: ExprId) -> Ptr<'ctx> {
        match self.ast.expr(expr) {
            Expr::Identifier(ident) => {
                let local = env.vars.iter().rev().find(|x| x.0 == ident).unwrap();

                builder.local_ptr(local.1)
            },


            Expr::Deref(v) => todo!(),


            Expr::AccessField { val, field_name } => {
                let ptr = self.assign_ptr(builder, env, val);
                todo!()
            },

            Expr::Unwrap(_) => todo!(),
            Expr::OrReturn(_) => todo!(),

            _ => unreachable!()
        }
    }


    pub fn expr(&mut self, builder: &mut Builder<'ctx>, env: &mut Env, expr: ExprId) -> Result<(Value<'ctx>, Type), ()> {
        let ty = match self.ty_info.expr(expr) {
            Ok(e) => e,
            Err(e) => {
                self.error(builder, e);
                return Err(());
            },
        };

        Ok((match self.ast.expr(expr) {
            Expr::Unit => *builder.const_unit(),


            Expr::Literal(v) => {
                match v {
                    Literal::Integer(v) => {
                        let ty = self.ctx.integer(64);
                        *builder.const_int(ty, v, true)
                    },


                    Literal::Float(v) => {
                        *builder.const_f64(v.inner())
                    },


                    Literal::String(v) => {
                        let ty = self.ty_to_llvm(Type::STR).as_struct();

                        let string = self.string_map.get(v);
                        let str = format!("\x01\x00\x00\x00\x00\x00\x00\x00{}", string);
                        let str = self.ctx.const_str(&str);
                        let ptr = self.module.add_global(*str.ty(), "str");
                        ptr.set_initialiser(*str);

                        let len_ty = self.ctx.integer(32);
                        let len = builder.const_int(len_ty, string.len() as i64, false);

                        *builder.const_struct(ty, &[*ptr, *len])
                    },


                    Literal::Bool(v) => {
                        *builder.const_bool(v)
                    },
                }
            },


            Expr::Identifier(v) => {
                let local = env.vars.iter().rev().find(|x| x.0 == v).unwrap().1;
                builder.local_get(local)
            },


            Expr::Deref(v) => {
                let (value, value_ty) = self.expr(builder, env, v)?;
                let value_ty = self.ty_to_llvm(value_ty);
                let value = value.as_ptr();

                let strct = builder.load(value, value_ty);
                let strct = strct.as_struct();

                builder.field_load(strct, 1)
            },


            Expr::Range { lhs, rhs } => {
                let lhs = self.expr(builder, env, lhs)?;
                let rhs = self.expr(builder, env, rhs)?;

                let ty = self.ctx.integer(64);
                let lhs = builder.int_cast(lhs.0.as_integer(), *ty, true);
                let rhs = builder.int_cast(rhs.0.as_integer(), *ty, true);

                let strct = self.ty_to_llvm(Type::RANGE).as_struct();
                *builder.const_struct(strct, &[lhs, rhs])
            },


            Expr::BinaryOp { operator, lhs, rhs } => {
                let lhs = self.expr(builder, env, lhs)?;
                let rhs = self.expr(builder, env, rhs)?;
                let sym = lhs.1.sym(self.syms).unwrap();

                if sym.is_int() {
                    let l = lhs.0.as_integer();
                    let r = rhs.0.as_integer();
                    let signed = sym.is_sint();

                    match operator {
                      BinaryOperator::Add => *builder.add_int(l, r),
                      BinaryOperator::Sub => *builder.sub_int(l, r),
                      BinaryOperator::Mul => *builder.mul_int(l, r),
                      BinaryOperator::Div => *builder.div_int(l, r, signed),
                      BinaryOperator::Rem => *builder.rem_int(l, r, signed),
                      BinaryOperator::BitshiftLeft => *builder.shl(l, r),
                      BinaryOperator::BitshiftRight => *builder.shr(l, r, signed),
                      BinaryOperator::BitwiseAnd => *builder.and(l, r),
                      BinaryOperator::BitwiseOr => *builder.or(l, r),
                      BinaryOperator::BitwiseXor => *builder.xor(l, r),
                      BinaryOperator::Eq => *builder.cmp_int(l, r, IntCmp::Eq),
                      BinaryOperator::Ne => *builder.cmp_int(l, r, IntCmp::Ne),
                      BinaryOperator::Gt => *builder.cmp_int(l, r, IntCmp::SignedGt),
                      BinaryOperator::Ge => *builder.cmp_int(l, r, IntCmp::SignedGe),
                      BinaryOperator::Lt => *builder.cmp_int(l, r, IntCmp::SignedLt),
                      BinaryOperator::Le => *builder.cmp_int(l, r, IntCmp::SignedLe), 
                    }

                } else if sym.is_num() {
                    let l = lhs.0.as_fp();
                    let r = rhs.0.as_fp();

                    match operator {
                      BinaryOperator::Add => *builder.add_fp(l, r),
                      BinaryOperator::Sub => *builder.sub_fp(l, r),
                      BinaryOperator::Mul => *builder.mul_fp(l, r),
                      BinaryOperator::Div => *builder.div_fp(l, r),
                      BinaryOperator::Rem => *builder.rem_fp(l, r),
                      BinaryOperator::Eq => *builder.cmp_fp(l, r, FPCmp::Eq),
                      BinaryOperator::Ne => *builder.cmp_fp(l, r, FPCmp::Ne),
                      BinaryOperator::Gt => *builder.cmp_fp(l, r, FPCmp::Gt),
                      BinaryOperator::Ge => *builder.cmp_fp(l, r, FPCmp::Ge),
                      BinaryOperator::Lt => *builder.cmp_fp(l, r, FPCmp::Lt),
                      BinaryOperator::Le => *builder.cmp_fp(l, r, FPCmp::Le), 

                      _ => unreachable!(),
                    }

                } else if sym == SymbolId::BOOL {
                    let l = lhs.0.as_bool();
                    let r = rhs.0.as_bool();

                    match operator {
                        BinaryOperator::Eq => *builder.bool_eq(l, r),
                        BinaryOperator::Ne => *builder.bool_ne(l, r),

                        _ => unreachable!(),
                    }
                } else if sym == SymbolId::UNIT {

                    match operator {
                        BinaryOperator::Eq => *builder.const_bool(true),
                        BinaryOperator::Ne => *builder.const_bool(false),

                        _ => unreachable!(),
                    }

                } else { unreachable!() }
            },


            Expr::UnaryOp { operator, rhs } => {
                let rhs = self.expr(builder, env, rhs)?;
                
                match operator {
                    UnaryOperator::Not => *builder.bool_not(rhs.0.as_bool()),
                    UnaryOperator::Neg => {
                        let c = builder.const_int(rhs.0.ty().as_integer(), -1, true);
                        *builder.mul_int(rhs.0.as_integer(), c)
                    },
                }
            },


            Expr::If { condition, body, else_block } => {
                let cond = self.expr(builder, env, condition)?;

                let ty = self.ty_to_llvm(ty);
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

                let llvm_ty = self.ty_to_llvm(ty).as_struct();
                let sym = ty.sym(self.syms).unwrap();
                let SymbolKind::Container(cont) = self.syms.sym(sym).kind
                else { unreachable!() };

                for sf in cont.fields {
                    let f = fields.iter().find(|f| f.0 == sf.0.unwrap()).unwrap();

                    let value = self.expr(builder, env, f.2)?;
                    vec.push(value.0);
                }

                *builder.const_struct(llvm_ty, &*vec)
            },


            Expr::AccessField { val, field_name } => {
                let value = self.expr(builder, env, val)?;
                let value_ty = value.1.sym(self.syms).unwrap();
                let SymbolKind::Container(cont) = self.syms.sym(value_ty).kind
                else { unreachable!() };

                let (i, _) = cont.fields.iter().enumerate().find(|x| x.1.0.unwrap() == field_name).unwrap();
                
                builder.field_load(value.0.as_struct(), i)
            },


            Expr::CallFunction { args, .. } => {
                let (sym, gens) = self.ty_info.funcs.get(&expr).unwrap();
                let (func, func_ty) = self.get_func(*sym, gens);

                let pool = Arena::tls_get_rec();
                let mut inouts = sti::vec::Vec::with_cap_in(&*pool, args.len());
                let args = {
                    let mut vec = Vec::with_cap_in(&*pool, args.len());
                    for a in args { 
                        let (val, ty) = self.expr(builder, env, a.0)?;

                        if a.1 {
                            let ptr = builder.alloca_store(val);

                            inouts.push((ptr, a.0, ty));
                            vec.push(*ptr);
                        } else {
                            vec.push(val)
                        }
                    };

                    vec
                };

                let ret = builder.call(func, func_ty, &*args);

                for i in inouts {
                    let ty = self.ty_to_llvm(i.2);
                    let val = builder.load(i.0, ty);
                    self.assign(builder, env, i.1, val)
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
                *builder.const_unit()
            },


            Expr::Continue => {
                builder.loop_continue(env.loop_id.unwrap());
                *builder.const_unit()
            },


            Expr::Break => {
                builder.loop_break(env.loop_id.unwrap());
                *builder.const_unit()
            },


            Expr::Tuple(_) => todo!(),


            Expr::AsCast { lhs, .. } => {
                let lhs = self.expr(builder, env, lhs)?;
                let lsym = lhs.1.sym(self.syms).unwrap();
                let dest = self.ty_to_llvm(ty);


                if lsym.is_int() {
                    builder.int_cast(lhs.0.as_integer(), dest, lsym.is_sint())
                } else {
                    builder.fp_cast(lhs.0.as_fp(), dest)
                }

            },


            Expr::Unwrap(_) => todo!(),


            Expr::OrReturn(_) => todo!(),
        }, ty))
    }

    fn error(&self, builder: &mut Builder<'_>, e: errors::ErrorId) {
        builder.unreachable();
    }


    fn get_func(&mut self, sym: SymbolId, gens: &[(StringIndex, Type)]) -> (FunctionPtr<'ctx>, FunctionType<'ctx>) {
        let pool = Arena::tls_get_rec();
        let sym = self.syms.sym(sym);
        let SymbolKind::Function(sym_func) = sym.kind
        else { unreachable!() };

        let ret = sym_func.ret.to_ty(gens, self.syms).unwrap();
        let ret = self.ty_to_llvm(ret);

        match sym_func.kind {
            FunctionKind::Extern(path) => {
                let args = {
                    let mut vec = Vec::with_cap_in(&*pool, sym_func.args.len());
                    for i in sym_func.args {
                        if i.inout {
                            let ptr = self.ctx.ptr();
                            vec.push(*ptr);
                        } else {
                            let ty = i.symbol.to_ty(gens, self.syms).unwrap();
                            let ty = self.ty_to_llvm(ty);
                            vec.push(ty);
                        }
                    }

                    vec.leak()
                };

                let func_ty = ret.fn_ty(args, false);
                let func = self.module.function(self.string_map.get(path), func_ty);
                func.set_linkage(Linkage::External);

                return (func, func_ty);
            },


            FunctionKind::UserDefined { decl } => {
                let args = {
                    let mut vec = Vec::with_cap_in(&*pool, sym_func.args.len());
                    for i in sym_func.args {
                        let ty = i.symbol.to_ty(gens, self.syms).unwrap();
                        let ty = self.ty_to_llvm(ty);
                        vec.push(ty);
                    }

                    vec.leak()
                };


                let func_ty = ret.fn_ty(args, false);
                let func = self.module.function(self.string_map.get(sym.name), func_ty);
                let mut builder = func.builder(self.ctx, func_ty);

                let Decl::Function { body, .. } = self.ast.decl(decl)
                else { unreachable!() };

                let mut env = Env {
                    vars: Vec::new_in(&*pool),
                    loop_id: None,
                };

                for (i, a) in sym_func.args.iter().enumerate() {
                    env.vars.push((a.name, builder.arg(i).unwrap()));
                }

                let result = self.block(&mut builder, &mut env, &*body);

                if let Some(e) = self.ty_info.decl(decl) {
                    self.error(&mut builder, e);
                } else if let Ok(val) = result {
                    builder.ret(val.0);
                }

                builder.build();
                (func, func_ty)
            },

            FunctionKind::Enum { sym } => todo!(),
        }
    }
}

