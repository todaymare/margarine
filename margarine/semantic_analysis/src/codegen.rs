use std::collections::HashMap;

use common::string_map::{StringIndex, StringMap};
use lexer::Literal;
use llvm_api::{builder::{Builder, FPCmp, IntCmp, Local, Loop}, ctx::{Context, ContextRef}, module::Module, tys::{func::FunctionType, Type as LLVMType}, values::{func::{FunctionPtr, Linkage}, ptr::Ptr, Value}};
use parser::nodes::{decl::Decl, expr::{BinaryOperator, Expr, ExprId, UnaryOperator}, stmt::{Stmt, StmtId}, NodeId, AST};
use sti::{arena::Arena, vec::Vec};

use crate::{namespace::NamespaceMap, types::{containers::ContainerKind, func::FunctionKind, ty::{Type, TypeHash}, GenListId, SymbolId, SymbolKind, SymbolMap}, TyChecker, TyInfo};

pub struct Codegen<'me, 'out, 'ast, 'str, 'ctx> {
    string_map: &'me StringMap<'str>,
    syms: &'me mut SymbolMap<'out>,
    ns: &'me NamespaceMap,
    ast: &'me AST<'ast>,

    ctx: ContextRef<'ctx>,
    module: Module<'ctx>,

    ty_info: &'me TyInfo,
    ty_mappings: HashMap<TypeHash, LLVMType<'ctx>>,
    func_mappings: HashMap<TypeHash, (FunctionPtr<'ctx>, FunctionType<'ctx>)>,

    abort_fn: (FunctionPtr<'ctx>, FunctionType<'ctx>),
}


pub struct Env<'me> {
    vars: Vec<(StringIndex, Local), &'me Arena>,
    inouts: Vec<(usize, Local), &'me Arena>,
    loop_id: Option<Loop>,
}


impl<'me, 'out, 'ast, 'str, 'ctx> Codegen<'me, 'out, 'ast, 'str, 'ctx> {
    pub fn run(ty_checker: &mut TyChecker) -> (Context<'me>, Module<'me>) {
        let ctx = Context::new();
        let module = ctx.module("margarine");

        let void = ctx.void();

        let abort_fn_ty = void.fn_ty(&[], false);
        let abort_fn = module.function("abort", abort_fn_ty);
        abort_fn.set_linkage(Linkage::External);
        abort_fn.set_noreturn(ctx.as_ctx_ref());

        let mut codegen = Codegen {
            string_map: ty_checker.string_map,
            syms: &mut ty_checker.syms,
            ast: ty_checker.ast,
            module,
            ty_info: &ty_checker.type_info,
            ty_mappings: HashMap::new(),
            func_mappings: HashMap::new(),
            ctx: ctx.as_ctx_ref(),
            ns: &ty_checker.namespaces,
            abort_fn: (abort_fn, abort_fn_ty),
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
            let (func, func_ty) = codegen.get_func(*sym, GenListId::EMPTY);
            b.call(func, func_ty, &[]);
        }


        b.ret_void();
        b.build();


        (ctx, module)
    }


    fn ty_to_llvm(&mut self, ty: Type) -> LLVMType<'ctx> {
        self.ty_to_llvm_ex(ty, true)
    }


    fn ty_to_llvm_ex(&mut self, ty: Type, early_out_ptr: bool) -> LLVMType<'ctx> {
        let hash = ty.hash(self.syms);
        if let Some(ty) = self.ty_mappings.get(&hash) { return *ty }

        let gens = ty.gens(self.syms);
        let gens = self.syms.get_gens(gens);

        let sym_id = ty.sym(self.syms).unwrap();
        debug_assert_ne!(sym_id, SymbolId::ERR);

        let sym = self.syms.sym(sym_id);

        let name = ty.display(self.string_map, self.syms);

        if early_out_ptr && sym_id == SymbolId::PTR {
            let ty = self.ctx.ptr();
            self.ty_mappings.insert(hash, *ty);
            return *ty;
        }

        match sym.kind {
            SymbolKind::Function(v) => {
                let ret = v.ret.to_ty(gens, self.syms).unwrap();
                let ret_llvm = self.ty_to_llvm(ret);

                let pool = Arena::tls_get_rec();
                let args = {
                    let mut vec = Vec::with_cap_in(&*pool, v.args.len());
                    for i in v.args {
                        let ty = i.symbol.to_ty(gens, self.syms).unwrap();
                        vec.push(self.ty_to_llvm(ty));
                    }

                    vec
                };

                *ret_llvm.fn_ty(&*args, false)
            },

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
                    if !self.stmt(builder, env, v) { return Err(()) }
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


    pub fn stmt(&mut self, builder: &mut Builder<'ctx>, env: &mut Env, stmt: StmtId) -> bool {
        if let Some(e) = self.ty_info.stmt(stmt) {
            self.error(builder, e);
        };


        let result = (|| {
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


            Stmt::ForLoop { binding, expr, body } => {
                let iter_expr = self.expr(builder, env, expr.1)?;
                let iter_expr_ptr = builder.alloca_store(iter_expr.0);

                let (iter_fn_ret_ty, func) = {
                    let sym = iter_expr.1.sym(self.syms).unwrap();
                    let ns = self.syms.sym_ns(sym);
                    let ns = self.ns.get_ns(ns);

                    let sym = ns.get_sym(StringMap::ITER_NEXT_FUNC).unwrap();
                    let func = self.get_func(sym, iter_expr.1.gens(&self.syms));

                    let ret_ty = self.syms.sym(sym);
                    let SymbolKind::Function(ret_ty) = ret_ty.kind
                    else { unreachable!() };

                    let gens = iter_expr.1.gens(self.syms);
                    let gens = self.syms.get_gens(gens);
                    let ret_ty = ret_ty.ret.to_ty(gens, self.syms).unwrap();

                    (ret_ty, func)
                };

                let iter_fn_ret_value_ty = iter_fn_ret_ty.gens(&self.syms);
                let iter_fn_ret_value_ty = self.syms.get_gens(iter_fn_ret_value_ty)[0].1;
                let iter_fn_ret_value_ty_llvm = self.ty_to_llvm(iter_fn_ret_value_ty);

                builder.loop_indefinitely(|builder, l| {
                    let call_ret_value = builder.call(func.0, func.1, &[*iter_expr_ptr]).as_struct();

                    let index = builder.field_load(call_ret_value, 0).as_integer();
                    let none_case = builder.const_int(index.as_integer().ty(), 1, false);
                    let cond = builder.cmp_int(index, none_case, IntCmp::Eq);

                    builder.ite(&mut (), cond, 
                    |builder, _| {
                        builder.loop_break(l);
                    }, |_, _| {});

                    let value = builder.field_load(call_ret_value, 1);
                    let value = builder.bitcast(value, iter_fn_ret_value_ty_llvm);

                    let local = builder.local(iter_fn_ret_value_ty_llvm);
                    builder.local_set(local, value);

                    env.vars.push((binding.1, local));

                    let _ = self.block(builder, env, &*body);

                    env.vars.pop();
                });
            },


        };
        Ok::<(), ()>(())

        })();


        result.is_ok()
    }


    pub fn assign(&mut self, builder: &mut Builder<'ctx>, env: &mut Env, expr: ExprId, value: Value) {
        let Some(ptr) = self.assign_ptr(builder, env, expr)
        else { return; };
        builder.store(ptr, value);
    }



    pub fn assign_ptr(&mut self, builder: &mut Builder<'ctx>, env: &mut Env, expr: ExprId) -> Option<Ptr<'ctx>> {
        match self.ast.expr(expr) {
            Expr::Identifier(ident) => {
                let local = env.vars.iter().rev().find(|x| x.0 == ident).unwrap();

                Some(builder.local_ptr(local.1))
            },


            Expr::Deref(v) => {
                let ptr = self.assign_ptr(builder, env, v)?;
                let rc_data_ty = self.ty_info.expr(v);
                let rc_data_ty = match rc_data_ty {
                    Ok(v) => v,
                    Err(v) => {
                        self.error(builder, v);
                        return None;
                    },
                };

                let rc_data_ty = self.ty_to_llvm_ex(rc_data_ty, false);

                Some(builder.field_ptr(ptr, rc_data_ty.as_struct(), 1))
            },


            Expr::AccessField { val, field_name } => {
                let ptr = self.assign_ptr(builder, env, val)?;
                let ty = self.ty_info.expr(val);
                let ty = match ty {
                    Ok(v) => v,
                    Err(v) => {
                        self.error(builder, v);
                        return None;
                    },
                };


                let sym = ty.sym(self.syms).unwrap();
                let sym = self.syms.sym(sym);

                let SymbolKind::Container(cont) = sym.kind
                else { unreachable!() };

                let ty = self.ty_to_llvm(ty);
                let (index, _) = cont.fields.iter().enumerate().find(|f| f.1.0.unwrap() == field_name).unwrap();
                Some(builder.field_ptr(ptr, ty.as_struct(), index))
            },


            Expr::Unwrap(v) => {
                let ptr = self.assign_ptr(builder, env, v)?;
                let ty = self.ty_info.expr(v);
                let ty = match ty {
                    Ok(v) => v,
                    Err(v) => {
                        self.error(builder, v);
                        return None;
                    },
                };

                let llvm_ty = self.ty_to_llvm(ty).as_struct();

                let strct = builder.load(ptr, *llvm_ty).as_struct();
                let value_index = builder.field_load(strct, 0).as_integer();
                let expected_index = builder.const_int(value_index.ty(), 0, false);

                let cmp = builder.cmp_int(value_index, expected_index, IntCmp::Ne);

                builder.ite(self, cmp,
                |builder, slf| slf.unwrap_fail(builder),
                |_, _| { });

                Some(builder.field_ptr(ptr, llvm_ty.as_struct(), 1))
            },


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

        let ty_sym = ty.sym(self.syms).unwrap();

        let val = match self.ast.expr(expr) {
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

                        *builder.struct_instance(ty, &[*ptr, *len])
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
                *builder.struct_instance(strct, &[lhs, rhs])
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

                let slf = self;
                builder.ite(&mut (slf, env), cond.0.as_bool(),
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


            Expr::Match { value, taken_as_inout, mappings } => {
                let val = self.expr(builder, env, value)?;
                let sym = val.1.sym(self.syms).unwrap();
                let sym = self.syms.sym(sym);

                let SymbolKind::Container(cont) = sym.kind
                else { unreachable!() };

                let value_ty = val.0.as_struct();
                let value_alloc = builder.alloca_store(val.0);
                let value_index = builder.field_load(value_ty, 0).as_integer();

                let iter = cont.fields.iter().map(|sf| {
                    let name = sf.0.unwrap();
                    (sf, mappings.iter().find(|x| x.name() == name).unwrap())
                });

                let ret_ty = self.ty_to_llvm(ty);
                let ret_local = builder.local(ret_ty);
                let inout_ptr = builder.field_ptr(value_alloc, value_ty.ty(), 1);

                builder.switch(value_index, iter,
                |builder, (field, mapping)| {
                    // initialize the binding
                    let gens = val.1.gens(self.syms);
                    let gens = self.syms.get_gens(gens);
                    let field_ty = field.1.to_ty(gens, self.syms).unwrap();
                    let field_ty_llvm = self.ty_to_llvm(field_ty);

                    let local = builder.local(field_ty_llvm);
                    let value = builder.field_load(val.0.as_struct(), 1);
                    let value = builder.bitcast(value, field_ty_llvm);
                    builder.local_set(local, value);

                    env.vars.push((mapping.binding(), local));

                    // run the body
                    let ret_val = self.expr(builder, env, mapping.expr());
                    debug_assert_eq!(env.vars.pop().unwrap(), (mapping.binding(), local));

                    let Ok(ret_val) = ret_val
                    else { return };

                    builder.local_set(ret_local, ret_val.0);

                    if mapping.is_inout() {
                        builder.store(inout_ptr, ret_val.0)
                    }
                });

                if taken_as_inout {
                    let val = builder.load(value_alloc, *value_ty.ty());
                    self.assign(builder, env, expr, val);
                }

                if ty_sym == SymbolId::NEVER { *builder.const_unit() }
                else { builder.local_get(ret_local) }
            },


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

                *builder.struct_instance(llvm_ty, &*vec)
            },


            Expr::AccessField { val, field_name } => {
                let value = self.expr(builder, env, val)?;
                let value_ty = value.1.sym(self.syms).unwrap();
                let SymbolKind::Container(cont) = self.syms.sym(value_ty).kind
                else { unreachable!() };

                let (i, f) = cont.fields.iter().enumerate().find(|x| x.1.0.unwrap() == field_name).unwrap();
                
                match cont.kind {
                    ContainerKind::Struct => builder.field_load(value.0.as_struct(), i),


                    ContainerKind::Enum => {
                        let field_ty = {
                            let gens = value.1.gens(self.syms);
                            let gens = self.syms.get_gens(gens);

                            f.1.to_ty(gens, self.syms).unwrap()
                        };

                        let gens_arr = self.syms.arena.alloc_new([(StringMap::T, field_ty)]);
                        let gens = self.syms.add_gens(gens_arr);
                        let opt_ty = Type::Ty(SymbolId::OPTION, gens);

                        let opt_ns = self.syms.sym_ns(SymbolId::OPTION);
                        let opt_ns = self.ns.get_ns(opt_ns);

                        let some_func = opt_ns.get_sym(StringMap::SOME).unwrap();
                        let none_func = opt_ns.get_sym(StringMap::NONE).unwrap();

                        let value_index = builder.field_load(value.0.as_struct(), 0);
                        let expected_index = builder.const_int(value_index.ty().as_integer(), i as i64, false);
                        let cmp = builder.cmp_int(value_index.as_integer(), expected_index, IntCmp::Eq);

                        let local = builder.local(self.ty_to_llvm(opt_ty));
                        builder.ite(self, cmp,
                        |builder, slf| {
                            let func = slf.get_func(some_func, gens);
                            let val = builder.field_load(value.0.as_struct(), 1);
                            let val = builder.bitcast(val, slf.ty_to_llvm(field_ty));
                            let ret = builder.call(func.0, func.1, &[val]);
                            builder.local_set(local, ret);
                        },

                        |builder, slf| {
                            let func = slf.get_func(none_func, gens);
                            let ret = builder.call(func.0, func.1, &[]);
                            builder.local_set(local, ret);
                        });


                        builder.local_get(local)
                    },


                    ContainerKind::Tuple => todo!(),
                }
            },


            Expr::CallFunction { args, is_accessor, .. } => {
                let (sym, gens) = self.ty_info.funcs.get(&expr).unwrap();
                let (func, func_ty) = self.get_func(*sym, *gens);

                let func_fields = {
                    let sym = self.syms.sym(*sym);
                    let SymbolKind::Function(func) = sym.kind
                    else { unreachable!() };

                    func.args
                };

                let pool = Arena::tls_get_rec();
                let mut inouts = sti::vec::Vec::with_cap_in(&*pool, args.len());
                let args = {
                    let mut vec = Vec::with_cap_in(&*pool, args.len());
                    for (i, (a, fa)) in args.iter().zip(func_fields).enumerate() { 
                        let (val, ty) = self.expr(builder, env, a.0)?;

                        let is_inout = if fa.inout && is_accessor && i == 0 { true }
                                        else { a.1 };
                        if is_inout {
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

                if !val.1.is_never(self.syms) {
                    Self::update_inouts(env, builder);
                    dbg!(val);
                    builder.ret(val.0);
                }

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


            Expr::Unwrap(val) => {
                let val = self.expr(builder, env, val)?;

                let sym = val.1.sym(self.syms).unwrap();
                let sym = self.syms.sym(sym);

                let SymbolKind::Container(cont) = sym.kind
                else { unreachable!() };

                let value_index = builder.field_load(val.0.as_struct(), 0).as_integer();
                let expected_index = builder.const_int(value_index.ty(), 0, false);

                let cmp = builder.cmp_int(value_index, expected_index, IntCmp::Ne);

                builder.ite(self, cmp,
                |builder, slf| slf.unwrap_fail(builder),
                |_, _| { });

                let gens = val.1.gens(self.syms);
                let gens = self.syms.get_gens(gens);
                let field_ty = cont.fields[0].1.to_ty(gens, self.syms).unwrap();
                let val = builder.field_load(val.0.as_struct(), 1);
                builder.bitcast(val, self.ty_to_llvm(field_ty))
            },


            Expr::OrReturn(_) => todo!(),
        }; 

        if ty_sym == SymbolId::NEVER {
            builder.unreachable();
            return Ok((*builder.const_unit(), ty))
        }

        Ok((val, ty))
    }

    fn error(&self, builder: &mut Builder<'_>, e: errors::ErrorId) {
        self.abort(builder)
    }


    fn abort(&self, builder: &mut Builder<'_>) {
        builder.call(self.abort_fn.0, self.abort_fn.1, &[]);
        builder.unreachable();
    }


    fn unwrap_fail(&self, builder: &mut Builder<'_>) {
        self.abort(builder)
    }


    fn get_func(&mut self, sym: SymbolId, gens: GenListId) -> (FunctionPtr<'ctx>, FunctionType<'ctx>) {
        let ty = Type::Ty(sym, gens);
        let hash = ty.hash(self.syms);
        if let Some(ty) = self.func_mappings.get(&hash) { return *ty }

        let pool = Arena::tls_get_rec();
        let fsym = self.syms.sym(sym);
        let SymbolKind::Function(sym_func) = fsym.kind
        else { unreachable!() };

        let gens = self.syms.gens[gens];
        let ret = sym_func.ret.to_ty(gens, self.syms).unwrap();
        let llvm_ret = self.ty_to_llvm(ret);

        let res = match sym_func.kind {
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

                let func_ty = llvm_ret.fn_ty(args, false);
                let func = self.module.function(self.string_map.get(path), func_ty);
                func.set_linkage(Linkage::External);

                return (func, func_ty);
            },


            FunctionKind::UserDefined { decl } => {
                let args = {
                    let mut vec = Vec::with_cap_in(&*pool, sym_func.args.len());
                    for i in sym_func.args {
                        if i.inout {
                            vec.push(*self.ctx.ptr());
                            continue;
                        }

                        let ty = i.symbol.to_ty(gens, self.syms).unwrap();
                        let ty = self.ty_to_llvm(ty);
                        vec.push(ty);
                    }

                    vec.leak()
                };


                let func_ty = llvm_ret.fn_ty(args, false);
                let func = self.module.function(self.string_map.get(fsym.name), func_ty);
                let mut builder = func.builder(self.ctx, func_ty);

                let Decl::Function { body, .. } = self.ast.decl(decl)
                else { unreachable!() };

                let mut env = Env {
                    vars: Vec::new_in(&*pool),
                    loop_id: None,
                    inouts: Vec::new_in(&*pool),
                };

                for (i, fa) in sym_func.args.iter().enumerate() {
                    let arg = builder.arg(i).unwrap();
                    let mut local = arg;
                    
                    if fa.inout {
                        let ty = fa.symbol.to_ty(gens, self.syms).unwrap();
                        let ty = self.ty_to_llvm(ty);

                        let ptr = builder.local_get(arg).as_ptr();
                        let load = builder.load(ptr, ty);

                        let new_local = builder.local(ty);
                        builder.local_set(new_local, load);
                        local = new_local;
                        env.inouts.push((i, new_local));
                    }

                    env.vars.push((fa.name, local));
                }

                let result = self.block(&mut builder, &mut env, &*body);
                
                // update inouts
                Self::update_inouts(&env, &mut builder);


                if let Some(e) = self.ty_info.decl(decl) {
                    self.error(&mut builder, e);
                } else if let Ok(val) = result {
                    if !val.1.is_never(self.syms) { dbg!(val); builder.ret(val.0); }
                    else { builder.unreachable() }
                }

                builder.build();
                (func, func_ty)
            },

            FunctionKind::Enum { sym: sym_id, index } => {
                let sym = self.syms.sym(sym_id);
                let SymbolKind::Container(cont) = sym.kind
                else { unreachable!() };

                let gens_id = self.syms.add_gens(gens);
                let ret_ty = Type::Ty(sym_id, gens_id);
                let arg = cont.fields[index];
                let arg_ty = arg.1.to_ty(gens, self.syms).unwrap();

                let llvm_ret_ty = self.ty_to_llvm(ret_ty);

                let is_unit = arg_ty.sym(self.syms).unwrap() == SymbolId::UNIT;
                let llvm_arg_ty = self.ty_to_llvm(arg_ty);

                let func_ty = if is_unit { llvm_ret_ty.fn_ty(&[], false) }
                              else { llvm_ret_ty.fn_ty(&[llvm_arg_ty], false) };
                let func = self.module.function(self.string_map.get(fsym.name), func_ty);

                let mut builder = func.builder(self.ctx, func_ty);

                let union_struct_fields = llvm_ret_ty.as_struct().fields(&*pool);
                let alloca = builder.alloca(llvm_ret_ty);

                if !is_unit {
                    let arg = builder.arg(0).unwrap();
                    let arg = builder.local_get(arg);
                    let fp = builder.field_ptr(alloca, llvm_ret_ty.as_struct(), 1);
                    builder.store(fp, arg);
                }

                let index = builder.const_int(union_struct_fields[0].as_integer(),
                                                index as i64, false);

                let fp = builder.field_ptr(alloca, llvm_ret_ty.as_struct(), 0);
                builder.store(fp, *index);

                let ret = builder.load(alloca, llvm_ret_ty);
                builder.ret(ret);
                builder.build();

                (func, func_ty)
            },
        };

        self.func_mappings.insert(hash, res);

        res
    }


    fn update_inouts(env: &Env, builder: &mut Builder<'_>) {
        for (arg_index, local) in env.inouts.iter() {
            let ptr = builder.arg(*arg_index).unwrap();
            let ptr = builder.local_get(ptr).as_ptr();

            let local = builder.local_get(*local);
            builder.store(ptr, local)
        }
    }
}

