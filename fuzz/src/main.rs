use margarine::{FileData, StringMap};
use sti::arena::Arena;

#[macro_use]
extern crate afl;

fn main() {
    fuzz!(|data: &[u8]| {
        let Ok(s) = std::str::from_utf8(data)
        else { return };

        let arena = Arena::new();
        let mut sm = StringMap::new(&arena);

        let file_data = FileData::new(s.to_string(), StringMap::VALUE, margarine::Extension::Mar);
        margarine::run(&mut sm, vec![file_data]);
    });

}

