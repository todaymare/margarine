use sti::prelude::{Arena, Vec};

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
            pub fn parse<I>(
                data: &mut I, 
                buffer: &mut Vec<Bytecode<'a>>, 
                arena: &'a sti::prelude::Arena
            )
            where I: Iterator<Item=u8>
            {
                while let Some(op) = Self::parse_one(data, arena) {
                    buffer.push(op);
                }
            }


            ///
            /// Parses a single bytecode instruction given an iterator
            /// This function will return `None` if the iterator is empty
            /// but it will panic if the bytecode instruction is corrupt
            ///
            pub fn parse_one<I>(data: &mut I, arena: &'a Arena) -> Option<Bytecode<'a>>
            where I: Iterator<Item=u8> 
            {
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


bytecode!(
    100 : Call(dst: Reg, func: u64, regs: &'a [Reg]);
    200 : AddI(dst: Reg, lhs: Reg, rhs: Reg);
    201 : AddF(dst: Reg, lhs: Reg, rhs: Reg);
    202 : AddU(dst: Reg, lhs: Reg, rhs: Reg);
    
    203 : SubI(dst: Reg, lhs: Reg, rhs: Reg);
    204 : SubF(dst: Reg, lhs: Reg, rhs: Reg);
    205 : SubU(dst: Reg, lhs: Reg, rhs: Reg);
    
    206 : MulI(dst: Reg, lhs: Reg, rhs: Reg);
    207 : MulF(dst: Reg, lhs: Reg, rhs: Reg);
    208 : MulU(dst: Reg, lhs: Reg, rhs: Reg);
    
    209 : DivI(dst: Reg, lhs: Reg, rhs: Reg);
    210 : DivF(dst: Reg, lhs: Reg, rhs: Reg);
    211 : DivU(dst: Reg, lhs: Reg, rhs: Reg);
    
    212 : RemI(dst: Reg, lhs: Reg, rhs: Reg);
    213 : RemF(dst: Reg, lhs: Reg, rhs: Reg);
    214 : RemU(dst: Reg, lhs: Reg, rhs: Reg);
);


pub trait BytecodeType<'a>: Sized {
    type Output;
    fn parse(iter: &mut impl Iterator<Item=u8>, arena: &'a Arena) -> Self::Output;
    fn generate(&self, buffer: &mut Vec<u8>); 
}


#[derive(Debug, Clone, Copy)]
pub struct Reg(pub u8);

impl BytecodeType<'_> for Reg {
    type Output = Self;

    fn parse(iter: &mut impl Iterator<Item=u8>, arena: &Arena) -> Self::Output {
        Reg(u8::parse(iter, arena))
    }

    fn generate(&self, buffer: &mut Vec<u8>) {
        self.0.generate(buffer)
    }
}


impl BytecodeType<'_> for u8 {
    type Output = Self;

    fn parse(iter: &mut impl Iterator<Item=u8>, _: &Arena) -> Self::Output {
        iter.next().unwrap()
    }
    
    #[inline(always)]
    fn generate(&self, buffer: &mut Vec<u8>) {
        buffer.push(*self)
    }
}


impl<'r, 'a, A: 'a, T: BytecodeType<'a, Output=A>> BytecodeType<'a> for &'r [T] {
    type Output = &'a [A];

    fn parse(iter: &mut impl Iterator<Item=u8>, arena: &'a Arena) -> Self::Output {
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


impl BytecodeType<'_> for u16 {
    type Output = Self;

    fn parse(iter: &mut impl Iterator<Item=u8>, _: &Arena) -> Self::Output {
        u16::from_le_bytes([
            iter.next().unwrap(), iter.next().unwrap(),
        ])
    }

    fn generate(&self, buffer: &mut Vec<u8>) {
        for i in self.to_le_bytes().iter() {
            buffer.push(*i)
        }
    }
}


impl BytecodeType<'_> for u32 {
    type Output = Self;

    fn parse(iter: &mut impl Iterator<Item=u8>, _: &Arena) -> Self::Output {
        u32::from_le_bytes([
            iter.next().unwrap(), iter.next().unwrap(),
            iter.next().unwrap(), iter.next().unwrap(),
        ])
    }

    fn generate(&self, buffer: &mut Vec<u8>) {
        for i in self.to_le_bytes().iter() {
            buffer.push(*i)
        }
    }
}


impl BytecodeType<'_> for u64 {
    type Output = Self;

    fn parse(iter: &mut impl Iterator<Item=u8>, _: &Arena) -> Self::Output {
        u64::from_le_bytes([
            iter.next().unwrap(), iter.next().unwrap(),
            iter.next().unwrap(), iter.next().unwrap(),
            iter.next().unwrap(), iter.next().unwrap(),
            iter.next().unwrap(), iter.next().unwrap(),
        ])
    }

    fn generate(&self, buffer: &mut Vec<u8>) {
        for i in self.to_le_bytes().iter() {
            buffer.push(*i)
        }
    }
}
