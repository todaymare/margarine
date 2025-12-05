use std::{collections::HashMap, hash::Hash};

use common::string_map::{StringIndex, StringMap};
use errors::ErrorId;
use parser::nodes::{decl::Decl, expr::{BinaryOperator, ExprId, UnaryOperator}, stmt::StmtId, NodeId, Pattern, PatternKind, AST};
use runtime::opcode::{self, runtime::builder::Builder, HEADER};
use sti::hash::fxhash::{FxHasher32, FxHasher64};

use crate::{namespace::NamespaceMap, syms::{self, containers::ContainerKind, sym_map::{GenListId, SymbolId, SymbolMap}, ty::{Sym, TypeHash}, SymbolKind}, TyChecker, TyInfo};

pub struct Conversion<'me, 'out, 'ast, 'str> {
    string_map: &'me mut StringMap<'str>,
    syms: &'me mut SymbolMap<'out>,
    ns: &'me NamespaceMap,
    ast: &'me AST<'ast>,

    ty_info: &'me TyInfo,

    funcs: HashMap<TypeHash, Function<'me>>,
    const_strs: Vec<StringIndex>,

    func_counter: u32,
}


#[derive(Clone, Copy, Debug)]
struct FuncIndex(u32);
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct BlockIndex(u32);


#[derive(Debug)]
struct Function<'a> {
    sym: Sym,

    name: StringIndex,
    index: FuncIndex,
    args: Vec<u32>,
    ret: u32,

    kind: FunctionKind<'a>,
    error: Option<ErrorId>,

    hits: u32,

    cached: bool,
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
    gens: GenListId,
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
    let mut conv = Conversion {
        string_map: ty_checker.string_map,
        syms: &mut ty_checker.syms,
        ns: &ty_checker.namespaces,
        ast: ty_checker.ast,
        ty_info: &ty_checker.type_info,
        funcs: HashMap::new(),
        const_strs: Vec::new(),
        func_counter: 0,
    };


    // create IR
    for (_, sym) in &ty_checker.startups {
        let _ = conv.get_func(Sym::Ty(*sym, GenListId::EMPTY));
    }


    // do the codegen
    let mut func_sec = vec![];
    let mut code = Builder::new();
    let mut funcs = conv.funcs.iter().collect::<Vec<_>>();
    funcs.sort_by_key(|x| x.1.index.0);
    for (_, func) in &funcs {
        func_sec.push(opcode::func::consts::Func);

        let name = conv.string_map.get(func.name);
        println!("Generating function: {} (hits: {}, index: {})", name, func.hits, func.index.0);
        println!("-----------------------------------");
        for bb in match &func.kind {
            FunctionKind::Code { blocks, .. } => &*blocks,
            _ => [].as_slice(),
        } {
            println!("Basic Block {:?}:", bb.index);
            println!("{:#?}", bb.bytecode);
            println!("Terminator: {:?}", bb.terminator);
            println!();
        }
        // func meta
        func_sec.extend_from_slice(&(name.len() as u32).to_le_bytes());
        func_sec.extend_from_slice(name.as_bytes());
        func_sec.push(func.args.len().try_into().unwrap());
        func_sec.extend_from_slice(&func.ret.to_le_bytes());

        func_sec.push(func.cached as u8);

        for arg in &func.args {
            func_sec.extend_from_slice(&arg.to_le_bytes());
        }

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

                    //dbg!(&block);
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
                                    code.err(0, error.0, error.1.0);
                                },


                                errors::ErrorId::Parser(error) => {
                                    code.err(1, error.0, error.1.0);

                                },

                                errors::ErrorId::Sema(sema_error) => {
                                    code.err(2, 0, sema_error.0);
                                },


                                errors::ErrorId::Bypass => {
                                    code.err(3, 0, 0);
                                }
                            }
                        }


                        BlockTerminator::Ret => {
                            code.ret(*local_count);
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
    final_product.extend_from_slice(&code.bytecode);

    final_product
}



impl<'me, 'out, 'ast, 'str> Conversion<'me, 'out, 'ast, 'str> {
    fn next_func_id(&mut self) -> FuncIndex {
        self.func_counter += 1;
        FuncIndex(self.func_counter-1)
    }


    fn get_func(&mut self, ty: Sym) -> Result<&Function<'me>, ErrorId> {
        assert!(ty.is_resolved(&mut self.syms));

        let sym = ty.sym(self.syms).unwrap();
        let gens_id = ty.gens(&self.syms);

        let hash = ty.hash(&self.syms);

        if let Some(func) = self.funcs.get(&hash) { 
            assert!(func.sym.eq(self.syms, ty));
            return Ok(self.funcs.get(&hash).unwrap())
        }

        // create
        let fsym = self.syms.sym(sym);
        let SymbolKind::Function(sym_func) = fsym.kind()
        else { unreachable!() };

        let gens = self.syms.gens()[gens_id];

        assert_eq!(gens.len(), fsym.generics().len());
        for ((g0, _), n1) in gens.iter().zip(fsym.generics()) {
            assert_eq!(*g0, *n1);
        }

        let ret = sym_func.ret().to_ty(gens, self.syms).unwrap();
        //if ret.is_err(self.syms) { return Err(ErrorId::Bypass) }

        let ret = ret.sym(self.syms).unwrap();
        let args = sym_func.args().iter().map(|x| x.symbol().to_ty(gens, self.syms).unwrap().sym(self.syms).unwrap().0).collect();

        match sym_func.kind() {
            crate::syms::func::FunctionKind::Extern(path) => {
                let func = Function {
                    sym: ty,
                    name: self.string_map.insert(self.string_map.get(path)),
                    index: self.next_func_id(),
                    kind: FunctionKind::Extern(path),
                    error: self.ty_info.decl(sym_func.decl().unwrap()),
                    args,
                    ret: ret.0,
                    cached: sym_func.cached,

                    hits: 0,
                };

                assert!(self.funcs.insert(hash, func).is_none());
                return Ok(self.funcs.get(&hash).unwrap())
            },


            crate::syms::func::FunctionKind::UserDefined => {
                let Decl::Function { body, .. } = self.ast.decl(sym_func.decl().unwrap())
                else { unreachable!() };

                let err = self.ty_info.decl(sym_func.decl().unwrap());

                if let Some(err) = err {
                    let block = Block {
                        index: BlockIndex(0),
                        bytecode: Builder::new(),
                        terminator: BlockTerminator::Err(err),
                    };


                    let func = Function {
                        sym: ty,
                        name: self.string_map.insert(ty.display(self.string_map, self.syms)),
                        index: self.next_func_id(),
                        kind: FunctionKind::Code {
                            local_count: 0,
                            entry: BlockIndex(0),
                            blocks: vec![block],
                        },
                        error: Some(err),
                        ret: ret.0,
                        args,
                        cached: sym_func.cached,

                        hits: 0,

                    };

                    assert!(self.funcs.insert(hash, func).is_none());
                    return Ok(self.funcs.get(&hash).unwrap());
                }

                assert!(err.is_none());

                let func = Function {
                    sym: ty,
                    name: self.string_map.insert(ty.display(self.string_map, self.syms)),
                    index: self.next_func_id(),
                    kind: FunctionKind::Code {
                        local_count: 0,
                        entry: BlockIndex(0),
                        blocks: vec![],
                    },
                    error: None,
                    ret: ret.0,
                    args,
                    cached: sym_func.cached,

                    hits: 0,

                };

                assert!(self.funcs.insert(hash, func).is_none());

                let mut env = Env {
                    vars: Vec::new(),
                    var_counter: 0,
                    block_counter: 0,
                    blocks: vec![],
                    loop_brek: None,
                    loop_cont: None,
                    gens: gens_id,
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

                let result = self.process(&mut env, &mut block, &*body);
                if let Err(err) = result {
                    egenerate_error(&mut env, &mut block, err);
                }

                if let Some(err) = err {
                    egenerate_error(&mut env, &mut block, err);
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
                block.bytecode.const_int(id.0 as i64);

                let func = Function {
                    sym: ty,
                    name: self.string_map.insert(ty.display(self.string_map, self.syms)),
                    index: self.next_func_id(),

                    kind: FunctionKind::Code {
                        local_count: 0,
                        entry: BlockIndex(0),
                        blocks: vec![block],
                    },

                    error: None,
                    args,
                    ret: ret.0,
                    cached: false,

                    hits: 0,
                };

                assert!(self.funcs.insert(hash, func).is_none());
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
                    sym: ty,
                    name: self.string_map.insert(ty.display(self.string_map, self.syms)),
                    index: self.next_func_id(),
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
                    args,
                    ret: ret.0,
                    cached: false,

                    hits: 0,
                };

                assert!(self.funcs.insert(hash, func).is_none());
                return Ok(self.funcs.get(&hash).unwrap())
            },


            crate::syms::func::FunctionKind::Closure(_) => unreachable!(),
        };
    }



    fn process(&mut self, env: &mut Env<'me>, block: &mut Block<'me>, instrs: &[NodeId]) -> Result<(), ErrorId> {
        let mut has_ret = false;

        for (i, &n) in instrs.iter().enumerate() {
            has_ret = false;
            match n {
                NodeId::Decl(_) => {
                    continue;
                },


                NodeId::Stmt(stmt_id) => {
                    self.stmt(env, block, stmt_id)?;
                    continue;

                },


                NodeId::Expr(expr_id) => {
                    has_ret = true;

                    self.expr(env, block, expr_id)?;
                    if i != instrs.len() - 1 {
                        block.bytecode.pop()
                    }
                },


                NodeId::Err(error_id) => {
                    egenerate_error(env, block, error_id);
                },
            };

        }


        if !has_ret { 
            block.bytecode.unit()
        }

        Ok(())
    }


    fn resolve_pattern(
        &mut self, env: &mut Env<'me>, block: &mut Block<'me>,
        pattern: Pattern,
    ) {
        match pattern.kind() {
            PatternKind::Variable(name) => {
                let index = env.alloc_var(name);
                block.bytecode.store(index.try_into().unwrap());
            },


            PatternKind::Tuple(items) => {
                let base = env.alloc_anon_var();
                block.bytecode.store(base.try_into().unwrap());

                for (i, item) in items.iter().enumerate() {
                    block.bytecode.load(base.try_into().unwrap());
                    block.bytecode.load_field(i.try_into().unwrap());

                    let index = env.alloc_var(*item);
                    block.bytecode.store(index.try_into().unwrap());
                }
            },
        }
    }


    fn stmt(&mut self, env: &mut Env<'me>, block: &mut Block<'me>, stmt: StmtId) -> Result<(), ErrorId> {
        macro_rules! out_if_err {
            () => {
               match self.ty_info.stmt(stmt) {
                    None => (),
                    Some(e) => {
                        return Err(e);
                    },
               }
            };
        }


        match self.ast.stmt(stmt) {
            parser::nodes::stmt::Stmt::Variable { pat, rhs, .. } => {
                self.expr(env, block, rhs)?;
                out_if_err!();

                self.resolve_pattern(env, block, pat);
            },

/*
            parser::nodes::stmt::Stmt::VariableTuple { names, rhs, .. } => {
                self.expr(env, block, rhs)?;
                out_if_err!();

                for &name in names.iter().rev() {
                    env.alloc_var(name);
                    let (_, index) = env.vars.iter().rev().find(|x| x.0 == name).unwrap();
                    block.bytecode.store((*index).try_into().unwrap());
                }
            },
*/

            parser::nodes::stmt::Stmt::UpdateValue { lhs, rhs } => {
                self.ty_info.expr(lhs)?;

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
                    else { unreachable!() };

                    let sym = Sym::Ty(sym, iter_expr.gens(&self.syms));
                    let sym = sym.resolve(&[], self.syms);

                    let func = self.get_func(sym)?.index;
                    func
                };

                let len = env.vars.len();

                let iterable = env.alloc_anon_var();
                block.bytecode.store(iterable.try_into().unwrap());


                let func = env.alloc_anon_var();
                let value = env.alloc_var(binding.0);

                block.bytecode.const_int(iter_fn.0 as i64);
                block.bytecode.create_func_ref(0);
                block.bytecode.store(func as u8);


                self.build_loop(env, block,
                |this, env, block| {
                    block.bytecode.load(iterable as u8);
                    block.bytecode.load(func as u8);
                    block.bytecode.call_func_ref(1);

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
                        cont_block.terminator = block.terminator;
                        block.terminator = BlockTerminator::Goto(goto);

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


    fn expr(&mut self, env: &mut Env<'me>, block: &mut Block<'me>, expr: ExprId) -> Result<(), ErrorId> {
        self.expr_ex(env, block, expr, false)
    }


    fn expr_ex(&mut self, env: &mut Env<'me>, block: &mut Block<'me>, expr: ExprId, is_fn_call_accessor: bool) -> Result<(), ErrorId> {
        macro_rules! out_if_err {
            () => {{

                match self.ty_info.expr(expr) {
                    Ok(e) => e,
                    Err(e) => {
                        return Err(e);
                    },
               }
            }};
        }


        let val = self.ast.expr(expr);

        match val {
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


            parser::nodes::expr::Expr::Identifier(string_index, _) => {
                let ty = out_if_err!();

                let env_gens = env.gens;
                let env_gens = self.syms.get_gens(env_gens);

                let ty = ty.resolve(&[env_gens], self.syms);

                // it's a function
                if let Some(Some(func)) = self.ty_info.idents.get(&expr) {
                    let func_gens = ty.gens(self.syms);
                    let env_gens = self.syms.get_gens(env.gens);

                    let func = Sym::Ty(*func, func_gens);

                    let func = func.resolve(&[env_gens], self.syms);

                    let func = self.get_func(func).unwrap();
                    block.bytecode.const_int(func.index.0 as i64);
                    block.bytecode.create_func_ref(0);

                    return Ok(());
                }


                if let Some(index) = env.find_var(string_index) {
                    block.bytecode.load(index.try_into().unwrap());
                    return Ok(());
                }


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

                    (_, BinaryOperator::Eq) => block.bytecode.eq_obj(),
                    (_, BinaryOperator::Ne) => block.bytecode.ne_obj(),

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
                self.process(env, block, &instrs)?;
                out_if_err!();
                env.vars.truncate(var_len);
            },


            parser::nodes::expr::Expr::CreateStruct { fields, .. } => {
                let ty = out_if_err!();
                let sym = ty.sym(self.syms).unwrap();
                let SymbolKind::Container(cont) = self.syms.sym(sym).kind()
                else { unreachable!() };

                for sf in cont.fields() {
                    let f = fields.iter().find(|f| f.0 == sf.0).unwrap();

                    self.expr(env, block, f.2)?;
                }

                block.bytecode.create_struct(fields.len().try_into().unwrap());
            },


            parser::nodes::expr::Expr::AccessField { val, field_name, .. } => {
                self.expr(env, block, val)?;
                let slf = out_if_err!();

                let env_gens = self.syms.get_gens(env.gens);

                let val = self.ty_info.expr(val).unwrap();
                let val = val.resolve(&[env_gens], self.syms);
                let ty = val.sym(self.syms).unwrap();

                if let SymbolKind::Container(cont) = self.syms.sym(ty).kind()
                && let Some((i, _)) = cont.fields().iter().enumerate().find(|(_, f)| {
                    let name = f.0;
                    field_name == name
                }) {
                    match cont.kind() {
                          ContainerKind::Tuple
                        | ContainerKind::Struct => block.bytecode.load_field(i.try_into().unwrap()),

                        ContainerKind::Enum => {
                            block.bytecode.load_enum_field(i.try_into().unwrap());
                        },


                        ContainerKind::Generic => unreachable!(),
                    }

                } else {
                    let sym_gens = self.syms.get_gens(val.gens(self.syms));

                    let ns = self.syms.sym_ns(ty);
                    let ns = self.ns.get_ns(ns);

                    let sym = ns.get_sym(field_name).unwrap().unwrap();
                    let gens = slf.gens(self.syms);

                    let sym = Sym::Ty(sym, gens)
                        .resolve(&[env_gens, sym_gens], self.syms);
                    assert!(sym.is_resolved(self.syms));

                    let func = self.get_func(sym)?;

                    if !is_fn_call_accessor {
                        block.bytecode.pop();
                    }

                    block.bytecode.const_int(func.index.0 as i64);
                    block.bytecode.create_func_ref(0);
                    return Ok(())
                };

            },


            parser::nodes::expr::Expr::CallFunction { args, lhs, .. } => {
                self.expr_ex(env, block, lhs, true)?;
                out_if_err!();

                let closure_var = env.alloc_anon_var();
                block.bytecode.store(closure_var.try_into().unwrap());

                let mut argc = args.len();
                if matches!(self.ast.expr(lhs), parser::nodes::expr::Expr::AccessField { .. }) {
                    argc += 1;
                }

                for arg in args {
                    self.expr(env, block, *arg)?;
                }

                block.bytecode.load(closure_var.try_into().unwrap());

                out_if_err!(); 

                block.bytecode.call_func_ref(argc as _);
            },


            parser::nodes::expr::Expr::CreateList { exprs } => {
                for &expr in exprs {
                    self.expr(env, block, expr)?;
                }

                out_if_err!();

                block.bytecode.create_list(exprs.len().try_into().unwrap())
            }


            parser::nodes::expr::Expr::IndexList { list, index } => {
                self.expr(env, block, list)?;
                self.expr(env, block, index)?;
                out_if_err!();

                block.bytecode.index_list();
            }


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

                    this.ty_info.expr(expr)?;
                    Ok(())
                })?;
            },


            parser::nodes::expr::Expr::Return(expr_id) => {
                self.expr(env, block, expr_id)?;
                out_if_err!();
                
                let mut cont_block = env.next_block();
                cont_block.terminator = block.terminator;
                block.terminator = BlockTerminator::Ret;

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
                else { return Err(ErrorId::Bypass) };

                let Ok(ty) = out_if_err!().sym(self.syms)
                else { return Err(ErrorId::Bypass) };

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


            parser::nodes::expr::Expr::Closure { args, body } => {
                let ty = out_if_err!();
                let closure = ty.sym(self.syms).unwrap();
                let sym = self.syms.sym(closure);
                let SymbolKind::Function(func_ty) = sym.kind()
                else { unreachable!() };

                let syms::func::FunctionKind::Closure(closure) = func_ty.kind()
                else { unreachable!() };

                let env_gens = self.syms.get_gens(env.gens);
                let ty = ty.resolve(&[env_gens], self.syms);


                // @todo performance
                let captured = self.syms.closure(closure).captured_variables.clone();


                let mut hash = FxHasher64::new();
                for capture in &captured {
                    let ty = capture.1.resolve(&[env_gens], self.syms);
                    ty.hash(self.syms).hash(&mut hash);
                }

                let hash = ty.hash_fn(self.syms, |h| {
                    expr.hash(h);
                    hash.hash.hash(h);
                    self.funcs.len().hash(h);
                });


                let closure = self.syms.closure(closure);
                for name in &closure.captured_variables {
                    let index = env.find_var(name.0).unwrap();
                    block.bytecode.load(index.try_into().unwrap());
                }
                
                let capture_count = closure.captured_variables.len();
                
                let func = {
                    let mut env = Env {
                        vars: Vec::new(),
                        var_counter: 0,
                        block_counter: 0,
                        blocks: vec![],
                        loop_brek: None,
                        loop_cont: None,
                        gens: env.gens,
                    };

                    for arg in args {
                        env.alloc_var(arg.0);
                    }

                    for capture in &closure.captured_variables {
                        env.alloc_var(capture.0);
                    }

                    let argc = args.len() + closure.captured_variables.len();

                    let entry_block = env.next_block_index();
                    let mut block = Block {
                        index: entry_block,
                        bytecode: Builder::new(),
                        terminator: BlockTerminator::Ret,
                    };

                    block.bytecode.push_local_space(u8::MAX);

                    let result = self.expr(&mut env, &mut block, body);
                    if let Err(err) = result {
                        egenerate_error(&mut env, &mut block, err);
                    }

                    env.blocks.push(block);
                    let block = env.blocks.iter_mut().find(|x| x.index == entry_block).unwrap();
                    block.bytecode.push_local_space_at(0, (env.var_counter - argc as u32).try_into().unwrap());

                    env.blocks.sort_by_key(|x| x.index.0);

                    let func = Function {
                        sym: ty,
                        name: self.string_map.insert(ty.display(self.string_map, self.syms)),
                        index: self.next_func_id(),
                        kind: FunctionKind::Code {
                            local_count: (env.var_counter - argc as u32).try_into().unwrap(),
                            entry: entry_block,
                            blocks: env.blocks,
                        },

                        error: None,
                        args: vec![0; argc],
                        ret: self.ty_info.expr(body).unwrap()
                            .sym(self.syms).unwrap().0,
                        cached: false,

                        hits: 0,
                    };

                    let ret = self.funcs.insert(hash, func);
                    assert!(ret.is_none());

                    self.funcs.get(&hash).unwrap()
                };

                block.bytecode.const_int(func.index.0 as i64);
                block.bytecode.create_func_ref(capture_count as _);
            }
        };
        Ok(())
    }



    fn build_loop(
        &mut self, env: &mut Env<'me>, block: &mut Block<'me>,
        f: impl FnOnce(&mut Self, &mut Env<'me>, &mut Block<'me>) -> Result<(), ErrorId>
    ) -> Result<(), ErrorId> {

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
        t: impl FnOnce(&mut Self, &mut Env<'me>, &mut Block<'me>) -> Result<(), ErrorId>,
        f: impl FnOnce(&mut Self, &mut Env<'me>, &mut Block<'me>) -> Result<(), ErrorId>,
    ) -> Result<(), ErrorId> {

        let mut continue_block = env.next_block();
        let var = env.alloc_anon_var();

        let true_case = {
            let mut body_block = env.next_block();
            let body_block_entry = body_block.index;

            t(self, env, &mut body_block)?;
            body_block.bytecode.store(var as u8);

            body_block.terminator = BlockTerminator::Goto(continue_block.index);
            env.blocks.push(body_block);

            body_block_entry
        };

        let false_case = {
            let mut body_block = env.next_block();
            let body_block_entry = body_block.index;


            f(self, env, &mut body_block)?;
            body_block.bytecode.store(var as u8);

            body_block.terminator = BlockTerminator::Goto(continue_block.index);
            env.blocks.push(body_block);

            body_block_entry
        };

        block.terminator = BlockTerminator::SwitchBool { op1: true_case, op2: false_case };

        core::mem::swap(block, &mut continue_block);

        block.bytecode.load(var as u8);
        env.blocks.push(continue_block);
        Ok(())

    }




    fn update_value(&mut self, env: &mut Env<'me>, block: &mut Block<'me>, expr: ExprId) -> Result<(), ErrorId> {
        match self.ast.expr(expr) {
            parser::nodes::expr::Expr::Identifier(ident, _) => {
                let (_, index) = env.vars.iter().rev().find(|x| x.0 == ident).unwrap();
                block.bytecode.store((*index).try_into().unwrap());
            },


            parser::nodes::expr::Expr::IndexList { list, index } => {
                self.expr(env, block, list)?;
                self.expr(env, block, index)?;
                block.bytecode.store_list();
            }


            parser::nodes::expr::Expr::AccessField { val, field_name, .. } => {
                self.expr(env, block, val)?;

                let val = self.ty_info.expr(val).unwrap();
                let ty = val.sym(self.syms).unwrap();
                let SymbolKind::Container(cont) = self.syms.sym(ty).kind()
                else { unreachable!() };

                let (i, _) = cont.fields().iter().enumerate().find(|(_, f)| {
                    let name = f.0;
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

                    ContainerKind::Generic => unreachable!(),
                }
            },


            parser::nodes::expr::Expr::Unwrap(expr_id) => {
                match self.ast.expr(expr_id) {
                    parser::nodes::expr::Expr::Identifier(ident, _) => {
                        let (_, index) = env.vars.iter().rev().find(|x| x.0 == ident).unwrap();
                        block.bytecode.load((*index).try_into().unwrap());
                        block.bytecode.unwrap_store();
                    },


                    parser::nodes::expr::Expr::AccessField { val, field_name, .. } => {
                        self.expr(env, block, val)?;

                        let val = self.ty_info.expr(val).unwrap();
                        let ty = val.sym(self.syms).unwrap();
                        let SymbolKind::Container(cont) = self.syms.sym(ty).kind()
                        else { unreachable!() };

                        let (i, _) = cont.fields().iter().enumerate().find(|(_, f)| {
                            let name = f.0;
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


                            ContainerKind::Generic => unreachable!(),
                        }
                    }


                    _ => {
                        self.expr(env, block, expr_id)?;
                        block.bytecode.unwrap_store();
                    }
                }
            },


            parser::nodes::expr::Expr::OrReturn(expr_id) => {
                match self.ast.expr(expr_id) {
                    parser::nodes::expr::Expr::Identifier(ident, _) => {
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


                   parser::nodes::expr::Expr::AccessField { val, field_name, .. } => {
                        self.expr(env, block, val)?;

                        let val = self.ty_info.expr(val).unwrap();
                        let ty = val.sym(self.syms).unwrap();
                        let SymbolKind::Container(cont) = self.syms.sym(ty).kind()
                        else { unreachable!() };

                        let (i, _) = cont.fields().iter().enumerate().find(|(_, f)| {
                            let name = f.0;

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


                            ContainerKind::Generic => unreachable!(),
                        }
                    },


                   _ => {
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


    pub fn find_var(&self, name: StringIndex) -> Option<u32> {
        self.vars.iter().rev().find(|x| x.0 == name).map(|x| x.1)
    }
}



fn egenerate_error<'buf>(env: &mut Env<'buf>, builder: &mut Block<'buf>, err: ErrorId) {
    let mut cont_block = env.next_block();
    cont_block.terminator = builder.terminator;
    builder.terminator = BlockTerminator::Err(err);

    core::mem::swap(builder, &mut cont_block);
    env.blocks.push(cont_block);

}

