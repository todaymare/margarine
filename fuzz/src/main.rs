use margarine::{Arena, CompilationSettings, CompilationTarget, Compiler, Extension, FileData};

#[macro_use]
extern crate afl;

fn main() {
    fuzz!(|data: &[u8]| {
        let Ok(s) = std::str::from_utf8(data)
        else { return };

        let arena = Arena::new();
        let mut compiler = Compiler::new(&arena);
        let name = compiler.string_map.insert("fuzz.mar");
        compiler.files.register(FileData::new(s.to_owned(), name, Extension::Mar));
        let _ = compiler.run(&CompilationSettings {
            compilation_target: CompilationTarget::Arm64AppleDarwin,
            preludes: vec![],
            entry: "fuzz.mar".to_owned(),
            output: "program".to_owned(),
            cache: "artifacts".to_owned(),
            arena: &arena,
            tests: false,
        });
    });

}
