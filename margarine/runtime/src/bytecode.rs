macro_rules! bytecode {
    ($($opcode: literal : $name: ident ( $($field: ident : $ty: ty),* ));* ;) => {

        #[allow(non_upper_case_globals)]
        pub mod consts {
            $(pub const $name : u8 = $opcode;)*
        }


        pub enum Bytecode {
            $($name { $($field: $ty,)* },)*
        }


        impl Bytecode {
            pub fn parse<I>(data: &mut I, buffer: &mut Vec<Bytecode>)
            where I: Iterator<Item=u8>
            {
                while let Some(op_code) = data.next() {
                    let op = match op_code {
                        $(consts::$name => Bytecode::$name { $($field: <$ty>::parse(data)),* },)*

                        _ => panic!("invalid bytecode op-code"),
                    };

                    buffer.push(op);
                }
            }


            pub fn generate<I>(data: &mut I, buffer: &mut Vec<u8>)
            where I: Iterator<Item=Bytecode>
            {
                while let Some(op) = data.next() {
                    match op {
                        $(
                            Bytecode::$name { $($field),* } => { 
                                buffer.push(consts::$name); 
                                $(<$ty>::generate(&$field, buffer);)*
                            },
                        )*
                    }
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

