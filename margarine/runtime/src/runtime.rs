use std::{hint::select_unpredictable, ops::Deref};

use crate::{obj_map::{ObjectData, ObjectIndex}, opcode::runtime::consts, CallFrame, FatalError, Object, Reader, Reg, Status, VM};

impl<'src> VM<'src> {
    pub fn run(&mut self, func: &str, args: &[Reg]) -> Status {
        let Some((index, _)) = &self.funcs.iter().enumerate().find(|x| x.1.name == func)
        else { return Status::err(FatalError::new(&format!("invalid entry function '{func}'"))) };

        for &arg in args {
            self.stack.push(arg);
        }

        self.cycle = 0;
        let now = std::time::Instant::now();
        let result = self.run_func(*index);
        println!("ran {} cycles in {:?}. {} MIPS", self.cycle, now.elapsed(), (self.cycle as f64) / (now.elapsed().as_secs_f64() * 1_000_000.0));
        result
    }

    pub extern "C" fn run_func(&mut self, index: usize) -> Status {

        let func = &self.funcs[index];

        let callframe = match func.kind {
            crate::FunctionKind::Code { byte_offset, byte_size } => {
                let cf = CallFrame::new(
                    &self.callstack.src[byte_offset..byte_offset + byte_size],
                    self.stack.bottom,
                    self.stack.curr - func.argc as usize,
                    func.argc,
                    index as _,
                );

                unsafe {
                    self.stack.set_bottom(self.stack.curr - func.argc as usize);
                }

                cf
            },


            crate::FunctionKind::Host(f) => {
                let mut reg = Reg::new_unit();
                let mut status = Status::ok();
                unsafe { f(self, &mut reg, &mut status) };
                return status
            },
        };


        let prev = std::mem::replace(&mut self.curr, callframe);
        let bottom = self.callstack.stack.len();

        unsafe {
        loop {
            //println!(" - {:?} ", &(*self.stack.values)[..self.stack.curr]);
            //let decode = crate::opcode::runtime::OpCode::decode(&mut self.curr.clone()).unwrap();
            let opcode = self.curr.next();
            //println!("{:?} - {}", decode.1, self.stack.curr);
            self.cycle += 1;


            if self.cycle % 1000000 == 0 { self.run_garbage_collection(); }
            //println!("{:?}", self.stack);
            
            match opcode {
                consts::PushLocalSpace => {
                    let amount = self.curr.next();
                    self.stack.curr += amount as usize;
                }


                consts::Ret => {
                    let local_count = self.curr.next();

                    let mut ret_value = Reg::new_unit();
                    ret_instr(self, local_count as _, &mut ret_value);

                    self.stack.set_bottom(self.curr.previous_offset);

                    self.stack.curr -= self.curr.argc as usize;

                    assert_eq!(self.stack.curr, self.curr.previous_top);

                    self.stack.push(ret_value);


                    if self.callstack.stack.len() == bottom { 
                        break; 
                    }

                    let Some(prev_frame) = self.callstack.pop()
                    else { break; };

                    self.curr = prev_frame; 

                },


                consts::CallFuncRef => {
                    let reg = self.stack.pop();
                    let (func_index, captures) =
                    if reg.is_nocap() { (reg.as_nocap(), [].as_slice()) }
                    else {
                        let func_ref = reg.as_obj();
                        let ObjectData::FuncRef { func: func_index, captures } = &self.objs[func_ref].data
                        else { unreachable!() };
                        (*func_index, captures.as_slice())
                    };

                    //println!("calling {}", self.funcs[func_index as usize].name);

                    let argc = self.curr.next();
                    
                    // 
                    // prepare the call frame
                    // the arguments should already be ordered at the top of the stack
                    //


                    //dbga(jitty::attempt_jit(self, *func_index as _));

                    let func = &self.funcs[func_index as usize];

                    match func.kind {
                        crate::FunctionKind::Code { byte_offset, byte_size } => {
                            for capture in captures {
                                self.stack.push(*capture);
                            }

                            let argc = argc + captures.len() as u8;

                            if let Some(cache) = &func.cache {
                                let curr = self.stack.curr;
                                let args : &[Reg] = &self.stack.values.deref()[curr-argc as usize..curr];
                                if let Some(result) = cache.get(&args) {
                                    self.stack.curr -= argc as usize;
                                    self.stack.push(*result);
                                    continue;
                                }
                            }

                            let mut call_frame = CallFrame::new(
                                &self.callstack.src[byte_offset..byte_offset+byte_size],
                                self.stack.bottom,
                                self.stack.curr - argc as usize,
                                argc,
                                func_index,
                            );

                            self.stack.set_bottom(self.stack.curr - argc as usize);

                            core::mem::swap(&mut self.curr, &mut call_frame);

                            self.callstack.push(call_frame);
                        },


                        crate::FunctionKind::Host(f) => {
                            debug_assert_eq!(captures.len(), 0);

                            let bottom = self.stack.bottom;
                            self.stack.bottom = self.stack.curr - argc as usize;


                            let start = self.stack.curr;
                            let mut ret = Reg::new_unit();
                            let mut status = Status::ok();

                            f(self, &mut ret, &mut status);

                            if status.as_err().is_some() {
                                return status;
                            }

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
                    debug_assert!(file <= file_count);
                    for _ in 0..file {
                        let err_count = reader.next_u32();

                        for _ in 0..err_count {
                            reader.next_str();
                        }
                    }

                    let err_count = reader.next_u32();
                    debug_assert!(index < err_count);
                    for _ in 0..index {
                        reader.next_str();
                    }

                    let str = reader.next_str();

                    return Status::err(FatalError::new(str))
                },


                consts::CreateFuncRef => {
                    let capture_count = self.curr.next();
                    let func = self.stack.pop().as_int() as u32;

                    if capture_count == 0 {
                        self.stack.push(Reg::new_nocap(func));
                        continue;
                    }

                    let mut vec = Vec::with_capacity(capture_count as usize);
                    for _ in 0..capture_count {
                        vec.push(self.stack.pop());
                    }

                    vec.reverse();

                    let obj = ObjectData::FuncRef {
                        func,
                        captures: vec
                    };

                    let obj = self.new_obj(obj)?;
                    self.stack.push(obj);
                }


                consts::CreateStruct => {
                    let field_count = self.curr.next();
                    let mut vec = Vec::with_capacity(field_count as usize);

                    for _ in 0..field_count {
                        vec.push(self.stack.pop());
                    }

                    vec.reverse();

                    let obj = ObjectData::Struct {
                        fields: vec,
                    };

                    let reg = self.new_obj(obj)?;
                    self.stack.push(reg);
                }


                consts::CreateList => {
                    let field_count = self.curr.next_u32();
                    let mut vec = Vec::with_capacity(field_count as usize);

                    for _ in 0..field_count {
                        vec.push(self.stack.pop());
                    }

                    vec.reverse();

                    let obj = ObjectData::List(vec);

                    let reg = self.new_obj(obj)?;
                    self.stack.push(reg);
                }


                consts::IndexList => {
                    let index = self.stack.pop().as_int();
                    let list = self.stack.pop();

                    let list = self.objs[list.as_obj()].as_list();
                    if index < 0 || index as usize >= list.len() {
                        return Status::err(FatalError::new("out of bounds access"))
                    }

                    self.stack.push(list[index as usize]);
                }


                consts::StoreList => {
                    let index = self.stack.pop().as_int();
                    let list = self.stack.pop();
                    let value = self.stack.pop();

                    let list = self.objs[list.as_obj()].as_mut_list();
                    if index < 0 || index as usize >= list.len() {
                        return Status::err(FatalError::new("out of bounds access"))
                    }

                    list[index as usize] = value;

                }


                consts::LoadField => {
                    let index = self.curr.next();
                    let val = self.stack.pop();
                    let obj_index = val.as_obj();
                    let obj = &self.objs[obj_index];

                    self.stack.push(obj.as_fields()[index as usize])
                }


                consts::StoreField => {
                    let index = self.curr.next();
                    let target = self.stack.pop();
                    let val = self.stack.pop();
                    let obj_index = target.as_obj();
                    let obj = &mut self.objs[obj_index];
                    obj.as_mut_fields()[index as usize] = val;
                }


                consts::CastIntToFloat => {
                    let v = self.stack.pop().as_int();
                    self.stack.push(Reg::new_float(v as f64));
                }


                consts::CastFloatToInt => {
                    let v = self.stack.pop().as_float();
                    self.stack.push(Reg::new_int(v as _));
                }


                consts::CastBoolToInt => {
                    let v = self.stack.pop().as_bool();
                    self.stack.push(Reg::new_int(v as _));
                }


                consts::UnwrapStore => {
                    let var = self.stack.pop();
                    let val = self.stack.pop();
                    
                    let obj = &mut self.objs[var.as_obj()];

                    if obj.as_fields()[0].as_int() == 1 {
                        return Status::err(FatalError::new("tried to unwrap an invalid value"));
                    }

                    obj.as_mut_fields()[1] = val;
                }


                consts::Unwrap => {
                    let val = self.stack.pop();
                    let obj_index = val.as_obj();
                    let obj = &self.objs[obj_index];
                    if obj.as_fields()[0].as_int() == 1 {
                        return Status::err(FatalError::new("tried to unwrap an invalid value"));
                    }

                    self.stack.push(obj.as_fields()[1])
                }


                consts::UnwrapFail => {
                    return Status::err(FatalError::new("tried to unwrap an invalid value"));
                }


                consts::LoadEnumField => {
                    let index = self.curr.next_u32();
                    let val = self.stack.pop();
                    let obj_index = val.as_obj();
                    let obj = &self.objs[obj_index];

                    let tag = obj.as_fields()[0].as_int();
                    let val = if tag as u32 == index {
                        self.new_obj(ObjectData::Struct {
                            fields: vec![Reg::new_int(0), obj.as_fields()[1]],
                        })
                    } else {
                        self.new_obj(ObjectData::Struct {
                            fields: vec![Reg::new_int(1), Reg::new_unit()],
                        })
                    }?;


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
                    self.stack.push(Reg::new_int(self.curr.next_i64() as _));
                }


                consts::ConstFloat => {
                    self.stack.push(Reg::new_float(self.curr.next_f64()));
                }


                consts::ConstBool => {
                    self.stack.push(Reg::new_bool(self.curr.next() == 1));
                }


                consts::ConstStr => {
                    let index = self.curr.next_u32();
                    self.stack.push(Reg::new_obj(ObjectIndex::new(index as _)));
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


                consts::EqObj => {
                    let lhs = self.stack.pop().as_obj();
                    let rhs = self.stack.pop().as_obj();

                    let mut stack = vec![];
                    stack.push((lhs, rhs));

                    let mut result = true;
                    while let Some((lhs, rhs)) = stack.pop() {
                        let lhs = self.objs.get(lhs);
                        let rhs = self.objs.get(rhs);

                        match (&lhs.data, &rhs.data) {
                              (ObjectData::List(f1), ObjectData::List(f2))
                            | (ObjectData::Struct { fields: f1 }, ObjectData::Struct { fields: f2 }) => {
                                if f1.len() != f2.len() {
                                    result = false;
                                    break;
                                }

                                for (lhs, rhs) in f1.iter().zip(f2.iter()) {
                                    if lhs.kind != rhs.kind {
                                        result = false;
                                        break;
                                    }

                                    let r = match lhs.kind {
                                        Reg::TAG_INT => lhs.as_int() == rhs.as_int(),
                                        Reg::TAG_FLOAT => lhs.as_float() == rhs.as_float(),
                                        Reg::TAG_BOOL => lhs.as_bool() == rhs.as_bool(),
                                        Reg::TAG_NOCAP => lhs.as_nocap() == rhs.as_nocap(),
                                        Reg::TAG_UNIT => true,
                                        Reg::TAG_OBJ => {
                                            stack.push((lhs.as_obj(), rhs.as_obj()));
                                            true
                                        },
                                        _ => unreachable!(),
                                    };


                                    if !r {
                                        result = false;
                                        break;
                                    }

                                }

                                if !result { break }
                            },


                            (ObjectData::String(s1), ObjectData::String(s2)) => {
                                if s1 != s2 {
                                    result = false;
                                    break;
                                }
                            },


                            (ObjectData::Dict(hm1), ObjectData::Dict(hm2)) => {
                                if hm1.len() != hm2.len() {
                                    result = false;
                                    break;
                                }

                                for (k, lhs) in hm1 {
                                    let Some(rhs) = hm2.get(k)
                                    else {
                                        result = false;
                                        break;
                                    };

                                    let r = match lhs.kind {
                                        Reg::TAG_INT => lhs.as_int() == rhs.as_int(),
                                        Reg::TAG_FLOAT => lhs.as_float() == rhs.as_float(),
                                        Reg::TAG_BOOL => lhs.as_bool() == rhs.as_bool(),
                                        Reg::TAG_NOCAP => lhs.as_nocap() == rhs.as_nocap(),
                                        Reg::TAG_UNIT => true,
                                        Reg::TAG_OBJ => {
                                            stack.push((lhs.as_obj(), rhs.as_obj()));
                                            true
                                        },
                                        _ => unreachable!(),
                                    };


                                    if !r {
                                        result = false;
                                        break;
                                    }


                                }

                                if !result { break }

                            },


                            (ObjectData::Ptr(p1), ObjectData::Ptr(p2)) => {
                                if p1 != p2 {
                                    result = false;
                                    break;
                                }
                            },
                            _ => {
                                result = false;
                                break;
                            },
                        }

                    };


                    self.stack.push(Reg::new_bool(result));
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

                    self.curr.offset(offset as i32);
                }


                _ => panic!("unimplemented opcode '{opcode}; {:?}", crate::opcode::runtime::OpCode::from_u8(opcode)),
            }
        }
        }


        assert_eq!(self.stack.bottom, 0);
        assert_eq!(self.stack.curr, 1);


        Status::ok()
    }
}



pub extern "C" fn ret_instr(vm: &mut VM, local_count: i64, ret: &mut Reg) {
    unsafe { 
    let return_val = vm.stack.pop();
    //dbg!(return_val);

    /*
    if let Some(cache) = &mut vm.funcs[vm.curr.func as usize].cache {
        let args = &vm.stack.values.deref()[vm.stack.bottom..vm.stack.bottom + vm.curr.argc as usize];
        let args = Vec::from(args).leak();
        cache.insert(args, return_val);
    }
    */

    vm.stack.curr -= local_count as usize;
    *ret = return_val;


    }
}
