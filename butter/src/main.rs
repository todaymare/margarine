use std::{env, fs, io::{Read, Write}, process::{Command, ExitCode, Stdio}};

use game_runtime::encode;
use margarine::{nodes::{decl::{Declaration, DeclarationNode}, Node}, DropTimer, FileData, SourceRange, StringMap};
use sti::prelude::Arena;

// const GAME_RUNTIME : &[u8] = include_bytes!("../../target/debug/game-runtime");

fn main() -> Result<(), &'static str> {
    DropTimer::with_timer("compilation", || {
       let string_map_arena = Arena::new();
       let mut string_map = StringMap::new(&string_map_arena);
       let files = {
           let mut files = Vec::new();
           for i in env::args().skip(1) {
               files.push(FileData::open(i, &mut string_map).unwrap());
           }

           files
       };

       butter::run(&mut string_map, files)
    })
}

