use std::{collections::HashMap, fmt::Write, fs};

use common::string_map::{StringIndex, StringMap};
use errors::ErrorId;
use parser::nodes::{decl::Decl, expr::{BinaryOperator, ExprId, UnaryOperator}, stmt::StmtId, NodeId, AST};
use runtime::opcode::{self, runtime::{builder::{self, Builder}, consts}, HEADER};
use sti::{arena::Arena, define_key, keyed::KVec};

use crate::{namespace::NamespaceMap, syms::{containers::ContainerKind, sym_map::{GenListId, SymbolId, SymbolMap}, ty::{Sym, TypeHash}, SymbolKind}, TyChecker, TyInfo};

#[derive(Debug)]
pub struct Conversion<'me, 'out, 'ast, 'str> {
    string_map: &'me mut StringMap<'str>,
    syms: &'me mut SymbolMap<'out>,
    ns: &'me NamespaceMap,
    ast: &'me AST<'ast>,

    ty_info: &'me TyInfo,

    funcs: HashMap<TypeHash, Function<'me>>,
    const_strs: Vec<StringIndex>,
    buf: &'me Arena,
}


#[derive(Clone, Copy, Debug)]
struct FuncIndex(u32);
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct BlockIndex(u32);


#[derive(Debug)]
struct Function<'a> {
    name: StringIndex,
    index: FuncIndex,

    kind: FunctionKind<'a>,
    error: Option<ErrorId>,
}


#[derive(Debug)]
enum FunctionKind<'a> {
    Code {
        local_count: u8,
        entry: BlockIndex,
        blocks: Vec<Block<'a>>,
    },


    Extern(StringIndex),
}


struct Env<'a> {
    vars: Vec<(StringIndex, u32)>,
    var_counter: u32,
    block_counter: u32,
    blocks: Vec<Block<'a>>,
    loop_cont: Option<BlockIndex>,
    loop_brek: Option<BlockIndex>,
}


#[derive(Debug)]
struct Block<'a> {
    index: BlockIndex,
    bytecode: Builder,
    terminator: BlockTerminator<'a>,
}


#[derive(Debug, Clone, Copy)]
enum BlockTerminator<'a> {
    Goto(BlockIndex),
    SwitchBool { op1: BlockIndex, op2: BlockIndex },
    Switch(&'a [BlockIndex]),
    Err(ErrorId),
    Ret,
}


pub fn run(ty_checker: &mut TyChecker, errors: [Vec<Vec<String>>; 3]) -> Vec<u8> {
    let out = Arena::new();
    let mut conv = Conversion {
        string_map: ty_checker.string_map,
        syms: &mut ty_checker.syms,
        ns: &ty_checker.namespaces,
        ast: ty_checker.ast,
        ty_info: &ty_checker.type_info,
        funcs: HashMap::new(),
        buf: &out,
        const_strs: Vec::new(),
    };


    // create IR
    for sym in &ty_checker.startups {
        conv.get_func(*sym, GenListId::EMPTY).unwrap();
    }

    // do the codegen
    let mut func_sec = vec![];
    let mut code = Builder::new();
    let mut funcs = conv.funcs.iter().collect::<Vec<_>>();
    funcs.sort_by_key(|x| x.1.index.0);
    for (_, func) in funcs {
        func_sec.push(opcode::func::consts::Func);

        let name = conv.string_map.get(func.name);
        // func meta
        func_sec.extend_from_slice(&(name.len() as u32).to_le_bytes());
        func_sec.extend_from_slice(name.as_bytes());

        match &func.kind {
            FunctionKind::Code { local_count, entry, blocks } => {
                func_sec.push(0);

                // code offset
                func_sec.extend_from_slice(&(code.len() as u32).to_le_bytes());

                let code_sec_start = code.len();

                let mut terminators = vec![];
                let mut bbs_hm = HashMap::with_capacity(blocks.len());

                let mut buf = vec![];
                let mut stack = vec![];
                stack.push(*entry);
                while let Some(bb) = stack.pop() {
                    let block = blocks.iter().find(|x| x.index == bb).unwrap();

                    if bbs_hm.contains_key(&block.index.0) { continue }

                    bbs_hm.insert(block.index.0, code.len());

                    code.append(&block.bytecode);

                    let start = code.len();
                    match block.terminator {
                        BlockTerminator::Goto(v) => {
                            stack.push(v);

                            code.jump(i32::MAX);
                        },


                        BlockTerminator::SwitchBool { op1, op2  } => {
                            stack.push(op1);
                            stack.push(op2);

                            code.switch_on(i32::MAX, i32::MAX);
                        },


                        BlockTerminator::Switch(bbs) => {
                            buf.clear();
                            buf.extend((0..(bbs.len() * 4)).map(|_| 255u8));

                            stack.extend_from_slice(bbs);

                            code.switch(&buf);
                        }


                        BlockTerminator::Err(err) => {
                            match err {
                                errors::ErrorId::Lexer(error) => {
                                    code.err(0, error.0, error.1.inner());
                                },


                                errors::ErrorId::Parser(error) => {
                                    code.err(1, error.0, error.1.inner());

                                },


                                errors::ErrorId::Sema(sema_error) => {
                                    code.err(2, 0, sema_error.inner());
                                },
                            }
                        }


                        BlockTerminator::Ret => {
                            code.pop_local_space(*local_count);

                            code.ret();
                        },
                    }

                    terminators.push((block.terminator, start, code.len()));

                }


                for (term, start_offset, end_offset) in terminators {
                    match term {
                        BlockTerminator::Goto(block_index) => {
                            let bb = bbs_hm.get(&block_index.0).unwrap();
                            let jmp_offset = *bb as i32 - end_offset as i32;
                            code.jump_at(start_offset, jmp_offset);
                        },


                        BlockTerminator::SwitchBool { op1, op2 } => {
                            let bb = bbs_hm.get(&op1.0).unwrap();
                            let op1_jmp_offset = *bb as i32 - end_offset as i32;

                            let bb = bbs_hm.get(&op2.0).unwrap();
                            let op2_jmp_offset = *bb as i32 - end_offset as i32;
                            code.switch_on_at(start_offset, op1_jmp_offset, op2_jmp_offset);
                        },


                        BlockTerminator::Switch(bbs) => {
                            buf.clear();

                            for bb in bbs {
                                let bb = *bbs_hm.get(&bb.0).unwrap();
                                let jmp_offset = bb as i32 - (start_offset + 9 + buf.len()) as i32;
                                buf.extend_from_slice(&jmp_offset.to_le_bytes());
                            }

                            code.switch_at(start_offset, &buf);
                        }


                        BlockTerminator::Ret => (),
                        BlockTerminator::Err(_) => (),
                    }
                }


                func_sec.extend(&((code.len() - code_sec_start) as u32).to_le_bytes());

            },


            FunctionKind::Extern(string_index) => {
                func_sec.push(1);
                let path = conv.string_map.get(*string_index);

                func_sec.extend_from_slice(&(path.len() as u32).to_le_bytes());
                func_sec.extend_from_slice(path.as_bytes());
            },
        }
    }


    func_sec.push(opcode::func::consts::Terminate);

    let mut errors_table = vec![];
    for error_files in errors {
        errors_table.extend_from_slice(&(error_files.len() as u32).to_le_bytes());
        for file in error_files {
            errors_table.extend_from_slice(&(file.len() as u32).to_le_bytes());

            for error in file {
                errors_table.extend_from_slice(&(error.len() as u32).to_le_bytes());
                errors_table.extend_from_slice(error.as_bytes());
            }
        }
    }

    
    let mut strs_table = vec![];
    strs_table.extend_from_slice(&(conv.const_strs.len() as u32).to_le_bytes());
    for str in &conv.const_strs {
        let str = conv.string_map.get(*str);

        strs_table.extend_from_slice(&(str.len() as u32).to_le_bytes());
        strs_table.extend_from_slice(str.as_bytes());
    }


    let mut final_product = vec![];
    final_product.extend_from_slice(&HEADER);
    final_product.extend_from_slice(&(func_sec.len() as u32).to_le_bytes());
    final_product.extend_from_slice(&(errors_table.len() as u32).to_le_bytes());
    final_product.extend_from_slice(&(strs_table.len() as u32).to_le_bytes());
    final_product.extend_from_slice(&errors_table);
    final_product.extend_from_slice(&strs_table);
    final_product.extend_from_slice(&func_sec);
    dbg!(&code);
    final_product.extend_from_slice(&code.bytecode);

    final_product
}



impl<'me, 'out, 'ast, 'str> Conversion<'me, 'out, 'ast, 'str> {
    fn get_func(&mut self, sym: SymbolId, gens: GenListId) -> Result<&Function<'me>, ()> {
        let ty = Sym::Ty(sym, gens);
        let hash = ty.hash(self.syms);
        if self.funcs.contains_key(&hash) { 
            return Ok(self.funcs.get(&hash).unwrap())
        }

        // create
        let fsym = self.syms.sym(sym);
        let SymbolKind::Function(sym_func) = fsym.kind()
        else { unreachable!() };

        let gens = self.syms.gens()[gens];;
        for g in gens { if g.1.is_err(self.syms) { return Err(()) } }

        let ret = sym_func.ret().to_ty(gens, self.syms).unwrap();
        if ret.is_err(self.syms) { return Err(()) }

        match sym_func.kind() {
            crate::syms::func::FunctionKind::Extern(path) => {
                let func = Function {
                    name: self.string_map.insert(ty.display(self.string_map, self.syms)),
                    index: FuncIndex(self.funcs.len() as u32),
                    kind: FunctionKind::Extern(path),
                    error: self.ty_info.decl(sym_func.decl().unwrap()),
                };

                self.funcs.insert(hash, func);
                return Ok(self.funcs.get(&hash).unwrap())
            },


            crate::syms::func::FunctionKind::UserDefined => {
                let Decl::Function { body, .. } = self.ast.decl(sym_func.decl().unwrap())
                else { unreachable!() };

                let err = self.ty_info.decl(sym_func.decl().unwrap());
                let func = Function {
                    name: self.string_map.insert(ty.display(self.string_map, self.syms)),
                    index: FuncIndex(self.funcs.len() as u32),
                    kind: FunctionKind::Code {
                        local_count: 0,
                        entry: BlockIndex(0),
                        blocks: vec![],
                    },
                    error: err,

                };

                self.funcs.insert(hash, func);

                let mut env = Env {
                    vars: Vec::new(),
                    var_counter: 0,
                    block_counter: 0,
                    blocks: vec![],
                    loop_brek: None,
                    loop_cont: None,
                };


                for arg in sym_func.args() {
                    env.alloc_var(arg.name());
                }


                let entry_block = env.next_block_index();
                let mut block = Block {
                    index: entry_block,
                    bytecode: Builder::new(),
                    terminator: BlockTerminator::Ret,
                };

                block.bytecode.push_local_space(u8::MAX);

                let _ = self.process(&mut env, &mut block, &*body);

                if let Some(err) = err {
                    generate_error(&mut env, &mut block, err);
                }

                env.blocks.push(block);
                let block = env.blocks.iter_mut().find(|x| x.index == entry_block).unwrap();
                block.bytecode.push_local_space_at(0, (env.var_counter - sym_func.args().len() as u32).try_into().unwrap());

                env.blocks.sort_by_key(|x| x.index.0);

                let func = self.funcs.get_mut(&hash).unwrap();
                func.kind = FunctionKind::Code {
                    local_count: (env.var_counter - sym_func.args().len() as u32).try_into().unwrap(),
                    entry: entry_block,
                    blocks: env.blocks,
                };

                return Ok(func)
            },



            crate::syms::func::FunctionKind::TypeId => {
                let mut block = Block {
                    index: BlockIndex(0),
                    bytecode: Builder::new(),
                    terminator: BlockTerminator::Ret,
                };

                let id = gens[0].1.sym(self.syms).unwrap();
                block.bytecode.const_int(id.inner() as i64);

                let func = Function {
                    name: self.string_map.insert(ty.display(self.string_map, self.syms)),
                    index: FuncIndex(self.funcs.len().try_into().unwrap()),

                    kind: FunctionKind::Code {
                        local_count: 0,
                        entry: BlockIndex(0),
                        blocks: vec![block],
                    },

                    error: None,
                };

                self.funcs.insert(hash, func);
                return Ok(self.funcs.get(&hash).unwrap())
            }


            crate::syms::func::FunctionKind::Enum { sym: sym_id, index } => {
                let sym = self.syms.sym(sym_id);
                let SymbolKind::Container(cont) = sym.kind()
                else { unreachable!() };

                let arg = cont.fields()[index];
                let arg_ty = arg.1.to_ty(gens, self.syms).unwrap();

                let is_unit = arg_ty.sym(self.syms).unwrap() == SymbolId::UNIT;

                let mut builder = Builder::new();
                if is_unit {
                    builder.const_int(index as i64);
                    builder.unit();
                    builder.create_struct(2);
                } else {
                    builder.const_int(index as i64);
                    builder.load(0);
                    builder.create_struct(2);
                }

                let err = sym_func.decl().map(|t| self.ty_info.decl(t)).flatten();
                let func = Function {
                    name: sym.name(),
                    index: FuncIndex(self.funcs.len() as u32),
                    kind: FunctionKind::Code {
                        local_count: 0,
                        entry: BlockIndex(0),
                        blocks: vec![Block {
                            index: BlockIndex(0),
                            bytecode: builder,
                            terminator: BlockTerminator::Ret,
                        }]
                    },
                    error: err,
                };

                self.funcs.insert(hash, func);
                return Ok(self.funcs.get(&hash).unwrap())

            },
        };
    }



    pub fn process(&mut self, env: &mut Env<'me>, block: &mut Block<'me>, instrs: &[NodeId]) -> Result<(), ()> {
        let mut has_ret = false;

        for (i, &n) in instrs.iter().enumerate() {
            has_ret = false;
            match n {
                NodeId::Decl(decl_id) => {
                    continue;
                },


                NodeId::Stmt(stmt_id) => {
                    self.stmt(env, block, stmt_id)?;
                    continue;

                },


                NodeId::Expr(expr_id) => {
                    if let Err(error_id) = self.ty_info.expr(expr_id) {
                        generate_error(env, block, error_id);
                        continue;
                    }

                    has_ret = true;

                    self.expr(env, block, expr_id)?;
                    if i != instrs.len() - 1 {
                        block.bytecode.pop()
                    }
                },


                NodeId::Err(error_id) => {
                    generate_error(env, block, error_id);
                },
            };

        }


        if !has_ret { 
            block.bytecode.unit()
        }

        Ok(())
    }


    fn stmt(&mut self, env: &mut Env<'me>, block: &mut Block<'me>, stmt: StmtId) -> Result<(), ()> {
        macro_rules! out_if_err {
            () => {
               match self.ty_info.stmt(stmt) {
                    None => (),
                    Some(e) => {
                        generate_error(env, block, e);
                        return Err(());
                    },
               }
            };
        }


        match self.ast.stmt(stmt) {
            parser::nodes::stmt::Stmt::Variable { name, rhs, .. } => {
                env.alloc_var(name);
                self.expr(env, block, rhs)?;
                out_if_err!();

                let (_, index) = env.vars.iter().rev().find(|x| x.0 == name).unwrap();
                block.bytecode.store((*index).try_into().unwrap());
            },


            parser::nodes::stmt::Stmt::VariableTuple { names, rhs, .. } => {
                self.expr(env, block, rhs)?;
                out_if_err!();

                for &name in names.iter().rev() {
                    env.alloc_var(name);
                    let (_, index) = env.vars.iter().rev().find(|x| x.0 == name).unwrap();
                    block.bytecode.store((*index).try_into().unwrap());
                }
            },


            parser::nodes::stmt::Stmt::UpdateValue { lhs, rhs } => {
                match self.ty_info.expr(lhs) {
                     Ok(_) => (),
                     Err(e) => {
                         generate_error(env, block, e);
                         return Err(());
                     },
                };

                self.expr(env, block, rhs)?;
                out_if_err!();

                self.update_value(env, block, lhs)?;

            },


            parser::nodes::stmt::Stmt::ForLoop { binding, expr, body } => {
                self.expr(env, block, expr)?;
                out_if_err!();

                let iter_expr = self.ty_info.expr(expr).unwrap();
                let iter_fn = {
                    let sym = iter_expr.sym(self.syms).unwrap();
                    let ns = self.syms.sym_ns(sym);
                    let ns = self.ns.get_ns(ns);

                    let Ok(sym) = ns.get_sym(StringMap::ITER_NEXT_FUNC).unwrap()
                    else { return Err(()) };

                    let func = self.get_func(sym, iter_expr.gens(&self.syms))?.index;
                    func
                };

                let len = env.vars.len();

                let iterable = env.alloc_anon_var();
                block.bytecode.store(iterable.try_into().unwrap());


                let value = env.alloc_var(binding.0);

                self.build_loop(env, block,
                |this, env, block| {

                    block.bytecode.load(iterable as u8);
                    block.bytecode.call(iter_fn.0, 1);

                    block.bytecode.copy();

                    block.bytecode.load_field(0);
                    block.bytecode.const_int(0);

                    block.bytecode.eq_int();

                    this.build_ite(env, block,
                    |this, env, block| {
                        block.bytecode.load_field(1);
                        block.bytecode.store(value.try_into().unwrap());

                        this.process(env, block, &body)?;
                        block.bytecode.pop();

                        Ok(())
                    },
                    |_, env, block| {
                        block.bytecode.pop();

                        let goto = env.loop_brek.unwrap();
                        let mut cont_block = env.next_block();
                        cont_block.terminator = BlockTerminator::Goto(goto);

                        core::mem::swap(block, &mut cont_block);
                        env.blocks.push(cont_block);
                        Ok(())
                    }
                    )?;
                    Ok(())
                })?;

                env.vars.truncate(len);
            },
        };

        Ok(())
    }


    fn expr(&mut self, env: &mut Env<'me>, block: &mut Block<'me>, expr: ExprId) -> Result<(), ()> {
        macro_rules! out_if_err {
            () => {
               match self.ty_info.expr(expr) {
                    Ok(e) => e,
                    Err(e) => {
                        generate_error(env, block, e);
                        return Err(());
                    },
               }
            };
        }


        match self.ast.expr(expr) {
            parser::nodes::expr::Expr::Unit => {
                block.bytecode.unit();
                out_if_err!();
            },


            parser::nodes::expr::Expr::Literal(literal) => {
                out_if_err!();
                match literal {
                    lexer::Literal::Integer(v) => block.bytecode.const_int(v),
                    lexer::Literal::Float(non_na_nf64) => block.bytecode.const_float(non_na_nf64.inner()),
                    lexer::Literal::String(string_index) => {
                        self.const_strs.push(string_index);
                        block.bytecode.const_str(self.const_strs.len() as u32 - 1);
                    },


                    lexer::Literal::Bool(v) => block.bytecode.const_bool(v as u8),
                };
            },


            parser::nodes::expr::Expr::Identifier(string_index) => {
                out_if_err!();
                let (_, index) = env.vars.iter().rev().find(|x| x.0 == string_index).unwrap();
                block.bytecode.load((*index).try_into().unwrap());
            },


            parser::nodes::expr::Expr::Range { lhs, rhs } => {
                self.expr(env, block, lhs)?;
                self.expr(env, block, rhs)?;
                out_if_err!();

                block.bytecode.create_struct(2);
            },


            parser::nodes::expr::Expr::BinaryOp { operator, lhs, rhs } => {
                self.expr(env, block, lhs)?;
                self.expr(env, block, rhs)?;
                out_if_err!();

                let Ok(ty) = self.ty_info.expr(lhs)
                // we can just return cos if an err happened then the program would terminate
                // at an earlier point anyways
                else { block.bytecode.unit(); return Ok(()) };
                let Ok(sym) = ty.sym(self.syms)
                else { block.bytecode.unit(); return Ok(()) };

                match (sym, operator) {
                    (SymbolId::I64, BinaryOperator::Add) => block.bytecode.add_int(),
                    (SymbolId::I64, BinaryOperator::Sub) => block.bytecode.sub_int(),
                    (SymbolId::I64, BinaryOperator::Mul) => block.bytecode.mul_int(),
                    (SymbolId::I64, BinaryOperator::Div) => block.bytecode.div_int(),
                    (SymbolId::I64, BinaryOperator::Rem) => block.bytecode.rem_int(),
                    (SymbolId::I64, BinaryOperator::Eq) => block.bytecode.eq_int(),
                    (SymbolId::I64, BinaryOperator::Ne) => block.bytecode.ne_int(),
                    (SymbolId::I64, BinaryOperator::Gt) => block.bytecode.gt_int(),
                    (SymbolId::I64, BinaryOperator::Ge) => block.bytecode.ge_int(),
                    (SymbolId::I64, BinaryOperator::Lt) => block.bytecode.lt_int(),
                    (SymbolId::I64, BinaryOperator::Le) => block.bytecode.le_int(),
                    (SymbolId::I64, BinaryOperator::BitwiseOr) => block.bytecode.bitwise_or(),
                    (SymbolId::I64, BinaryOperator::BitwiseAnd) => block.bytecode.bitwise_and(),
                    (SymbolId::I64, BinaryOperator::BitwiseXor) => block.bytecode.bitwise_xor(),
                    (SymbolId::I64, BinaryOperator::BitshiftLeft) => block.bytecode.bitshift_left(),
                    (SymbolId::I64, BinaryOperator::BitshiftRight) => block.bytecode.bitshift_right(),

                    (SymbolId::F64, BinaryOperator::Add) => block.bytecode.add_float(),
                    (SymbolId::F64, BinaryOperator::Sub) => block.bytecode.sub_float(),
                    (SymbolId::F64, BinaryOperator::Mul) => block.bytecode.mul_float(),
                    (SymbolId::F64, BinaryOperator::Div) => block.bytecode.div_float(),
                    (SymbolId::F64, BinaryOperator::Rem) => block.bytecode.rem_float(),
                    (SymbolId::F64, BinaryOperator::Eq) => block.bytecode.eq_float(),
                    (SymbolId::F64, BinaryOperator::Ne) => block.bytecode.ne_float(),
                    (SymbolId::F64, BinaryOperator::Gt) => block.bytecode.gt_float(),
                    (SymbolId::F64, BinaryOperator::Ge) => block.bytecode.ge_float(),
                    (SymbolId::F64, BinaryOperator::Lt) => block.bytecode.lt_float(),
                    (SymbolId::F64, BinaryOperator::Le) => block.bytecode.le_float(),


                    (SymbolId::BOOL, BinaryOperator::Eq) => block.bytecode.eq_bool(),
                    (SymbolId::BOOL, BinaryOperator::Ne) => block.bytecode.ne_bool(),

                    _ => unreachable!(),
                };
            },


            parser::nodes::expr::Expr::UnaryOp { operator, rhs } => {
                self.expr(env, block, rhs)?;
                out_if_err!();

                let Ok(ty) = self.ty_info.expr(rhs)
                // we can just return cos if an err happened then the program would terminate
                // at an earlier point anyways
                else { block.bytecode.unit(); return Ok(()) };
                let Ok(sym) = ty.sym(self.syms)
                else { block.bytecode.unit(); return Ok(()) };

                match (sym, operator) {
                    (SymbolId::I64, UnaryOperator::Neg) => block.bytecode.neg_int(),
                    (SymbolId::F64, UnaryOperator::Neg) => block.bytecode.neg_float(),
                    (SymbolId::BOOL, UnaryOperator::Not) => block.bytecode.not_bool(),

                    _ => unreachable!(),
                }

            },


            parser::nodes::expr::Expr::If { condition, body, else_block } => {
                self.expr(env, block, condition)?;
                out_if_err!();

                self.build_ite(env, block,
                |this, env, block| {
                    this.expr(env, block, body)?;
                    Ok(())
                },

                |this, env, block| {
                    if let Some(else_block) = else_block {
                        this.expr(env, block, else_block)?;
                    } else {
                        block.bytecode.unit();
                    }

                    Ok(())
                })?;

            },


            parser::nodes::expr::Expr::Match { value, mappings } => {
                self.expr(env, block, value)?;
                block.bytecode.copy();
                block.bytecode.load_field(0);


                let mut cont_block = env.next_block();
                cont_block.terminator = block.terminator;

                let mut blocks = Vec::with_capacity(mappings.len());
                for mm in mappings {
                    let mut match_block = env.next_block();
                    blocks.push(match_block.index);

                    let v = env.alloc_var(mm.binding());
                    match_block.bytecode.load_field(1);
                    match_block.bytecode.store(v.try_into().unwrap());

                    let _ = self.expr(env, &mut match_block, mm.expr());

                    match_block.terminator = BlockTerminator::Goto(cont_block.index);
                    env.blocks.push(match_block);
                }

                block.terminator = BlockTerminator::Switch(blocks.leak());

                core::mem::swap(block, &mut cont_block);
                env.blocks.push(cont_block);
            },


            parser::nodes::expr::Expr::Block { block: instrs } => {
                let var_len = env.vars.len();
                self.process(env, block, &instrs).unwrap();
                out_if_err!();
                env.vars.truncate(var_len);
            },


            parser::nodes::expr::Expr::CreateStruct { fields, .. } => {
                let ty = out_if_err!();
                let sym = ty.sym(self.syms).unwrap();
                let SymbolKind::Container(cont) = self.syms.sym(sym).kind()
                else { unreachable!() };

                for sf in cont.fields() {
                    let f = fields.iter().find(|f| f.0 == sf.0.unwrap()).unwrap();

                    self.expr(env, block, f.2)?;
                }

                block.bytecode.create_struct(fields.len().try_into().unwrap());
            },


            parser::nodes::expr::Expr::AccessField { val, field_name } => {
                self.expr(env, block, val)?;
                out_if_err!();

                let val = self.ty_info.expr(val).unwrap();
                let ty = val.sym(self.syms).unwrap();
                let SymbolKind::Container(cont) = self.syms.sym(ty).kind()
                else { unreachable!() };

                let mut str = sti::string::String::new_in(self.syms.arena());
                let (i, _) = cont.fields().iter().enumerate().find(|(i, f)| {
                    let name = match f.0.to_option() {
                        Some(v) => v,
                        None => {
                            str.clear();
                            self.string_map.num(*i)
                        },
                    };

                    field_name == name
                }).unwrap();

                match cont.kind() {
                      ContainerKind::Tuple
                    | ContainerKind::Struct => block.bytecode.load_field(i.try_into().unwrap()),

                    ContainerKind::Enum => {
                        block.bytecode.load_enum_field(i.try_into().unwrap());
                    },
                }


            },


            parser::nodes::expr::Expr::CallFunction { args, .. } => {
                for arg in args {
                    self.expr(env, block, *arg)?;
                }

                out_if_err!();

                let (sym, gens) = self.ty_info.funcs.get(&expr).unwrap();
                if *sym == SymbolId::ERR { return Err(()) }

                let func = self.get_func(*sym, *gens)?;
                if let Some(err) = func.error {
                    generate_error(env, block, err);
                    block.bytecode.unit();
                    return Ok(());
                }



                block.bytecode.call(func.index.0, args.len().try_into().unwrap());
            },


            parser::nodes::expr::Expr::WithinNamespace { action, .. } => {
                out_if_err!();
                self.expr(env, block, action)?;
            },


            parser::nodes::expr::Expr::WithinTypeNamespace { action, .. } => {
                out_if_err!();
                self.expr(env, block, action)?;
            },


            parser::nodes::expr::Expr::Loop { body } => {
                self.build_loop(env, block, 
                |this, env, block| {
                    this.process(env, block, &body)?;
                    block.bytecode.pop();
                    match this.ty_info.expr(expr){
                        Ok(e) => e,
                        Err(e) => {
                            generate_error(env, block, e);
                            return Err(());
                        },
                    };
                    Ok(())
                })?;
            },


            parser::nodes::expr::Expr::Return(expr_id) => {
                self.expr(env, block, expr_id)?;
                out_if_err!();
                
                block.bytecode.ret();
                let mut cont_block = env.next_block();
                cont_block.terminator = block.terminator;

                core::mem::swap(block, &mut cont_block);
                env.blocks.push(cont_block);
                block.bytecode.unit();
            },


            parser::nodes::expr::Expr::Continue => {
                out_if_err!();

                let term = block.terminator;
                let mut cont_block = env.next_block();

                block.terminator = BlockTerminator::Goto(env.loop_cont.unwrap());
                cont_block.terminator = term;

                core::mem::swap(block, &mut cont_block);
                env.blocks.push(cont_block);
            },


            parser::nodes::expr::Expr::Break => {
                out_if_err!();

                block.bytecode.unit();

                let term = block.terminator;
                let mut cont_block = env.next_block();

                block.terminator = BlockTerminator::Goto(env.loop_brek.unwrap());
                cont_block.terminator = term;

                core::mem::swap(block, &mut cont_block);
                env.blocks.push(cont_block);
            },


            parser::nodes::expr::Expr::Tuple(expr_ids) => {
                for &e in expr_ids {
                    self.expr(env, block, e)?;
                }

                out_if_err!();
                block.bytecode.create_struct(expr_ids.len().try_into().unwrap());
            },


            parser::nodes::expr::Expr::AsCast { lhs, .. } => {
                self.expr(env, block, lhs)?;

                let Ok(lsym) = self.ty_info.expr(lhs).unwrap().sym(self.syms)
                else { return Err(()) };

                let Ok(ty) = out_if_err!().sym(self.syms)
                else { return Err(()) };

                if lsym == ty {
                    // no op
                    return Ok(())
                }

                match (lsym, ty) {
                    (SymbolId::BOOL, SymbolId::I64) => block.bytecode.cast_bool_to_int(),
                    (SymbolId::I64, SymbolId::F64) => block.bytecode.cast_int_to_float(),
                    (SymbolId::F64, SymbolId::I64) => block.bytecode.cast_float_to_int(),
                    _ => unreachable!(),
                }

            },


            parser::nodes::expr::Expr::Unwrap(expr_id) => {
                self.expr(env, block, expr_id)?;
                out_if_err!();
                block.bytecode.unwrap();
            },


            parser::nodes::expr::Expr::OrReturn(expr_id) => {
                self.expr(env, block, expr_id)?;
                out_if_err!();

                block.bytecode.copy();
                block.bytecode.load_field(0);

                block.bytecode.const_int(0);
                block.bytecode.eq_int();

                self.build_ite(env, block,
                |_, _, block| {
                    block.bytecode.load_field(1);
                    Ok(())
                },
                |_, env, block| {
                    let mut cont_block = env.next_block();
                    cont_block.terminator = block.terminator;
                    block.terminator = BlockTerminator::Ret;

                    core::mem::swap(block, &mut cont_block);
                    env.blocks.push(cont_block);

                    Ok(())
                })?;
            },
        };
        Ok(())
    }



    fn build_loop(
        &mut self, env: &mut Env<'me>, block: &mut Block<'me>,
        f: impl FnOnce(&mut Self, &mut Env<'me>, &mut Block<'me>) -> Result<(), ()>
    ) -> Result<(), ()> {

        let mut loop_block = env.next_block();

        let mut next_block = env.next_block();
        next_block.terminator = block.terminator;

        let loop_block_entry = loop_block.index;
        let term = block.terminator;
        let loop_cont = env.loop_cont;
        let loop_brek = env.loop_brek;

        env.loop_cont = Some(loop_block_entry);
        env.loop_brek = Some(next_block.index);

        block.terminator = BlockTerminator::Goto(loop_block_entry);

        let var_len = env.vars.len();

        /*
        self.process(env, &mut loop_block, &body)?;
        loop_block.bytecode.pop();
        */
        f(self, env, &mut loop_block)?;
        loop_block.terminator = BlockTerminator::Goto(loop_block_entry);

        env.vars.truncate(var_len);


        env.loop_cont = Some(loop_block_entry);
        env.loop_brek = Some(next_block.index);

        next_block.terminator = term;

        core::mem::swap(block, &mut next_block);
        env.blocks.push(next_block);
        env.blocks.push(loop_block);

        env.loop_brek = loop_brek;
        env.loop_cont = loop_cont;
        Ok(())
    }




    fn build_ite(&mut self, env: &mut Env<'me>, block: &mut Block<'me>,
        t: impl FnOnce(&mut Self, &mut Env<'me>, &mut Block<'me>) -> Result<(), ()>,
        f: impl FnOnce(&mut Self, &mut Env<'me>, &mut Block<'me>) -> Result<(), ()>,
    ) -> Result<(), ()> {

        let mut continue_block = env.next_block();

        let true_case = {
            let mut body_block = env.next_block();
            let body_block_entry = body_block.index;


            t(self, env, &mut body_block)?;

            body_block.terminator = BlockTerminator::Goto(continue_block.index);
            env.blocks.push(body_block);

            body_block_entry
        };

        let false_case = {
            let mut body_block = env.next_block();
            let body_block_entry = body_block.index;


            f(self, env, &mut body_block)?;

            body_block.terminator = BlockTerminator::Goto(continue_block.index);
            env.blocks.push(body_block);

            body_block_entry
        };

        block.terminator = BlockTerminator::SwitchBool { op1: true_case, op2: false_case };

        core::mem::swap(block, &mut continue_block);
        env.blocks.push(continue_block);
        Ok(())

    }




    fn update_value(&mut self, env: &mut Env<'me>, block: &mut Block<'me>, expr: ExprId) -> Result<(), ()> {
        match self.ast.expr(expr) {
            parser::nodes::expr::Expr::Identifier(ident) => {
                let (_, index) = env.vars.iter().rev().find(|x| x.0 == ident).unwrap();
                block.bytecode.store((*index).try_into().unwrap());
            },


            parser::nodes::expr::Expr::AccessField { val, field_name } => {
                self.expr(env, block, val)?;

                let val = self.ty_info.expr(val).unwrap();
                let ty = val.sym(self.syms).unwrap();
                let SymbolKind::Container(cont) = self.syms.sym(ty).kind()
                else { unreachable!() };

                let mut str = sti::string::String::new_in(self.syms.arena());
                let (i, _) = cont.fields().iter().enumerate().find(|(i, f)| {
                    let name = match f.0.to_option() {
                        Some(v) => v,
                        None => {
                            str.clear();
                            self.string_map.num(*i)
                        },
                    };

                    field_name == name
                }).unwrap();


                match cont.kind() {
                      ContainerKind::Tuple
                    | ContainerKind::Struct => {
                        block.bytecode.store_field(i.try_into().unwrap());
                    }

                    // you can't assign to an enum field that's not unwrapped
                    // that just doesn't make sense
                    ContainerKind::Enum => unreachable!(),
                }
            },


            parser::nodes::expr::Expr::Unwrap(expr_id) => {
                match self.ast.expr(expr_id) {
                    parser::nodes::expr::Expr::Identifier(ident) => {
                        let (_, index) = env.vars.iter().rev().find(|x| x.0 == ident).unwrap();
                        block.bytecode.load((*index).try_into().unwrap());
                        block.bytecode.unwrap_store();
                    },


                    parser::nodes::expr::Expr::AccessField { val, field_name } => {
                        self.expr(env, block, val)?;

                        let val = self.ty_info.expr(val).unwrap();
                        let ty = val.sym(self.syms).unwrap();
                        let SymbolKind::Container(cont) = self.syms.sym(ty).kind()
                        else { unreachable!() };

                        let (i, _) = cont.fields().iter().enumerate().find(|(i, f)| {
                            let name = match f.0.to_option() {
                                Some(v) => v,
                                None => {
                                    self.string_map.num(*i)
                                },
                            };

                            field_name == name
                        }).unwrap();


                        match cont.kind() {
                              ContainerKind::Tuple
                            | ContainerKind::Struct => {
                                block.bytecode.load_field(i.try_into().unwrap());
                                block.bytecode.unwrap_store();
                            }

                            ContainerKind::Enum => {
                                block.bytecode.copy();
                                block.bytecode.load_field(0); // the tag

                                block.bytecode.const_int(i.try_into().unwrap());
                                block.bytecode.eq_int();

                                self.build_ite(env, block,
                                |_, _, block| {
                                    block.bytecode.store_field(1);
                                    Ok(())
                                },
                                |_, _, block| {
                                    block.bytecode.unwrap_fail();
                                    Ok(())
                                })?;

                            },
                        }
                    }


                    parser::nodes::expr::Expr::Unwrap(_) => {
                        self.expr(env, block, expr_id)?;
                        block.bytecode.unwrap_store();

                    }


                    parser::nodes::expr::Expr::OrReturn(_) => {
                        self.expr(env, block, expr_id)?;
                        block.bytecode.unwrap_store();

                    }


                    _ => unreachable!()
                }
            },


            parser::nodes::expr::Expr::OrReturn(expr_id) => {
                match self.ast.expr(expr_id) {
                    parser::nodes::expr::Expr::Identifier(ident) => {
                        let (_, index) = env.vars.iter().rev().find(|x| x.0 == ident).unwrap();

                        block.bytecode.load((*index).try_into().unwrap());
                        block.bytecode.load_field(0);

                        block.bytecode.const_int(0);
                        block.bytecode.eq_int();


                        self.build_ite(env, block,
                        |_, _, block| {
                            block.bytecode.store_field(1);
                            Ok(())
                        },
                        |_, env, block| {
                            let mut cont_block = env.next_block();
                            cont_block.terminator = block.terminator;
                            block.terminator = BlockTerminator::Ret;

                            core::mem::swap(block, &mut cont_block);
                            env.blocks.push(cont_block);

                            Ok(())
                        })?;
                    },


                   parser::nodes::expr::Expr::AccessField { val, field_name } => {
                        self.expr(env, block, val)?;

                        let val = self.ty_info.expr(val).unwrap();
                        let ty = val.sym(self.syms).unwrap();
                        let SymbolKind::Container(cont) = self.syms.sym(ty).kind()
                        else { unreachable!() };

                        let (i, _) = cont.fields().iter().enumerate().find(|(i, f)| {
                            let name = match f.0.to_option() {
                                Some(v) => v,
                                None => {
                                    self.string_map.num(*i)
                                },
                            };

                            field_name == name
                        }).unwrap();


                        match cont.kind() {
                              ContainerKind::Tuple
                            | ContainerKind::Struct => {
                                block.bytecode.load_field(i.try_into().unwrap());

                                block.bytecode.copy();
                                block.bytecode.load_field(0);

                                block.bytecode.const_int(0);
                                block.bytecode.eq_int();


                                self.build_ite(env, block,
                                |_, _, block| {
                                    block.bytecode.store_field(1);
                                    Ok(())
                                },
                                |_, env, block| {
                                    let mut cont_block = env.next_block();
                                    cont_block.terminator = block.terminator;
                                    block.terminator = BlockTerminator::Ret;

                                    core::mem::swap(block, &mut cont_block);
                                    env.blocks.push(cont_block);

                                    Ok(())
                                })?;
                            }

                            ContainerKind::Enum => {
                                block.bytecode.copy();
                                block.bytecode.load_field(0); // the tag

                                block.bytecode.const_int(i.try_into().unwrap());
                                block.bytecode.eq_int();

                                self.build_ite(env, block,
                                |_, _, block| {
                                    block.bytecode.store_field(1);
                                    Ok(())
                                },
                                |_, env, block| {
                                    let mut cont_block = env.next_block();
                                    cont_block.terminator = block.terminator;
                                    block.terminator = BlockTerminator::Ret;

                                    core::mem::swap(block, &mut cont_block);
                                    env.blocks.push(cont_block);

                                    Ok(())
                                })?;

                            },
                        }
                    },


                    parser::nodes::expr::Expr::Unwrap(_) => {
                        self.expr(env, block, expr_id)?;
                        block.bytecode.copy();
                        block.bytecode.load_field(0); // the tag

                        block.bytecode.const_int(0);
                        block.bytecode.eq_int();

                        self.build_ite(env, block,
                        |_, _, block| {
                            block.bytecode.store_field(1);
                            Ok(())
                        },
                        |_, env, block| {
                            let mut cont_block = env.next_block();
                            cont_block.terminator = block.terminator;
                            block.terminator = BlockTerminator::Ret;

                            core::mem::swap(block, &mut cont_block);
                            env.blocks.push(cont_block);

                            Ok(())
                        })?;

                    }


                    parser::nodes::expr::Expr::OrReturn(_) => {
                        self.expr(env, block, expr_id)?;
                        block.bytecode.copy();
                        block.bytecode.load_field(0); // the tag

                        block.bytecode.const_int(0);
                        block.bytecode.eq_int();

                        self.build_ite(env, block,
                        |_, _, block| {
                            block.bytecode.store_field(1);
                            Ok(())
                        },
                        |_, env, block| {
                            let mut cont_block = env.next_block();
                            cont_block.terminator = block.terminator;
                            block.terminator = BlockTerminator::Ret;

                            core::mem::swap(block, &mut cont_block);
                            env.blocks.push(cont_block);

                            Ok(())
                        })?;



                    }


                    _ => unreachable!()
                }

            },

            _ => unreachable!(),
        };

        Ok(())
    }


}


impl<'buf> Env<'buf> {
    pub fn alloc_var(&mut self, str: StringIndex) -> u32 {
        let value = self.alloc_anon_var();
        self.vars.push((str, value));
        value
    }


    pub fn alloc_anon_var(&mut self) -> u32 {
        self.var_counter += 1;
        self.var_counter - 1
    }



    pub fn next_block_index(&mut self) -> BlockIndex {
        self.block_counter += 1;
        BlockIndex(self.block_counter - 1)
    }


    pub fn next_block(&mut self) -> Block<'buf> {
        Block {
            index: self.next_block_index(),
            bytecode: Builder::new(),
            terminator: BlockTerminator::Ret,
        }
    }
}



fn generate_error<'buf>(env: &mut Env<'buf>, builder: &mut Block<'buf>, err: ErrorId) {
    let mut cont_block = env.next_block();
    cont_block.terminator = builder.terminator;
    builder.terminator = BlockTerminator::Err(err);

    core::mem::swap(builder, &mut cont_block);
    env.blocks.push(cont_block);
    

    /*
    match err {
        errors::ErrorId::Lexer(error) => {
            builder.err(0, error.0, error.1.inner());
        },


        errors::ErrorId::Parser(error) => {
            builder.err(1, error.0, error.1.inner());

        },


        errors::ErrorId::Sema(sema_error) => {
            builder.err(2, 0, sema_error.inner());
        },
    }*/
}

