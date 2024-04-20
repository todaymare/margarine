use margarine::{nodes::{decl::{Declaration, DeclarationNode}, Node}, DropTimer, FileData, SourceRange, StringMap};
use sti::arena::Arena;

pub fn run(string_map: &mut StringMap<'_>, files: Vec<FileData>) -> Result<(), &'static str> {
    let mut global = vec![];
    let mut lex_errors = vec![];
    let mut parse_errors = vec![];

    let arena = Arena::new();
    let mut source_offset = 0;
    for (i, f) in files.iter().enumerate() {
        let (tokens, le) = DropTimer::with_timer("tokenisation", || {
            let tokens = margarine::lex(&f, string_map, source_offset);
            tokens
        });

        let (ast, pe) = DropTimer::with_timer("parsing", || {
            let ast = margarine::parse(tokens, i.try_into().unwrap(), &arena, string_map);
            ast
        });

        {
            for n in ast.iter() {
                if matches!(n, Node::Declaration(_)) { continue }
                if let Node::Attribute(attr) = n {
                    if matches!(attr.node(), Node::Declaration(_)) { continue }
                }

                ()
            }
        }

        lex_errors.push(le);
        parse_errors.push(pe);

        global.push(DeclarationNode::new(
            Declaration::Module {
                name: f.name(),
                body: ast.inner(),
                header: SourceRange::new(0, 0),
           },

           SourceRange::new(source_offset, source_offset + f.read().len() as u32),
        ).into());

       source_offset += f.read().len() as u32;
    }

    assert_eq!(lex_errors.len(), files.len());
    assert_eq!(parse_errors.len(), files.len());


    let sema_arena = Arena::new();
    let _scopes = Arena::new();
    let (sema, ctx) = {
        let _1 = DropTimer::new("semantic analysis");
        margarine::Analyzer::run(&sema_arena, &global, string_map)
    };

    // todo: find a way to comrpess these errors into vecs
    let mut lex_error_files = Vec::with_capacity(lex_errors.len());
    for l in lex_errors {
        let mut file = Vec::with_capacity(l.len());
        for e in l.iter() {
            let report = margarine::display(e.1, &sema.string_map, &files, &());
            #[cfg(not(feature = "fuzzer"))]
            println!("{report}");
            file.push(report);
        }

        lex_error_files.push(file);
    }

    let mut parse_error_files = Vec::with_capacity(parse_errors.len());
    for l in parse_errors {
        let mut file = Vec::with_capacity(l.len());
        for e in l.iter() {
            let report = margarine::display(e.1, &sema.string_map, &files, &());
            #[cfg(not(feature = "fuzzer"))]
            println!("{report}");
            file.push(report);
        }

        parse_error_files.push(file);
    }

    let mut sema_errors = Vec::with_capacity(sema.errors.len());
    for s in sema.errors.iter() {
        let report = margarine::display(s.1, &sema.string_map, &files, &sema.types);
        #[cfg(not(feature = "fuzzer"))]
        println!("{report}");
        sema_errors.push(report);
    } 

    ctx.module(sema.module).dump();

     Ok(())
}
