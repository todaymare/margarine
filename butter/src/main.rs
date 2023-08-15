use std::collections::HashMap;

use margarine::{FileData, SymbolMap};

fn main() -> Result<(), &'static str> {
    let mut symbol_map = SymbolMap::new();
    let file = FileData::open("features.txt", &mut symbol_map).unwrap();

    let tokens = margarine::lex(&file, &mut symbol_map);
    let tokens = match tokens {
        Ok(v)  => v,
        Err(e) => {
            let report = e.build(&HashMap::from([(file.name(), file)]), &symbol_map);
            println!("{report}");
            return Err("failed to compile because of the previous errors")
        },
    };

    println!("{tokens:?}");


    let instructions = margarine::parse(tokens, &mut symbol_map);
    let mut instructions = match instructions {
        Ok(v)  => v,
        Err(e) => {
            let report = e.build(&HashMap::from([(file.name(), file)]), &symbol_map);
            println!("{report}");
            return Err("failed to compile because of the previous errors")
        },
    };

    println!("{instructions:#?}");

    let state = margarine::semantic_analysis(&mut symbol_map, &mut instructions);
    let state = match state {
        Ok(v)  => v,
        Err(e) => {
            let report = e.build(&HashMap::from([(file.name(), file)]), &symbol_map);
            println!("{report}");
            println!("{symbol_map:?}");
            return Err("failed to compile because of the previous errors")
        },
    };

    println!("{state:#?}");
    println!("{symbol_map:?}");
    Ok(())
}
