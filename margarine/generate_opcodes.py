spec = {
  "Ret": { "local_count": "u8" },
  "Unit": {},
  "PushLocalSpace": { "amount": "u8" },
  "PopLocalSpace": { "amount": "u8" },
  "Err": { "ty": "u8", "file": "u32", "index": "u32" },
  "ConstInt": { "val": "i64" },
  "ConstFloat": { "val": "f64" },
  "ConstBool": { "val": "u8" },
  "ConstStr": { "val": "u32" },

  "Call": { "func": "u32", "argc": "u8" },

  "Pop": {},
  "Copy": {},

  "CreateList": { "elem_count": "u32" },
  "CreateStruct": { "field_count": "u8" },
  "LoadField": { "field_index": "u8" },
  "IndexList": { },
  "StoreList": { },
  "StoreField": { "field_index": "u8" },
  "LoadEnumField": { "enum_index": "u32" },

  "Unwrap": {},
  "UnwrapFail": {},

  "CastIntToFloat": {},
  "CastFloatToInt": {},
  "CastBoolToInt": {},

  "NegInt": {},
  "AddInt": {},
  "SubInt": {},
  "MulInt": {},
  "DivInt": {},
  "RemInt": {},
  "EqInt": {},
  "NeInt": {},
  "GtInt": {},
  "GeInt": {},
  "LtInt": {},
  "LeInt": {},
  "BitwiseOr": {},
  "BitwiseAnd": {},
  "BitwiseXor": {},
  "BitshiftLeft": {},
  "BitshiftRight": {},

  "NegFloat": {},
  "AddFloat": {},
  "SubFloat": {},
  "MulFloat": {},
  "DivFloat": {},
  "RemFloat": {},
  "EqFloat": {},
  "NeFloat": {},
  "GtFloat": {},
  "GeFloat": {},
  "LtFloat": {},
  "LeFloat": {},

  "EqBool": {},
  "NeBool": {},
  "NotBool": {},

  "Load": { "index": "u8" },
  "Store": { "index": "u8" },
  "UnwrapStore": {},

  "Jump": { "offset": "i32" },
  "SwitchOn": { "true_offset": "i32", "false_offset": "i32" },
  "Switch": { "offsets": "&[u8]" },
}

from textwrap import indent
import re

def to_snake_case(name: str) -> str:
    return re.sub(r'(?<!^)(?=[A-Z])', '_', name).lower()

def generate_opcodes(spec: dict, enum_name="OpCode", repr_ty="u8", vis="pub"):
    # --- Enum definition ---
    enum_variants = "\n".join(f"    {name}," for name in spec.keys())
    enum_def = f"""\
#[repr({repr_ty})]
#[derive(Debug)]
{vis} enum {enum_name} {{
{enum_variants}
}}"""

    # --- Consts module ---
    consts = "\n".join(
        f"    pub const {name}: {repr_ty} = super::{enum_name}::{name} as {repr_ty};"
        for name in spec.keys()
    )
    consts_mod = f"""\
#[allow(non_upper_case_globals)]
pub mod consts {{
{consts}
}}"""

    # --- Builder methods ---
    methods = []
    for name, fields in spec.items():
        method_name = to_snake_case(name)

        args = ", ".join(f"{ident}: {ty}" for ident, ty in fields.items())
        arglist = f"(&mut self{', ' if args else ''}{args})"
        body = [f"self.bytecode.push(super::{enum_name}::{name}.as_u8());"]
        for ident, ty in fields.items():
            if ty == "&[u8]":
                body.append(f"    self.bytecode.extend_from_slice(&( {ident}.len() as u32 ).to_le_bytes());")
                body.append(f"    self.bytecode.extend_from_slice({ident});")
            else:
                body.append(f"    self.bytecode.extend_from_slice(&{ident}.to_le_bytes());")

        body_block = "\n        ".join(body)
        methods.append(f"""\
pub fn {method_name}{arglist} {{
            {body_block}
        }}""")

    for name, fields in spec.items():
        method_name = to_snake_case(name)

        args = "_at: usize, " + ", ".join(f"{ident}: {ty}" for ident, ty in fields.items())
        arglist = f"_at(&mut self{', ' if args else ''}{args})"
        body = []
        body.append(f"            self.bytecode[_at] = super::{enum_name}::{name}.as_u8();")
        body.append(f"            let mut _offset = 1;")
        for ident, ty in fields.items():
            if ty == "&[u8]":
                body.append(f"            let _len = {ident}.len() as u32;")
                body.append(f"            self.bytecode[_at+_offset.._at+_offset+4].copy_from_slice(&_len.to_le_bytes());")
                body.append(f"            _offset += 4;")
                body.append(f"            self.bytecode[_at+_offset.._at+_offset+_len as usize].copy_from_slice({ident});")
                body.append(f"            _offset += _len as usize;")
            else:
                body.append(f"            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&{ident})].copy_from_slice(&{ident}.to_le_bytes());")
                body.append(f"            _offset += core::mem::size_of_val(&{ident});")

        body_block = "\n".join(body)
        methods.append(f"""\
pub fn {method_name}{arglist} {{
{body_block}
        }}""")

    builder_mod = f"""\
#[allow(non_upper_case_globals, non_snake_case, unused)]
pub mod builder {{
    pub struct Builder {{
        pub bytecode: Vec<u8>,
    }}

    impl Builder {{
        pub fn new() -> Self {{
            Self {{ bytecode: vec![] }}
        }}

        pub fn len(&self) -> usize {{ self.bytecode.len() }}

        pub fn append(&mut self, oth: &Builder) {{
            self.bytecode.extend_from_slice(&oth.bytecode);
        }}

        {"\n\n        ".join(methods)}
    }}

    impl core::fmt::Debug for Builder {{
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {{
            use core::fmt::Write;

            let mut strct = f.debug_map();

            let mut iter = crate::Reader::new(&self.bytecode);
            while let Some([opcode]) = iter.try_next_n::<1>() {{
                let offset = unsafe {{ iter.src.offset_from(self.bytecode.as_ptr()) }} - 1;

                let Some(opcode) = super::{enum_name}::from_u8(opcode)
                else {{
                    strct.entry(&offset, &"<invalid opcode>".to_string());
                    break;
                }};

                match opcode {{
{
        "".join(
            f'''\
                    super::{enum_name}::{name} => {{
                        let mut fields = String::new();
                        unsafe {{
{
                "".join(
                    (
                        f"""\
                            {{
                                if !fields.is_empty() {{
                                    fields.push_str(", ");
                                }}
                                let len = <u32>::from_le_bytes(iter.next_n::<4>());
                                let data = iter.next_slice(len as usize);
                                write!(&mut fields, "{ident}: [len={{}} bytes]", len).unwrap();
                            }}
""" if ty == "&[u8]" else f"""\
                            {{
                                if !fields.is_empty() {{
                                    fields.push_str(", ");
                                }}
                                write!(
                                    &mut fields,
                                    "{ident}: {{}}",
                                    <{ty}>::from_le_bytes(
                                        iter.next_n::<{{{{ core::mem::size_of::<{ty}>() }}}}>()
                                    )
                                ).unwrap();
                            }}
"""
                    )
                    for ident, ty in fields.items()
                )
            }
                        }}
                        strct.entry(&offset,
                            &format!(
                            "{name}({{fields}})",
                        ));
                    }}
'''
            for name, fields in spec.items()
        )
    }
                }}
            }}

            strct.finish();
            Ok(())
        }}
    }}
}}"""

    # --- as_u8/from_u8 impl ---
    match_arms = "\n".join(
        f"            consts::{name} => Some(Self::{name})," for name in spec.keys()
    )
    impl_block = f"""\
impl {enum_name} {{
    #[inline(always)]
    #[must_use]
    pub fn as_u8(self) -> {repr_ty} {{ self as _ }}

    #[inline(always)]
    #[must_use]
    pub fn from_u8(value: {repr_ty}) -> Option<Self> {{
        match value {{
{match_arms}
            _ => None,
        }}
    }}
}}"""

    return "\n\n".join([enum_def, consts_mod, builder_mod, impl_block])


# Example usage
if __name__ == "__main__":
    open("margarine/runtime/src/opcode/runtime.rs", "w").write(generate_opcodes(spec))
