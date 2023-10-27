pub mod scope;
pub mod namespace;
pub mod types;

use errors::ErrorId;
use parser::nodes::{Node, NodeKind};
use scope::{ScopeId, ScopeMap, Scope};
use types::Type;
use wasm::{WasmModuleBuilder, WasmFunctionBuilder};

struct Analyzer<'out> {
    scopes: ScopeMap,

    module_builder: WasmModuleBuilder<'out>,
}


struct AnalysisResult {
    ty: Type,
}


impl Analyzer<'_> {
    pub fn block(
        &mut self,
        builder: &mut WasmFunctionBuilder,
        scope: ScopeId,
        nodes: &[Node],
    ) -> AnalysisResult {
        let scope = {
        };

        AnalysisResult { ty: Type::Unit }
    }
}


impl Analyzer<'_> {
    pub fn collect_names(
        nodes: &[Node],
        scope: Scope,
        
        buf_err: Vec<ErrorId>,
    ) {
        for node in nodes {
            let source = node.range();

            let NodeKind::Declaration(decl) = node.kind()
            else { continue };

            match decl {
                parser::nodes::Declaration::Struct { kind, name, header, fields } => {

                },
                parser::nodes::Declaration::Enum { name, header, mappings } => todo!(),
                parser::nodes::Declaration::Function { is_system, name, header, arguments, return_type, body } => todo!(),
                parser::nodes::Declaration::Impl { data_type, body } => todo!(),
                parser::nodes::Declaration::Using { file } => todo!(),
                parser::nodes::Declaration::Module { name, body } => todo!(),
                parser::nodes::Declaration::Extern { file, functions } => todo!(),
            }
        }
    }
}
