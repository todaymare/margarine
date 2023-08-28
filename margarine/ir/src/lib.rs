use std::fmt::{Write, Display};
#[allow(unused)]
#[allow(dead_code)]
use std::{collections::{HashMap, BTreeMap}, sync::RwLock};

use common::{SymbolIndex, SymbolMap};
use lexer::Literal;
use parser::{nodes::{Node, Statement, Expression, BinaryOperator, UnaryOperator, NodeKind, FunctionArgument}, DataTypeKind};
use semantic_analysis::{Infer, Symbol, is_l_val};


const RESERVED_INTERNAL_TYPE_IDS : u32 = 256;


pub fn convert(ctx: Infer) -> State {
    println!("converting");
    let mut state : State = State {
        constants: HashMap::new(),
        types: HashMap::with_capacity(ctx.symbols.len()),
        loop_break_point: None,
        loop_cont_point: None,
        functions: Vec::with_capacity(ctx.symbols.as_slice().iter().filter(|x| matches!(x, Symbol::Function(semantic_analysis::Function { is_extern: None, .. }))).count()),
        extern_functions: HashMap::with_capacity(ctx.symbols.as_slice().iter().filter(|x| matches!(x, Symbol::Function(semantic_analysis::Function { is_extern: Some(_), .. }))).count()),
    };


    for s in ctx.symbols.as_slice() {
        let Symbol::Function(semantic_analysis::Function { is_extern: Some(path), .. }) = s else { continue };
        state.extern_functions.insert(s.full_name(), *path);
    }

    
    for s in ctx.symbols.as_slice() {
        let Symbol::Function(f) = s else { continue };

        if f.is_extern.is_some() {
            continue
        }
        
        if let Some(v) = f.is_enum_variant_function {
            let function = Function {
                name: s.full_name(),
                blocks: Vec::from([
                    Block {
                        label: Label(0),
                        body: Vec::from([
                            IR::SetEnumVariant { dst: Reg(0), src: Reg(1), variant: v }
                        ]),
                        ending: Terminator::Ret,
                        named_regs: 1,
                        regs: 1,
                    }
                ]),
                reg_count: 1,
                named_regs: f.args.iter().enumerate().map(|x| (x.1.name(), Reg(x.0 + 1))).collect(),
                block_count: 1,
                args: f.args.iter()
                    .map(|x| (x.name(), x.is_inout(), state.get_type_id(x.data_type().kind())))
                    .collect(),
                return_type: state.get_type_id(f.return_type.kind()),
                is_system: f.is_system,
            };

            state.functions.push(function);
            
            continue
        }
        
        let ret_typ = state.get_type_id(f.return_type.kind());
        let function = Function::from_body(&mut state, s.full_name(), &f.args, &f.body, ret_typ, f.is_system);
        state.functions.push(function);
    }

    for f in &state.functions {
        let mut string = String::new();
        f.pretty_print(&mut string, &ctx.symbol_map);
        println!("{string}");
    }

    assert!(state.loop_break_point.is_none());
    assert!(state.loop_cont_point.is_none());
    state
}



#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum IR {
    DebugName { dst: Reg, name: SymbolIndex },

    
    Unit { dst: Reg },
    Copy { dst: Reg, src: Reg },
    Literal { dst: Reg, lit: ConstIndex },

    CastAny { dst: Reg, src: Reg, target: TypeId },

    CreateStruct { dst: Reg, type_id: TypeId, fields: Vec<Reg> },
    AccField { dst: Reg, src: Reg, field_index: u16 },
    SetField { dst: Reg, val: Reg, field_index: Vec<u16> },
    AccEnumVariant { dst: Reg, src: Reg, variant: u16 },
    SetEnumVariant { dst: Reg, src: Reg, variant: u16 },

    Call { dst: Reg, function: SymbolIndex, args: Vec<(Reg, Reg)> },    
    ExternCall { dst: Reg, function: SymbolIndex, args: Vec<(Reg, Reg)> },    

    Unwrap { src: Reg },
    OrReturn { src: Reg },

    Not { dst: Reg, src: Reg },
    NegI { dst: Reg, src: Reg },
    NegF { dst: Reg, src: Reg },


    AddI { dst: Reg, lhs: Reg, rhs: Reg },
    AddF { dst: Reg, lhs: Reg, rhs: Reg },
    AddU { dst: Reg, lhs: Reg, rhs: Reg },

    SubI { dst: Reg, lhs: Reg, rhs: Reg },
    SubF { dst: Reg, lhs: Reg, rhs: Reg },
    SubU { dst: Reg, lhs: Reg, rhs: Reg },

    MulI { dst: Reg, lhs: Reg, rhs: Reg },
    MulF { dst: Reg, lhs: Reg, rhs: Reg },
    MulU { dst: Reg, lhs: Reg, rhs: Reg },

    DivI { dst: Reg, lhs: Reg, rhs: Reg },
    DivF { dst: Reg, lhs: Reg, rhs: Reg },
    DivU { dst: Reg, lhs: Reg, rhs: Reg },

    RemI { dst: Reg, lhs: Reg, rhs: Reg },
    RemF { dst: Reg, lhs: Reg, rhs: Reg },
    RemU { dst: Reg, lhs: Reg, rhs: Reg },

    LeftShiftI { dst: Reg, lhs: Reg, rhs: Reg },
    LeftShiftU { dst: Reg, lhs: Reg, rhs: Reg },

    RightShiftI { dst: Reg, lhs: Reg, rhs: Reg },
    RightShiftU { dst: Reg, lhs: Reg, rhs: Reg },

    BitwiseAndI { dst: Reg, lhs: Reg, rhs: Reg },
    BitwiseAndU { dst: Reg, lhs: Reg, rhs: Reg },

    BitwiseOrI { dst: Reg, lhs: Reg, rhs: Reg },
    BitwiseOrU { dst: Reg, lhs: Reg, rhs: Reg },

    BitwiseXorI { dst: Reg, lhs: Reg, rhs: Reg },
    BitwiseXorU { dst: Reg, lhs: Reg, rhs: Reg },

    EqI { dst: Reg, lhs: Reg, rhs: Reg },
    EqF { dst: Reg, lhs: Reg, rhs: Reg },
    EqU { dst: Reg, lhs: Reg, rhs: Reg },
    EqB { dst: Reg, lhs: Reg, rhs: Reg },

    NeI { dst: Reg, lhs: Reg, rhs: Reg },
    NeF { dst: Reg, lhs: Reg, rhs: Reg },
    NeU { dst: Reg, lhs: Reg, rhs: Reg },
    NeB { dst: Reg, lhs: Reg, rhs: Reg },

    GtI { dst: Reg, lhs: Reg, rhs: Reg },
    GtF { dst: Reg, lhs: Reg, rhs: Reg },
    GtU { dst: Reg, lhs: Reg, rhs: Reg },

    GeI { dst: Reg, lhs: Reg, rhs: Reg },
    GeF { dst: Reg, lhs: Reg, rhs: Reg },
    GeU { dst: Reg, lhs: Reg, rhs: Reg },

    LtI { dst: Reg, lhs: Reg, rhs: Reg },
    LtF { dst: Reg, lhs: Reg, rhs: Reg },
    LtU { dst: Reg, lhs: Reg, rhs: Reg },

    LeI { dst: Reg, lhs: Reg, rhs: Reg },
    LeF { dst: Reg, lhs: Reg, rhs: Reg },
    LeU { dst: Reg, lhs: Reg, rhs: Reg },
}


#[derive(Debug, Clone)]
pub enum Terminator {
    Ret,
    Jmp(Label),
    Jif { cond: Reg, if_true: Label, if_false: Label },
    Match { src: Reg, jumps: Vec<Label> },
}


#[derive(Debug, Clone, Copy)]
pub struct TypeId(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct Label(u32);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Reg(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct ConstIndex(pub u32);

#[derive(Debug)]
pub struct Block {
    label: Label,
    body: Vec<IR>,
    ending: Terminator,

    named_regs: usize,
    regs: usize,
}


#[derive(Debug)]
pub struct Function {
    name: SymbolIndex,
    blocks: Vec<Block>,

    reg_count: usize,
    named_regs: Vec<(SymbolIndex, Reg)>,

    pub args: Vec<(SymbolIndex, bool, TypeId)>,
    pub return_type: TypeId,
    pub is_system: bool,

    block_count: u32,
}


#[derive(Debug)]
pub struct State {
    pub constants: HashMap<Literal, ConstIndex>,
    pub types: HashMap<DataTypeKind, TypeId>,

    loop_break_point: Option<(Label, Reg)>,
    loop_cont_point: Option<Label>,
    pub functions: Vec<Function>,
    pub extern_functions: HashMap<SymbolIndex, (SymbolIndex, SymbolIndex)>,
}


impl State {
    pub fn get_type_id(&mut self, data_kind: &DataTypeKind) -> TypeId {
        if let Some(v) = self.types.get(data_kind) { *v }
        else {
            let typeid = TypeId(self.types.len() as u32 + RESERVED_INTERNAL_TYPE_IDS);
            self.types.insert(data_kind.clone(), typeid);
            typeid
        }
    }
}


impl Block {
    #[inline(always)]
    pub fn label(&self) -> Label { self.label }
    #[inline(always)]
    pub fn body(&self) -> &[IR] { &self.body }
    #[inline(always)]
    pub fn terminator(&self) -> &Terminator { &self.ending }
}


impl Function {
    pub fn from_body(constants: &mut State, name: SymbolIndex, args: &[FunctionArgument], body: &[Node], return_type: TypeId, is_system: bool) -> Self {
        let mut function = Function {
            name,
            blocks: vec![],
            reg_count: 0,
            named_regs: Vec::with_capacity(args.len()),
            block_count: 0,
            args: args.iter().map(|x| (x.name(), x.is_inout(), constants.get_type_id(x.data_type().kind()))).collect(),
            return_type,
            is_system,

        };

        let return_reg = function.new_reg();
        
        args.iter().for_each(|x| {
            let reg = function.new_reg();
            function.named_regs.push((x.name(), reg))
        });


        let mut block = function.new_block();
        let reg = function.convert_block(constants, &mut block, body);
        block.emit(IR::Copy { dst: return_reg, src: reg });
        function.finish_block(block, Terminator::Ret);
        function.blocks.sort_by_key(|x| x.label.0);

        function
    }


    #[inline(always)]
    pub fn blocks(&self) -> &[Block] { &self.blocks }
    #[inline(always)]
    pub fn name(&self) -> SymbolIndex { self.name }
    

    fn convert_block(&mut self, state: &mut State, block: &mut Block, body: &[Node]) -> Reg {
        let mut last_dst = self.new_reg();
        for n in body.iter() {
            last_dst = self.convert_node(state, block, n);
        }
        last_dst
    }


    fn convert_node(&mut self, state: &mut State, block: &mut Block, node: &Node) -> Reg {
        match node.kind() {
                NodeKind::Declaration(_) => self.new_reg(),

                
                NodeKind::Statement(v) => {
                    self.convert_statement(state, block, v);
                    self.new_reg()
                },

                
                NodeKind::Expression(e) => self.convert_expression(state, block, e, &node.data_kind),
            }
    }


    fn convert_statement(&mut self, state: &mut State, block: &mut Block, statement: &Statement) {
        match statement {
            Statement::Variable { name, rhs, .. } => {
                let reg = self.new_reg();
                let rhs = self.convert_node(state, block, rhs);

                block.emit(IR::DebugName { dst: reg, name: *name });
                block.emit(IR::Copy { dst: reg, src: rhs });
                self.named_regs.push((*name, reg));
            },

            
            Statement::UpdateValue { lhs, rhs } => {
                let lhs_reg = self.convert_node(state, block, lhs);
                let rhs_reg = self.convert_node(state, block, rhs);

                self.update_value(block, lhs.kind(), rhs_reg);
            },
        }
    }


    fn convert_expression(&mut self, state: &mut State, block: &mut Block, expr: &Expression, typ: &DataTypeKind) -> Reg {
        println!("expression {expr:?}");
        match expr {
            Expression::Unit => {
                let reg = self.new_reg();
                block.emit(IR::Unit { dst: reg });
                reg
            },

            
            Expression::Literal(v) => {
                let dst = self.new_reg();
                let index = literal(state, v);

                block.emit(IR::Literal { dst, lit: index });
                dst
            },

            
            Expression::Identifier(v) => {
                self.find_named_reg(*v)
            },

            
            Expression::BinaryOp { operator, lhs, rhs } => {
                let dst = self.new_reg();
                let lhs = self.convert_node(state, block, lhs);
                let rhs = self.convert_node(state, block, rhs);

                let ir = match (operator, typ) {
                    (BinaryOperator::Add, DataTypeKind::Int)   => IR::AddI { dst, lhs, rhs },
                    (BinaryOperator::Add, DataTypeKind::Float) => IR::AddF { dst, lhs, rhs },
                    (BinaryOperator::Sub, DataTypeKind::Int)   => IR::SubI { dst, lhs, rhs },
                    (BinaryOperator::Sub, DataTypeKind::Float) => IR::SubF { dst, lhs, rhs },
                    (BinaryOperator::Mul, DataTypeKind::Int)   => IR::MulI { dst, lhs, rhs },
                    (BinaryOperator::Mul, DataTypeKind::Float) => IR::MulF { dst, lhs, rhs },
                    (BinaryOperator::Div, DataTypeKind::Int)   => IR::DivI { dst, lhs, rhs },
                    (BinaryOperator::Div, DataTypeKind::Float) => IR::DivF { dst, lhs, rhs },
                    (BinaryOperator::Rem, DataTypeKind::Int)   => IR::RemI { dst, lhs, rhs },
                    (BinaryOperator::Rem, DataTypeKind::Float) => IR::RemF { dst, lhs, rhs },
                    (BinaryOperator::BitshiftLeft, DataTypeKind::Int)  => IR::LeftShiftI { dst, lhs, rhs },
                    (BinaryOperator::BitshiftRight, DataTypeKind::Int) => IR::RightShiftI { dst, lhs, rhs },
                    (BinaryOperator::BitwiseAnd, DataTypeKind::Int)    => IR::BitwiseAndI { dst, lhs, rhs },
                    (BinaryOperator::BitwiseOr, DataTypeKind::Int)     => IR::BitwiseOrI { dst, lhs, rhs },
                    (BinaryOperator::BitwiseXor, DataTypeKind::Int)    => IR::BitwiseXorI { dst, lhs, rhs },
                    (BinaryOperator::Eq, DataTypeKind::Int)   => IR::EqI { dst, lhs, rhs },
                    (BinaryOperator::Eq, DataTypeKind::Bool)  => IR::EqB { dst, lhs, rhs },
                    (BinaryOperator::Eq, DataTypeKind::Float) => IR::EqF { dst, lhs, rhs },
                    (BinaryOperator::Eq, DataTypeKind::Unit)  => IR::Literal { dst, lit: literal(state, &Literal::Bool(true)) },
                    (BinaryOperator::Ne, DataTypeKind::Int)   => IR::NeI { dst, lhs, rhs },
                    (BinaryOperator::Ne, DataTypeKind::Bool)  => IR::NeB { dst, lhs, rhs },
                    (BinaryOperator::Ne, DataTypeKind::Float) => IR::NeF { dst, lhs, rhs },
                    (BinaryOperator::Ne, DataTypeKind::Unit)  => IR::Literal { dst, lit: literal(state, &Literal::Bool(false)) },
                    (BinaryOperator::Gt, DataTypeKind::Int)   => IR::GtI { dst, lhs, rhs },
                    (BinaryOperator::Gt, DataTypeKind::Float) => IR::GtF { dst, lhs, rhs },
                    (BinaryOperator::Ge, DataTypeKind::Int)   => IR::GeI { dst, lhs, rhs },
                    (BinaryOperator::Ge, DataTypeKind::Float) => IR::GeF { dst, lhs, rhs },
                    (BinaryOperator::Lt, DataTypeKind::Int)   => IR::LtI { dst, lhs, rhs },
                    (BinaryOperator::Lt, DataTypeKind::Float) => IR::LtF { dst, lhs, rhs },
                    (BinaryOperator::Le, DataTypeKind::Int)   => IR::LeI { dst, lhs, rhs },
                    (BinaryOperator::Le, DataTypeKind::Float) => IR::LeF { dst, lhs, rhs },

                    _ => unreachable!(),
                };

                block.emit(ir);
                dst
            },

            
            Expression::UnaryOp { operator, rhs } => {
                let dst = self.new_reg();
                let rhs = self.convert_node(state, block, rhs);

                let ir = match (operator, typ) {
                    (UnaryOperator::Not, DataTypeKind::Bool) => IR::Not { dst, src: rhs },
                    (UnaryOperator::Neg, DataTypeKind::Int) => IR::NegI { dst, src: rhs },
                    (UnaryOperator::Neg, DataTypeKind::Float) => IR::NegF { dst, src: rhs },

                    _ => unreachable!()
                };

                block.emit(ir);
                dst
            },
            
            
            Expression::If { condition, body, else_block } => {
                let dst = self.new_reg();
                let cond = self.convert_node(state, block, condition);
                let continuation = self.new_block();

                let body_block = {
                    let mut body_block = self.new_block();
                    let label = body_block.label;
                    let ret_reg = self.convert_block(state, &mut body_block, body);
                    body_block.emit(IR::Copy { dst, src: ret_reg  });
                    self.finish_block(body_block, Terminator::Jmp(continuation.label));

                    label
                };


                let old_block = std::mem::replace(block, continuation);
                

                if let Some(else_body) = else_block {
                    let mut else_block = self.new_block();
                    let label = else_block.label;

                    let ret_reg = self.convert_node(state, &mut else_block, else_body);
                    else_block.emit(IR::Copy { dst, src: ret_reg  });
                    self.finish_block(else_block, Terminator::Jmp(block.label));

                    self.finish_block(old_block, Terminator::Jif { cond, if_true: body_block, if_false: label });

                } else {
                    self.finish_block(old_block, Terminator::Jif { cond, if_true: body_block, if_false: block.label });
                }
                
                                
                dst
            },

            
            Expression::Match { value, mappings } => {
                let dst = self.new_reg();
                let val = self.convert_node(state, block, value);

                let continuation = self.new_block();

                let binding = self.new_reg();
                let mut jumps = Vec::with_capacity(mappings.len());
                for m in mappings.iter() {
                    self.named_regs.push((m.binding(), binding));
                    let mut mblock = self.new_block();

                    let reg = self.convert_node(state, &mut mblock, m.node());
                    mblock.emit(IR::Copy { dst, src: reg });

                    jumps.push(mblock.label);
                    self.finish_block(mblock, Terminator::Jmp(continuation.label));
                    
                    assert!(self.named_regs.pop().unwrap() == (m.binding(), binding));
                }
            
                let old_block = std::mem::replace(block, continuation);
                self.finish_block(old_block, Terminator::Match { src: val, jumps });

                dst
            },

            
            Expression::Block { block: body } => self.convert_block(state, block, body),

            
            Expression::CreateStruct { data_type, fields } => {
                let dst = self.new_reg();
                let type_id = state.get_type_id(data_type.kind());

                let mut field_mappings = Vec::with_capacity(fields.len());
                for f in fields {
                    let reg = self.convert_node(state, block, &f.2);
                    field_mappings.push(reg);
                }


                block.emit(IR::CreateStruct { dst, type_id, fields: field_mappings });
                
                dst
            },

            
            Expression::AccessField { val, field: _, field_meta } => {
                let dst = self.new_reg();
                let val = self.convert_node(state, block, val);

                match field_meta.1 {
                    true => block.emit(IR::AccEnumVariant { dst, src: val, variant: field_meta.0 }),
                    false => block.emit(IR::AccField { dst, src: val, field_index: field_meta.0 }),
                }
                
                dst
            },

            
            Expression::CallFunction { name, is_accessor: _, args } => {
                let dst = self.new_reg();
                let mut arg_regs = Vec::with_capacity(args.len());

                for a in args.iter() {
                    let reg = self.convert_node(state, block, &a.0);
                    let inout = if a.1 { self.new_reg() } else { dst };
                    arg_regs.push((reg, inout));
                }


                if state.extern_functions.contains_key(name) {
                    block.emit(IR::Call { dst, function: *name, args: arg_regs.clone() });
                } else {
                    block.emit(IR::Call { dst, function: *name, args: arg_regs.clone() });
                }


                // fuck me.
                assert_eq!(arg_regs.len(), args.len());
                for (reg, arg) in arg_regs.iter().zip(args.iter()) {
                    let kind = arg.0.kind();
                    self.update_value(block, kind, reg.1);
                }
                

                dst
            },
            Expression::WithinNamespace { action, .. } => self.convert_node(state, block, action),
            Expression::WithinTypeNamespace { action, .. } => self.convert_node(state, block, action),


            Expression::Loop { body } => {
                let mut body_block = self.new_block();
                let label = body_block.label;

                let break_block = self.new_block();
                let reg = self.new_reg();

                state.loop_break_point = Some((break_block.label, reg));
                state.loop_cont_point = Some(body_block.label);

                let body_reg = self.convert_block(state, &mut body_block, body);
                self.finish_block(body_block, Terminator::Jmp(label));

                state.loop_break_point = None;
                state.loop_cont_point = None;

                let old_block = std::mem::replace(block, break_block);
                self.finish_block(old_block, Terminator::Jmp(label));

                block.emit(IR::Copy { dst: reg, src: body_reg });

                reg
            },

            
            Expression::Return(v) => {
                let reg = self.convert_node(state, block, v);

                block.emit(IR::Copy { dst: Reg(0), src: reg });
                let block = std::mem::replace(block, self.new_block());
                self.finish_block(block, Terminator::Ret);
                Reg(0)
            },

            
            Expression::Continue => {
                let old_block = std::mem::replace(block, self.new_block());
                self.finish_block(old_block, Terminator::Jmp(state.loop_cont_point.unwrap()));
                self.new_reg()
            },

            
            Expression::Break => {
                let old_block = std::mem::replace(block, self.new_block());
                self.finish_block(old_block, Terminator::Jmp(state.loop_break_point.unwrap().0));
                self.new_reg()
            },

            
            Expression::CastAny { lhs, data_type } => {
                let dst = self.new_reg();
                let lhs = self.convert_node(state, block, lhs);
                
                block.emit(IR::CastAny { dst, src: lhs, target: state.get_type_id(data_type.kind()) });
                
                dst
            },

            
            Expression::Unwrap(v) => {
                let reg = self.convert_node(state, block, v);
                block.emit(IR::Unwrap { src: reg });
                reg
            },

            
            Expression::OrReturn(v) => {
                let reg = self.convert_node(state, block, v);
                block.emit(IR::OrReturn { src: reg });
                reg
                
            },
        }
    }


    fn new_reg(&mut self) -> Reg {
        self.reg_count += 1;
        Reg(self.reg_count-1)
    }


    fn new_block(&mut self) -> Block {
        self.block_count += 1;
        Block {
            label: Label(self.block_count-1),
            body: vec![],
            ending: Terminator::Ret,
            named_regs: self.named_regs.len(),
            regs: self.reg_count,
        }
    }
    

    fn finish_block(&mut self, mut block: Block, terminator: Terminator) {
        block.ending = terminator;
        self.named_regs.truncate(block.named_regs);
        self.reg_count = block.regs;
        self.blocks.push(block)
    }


    fn find_named_reg(&self, name: SymbolIndex) -> Reg {
        self.named_regs.iter().rev().find(|x| x.0 == name).unwrap().1
    }


    fn update_value(&self, block: &mut Block, node: &NodeKind, src: Reg) {
        match node {
            NodeKind::Expression(Expression::Unwrap(val)) => self.update_value(block, val.kind(), src),
            NodeKind::Expression(Expression::OrReturn(val)) => self.update_value(block, val.kind(), src),

            NodeKind::Expression(Expression::Identifier(val)) => {
                let named_reg = self.find_named_reg(*val);
                block.emit(IR::Copy { dst: named_reg, src });
            },


            NodeKind::Expression(Expression::AccessField { val, field: _, field_meta }) => {
                let mut current = val.kind();
                // PERFORMANCE: Cache the vec
                let mut field_accesses = vec![field_meta.0];
                loop {
                    match current {
                        NodeKind::Expression(Expression::Identifier(v)) => {
                            field_accesses.reverse();
                            block.emit(IR::SetField { dst: self.find_named_reg(*v), val: src, field_index: field_accesses });
                            break
                        },


                        NodeKind::Expression(Expression::AccessField { val, field: _, field_meta }) => {
                            field_accesses.push(field_meta.0);
                            current = val.kind();
                        },


                        NodeKind::Expression(Expression::Unwrap(val)) => current = val.kind(),
                        NodeKind::Expression(Expression::OrReturn(val)) => current = val.kind(),
                        
                        
                        _ => break
                    }
                }
            }

           

            // If the value is not an L value
            // we can just, not assign it. This
            // way you can still pass in R values
            // as inouts but they have no affect
            _ => assert!(!is_l_val(node))
        }
    }
}


impl Function {
    pub fn pretty_print(&self, f: &mut impl Write, symbol_map: &SymbolMap) {
        let _ = write!(f, "fn {} (", symbol_map.get(self.name));

        for i in 0..self.args.len() {
            if i != 0 {
                let _ = write!(f, ", ");
            }

            let _ = write!(f, "{}", symbol_map.get(self.named_regs[i].0));
        }
        
        let _ = write!(f, ")");
        let _ = writeln!(f);

        for b in self.blocks.iter() {
            b.pretty_print(f, symbol_map, true);
            let _ = writeln!(f);
        }
    }
}


impl Block {
    pub fn pretty_print(&self, f: &mut impl Write, symbol_map: &SymbolMap, indent: bool) {
        const SPACING : &str = "    ";
        if indent {
            let _ = write!(f, "{SPACING}");
        }
        let _ = writeln!(f, "{}:", self.label);

        for i in self.body.iter() {
            if indent {
                let _ = write!(f, "{SPACING}");
            }

            let _ = write!(f, "{SPACING}");

            let _ = match i {
                IR::DebugName { dst, name } => write!(f, "debug '{}' {dst}", symbol_map.get(*name)),
                IR::Unit { dst } => write!(f, "unit {dst}"),
                IR::Copy { dst, src } => write!(f, "copy {dst} {src}"),
                IR::Literal { dst, lit } => write!(f, "literal {dst} lit({})", lit.0),

                
                IR::CastAny { dst, src, target } => write!(f, "castany {dst} {src} {target}"),

                
                IR::CreateStruct { dst, type_id, fields } => {
                    let _ = writeln!(f, "createstruct {dst} {type_id} {{");
                    for j in fields.iter() {
                        if indent {
                            let _ = write!(f, "{SPACING}");
                        }

                        let _ = writeln!(f, "{SPACING}{SPACING}{},", j);
                    }

                    if indent {
                        let _ = write!(f, "{SPACING}");
                    }
                    let _ = write!(f, "{SPACING}");
                    write!(f, "}}")
                },

                
                IR::AccField { dst, src, field_index } => write!(f, "accfield {dst} {src} index({field_index})"),
                IR::SetField { dst, val, field_index } => {
                    let _ = write!(f, "setfield {dst} {val} (");
                    
                    for i in field_index.iter().enumerate() {
                        if i.0 != 0 {
                            let _ = write!(f, ", ");
                        }

                        let _ = write!(f, "{}", i.1);
                    }

                    write!(f, ")")
                },

                
                IR::AccEnumVariant { dst, src, variant } => write!(f, "accvariant {dst} {src} index({variant})"),
                IR::SetEnumVariant { dst, src, variant } => write!(f, "setvariant {dst} {src} index({variant})"),
                IR::Call { dst, function, args } => {
                    let _ = write!(f, "call {dst} {} (", symbol_map.get(*function));
                    for i in args.iter().enumerate() {
                        if i.0 != 0 {
                            let _ = write!(f, ", ");
                        }

                        let _ = write!(f, "{} -> {}", i.1.0, i.1.1);
                    }

                    write!(f, ")")
                },

                IR::ExternCall { dst, function, args } => {
                    let _ = write!(f, "ecall {dst} {} (", symbol_map.get(*function));
                    for i in args.iter().enumerate() {
                        if i.0 != 0 {
                            let _ = write!(f, ", ");
                        }

                        let _ = write!(f, "{} -> {}", i.1.0, i.1.1);
                    }

                    write!(f, ")")
                },

                IR::Unwrap { src } => write!(f, "unwrap {src}"),
                IR::OrReturn { src } => write!(f, "orreturn {src}"),
                
                IR::Not { dst, src } => write!(f, "not {dst} {src}"),

                
                IR::NegI { dst, src } => write!(f, "negi {dst} {src}"),
                IR::NegF { dst, src } => write!(f, "negf {dst} {src}"),
                IR::AddI { dst, lhs, rhs } => write!(f, "addi {dst} {lhs} {rhs}"),
                IR::AddF { dst, lhs, rhs } => write!(f, "addf {dst} {lhs} {rhs}"),
                IR::AddU { dst, lhs, rhs } => write!(f, "addu {dst} {lhs} {rhs}"),
                IR::SubI { dst, lhs, rhs } => write!(f, "subi {dst} {lhs} {rhs}"),
                IR::SubF { dst, lhs, rhs } => write!(f, "subf {dst} {lhs} {rhs}"),
                IR::SubU { dst, lhs, rhs } => write!(f, "subu {dst} {lhs} {rhs}"),
                IR::MulI { dst, lhs, rhs } => write!(f, "muli {dst} {lhs} {rhs}"),
                IR::MulF { dst, lhs, rhs } => write!(f, "mulf {dst} {lhs} {rhs}"),
                IR::MulU { dst, lhs, rhs } => write!(f, "mulu {dst} {lhs} {rhs}"),
                IR::DivI { dst, lhs, rhs } => write!(f, "divi {dst} {lhs} {rhs}"),
                IR::DivF { dst, lhs, rhs } => write!(f, "divf {dst} {lhs} {rhs}"),
                IR::DivU { dst, lhs, rhs } => write!(f, "divu {dst} {lhs} {rhs}"),
                IR::RemI { dst, lhs, rhs } => write!(f, "remi {dst} {lhs} {rhs}"),
                IR::RemF { dst, lhs, rhs } => write!(f, "remf {dst} {lhs} {rhs}"),
                IR::RemU { dst, lhs, rhs } => write!(f, "remu {dst} {lhs} {rhs}"),
                IR::LeftShiftI { dst, lhs, rhs } => write!(f, "lsi {dst} {lhs} {rhs}"),
                IR::LeftShiftU { dst, lhs, rhs } => write!(f, "lsu {dst} {lhs} {rhs}"),
                IR::RightShiftI { dst, lhs, rhs } => write!(f, "rsi {dst} {lhs} {rhs}"),
                IR::RightShiftU { dst, lhs, rhs } => write!(f, "rsu {dst} {lhs} {rhs}"),
                IR::BitwiseAndI { dst, lhs, rhs } => write!(f, "bwandi {dst} {lhs} {rhs}"),
                IR::BitwiseAndU { dst, lhs, rhs } => write!(f, "bwandu {dst} {lhs} {rhs}"),
                IR::BitwiseOrI { dst, lhs, rhs } => write!(f, "bwori {dst} {lhs} {rhs}"),
                IR::BitwiseOrU { dst, lhs, rhs } => write!(f, "bworu {dst} {lhs} {rhs}"),
                IR::BitwiseXorI { dst, lhs, rhs } => write!(f, "bwxori {dst} {lhs} {rhs}"),
                IR::BitwiseXorU { dst, lhs, rhs } => write!(f, "bwxoru {dst} {lhs} {rhs}"),
                IR::EqI { dst, lhs, rhs } => write!(f, "eqi {dst} {lhs} {rhs}"),
                IR::EqF { dst, lhs, rhs } => write!(f, "eqf {dst} {lhs} {rhs}"),
                IR::EqU { dst, lhs, rhs } => write!(f, "equ {dst} {lhs} {rhs}"),
                IR::EqB { dst, lhs, rhs } => write!(f, "eqb {dst} {lhs} {rhs}"),
                IR::NeI { dst, lhs, rhs } => write!(f, "nei {dst} {lhs} {rhs}"),
                IR::NeF { dst, lhs, rhs } => write!(f, "nef {dst} {lhs} {rhs}"),
                IR::NeU { dst, lhs, rhs } => write!(f, "neu {dst} {lhs} {rhs}"),
                IR::NeB { dst, lhs, rhs } => write!(f, "neb {dst} {lhs} {rhs}"),
                IR::GtI { dst, lhs, rhs } => write!(f, "gti {dst} {lhs} {rhs}"),
                IR::GtF { dst, lhs, rhs } => write!(f, "gtf {dst} {lhs} {rhs}"),
                IR::GtU { dst, lhs, rhs } => write!(f, "gtu {dst} {lhs} {rhs}"),
                IR::GeI { dst, lhs, rhs } => write!(f, "gei {dst} {lhs} {rhs}"),
                IR::GeF { dst, lhs, rhs } => write!(f, "gef {dst} {lhs} {rhs}"),
                IR::GeU { dst, lhs, rhs } => write!(f, "geu {dst} {lhs} {rhs}"),
                IR::LtI { dst, lhs, rhs } => write!(f, "lti {dst} {lhs} {rhs}"),
                IR::LtF { dst, lhs, rhs } => write!(f, "lti {dst} {lhs} {rhs}"),
                IR::LtU { dst, lhs, rhs } => write!(f, "ltu {dst} {lhs} {rhs}"),
                IR::LeI { dst, lhs, rhs } => write!(f, "lei {dst} {lhs} {rhs}"),
                IR::LeF { dst, lhs, rhs } => write!(f, "lef {dst} {lhs} {rhs}"),
                IR::LeU { dst, lhs, rhs } => write!(f, "leu {dst} {lhs} {rhs}"),
            };

            let _ = writeln!(f);
            
        }

        if indent {
            let _ = write!(f, "{SPACING}");
        }

        let _ = write!(f, "{SPACING}");
        let _ = match &self.ending {
            Terminator::Ret => write!(f, "ret"),
            Terminator::Jmp(v) => write!(f, "jmp {v}"),
            Terminator::Jif { cond, if_true, if_false } => write!(f, "jif {cond} {if_true} {if_false}"),
            Terminator::Match { src, jumps } => {
                let _ = writeln!(f, "match {src} {{");
                for j in jumps.iter().enumerate() {
                    if indent {
                        let _ = write!(f, "{SPACING}");
                    }

                    let _ = writeln!(f, "{SPACING}{SPACING}{}: {},", j.0, j.1);
                }

                if indent {
                    let _ = write!(f, "{SPACING}");
                }
                let _ = write!(f, "{SPACING}");
                write!(f, "}}")
            },
        };
    }
}



impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}


impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bb{}", self.0)
    }
}


impl Display for TypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type({})", self.0)
    }
}


fn literal(state: &mut State, literal: &Literal) -> ConstIndex {
    if let Some(v) = state.constants.get(literal) { *v }
    else {
        let index = ConstIndex(state.constants.len().try_into().expect("too many constants (4b+)"));
        state.constants.insert(*literal, index);
        index
    }
}


impl Block {
    pub fn emit(&mut self, ir: IR) {
        self.body.push(ir);
    }
}
