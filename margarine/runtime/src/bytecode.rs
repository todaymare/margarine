macro_rules! bytecode {
    ($($opcode: literal : $name: ident ( $($field: ident : $ty: ty),* ));* ;) => {

        #[allow(non_upper_case_globals)]
        pub mod consts {
            $(pub const $name : u8 = $opcode;)*
        }


        ///
        /// A bytecode instruction
        ///
        pub enum Bytecode {
            $($name { $($field: $ty,)* },)*
        }


        impl Bytecode {
            ///
            /// Calls `Self::parse_one` with `data` and puts the result in
            /// the buffer until the iterator is exhausted
            ///
            pub fn parse<I>(data: &mut I, buffer: &mut Vec<Bytecode>)
            where I: Iterator<Item=u8>
            {
                while let Some(op) = Self::parse_one(data) {
                    buffer.push(op);
                }
            }


            ///
            /// Parses a single bytecode instruction given an iterator
            /// This function will return `None` if the iterator is empty
            /// but it will panic if the bytecode instruction is corrupt
            ///
            pub fn parse_one<I>(data: &mut I) -> Option<Bytecode>
            where I: Iterator<Item=u8> 
            {
                Some(match data.next()? {
                    $(consts::$name => Bytecode::$name { $($field: <$ty>::parse(data)),* },)*

                    _ => panic!("invalid bytecode op-code"),
                })
                
            }

            
            ///
            /// Calls `Self::generate_one` for each element in `data`
            /// putting the result into the `buffer`
            ///
            pub fn generate<I>(data: &mut I, buffer: &mut Vec<u8>)
            where I: Iterator<Item=Bytecode>
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


pub trait BytecodeType: Sized {
    fn parse<T>(iter: &mut T) -> Self where T: Iterator<Item = u8>;
    fn generate(&self, buffer: &mut Vec<u8>); 
}


bytecode!(
    100 : AddI(dst: Reg, lhs: Reg, rhs: Reg);
    101 : AddF(dst: Reg, lhs: Reg, rhs: Reg);
    102 : AddU(dst: Reg, lhs: Reg, rhs: Reg);
);


#[derive(Debug, Clone, Copy)]
pub struct Reg(pub u8);

impl BytecodeType for Reg {
    #[inline(always)]
    fn parse<T>(iter: &mut T) -> Self
    where T: Iterator<Item = u8> {
        let reg = iter.next().unwrap();
        Self(reg)
    }

    #[inline(always)]
    fn generate(&self, buffer: &mut Vec<u8>) 
    {
        buffer.push(self.0)
    }
}

