pub mod runtime;


macro_rules! opcode {
    ( $(#[$attr:meta])* $vis:vis enum $name:ident : $type:ty {
        $($(#[$($attrss:tt)*])* $variant:ident $(($($ident:ident :$ty:ty),*))?),* $(,)?
    } ) => {
        #[repr($type)]
        $(#[$attr])*
        $vis enum $name {
            $($(#[$($attrss)*])* $variant,)*
        }
        #[allow(non_upper_case_globals)]
        pub mod consts {
            $($(#[$($attrss)*])* pub const $variant: $type = super::$name::$variant as $type;)*
        }
        #[allow(non_upper_case_globals, non_snake_case, unused)]
        pub mod builder {
            pub struct Builder {
                pub bytecode: Vec<u8>,
            }


            impl Builder {
                pub fn new() -> Self {
                    Self {
                        bytecode: vec![],
                    }
                }


                pub fn len(&self) -> usize { self.bytecode.len() }


                pub fn append(&mut self, oth: &Builder) {
                    self.bytecode.extend_from_slice(&oth.bytecode);
                }


                $($(#[$($attrss)*])*
                    pub fn $variant(&mut self, $($($ident: $ty),*)*) {
                        self.bytecode.push(super::$name::$variant.as_u8());
                        $($(
                            self.bytecode.extend_from_slice(&$ident.to_le_bytes());
                        )*)*
                    }
                )*
            }


            impl core::fmt::Debug for Builder {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                    use core::fmt::Write;

                    let mut strct = f.debug_list();

                    let mut iter = crate::Reader::new(&self.bytecode);
                    while let Some([opcode]) = iter.try_next_n::<1>() {
                        let Some(opcode) = super::$name::from_u8(opcode)
                        else {
                            strct.entry(&"<invalid opcode>".to_string());
                            break;
                        };

                        match opcode {
                            $(
                                super::$name::$variant => {
                                    let mut fields = String::new();
                                    unsafe {
                                    $($(
                                    {
                                        if !fields.is_empty() {
                                            fields.push_str(", ");
                                        }

                                        write!(
                                            &mut fields, 
                                            "{}: {}", 
                                            stringify!($ident), 
                                            <$ty>::from_le_bytes(iter.next_n::<{ core::mem::size_of::<$ty>() }>()));
                                    }

                                    );*)*
                                    }

                                    strct.entry(&format!(
                                        "{}({fields})",
                                        stringify!($variant),
                                    ));
                                }
                            )*
                        }
                    }


                    strct.finish();
                    Ok(())
                }
            }
        }
        impl $name {
            #[inline(always)]
            #[must_use]
            pub fn as_u8(self) -> $type { self as _ }

            #[inline(always)]
            #[must_use]
            pub fn from_u8(value: $type) -> Option<Self> {
                match value {
                    $(consts::$variant => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }
    };
}


pub const HEADER : [u8; 7] = *b"BUTTERY";


pub mod func {
opcode! {
#[derive(Hash, PartialEq, Eq, Debug)]
pub enum OpCode : u8 {
    ///
    /// usage: `term`
    ///
    Terminate,


    ///
    /// usage: `fn <$name: str> <$code: u32>`
    ///
    /// The code offset is relative to the code section's start
    ///
    Func,
}
}
}



