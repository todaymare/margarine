use std::{collections::linked_list, mem::size_of, sync::Mutex};

use common::{SymbolIndex, SymbolMap};
use ir::{State, Function, Reg, TypeId};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use runtime::bytecode::Bytecode;

pub fn codegen(symbol_map: &SymbolMap, state: &State) {
    let function_calls = Mutex::new(Vec::new());
    let bodies = state.functions
        .iter()
        .map(|x| codegen_function(x, symbol_map, &function_calls))
        .collect::<Vec<_>>();

    for b in bodies.iter() {
        println!("{:?}", b);
    }
}


fn codegen_function(function: &Function, symbol_map: &SymbolMap, function_calls: &Mutex<Vec<(SymbolIndex, SymbolIndex, usize)>>) -> BytecodeVec {
    let mut bytes = BytecodeVec::with_capacity(128);
    let mut labels = Vec::with_capacity(function.blocks().len());
    let mut endings = Vec::with_capacity(function.blocks().len());

    let size_offset;
    {
        bytes.emit_bytecode(Bytecode::Func);
        bytes.emit_str(symbol_map.get(function.name()));
        bytes.emit_byte(function.is_system as u8);
        size_offset = bytes.offset();
        bytes.emit_u32(u32::MAX);
        bytes.emit_type_id(function.return_type);
        bytes.emit_byte(function.args.len().try_into().unwrap());
        for a in &function.args {
            bytes.emit_str(symbol_map.get(a.0));
            bytes.emit_byte(a.1 as u8);
            bytes.emit_type_id(a.2);
        }
    }
    let starting_pos = bytes.offset();

    for block in function.blocks() {
        labels.push((block.label(), bytes.offset()));
        

        for ir in block.body() {
            match ir {
                ir::IR::DebugName { .. } => (),

                
                ir::IR::Unit { dst } => {
                    bytes.emit_bytecode(Bytecode::Unit);
                    bytes.emit_reg(*dst);
                },

                
                ir::IR::Copy { dst, src } => {
                    bytes.emit_bytecode(Bytecode::Copy);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*src);
                },

                
                ir::IR::Literal { dst, lit } => {
                    bytes.emit_bytecode(Bytecode::Lit);
                    bytes.emit_reg(*dst);
                    bytes.emit_u32(lit.0)
                },

                
                ir::IR::CastAny { dst, src, target } => {
                    bytes.emit_bytecode(Bytecode::CastAny);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*src);
                    bytes.emit_u32(target.0);
                },

                
                ir::IR::CreateStruct { dst, type_id, fields } => {
                    bytes.emit_bytecode(Bytecode::CreateStruct);
                    bytes.emit_type_id(*type_id);
                    bytes.emit_u16(fields.len().try_into().unwrap());
                    for f in fields {
                        bytes.emit_reg(*f);
                    }
                },

                
                ir::IR::AccField { dst, src, field_index } => {
                    bytes.emit_bytecode(Bytecode::AccField);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*src);
                    bytes.emit_u16(*field_index);
                },

                
                ir::IR::SetField { dst, val, field_index } => {
                    bytes.emit_bytecode(Bytecode::SetField);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*val);
                    bytes.emit_u16(field_index.len().try_into().unwrap());
                    for f in field_index {
                        bytes.emit_u16(*f);
                    }
                },

                
                ir::IR::AccEnumVariant { dst, src, variant } => {
                    bytes.emit_bytecode(Bytecode::AccVariant);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*src);
                    bytes.emit_u16(*variant);
                },

                
                ir::IR::SetEnumVariant { dst, src, variant } => {
                    bytes.emit_bytecode(Bytecode::SetVariant);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*src);
                    bytes.emit_u16(*variant);
                },

                
                ir::IR::Call { dst, function: target, args } => {
                    bytes.emit_bytecode(Bytecode::Call);
                    bytes.emit_reg(*dst);
                    
                    function_calls.lock().unwrap().push((function.name(), *target, bytes.offset()));
                    bytes.reserve(size_of::<u32>());
                    
                    bytes.emit_byte(args.len().try_into().unwrap());
                    for a in args {
                        bytes.emit_reg(a.0);
                        bytes.emit_reg(a.1);
                    }
                    
                },

                
                ir::IR::ExternCall { dst, function: target, args } => {
                    bytes.emit_bytecode(Bytecode::Call);
                    bytes.emit_reg(*dst);
                    
                    function_calls.lock().unwrap().push((function.name(), *target, bytes.offset()));
                    bytes.reserve(size_of::<u32>());
                    
                    bytes.emit_byte(args.len().try_into().unwrap());
                    for a in args {
                        bytes.emit_reg(a.0);
                        bytes.emit_reg(a.1);
                    }
                }

                
                ir::IR::Unwrap { src } => {
                    bytes.emit_bytecode(Bytecode::Unwrap);
                    bytes.emit_reg(*src);
                },

                
                ir::IR::OrReturn { src } => {
                    bytes.emit_bytecode(Bytecode::Unwrap);
                    bytes.emit_reg(*src);
                },

                
                ir::IR::Not { dst, src } => {
                    bytes.emit_bytecode(Bytecode::Not);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*src);
                },

                
                ir::IR::NegI { dst, src } => {
                    bytes.emit_bytecode(Bytecode::NegI);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*src);
                },

                
                ir::IR::NegF { dst, src } => {
                    bytes.emit_bytecode(Bytecode::NegF);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*src);
                },

                
                ir::IR::AddI { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::AddI);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::AddF { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::AddF);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::AddU { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::AddU);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::SubI { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::SubI);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::SubF { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::SubF);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::SubU { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::SubU);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::MulI { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::MulI);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::MulF { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::MulF);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::MulU { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::MulU);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::DivI { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::DivI);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::DivF { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::DivF);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::DivU { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::DivU);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::RemI { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::RemI);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::RemF { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::RemF);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::RemU { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::RemU);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::LeftShiftI { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::LeftShiftI);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::LeftShiftU { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::LeftShiftU);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::RightShiftI { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::RightShiftI);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::RightShiftU { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::RightShiftU);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::BitwiseAndI { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::BitwiseAndI);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::BitwiseAndU { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::BitwiseAndU);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::BitwiseOrI { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::BitwiseOrI);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::BitwiseOrU { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::BitwiseOrU);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::BitwiseXorI { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::BitwiseXorI);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::BitwiseXorU { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::BitwiseXorU);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::EqI { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::EqI);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::EqF { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::EqF);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::EqU { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::EqU);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::EqB { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::EqB);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::NeI { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::NeI);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::NeF { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::NeF);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::NeU { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::NeU);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::NeB { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::NeB);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::GtI { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::GtI);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::GtF { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::GtF);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::GtU { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::GtU);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::GeI { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::GeI);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::GeF { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::GeF);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::GeU { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::GeU);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::LtI { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::LtI);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::LtF { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::LtF);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::LtU { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::LtU);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::LeI { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::LeI);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::LeF { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::LeF);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
                ir::IR::LeU { dst, lhs, rhs } => {
                    bytes.emit_bytecode(Bytecode::LeU);
                    bytes.emit_reg(*dst);
                    bytes.emit_reg(*lhs);
                    bytes.emit_reg(*rhs);
                },
            }
        }


        endings.push((block.terminator(), bytes.offset()));
        match block.terminator() {
            ir::Terminator::Ret => bytes.emit_bytecode(Bytecode::Ret),
            ir::Terminator::Jmp(_) => {
                bytes.emit_bytecode(Bytecode::Jmp);
                bytes.reserve(size_of::<u32>());
            },

            ir::Terminator::Jif { cond, .. } => {
                bytes.emit_bytecode(Bytecode::Jif);
                bytes.emit_reg(*cond);
                bytes.reserve(size_of::<u32>() * 2);
            },
            ir::Terminator::Match { src, jumps } => {
                bytes.emit_bytecode(Bytecode::Match);
                bytes.emit_reg(*src);
                bytes.reserve(size_of::<u32>() * jumps.len());
            },
        };
    }
    
    {
        let size = bytes.offset()-starting_pos;
        let size = size as u32;
        let size = size.to_le_bytes();
        bytes.0[size_offset] = size[0];
        bytes.0[size_offset + 1] = size[1];
        bytes.0[size_offset + 2] = size[2];
        bytes.0[size_offset + 3] = size[3];
    }
    bytes
}


#[derive(Debug)]
pub struct BytecodeVec(Vec<u8>);

impl BytecodeVec {
    pub fn with_capacity(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }


    #[inline(always)]
    pub fn emit_bytecode(&mut self, bytecode: Bytecode) {
        self.0.push(bytecode as u8)
    }


    #[inline(always)]
    pub fn emit_reg(&mut self, byte: Reg) {
        self.0.push(byte.0.try_into().unwrap())
    }


    #[inline(always)]
    pub fn emit_type_id(&mut self, byte: TypeId) {
        self.emit_u32(byte.0)
    }


    #[inline(always)]
    pub fn emit_byte(&mut self, byte: u8) {
        self.0.push(byte)
    }


    #[inline(always)]
    pub fn emit_u16(&mut self, byte: u16) {
        self.0.extend_from_slice(&byte.to_le_bytes());
    }


    #[inline(always)]
    pub fn emit_u32(&mut self, byte: u32) {
        self.0.extend_from_slice(&byte.to_le_bytes());
    }


    #[inline(always)]
    pub fn emit_str(&mut self, byte: &str) {
        self.emit_u32(byte.len().try_into().unwrap());
        self.0.extend_from_slice(byte.as_bytes());
    }


    #[inline(always)]
    pub fn reserve(&mut self, amount: usize) {
        self.0.reserve(amount);
        for _ in 0..amount {
            self.0.push(u8::MAX);
        }
    }


    #[inline(always)]
    pub fn offset(&self) -> usize {
        self.0.len()
    }
}