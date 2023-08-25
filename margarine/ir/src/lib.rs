#[allow(unused)]
#[allow(dead_code)]
use std::{collections::{HashMap, BTreeMap}, sync::RwLock};

use common::SymbolIndex;
use lexer::Literal;
use parser::{nodes::{StructKind, Node, Statement, Expression, BinaryOperator, UnaryOperator, NodeKind}, DataTypeKind};
use semantic_analysis::{Infer, Symbol};


pub fn convert(ctx: Infer) {
    for s in ctx.symbols.as_slice() {
        let Symbol::Function(f) = s else { continue };
        
    }
}



#[derive(Debug, Clone)]
enum IR {
    DebugName { dst: Reg, name: SymbolIndex },

    
    Unit { dst: Reg },
    Copy { dst: Reg, src: Reg },
    Literal { dst: Reg, lit: ConstIndex },

    Match { src: Reg, jumps: Vec<Label> },
    CastAny { dst: Reg, src: Reg, target: TypeId },

    CreateStruct { dst: Reg, type_id: TypeId, fields: Vec<Reg> },
    AccField { dst: Reg, src: Reg, field_index: u16 },
    SetField { dst: Reg, src: Reg, field_index: u16 },
    SetEnumVariant { dst: Reg, src: Reg, variant: u16 },

    Call { dst: Reg, function: FunctionName, args: Vec<Reg> },    

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
enum Terminator {
    Ret,
    Jmp(Label),
    Jif { cond: Reg, if_true: Label, if_false: Label },
}


#[derive(Debug, Clone, Copy)]
pub struct TypeId(u32);

#[derive(Debug, Clone, Copy)]
struct Label(u32);

#[derive(Debug, PartialEq, Clone, Copy)]
struct Reg(usize);

#[derive(Debug, Clone, Copy)]
pub struct ConstIndex(u32);

#[derive(Debug, Clone, Copy)]
struct FunctionName(SymbolIndex);


struct Block {
    label: Label,
    body: Vec<IR>,
    ending: Terminator,

    named_regs: usize,
    regs: usize,
}


pub struct Function {
    name: FunctionName,
    blocks: Vec<Block>,
    entry: Label,

    reg_count: usize,
    named_regs: Vec<(SymbolIndex, Reg)>,

    block_count: u32,
}


type State = (RwLock<Vec<(Literal, ConstIndex)>>, HashMap<DataTypeKind, TypeId>);


impl Function {
    pub fn from_body(constants: &State, name: SymbolIndex, body: &[Node]) -> Self {
        let mut function = Function {
            name: FunctionName(name),
            blocks: vec![],
            entry: Label(0),
            reg_count: 0,
            named_regs: vec![],
            block_count: 0,
        };


        let mut block = function.new_block();
        function.convert_block(constants, &mut block, body);

        function
    }


    fn convert_block(&mut self, constants: &State, block: &mut Block, body: &[Node]) -> Reg {
        let mut last_dst = self.new_reg();
        for n in body.iter() {
            last_dst = self.convert_node(constants, block, n);
        }
        last_dst
    }


    fn convert_node(&mut self, constants: &State, block: &mut Block, node: &Node) -> Reg {
        match node.kind() {
                NodeKind::Declaration(_) => self.new_reg(),

                
                NodeKind::Statement(v) => {
                    self.convert_statement(constants, block, v);
                    self.new_reg()
                },

                
                NodeKind::Expression(e) => self.convert_expression(constants, block, e, &node.data_kind),
            }
    }


    fn convert_statement(&mut self, constants: &State, block: &mut Block, statement: &Statement) {
        match statement {
            Statement::Variable { name, hint, is_mut, rhs } => {
                let reg = self.new_reg();
                let rhs = self.convert_node(constants, block, rhs);

                block.emit(IR::DebugName { dst: reg, name: *name });
                block.emit(IR::Copy { dst: reg, src: rhs });
                self.named_regs.push((*name, reg));
            },

            
            Statement::UpdateValue { lhs, rhs } => {
                let lhs = self.convert_node(constants, block, lhs);
                let rhs = self.convert_node(constants, block, rhs);

                block.emit(IR::Copy { dst: lhs, src: rhs });
            },
        }
    }


    fn convert_expression(&mut self, state: &State, block: &mut Block, expr: &Expression, typ: &DataTypeKind) -> Reg {
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
                self.named_regs.iter().rev().find(|x| x.0 == *v).unwrap().1
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
                    let ret_reg = self.convert_block(state, &mut body_block, body);
                    body_block.emit(IR::Copy { dst, src: ret_reg  });
                    let label = body_block.label;
                    self.finish_block(body_block, Terminator::Jmp(continuation.label));

                    label
                };


                let old_block = std::mem::replace(block, continuation);
                

                if let Some(else_body) = else_block {
                    let mut else_block = self.new_block();
                    let ret_reg = self.convert_node(state, &mut else_block, else_body);
                    else_block.emit(IR::Copy { dst, src: ret_reg  });
                    let label = else_block.label;
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


                let binding = self.new_reg();
                let mut jumps = Vec::with_capacity(mappings.len());
                for m in mappings.iter() {
                    self.named_regs.push((m.binding(), binding));
                    let mut mblock = self.new_block();

                    let reg = self.convert_node(state, &mut mblock, m.node());
                    mblock.emit(IR::Copy { dst, src: reg });

                    jumps.push(mblock.label);
                    self.finish_block(mblock, Terminator::Jmp(block.label));
                    
                    assert!(self.named_regs.pop().unwrap() == (m.binding(), binding));
                }
            
                block.emit(IR::Match { src: val, jumps });

                dst
            },

            
            Expression::Block { block: body } => self.convert_block(state, block, body),

            
            Expression::CreateStruct { data_type, fields } => {
                let dst = self.new_reg();
                let type_id = *state.1.get(data_type.kind()).unwrap();

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
                    true => block.emit(IR::SetEnumVariant { dst, src: val, variant: field_meta.0 }),
                    false => block.emit(IR::SetField { dst, src: val, field_index: field_meta.0 }),
                }
                
                dst
            },

            
            Expression::CallFunction { name, is_accessor, args } => {
                let dst = self.new_reg();
                let mut arg_regs = Vec::with_capacity(args.len());

                for a in args.iter() {
                    let reg = self.convert_node(state, block, &a.0);
                    arg_regs.push(reg);
                }


                block.emit(IR::Call { dst, function: FunctionName(*name), args: arg_regs });


                // fuck me.
                for a in arg_regs {
                    self.convert_statement(
                        state, 
                        block, 
                        &Statement::UpdateValue { 
                            lhs: Box::newI DONT FUICKING KNOW WHAT TO DO HERE, 
                            rhs:  
                        }
                    )
                }
                

                dst
            },
            Expression::WithinNamespace { namespace, namespace_source, action } => todo!(),
            Expression::WithinTypeNamespace { namespace, action } => todo!(),
            Expression::Loop { body } => todo!(),
            Expression::Return(_) => todo!(),
            Expression::Continue => todo!(),
            Expression::Break => todo!(),
            Expression::CastAny { lhs, data_type } => todo!(),
            Expression::Unwrap(_) => todo!(),
            Expression::OrReturn(_) => todo!(),
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
}


fn literal(state: &State, literal: &Literal) -> ConstIndex {
    let read = state.0.read().unwrap();
    if let Some(v) = read.iter().find(|x| &x.0 == literal) { v.1 }
    else {
        drop(read);
        let mut write = state.0.write().unwrap();
        let len = write.len();
        let index = ConstIndex(len.try_into().expect("too many constants (4b+)"));
        write.push((*literal, index));
        index
    }
}


impl Block {
    pub fn emit(&mut self, ir: IR) {
        self.body.push(ir);
    }
}
