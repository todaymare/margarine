use std::hint::select_unpredictable;

use crate::{jit::{self, attempt_jit}, opcode::runtime::consts, CallFrame, FatalError, Object, Reader, Reg, Status, VM};

impl<'src> VM<'src> {
    pub fn run(&mut self, func: &str) -> Status {
        let Some((index, _)) = &self.funcs.iter().enumerate().find(|x| x.1.name == func)
        else { return Status::err(FatalError::new(format!("invalid entry function '{func}'"))) };

        self.run_func(*index)
    }

    pub extern "C" fn run_func(&mut self, index: usize) -> Status {
        //dbg!(jit::attempt_jit(self, index));

        let func = &self.funcs[index];
        match func.kind {
            crate::FunctionKind::Code { byte_offset, byte_size } => {
                assert!(self.callstack.stack.is_empty());
                self.curr = CallFrame::new(
                    &self.callstack.src[byte_offset..byte_offset + byte_size],
                    0,
                    0,
                );

            },


            crate::FunctionKind::Host(f) => {
                let mut reg = Reg::new_unit();
                unsafe { f(self, &mut reg) };
                return Status::ok()
            },
        };


        unsafe {
        loop {
            let opcode = self.curr.next();
            println!("{:?}", crate::opcode::runtime::OpCode::from_u8(opcode));
            //println!("{:?}", self.stack);
            
            match opcode {
                consts::PushLocalSpace => {
                    let amount = self.curr.next();
                    self.stack.curr += amount as usize;
                }


                consts::Ret => {
                    let local_count = self.curr.next();

                    let Some(prev_frame) = self.callstack.pop()
                    else { break };

                    let return_val = self.stack.pop();
                    self.stack.curr -= self.curr.argc as usize + local_count as usize;
                    self.stack.set_bottom(self.curr.previous_offset);
                    self.stack.push(return_val);

                    self.curr = prev_frame;
                },


                consts::Call => {
                    let func = self.curr.next_u32();
                    let argc = self.curr.next();
                    
                    // 
                    // prepare the call frame
                    // the arguments should already be ordered at the top of the stack
                    //

                    //dbg!(attempt_jit(self, func as _));

                    let func = &self.funcs[func as usize];

                    match func.kind {
                        crate::FunctionKind::Code { byte_offset, byte_size } => {
                            let mut call_frame = CallFrame::new(
                                &self.callstack.src[byte_offset..byte_offset+byte_size],
                                self.stack.bottom,
                                argc,
                            );

                            self.stack.set_bottom(self.stack.curr - argc as usize);

                            core::mem::swap(&mut self.curr, &mut call_frame);

                            self.callstack.push(call_frame);
                        },


                        crate::FunctionKind::Host(f) => {
                            let bottom = self.stack.bottom;
                            self.stack.bottom = self.stack.curr - argc as usize;

                            let start = self.stack.curr;
                            let mut ret = Reg::new_unit();
                            f(self, &mut ret);
                            debug_assert_eq!(self.stack.curr, start);

                            self.stack.curr -= argc as usize;
                            self.stack.set_bottom(bottom);
                            self.stack.push(ret);
                        },
                    }
                },


                consts::CallFuncRef => {
                    let func_ref = self.stack.pop().as_obj();
                    let Object::FuncRef { func, captures } = &self.objs[func_ref as usize]
                    else { unreachable!() };

                    let argc = self.curr.next();
                    
                    // 
                    // prepare the call frame
                    // the arguments should already be ordered at the top of the stack
                    //

                    let func = &self.funcs[*func as usize];

                    match func.kind {
                        crate::FunctionKind::Code { byte_offset, byte_size } => {
                            for capture in captures {
                                self.stack.push(*capture);
                            }

                            let argc = argc + captures.len() as u8;

                            let mut call_frame = CallFrame::new(
                                &self.callstack.src[byte_offset..byte_offset+byte_size],
                                self.stack.bottom,
                                argc,
                            );

                            self.stack.set_bottom(self.stack.curr - argc as usize);

                            core::mem::swap(&mut self.curr, &mut call_frame);

                            self.callstack.push(call_frame);
                        },


                        crate::FunctionKind::Host(f) => {
                            assert_eq!(captures.len(), 0);

                            let bottom = self.stack.bottom;
                            self.stack.bottom = self.stack.curr - argc as usize;

                            let start = self.stack.curr;
                            let mut ret = Reg::new_unit();
                            f(self, &mut ret);
                            debug_assert_eq!(self.stack.curr, start);

                            self.stack.curr -= argc as usize;
                            self.stack.set_bottom(bottom);
                            self.stack.push(ret);
                        },
                    }
                },


                consts::Err => {
                    let ty = self.curr.next();
                    let file = self.curr.next_u32();
                    let index = self.curr.next_u32();

                    if ty == 3 {
                        panic!("a bypass error was reached. uh oh");
                    }

                    let mut reader = Reader::new(self.error_table);
                    for _ in 0..ty {
                        let file_count = reader.next_u32();
                        for _ in 0..file_count {
                            let err_count = reader.next_u32();
                            for _ in 0..err_count {
                                reader.next_str();
                            }
                        }
                    }

                    let file_count = reader.next_u32();
                    assert!(file <= file_count);
                    for _ in 0..file {
                        let err_count = reader.next_u32();

                        for _ in 0..err_count {
                            reader.next_str();
                        }
                    }

                    let err_count = reader.next_u32();
                    assert!(index < err_count);
                    for _ in 0..index {
                        reader.next_str();
                    }

                    let str = reader.next_str();

                    return Status::err(FatalError::new(format!("{str}")))
                },


                consts::CreateFuncRef => {
                    let capture_count = self.curr.next();
                    let func = self.stack.pop().as_int() as u32;

                    let mut vec = Vec::with_capacity(capture_count as usize);
                    for _ in 0..capture_count {
                        vec.push(self.stack.pop());
                    }

                    vec.reverse();

                    let obj = Object::FuncRef {
                        func,
                        captures: vec
                    };

                    let obj = self.new_obj(obj);
                    self.stack.push(obj);
                }


                consts::CreateStruct => {
                    let field_count = self.curr.next();
                    let mut vec = Vec::with_capacity(field_count as usize);

                    for _ in 0..field_count {
                        vec.push(self.stack.pop());
                    }

                    vec.reverse();

                    let obj = Object::Struct {
                        fields: vec,
                    };

                    let reg = self.new_obj(obj);
                    self.stack.push(reg);
                }


                consts::CreateList => {
                    let field_count = self.curr.next_u32();
                    let mut vec = Vec::with_capacity(field_count as usize);

                    for _ in 0..field_count {
                        vec.push(self.stack.pop());
                    }

                    vec.reverse();

                    let obj = Object::List(vec);

                    let reg = self.new_obj(obj);
                    self.stack.push(reg);
                }


                consts::IndexList => {
                    let index = self.stack.pop().as_int();
                    let list = self.stack.pop();

                    let list = self.objs[list.as_obj() as usize].as_list();
                    if index < 0 || index as usize >= list.len() {
                        return Status::err(FatalError::new(String::from("out of bounds access")))
                    }

                    self.stack.push(list[index as usize]);
                }


                consts::StoreList => {
                    let index = self.stack.pop().as_int();
                    let list = self.stack.pop();
                    let value = self.stack.pop();

                    let list = self.objs[list.as_obj() as usize].as_mut_list();
                    if index < 0 || index as usize >= list.len() {
                        return Status::err(FatalError::new(String::from("out of bounds access")))
                    }

                    list[index as usize] = value
                }


                consts::LoadField => {
                    let index = self.curr.next();
                    let val = self.stack.pop();
                    let obj_index = val.as_obj();
                    let obj = &self.objs[obj_index as usize];
                    self.stack.push(obj.as_fields()[index as usize])
                }


                consts::StoreField => {
                    let index = self.curr.next();
                    let target = self.stack.pop();
                    let val = self.stack.pop();
                    let obj_index = target.as_obj();
                    let obj = &mut self.objs[obj_index as usize];
                    obj.as_mut_fields()[index as usize] = val;
                }


                consts::CastIntToFloat => {
                    let v = self.stack.pop().as_int();
                    self.stack.push(Reg::new_float(v as f64));
                }


                consts::CastFloatToInt => {
                    let v = self.stack.pop().as_float();
                    self.stack.push(Reg::new_int(v as i64));
                }


                consts::CastBoolToInt => {
                    let v = self.stack.pop().as_bool();
                    self.stack.push(Reg::new_int(v as i64));
                }


                consts::UnwrapStore => {
                    let var = self.stack.pop();
                    let val = self.stack.pop();
                    
                    let obj = &mut self.objs[var.as_obj() as usize];

                    if obj.as_fields()[0].as_int() == 1 {
                        return Status::err(FatalError::new("tried to unwrap an invalid value".to_string()));
                    }

                    obj.as_mut_fields()[1] = val;
                }


                consts::Unwrap => {
                    let val = self.stack.pop();
                    let obj_index = val.as_obj();
                    let obj = &self.objs[obj_index as usize];
                    if obj.as_fields()[0].as_int() == 1 {
                        return Status::err(FatalError::new("tried to unwrap an invalid value".to_string()));
                    }

                    self.stack.push(obj.as_fields()[1])
                }


                consts::UnwrapFail => {
                    return Status::err(FatalError::new("tried to unwrap an invalid value".to_string()));
                }


                consts::LoadEnumField => {
                    let index = self.curr.next_u32();
                    let val = self.stack.pop();
                    let obj_index = val.as_obj();
                    let obj = &self.objs[obj_index as usize];

                    let tag = obj.as_fields()[0].as_int();
                    let val = if tag as u32 == index {
                        self.new_obj(Object::Struct {
                            fields: vec![Reg::new_int(0), obj.as_fields()[1]],
                        })
                    } else {
                        self.new_obj(Object::Struct {
                            fields: vec![Reg::new_int(1), Reg::new_unit()],
                        })
                    };


                    self.stack.push(val);
                }


                consts::Copy => {
                    let val = self.stack.read();
                    self.stack.push(val);
                }


                consts::Unit => {
                    self.stack.push(Reg::new_unit());
                }


                consts::ConstInt => {
                    self.stack.push(Reg::new_int(self.curr.next_i64()));
                }


                consts::ConstFloat => {
                    self.stack.push(Reg::new_float(self.curr.next_f64()));
                }


                consts::ConstBool => {
                    self.stack.push(Reg::new_bool(self.curr.next() == 1));
                }


                consts::ConstStr => {
                    let index = self.curr.next_u32();
                    self.stack.push(Reg::new_obj(index as u64));
                }


                consts::NegInt => {
                    let val = self.stack.pop().as_int();
                    self.stack.push(Reg::new_int(-val));
                }


                consts::AddInt => {
                    let rhs = self.stack.pop();
                    let lhs = self.stack.pop();
                    self.stack.push(Reg::new_int(lhs.as_int() + rhs.as_int()));
                }


                consts::SubInt => {
                    let rhs = self.stack.pop().as_int();
                    let lhs = self.stack.pop().as_int();
                    self.stack.push(Reg::new_int(lhs - rhs));
                }


                consts::MulInt => {
                    let rhs = self.stack.pop().as_int();
                    let lhs = self.stack.pop().as_int();
                    self.stack.push(Reg::new_int(lhs * rhs));
                }


                consts::DivInt => {
                    let rhs = self.stack.pop().as_int();
                    let lhs = self.stack.pop().as_int();
                    self.stack.push(Reg::new_int(lhs / rhs));
                }


                consts::RemInt => {
                    let rhs = self.stack.pop().as_int();
                    let lhs = self.stack.pop().as_int();
                    self.stack.push(Reg::new_int(lhs % rhs));
                }


                consts::EqInt => {
                    let rhs = self.stack.pop().as_int();
                    let lhs = self.stack.pop().as_int();
                    self.stack.push(Reg::new_bool(lhs == rhs));
                }


                consts::NeInt => {
                    let rhs = self.stack.pop().as_int();
                    let lhs = self.stack.pop().as_int();
                    self.stack.push(Reg::new_bool(lhs != rhs));
                }


                consts::GtInt => {
                    let rhs = self.stack.pop().as_int();
                    let lhs = self.stack.pop().as_int();
                    self.stack.push(Reg::new_bool(lhs > rhs));
                }


                consts::GeInt => {
                    let rhs = self.stack.pop().as_int();
                    let lhs = self.stack.pop().as_int();
                    self.stack.push(Reg::new_bool(lhs >= rhs));
                }


                consts::LtInt => {
                    let rhs = self.stack.pop().as_int();
                    let lhs = self.stack.pop().as_int();
                    self.stack.push(Reg::new_bool(lhs < rhs));
                }


                consts::LeInt => {
                    let rhs = self.stack.pop().as_int();
                    let lhs = self.stack.pop().as_int();
                    self.stack.push(Reg::new_bool(lhs <= rhs));
                }


                consts::BitwiseOr => {
                    let rhs = self.stack.pop().as_int();
                    let lhs = self.stack.pop().as_int();
                    self.stack.push(Reg::new_int(lhs | rhs));
                }


                consts::BitwiseAnd => {
                    let rhs = self.stack.pop().as_int();
                    let lhs = self.stack.pop().as_int();
                    self.stack.push(Reg::new_int(lhs & rhs));
                }


                consts::BitwiseXor => {
                    let rhs = self.stack.pop().as_int();
                    let lhs = self.stack.pop().as_int();
                    self.stack.push(Reg::new_int(lhs ^ rhs));
                }


                consts::BitshiftLeft => {
                    let rhs = self.stack.pop().as_int();
                    let lhs = self.stack.pop().as_int();
                    self.stack.push(Reg::new_int(lhs << rhs));
                }


                consts::BitshiftRight => {
                    let rhs = self.stack.pop().as_int();
                    let lhs = self.stack.pop().as_int();
                    self.stack.push(Reg::new_int(lhs >> rhs));
                }


                consts::NegFloat => {
                    let val = self.stack.pop().as_float();
                    self.stack.push(Reg::new_float(-val));
                }

                consts::AddFloat => {
                    let rhs = self.stack.pop().as_float();
                    let lhs = self.stack.pop().as_float();
                    self.stack.push(Reg::new_float(lhs + rhs));
                }


                consts::SubFloat => {
                    let rhs = self.stack.pop().as_float();
                    let lhs = self.stack.pop().as_float();
                    self.stack.push(Reg::new_float(lhs - rhs));
                }


                consts::MulFloat => {
                    let rhs = self.stack.pop().as_float();
                    let lhs = self.stack.pop().as_float();
                    self.stack.push(Reg::new_float(lhs * rhs));
                }


                consts::DivFloat => {
                    let rhs = self.stack.pop().as_float();
                    let lhs = self.stack.pop().as_float();
                    self.stack.push(Reg::new_float(lhs / rhs));
                }


                consts::RemFloat => {
                    let rhs = self.stack.pop().as_float();
                    let lhs = self.stack.pop().as_float();
                    self.stack.push(Reg::new_float(lhs % rhs));
                }


                consts::EqFloat => {
                    let rhs = self.stack.pop().as_float();
                    let lhs = self.stack.pop().as_float();
                    self.stack.push(Reg::new_bool(lhs == rhs));
                }


                consts::NeFloat => {
                    let rhs = self.stack.pop().as_float();
                    let lhs = self.stack.pop().as_float();
                    self.stack.push(Reg::new_bool(lhs != rhs));
                }


                consts::GtFloat => {
                    let rhs = self.stack.pop().as_float();
                    let lhs = self.stack.pop().as_float();
                    self.stack.push(Reg::new_bool(lhs > rhs));
                }


                consts::GeFloat => {
                    let rhs = self.stack.pop().as_float();
                    let lhs = self.stack.pop().as_float();
                    self.stack.push(Reg::new_bool(lhs >= rhs));
                }


                consts::LtFloat => {
                    let rhs = self.stack.pop().as_float();
                    let lhs = self.stack.pop().as_float();
                    self.stack.push(Reg::new_bool(lhs < rhs));
                }


                consts::LeFloat => {
                    let rhs = self.stack.pop().as_float();
                    let lhs = self.stack.pop().as_float();
                    self.stack.push(Reg::new_bool(lhs <= rhs));
                }


                consts::EqBool => {
                    let rhs = self.stack.pop().as_bool();
                    let lhs = self.stack.pop().as_bool();
                    self.stack.push(Reg::new_bool(lhs == rhs));
                }


                consts::NeBool => {
                    let rhs = self.stack.pop().as_bool();
                    let lhs = self.stack.pop().as_bool();
                    self.stack.push(Reg::new_bool(lhs != rhs));
                }


                consts::NotBool => {
                    let rhs = self.stack.pop().as_bool();
                    self.stack.push(Reg::new_bool(!rhs));
                }


                consts::Load => {
                    let reg = self.curr.next();
                    let val = self.stack.reg(reg);
                    self.stack.push(val);
                }


                consts::Store => {
                    let reg = self.curr.next();
                    let data = self.stack.pop();
                    self.stack.set_reg(reg, data);
                }


                consts::Pop => {
                    self.stack.pop();
                }


                consts::Jump => {
                    let offset = self.curr.next_i32();

                    self.curr.offset(offset);
                }


                consts::SwitchOn => {
                    let t = self.curr.next_i32();
                    let f = self.curr.next_i32();

                    let val = self.stack.pop().as_bool();
                    let offset = select_unpredictable(val, t, f);
                    self.curr.offset(offset);
                }


                consts::Switch => {
                    let index = self.stack.pop().as_int() as u32;
                    let byte_size = self.curr.next_u32();
                    let count = byte_size / 4;
                    debug_assert!(index < count);

                    self.curr.next_slice(index as usize * 4);

                    let offset = self.curr.next_i32();
                    //self.curr.next_slice((count - index) as usize * 4);

                    self.curr.offset(offset as i32);
                }


                _ => panic!("unimplemented opcode '{opcode}; {:?}", crate::opcode::runtime::OpCode::from_u8(opcode)),
            }
        }
        }


        //debug_assert_eq!(self.stack.top, 1);
        debug_assert!(self.stack.bottom == 0);


        Status::ok()
    }
}

