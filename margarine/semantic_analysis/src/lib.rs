pub mod scope;
pub mod namespace;
pub mod types;

use wasm::WasmModuleBuilder;

struct Analyzer<'out> {
    module_builder: WasmModuleBuilder<'out>,
}


