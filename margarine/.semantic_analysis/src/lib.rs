use std::collections::HashMap;

use common::string_map::StringMap;
use errors::Error;
use ::errors::SemaError;
use funcs::FunctionMap;
use llvm_api::{values::Value, Context, Module};
use namespace::NamespaceMap;
use parser::DataType;
use scope::{ScopeId, ScopeMap};
use sti::{arena::Arena, keyed::KVec};
use types::{ty::Type, ty_map::{TypeId, TypeMap}};

pub mod scope;
pub mod namespace;
pub mod funcs;
pub mod types;
pub mod errors;

#[derive(Debug)]
pub struct Analyzer<'me, 'out, 'str> {
    output    : &'out Arena,
    string_map: &'me mut StringMap<'str>,

    module    : Module,
    context   : Context,

    scopes    : ScopeMap,
    namespaces: NamespaceMap<'out>,
    types     : TypeMap<'out>,
    funcs     : FunctionMap<'out>,

    errors    : KVec<SemaError, Error>,
}


#[derive(Debug, Clone, Copy)]
pub struct AnalysisResult {
    ty    : Type,
    value : Value,
    is_mut: bool,
}


impl<'me, 'out, 'str> Analyzer<'me, 'out, 'str> {
    fn dt_to_ty(&mut self, scope: ScopeId,
                dt: DataType) -> Result<Type, Error> {
    }
}
