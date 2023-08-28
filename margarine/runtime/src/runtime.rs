#![allow(non_upper_case_globals)]
use std::mem::{size_of, ManuallyDrop};
use std::sync::Arc;

use crate::{VM, Code, ProgramCounter, FunctionDebugInfo, FunctionArgument, StructureKind, Structure, FunctionIndex, Data, TypeId, Object, DataUnion, DataMetadata, Stackframe};
use crate::bytecode::bytecode_consts::*;

impl VM<'_> {
    pub fn run(&mut self) {
        
    }


    pub fn register_declarations(&mut self, code: Code) {
        self.current = ProgramCounter::new(code);
        
        loop {
            match self.current.next() {
                Func => {
                    dbg!(&self);
                    // PERFORMANCE: Might wanna move these to another
                    // part instead of the bytecode for cache reasons
                    let name = self.current.next_str();
                    let is_system = self.current.next() == 1;
                    let size = self.current.next_u32();
                    let return_type = self.current.next_type();
                    let args_len = self.current.next_u16() as usize;
                    let mut args = Vec::with_capacity(args_len);

                    for _ in 0..args_len {
                        let name = self.current.next_str();
                        let is_inout = self.current.next() == 1;
                        let type_id = self.current.next_type();

                        args.push(FunctionArgument {
                            name,
                            data_type: type_id,
                            is_inout,
                        })
                    }


                    let code = Code::new(self.current.counter, #[cfg(debug_assertions)] unsafe { self.current.counter.add(size as usize) });

                    self.current.skip(size as usize);

                    self.functions.push(code);
                    self.functions_debug_info.push(FunctionDebugInfo { name, return_type, arguments: args, is_system });
                    self.name_to_func.insert(name, FunctionIndex(self.functions.len()-1));
                }


                Struct => {
                    // PERFORMANCE: Might wanna move these to another
                    // part instead of the bytecode for cache reasons
                    let name = self.current.next_str();
                    let type_id = self.current.next_type();
                    let kind = self.current.next();
                    let kind = match kind {
                        0 => StructureKind::Normal,
                        1 => StructureKind::Resource,
                        2 => StructureKind::Component,
                        _ => unreachable!()
                    };
                    

                    let fields_len = self.current.next_u16();
                    let mut fields = Vec::with_capacity(fields_len as usize);
                    for _ in 0..fields_len {
                        let name = self.current.next_str();
                        let type_id = self.current.next_type();

                        fields.push((name, type_id));
                    }

                    self.structures.insert(type_id, Structure {
                        name,
                        type_id,
                        fields,
                        kind,
                    });

                    self.name_to_typeid.insert(name, type_id);
                }


                // There should only be 1 return
                // in the top level. Which is to 
                // indicate end of file
                Ret => {
                    // We sub(1) cos the counter will be pointing
                    // at the next spot which will be one byte ahead
                    #[cfg(debug_assertions)]
                    assert_eq!(unsafe { self.current.counter.sub(1) }, self.current.code.end,
                            "ret isn't at the end of the code block");

                    return
                },
                
                _ => unreachable!(),
            }
        }
    }


    pub fn run_bytecode(&mut self, code: Code) {
        macro_rules! arithmetic {
            ($kind: ident, $new: ident, $tt: tt) => {{
                let dst = self.current.next();
                let lhs = self.current.next();
                let rhs = self.current.next();


                let lhs = self.stack.reg(lhs).$kind();
                let rhs = self.stack.reg(rhs).$kind();

                let result = Data::$new(lhs $tt rhs);
                self.stack.set_reg(dst, result);
            }}
        }

        
        assert!(self.callstack.is_empty());
        self.current = ProgramCounter::new(code);
        loop {
            let value = self.current.next();
            match value {
                Ret => {
                    let Some(current) = self.callstack.pop()
                    else { return };
                    let ret_val = self.stack.take_reg(0);

                    self.stack.bottom = current.stack_bottom;
                    self.stack.set_reg(current.dst, ret_val);
                    self.current = current.pc;
                }

                
                Copy => {
                    let dst = self.current.next();
                    let src = self.current.next();

                    let src = self.stack.reg(src).clone();
                    self.stack.set_reg(dst, src);
                },


                Lit => {
                    let dst = self.current.next();
                    let val = self.current.next_u32();

                    let val = self.constants[val as usize].clone();
                    self.stack.set_reg(dst, val)
                },


                Push => {
                    let amount = self.current.next();
                    self.stack.push(amount as usize);
                },


                Pop => {
                    let amount = self.current.next();
                    self.stack.pop(amount as usize);
                },


                Jmp => {
                    let dst = self.current.next_u32();
                    self.current.jump(dst as usize);
                }


                Jif => {
                    let cond = self.current.next();
                    let cond = self.stack.reg(cond).as_bool();
                    let true_dst = self.current.next_u32();
                    let false_dst = self.current.next_u32();

                    if cond {
                        self.current.jump(true_dst as usize);
                    } else {
                        self.current.jump(false_dst as usize);
                    }
                }


                Match => {
                    let val = self.current.next();
                    let val = self.stack.reg(val);
                    let variant = val.metadata.variant();

                    self.current.skip(variant as usize * size_of::<u32>());

                    let dst = self.current.next_u32();
                    self.current.jump(dst as usize);
                },
                

                CastAny => {
                    let dst = self.current.next();
                    let src = self.current.next();
                    let target = self.current.next_type();

                    let src = self.stack.reg(src);
                    let src_type = src.type_id;
                    if src_type != target {
                        self.stack.set_reg(dst, Data::new(
                            DataUnion { obj: ManuallyDrop::new(Object::String(
                                format!(
                                    "any can only cast between the same types. tried to cast a {} into {}",
                                    self.get_type(src_type).unwrap().name,
                                    self.get_type(target).unwrap().name,
                                ).into())) },

                            crate::DataMetadata(1),
                            TypeId::CAST_ERROR
                        ))
                    } else {
                        let mut val = src.clone();
                        val.metadata = DataMetadata(0);
                        self.stack.set_reg(dst, val);
                    }
                },


                CreateStruct => {
                    let dst = self.current.next();
                    let type_id = self.current.next_type();
                    let fields_len = self.current.next();
                    let mut vec = Vec::with_capacity(fields_len as usize);

                    for _ in 0..fields_len {
                        let reg = self.current.next();
                        vec.push(self.stack.reg(reg).clone());
                    }

                    self.stack.set_reg(
                        dst, 
                        Data::new_obj(
                            Object::Structure(vec.into()), 
                            type_id
                        )
                    );
                },


                AccField => {
                    let dst = self.current.next();
                    let src = self.current.next();
                    let index = self.current.next();

                    let val = self.stack.reg(src)
                        .as_obj()
                        .as_struct()[index as usize]
                        .clone();
                    self.stack.set_reg(dst, val);
                },


                SetField => {
                    let dst = self.current.next();
                    let src = self.current.next();
                    let index = self.current.next();

                    let src = self.stack.reg(src).clone();

                    let val = self.stack.reg_mut(dst);
                    let val = val.as_obj_mut().as_struct_mut();

                    val[index as usize] = src;
                },


                Call => {
                    let dst = self.current.next();
                    let func = self.current.next_u32();
                    let args_len = self.current.next() as usize;

                    self.stack.push(args_len + 1);

                    for i in (0..args_len).rev() {
                        let index = self.current.next();
                        let value = self.stack.reg(index).clone();
                        self.stack.set_reg((self.stack.top - i) as u8, value);
                    }

                    let stackframe = Stackframe {
                        pc: self.current.clone(),
                        stack_bottom: self.stack.bottom,
                        dst,
                    };

                    self.stack.bottom = self.stack.top - args_len - 1;
                    self.callstack.push(stackframe);
                    self.current = ProgramCounter::new(self.functions[func as usize]);
                },
                

                Not => {
                    let dst = self.current.next();
                    let src = self.current.next();

                    let src = self.stack.reg(src).as_bool();
                    self.stack.set_reg(dst, Data::new_bool(!src));
                },


                NegI => {
                    let dst = self.current.next();
                    let src = self.current.next();

                    let src = self.stack.reg(src).as_int();
                    self.stack.set_reg(dst, Data::new_int(-src));
                }


                NegF => {
                    let dst = self.current.next();
                    let src = self.current.next();

                    let src = self.stack.reg(src).as_float();
                    self.stack.set_reg(dst, Data::new_float(-src));
                },


                AddI => arithmetic!(as_int  , new_int  , +),
                AddF => arithmetic!(as_float, new_float, +),
                AddU => arithmetic!(as_uint , new_uint , +),

                SubI => arithmetic!(as_int  , new_int  , -),
                SubF => arithmetic!(as_float, new_float, -),
                SubU => arithmetic!(as_uint , new_uint , -),

                MulI => arithmetic!(as_int  , new_int  , *),
                MulF => arithmetic!(as_float, new_float, *),
                MulU => arithmetic!(as_uint , new_uint , *),

                DivI => arithmetic!(as_int  , new_int  , /),
                DivF => arithmetic!(as_float, new_float, /),
                DivU => arithmetic!(as_uint , new_uint , /),

                RemI => arithmetic!(as_int  , new_int  , %),
                RemF => arithmetic!(as_float, new_float, %),
                RemU => arithmetic!(as_uint , new_uint , %),

                
                LeftShiftI => arithmetic!(as_int , new_int , <<),
                LeftShiftU => arithmetic!(as_uint, new_uint, <<),

                RightShiftI => arithmetic!(as_int , new_int , >>),
                RightShiftU => arithmetic!(as_uint, new_uint, >>),

                BitwiseAndI => arithmetic!(as_int , new_int , &),
                BitwiseAndU => arithmetic!(as_uint, new_uint, &),

                BitwiseOrI => arithmetic!(as_int , new_int , |),
                BitwiseOrU => arithmetic!(as_uint, new_uint, |),

                BitwiseXorI => arithmetic!(as_int , new_int , ^),
                BitwiseXorU => arithmetic!(as_uint, new_uint, ^),


                EqI => arithmetic!(as_int  , new_bool, ==),
                EqF => arithmetic!(as_float, new_bool, ==),
                EqU => arithmetic!(as_uint , new_bool, ==),
                EqB => arithmetic!(as_bool , new_bool, ==),

                NeI => arithmetic!(as_int  , new_bool, !=),
                NeF => arithmetic!(as_float, new_bool, !=),
                NeU => arithmetic!(as_uint , new_bool, !=),
                NeB => arithmetic!(as_bool , new_bool, !=),

                GtI => arithmetic!(as_int  , new_bool, >),
                GtF => arithmetic!(as_float, new_bool, >),
                GtU => arithmetic!(as_uint , new_bool, >),

                GeI => arithmetic!(as_int  , new_bool, >=),
                GeF => arithmetic!(as_float, new_bool, >=),
                GeU => arithmetic!(as_uint , new_bool, >=),

                LtI => arithmetic!(as_int  , new_bool, <),
                LtF => arithmetic!(as_float, new_bool, <),
                LtU => arithmetic!(as_uint , new_bool, <),

                LeI => arithmetic!(as_int  , new_bool, <=),
                LeF => arithmetic!(as_float, new_bool, <=),
                LeU => arithmetic!(as_uint , new_bool, <=),

                
                _ => unreachable!(),
            }
        }
    }
}
