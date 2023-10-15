use sti::{reader::Reader, prelude::{Arena, Vec}};

macro_rules! bytecode {
    ($($opcode: literal : $name: ident ( $($field: ident : $ty: ty),* ));* ;) => {

        #[allow(non_upper_case_globals)]
        pub mod consts {
            $(pub const $name : u8 = $opcode;)*
        }


        ///
        /// A bytecode instruction
        ///
        pub enum Bytecode<'a> {
            $($name { $($field: $ty,)* },)*
        }


        #[forbid(unreachable_patterns)]
        impl<'a> Bytecode<'a> {
            ///
            /// Calls `Self::parse_one` with `data` and puts the result in
            /// the buffer until the iterator is exhausted
            ///
            pub fn parse(
                data: &mut sti::reader::Reader<u8>, 
                buffer: &mut Vec<Bytecode<'a>>, 
                arena: &'a sti::prelude::Arena
            ) {
                while let Some(op) = Self::parse_one(data, arena) {
                    buffer.push(op);
                }
            }


            ///
            /// Parses a single bytecode instruction given an iterator
            /// This function will return `None` if the iterator is empty
            /// but it will panic if the bytecode instruction is corrupt
            ///
            pub fn parse_one(
                data: &mut sti::reader::Reader<u8>,
                arena: &'a Arena
            ) -> Option<Bytecode<'a>> {
                
                Some(match data.next()? {
                    $(consts::$name => Bytecode::$name { $($field: <$ty>::parse(data, arena)),* },)*

                    _ => panic!("invalid bytecode op-code"),
                })
                
            }

            
            ///
            /// Calls `Self::generate_one` for each element in `data`
            /// putting the result into the `buffer`
            ///
            pub fn generate<I>(data: &mut I, buffer: &mut Vec<u8>)
            where I: Iterator<Item=Bytecode<'a>>
            {
                while let Some(op) = data.next() {
                    op.generate_one(buffer);
                }
            }

            
            ///
            /// Generates compact bytes from a bytecode instruction
            /// and puts the data in the `buffer`
            ///
            pub fn generate_one(self, buffer: &mut Vec<u8>) {
                match self {
                    $(
                        Bytecode::$name { $($field),* } => { 
                            buffer.push(consts::$name); 
                            $(<$ty>::generate(&$field, buffer);)*
                        },
                    )*
                }
                
            }
        }
    };
}


macro_rules! wrapper {
    (
        $(
            $(#[$trait: ident ($($ident: ident),*)])*
            struct $name: ident ($ty: ty);
        )*
    ) => {
        
        $(
            $(#[$trait($($ident),*)])*
            pub struct $name(pub $ty);

            impl BytecodeType<'_> for $name {
                type Output = Self;

                fn parse(iter: &mut Reader<u8>, arena: &Arena) -> Self::Output {
                    Self(<$ty>::parse(iter, arena))
                }

                fn generate(&self, buffer: &mut Vec<u8>) {
                    self.0.generate(buffer)
                }
            }
        )*
    };
}


bytecode!(
      0 :  Ret ();
      // 1 :  Jmp (at: JumpIndex);
      // 2 :  Jif (cnd: Reg, if_true: JumpIndex, if_false: JumpIndex);
      // 3 : Match(cnd: Reg, jmps: &'a [JumpIndex]);
    
    100 :  Call(dst: Reg, func: u64, regs: &'a [Reg]);

    150 :  Unit(dst: Reg);
    151 :  LitS(dst: Reg, lit: u64);
    152 :  LitI(dst: Reg, lit: i64);
    153 :  LitF(dst: Reg, lit: f64);
    154 :  LitB(dst: Reg, lit: bool);
    
    200 :  AddI(dst: Reg, lhs: Reg, rhs: Reg);
    201 :  AddF(dst: Reg, lhs: Reg, rhs: Reg);
    202 :  AddU(dst: Reg, lhs: Reg, rhs: Reg);
    
    203 :  SubI(dst: Reg, lhs: Reg, rhs: Reg);
    204 :  SubF(dst: Reg, lhs: Reg, rhs: Reg);
    205 :  SubU(dst: Reg, lhs: Reg, rhs: Reg);
    
    206 :  MulI(dst: Reg, lhs: Reg, rhs: Reg);
    207 :  MulF(dst: Reg, lhs: Reg, rhs: Reg);
    208 :  MulU(dst: Reg, lhs: Reg, rhs: Reg);
    
    209 :  DivI(dst: Reg, lhs: Reg, rhs: Reg);
    210 :  DivF(dst: Reg, lhs: Reg, rhs: Reg);
    211 :  DivU(dst: Reg, lhs: Reg, rhs: Reg);
    
    212 :  RemI(dst: Reg, lhs: Reg, rhs: Reg);
    213 :  RemF(dst: Reg, lhs: Reg, rhs: Reg);
    214 :  RemU(dst: Reg, lhs: Reg, rhs: Reg);

    255 :  Error(err_index: u64);
);


pub trait BytecodeType<'a>: Sized {
    type Output;
    fn parse(iter: &mut Reader<u8>, arena: &'a Arena) -> Self::Output;
    fn generate(&self, buffer: &mut Vec<u8>); 
}

wrapper!(
    #[derive(Debug, Clone, Copy)] 
    struct Reg(u8);
    
    #[derive(Debug, Clone, Copy)] 
    struct JumpIndex(u8);
);


impl<'r, 'a, A: 'a, T: BytecodeType<'a, Output=A>> BytecodeType<'a> for &'r [T] {
    type Output = &'a [A];

    fn parse(iter: &mut Reader<u8>, arena: &'a Arena) -> Self::Output {
        let len = u32::parse(iter, arena);
        let mut vec = Vec::new_in(arena);

        for _ in 0..len {
            vec.push(T::parse(iter, arena))
        }

        vec.leak()
    }
    

    fn generate(&self, buffer: &mut Vec<u8>) {
        let len : u32 = self.len().try_into().expect("length of a map can't exceed a u32");
        len.generate(buffer);

        for i in self.iter() {
            i.generate(buffer);
        }
    }
}


impl BytecodeType<'_> for u8 {
    type Output = Self;

    fn parse(iter: &mut Reader<u8>, _: &Arena) -> Self::Output {
        iter.next().unwrap()
    }
    
    #[inline(always)]
    fn generate(&self, buffer: &mut Vec<u8>) {
        buffer.push(*self)
    }
}


impl BytecodeType<'_> for u16 {
    type Output = Self;

    fn parse(iter: &mut Reader<u8>, _: &Arena) -> Self::Output {
        u16::from_le_bytes(iter.next_array().unwrap())
    }

    fn generate(&self, buffer: &mut Vec<u8>) {
        for i in self.to_le_bytes().iter() {
            buffer.push(*i)
        }
    }
}


impl BytecodeType<'_> for u32 {
    type Output = Self;

    fn parse(iter: &mut Reader<u8>, _: &Arena) -> Self::Output {
        u32::from_le_bytes(iter.next_array().unwrap())
    }

    fn generate(&self, buffer: &mut Vec<u8>) {
        for i in self.to_le_bytes().iter() {
            buffer.push(*i)
        }
    }
}


impl BytecodeType<'_> for u64 {
    type Output = Self;

    fn parse(iter: &mut Reader<u8>, _: &Arena) -> Self::Output {
        u64::from_le_bytes(iter.next_array().unwrap())
    }

    fn generate(&self, buffer: &mut Vec<u8>) {
        for i in self.to_le_bytes().iter() {
            buffer.push(*i)
        }
    }
}


impl BytecodeType<'_> for i64 {
    type Output = Self;

    fn parse(iter: &mut Reader<u8>, _: &Arena) -> Self::Output {
        i64::from_le_bytes(iter.next_array().unwrap())
    }

    fn generate(&self, buffer: &mut Vec<u8>) {
        for i in self.to_le_bytes().iter() {
            buffer.push(*i)
        }
    }
}


impl BytecodeType<'_> for f64 {
    type Output = Self;

    fn parse(iter: &mut Reader<u8>, _: &Arena) -> Self::Output {
        f64::from_le_bytes(iter.next_array().unwrap())
    }

    fn generate(&self, buffer: &mut Vec<u8>) {
        for i in self.to_le_bytes().iter() {
            buffer.push(*i)
        }
    }
}


impl BytecodeType<'_> for bool {
    type Output = Self;

    fn parse(iter: &mut Reader<u8>, _: &Arena) -> Self::Output {
        iter.next().unwrap() == 1
    }

    fn generate(&self, buffer: &mut Vec<u8>) {
        buffer.push(*self as u8)
    }
}
