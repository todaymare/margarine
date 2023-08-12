use std::{fmt::Write, collections::HashMap};

use colored::{Color, Colorize};
use common::{SymbolIndex, SourceRange, FileData, SymbolMap};


const ORANGE: Color = Color::TrueColor {
    r: 255,
    g: 160,
    b: 100,
};

// Error Creation
#[repr(usize)]
pub enum ErrorCode {
    LInvalidCharacter  = 101,
    LUnterminatedStr   = 102,
    LCorruptUnicodeEsc = 103,
    LUnicodeNotBase16  = 104,
    LInvalidUnicodeChr = 105,
    LInvalidNumForBase = 106,
    LTooManyDots       = 107,
    LNumTooLarge       = 108,


    PUnexpectedToken   = 201,


    SNameAlrDefined    = 301,
    STypeDoesntExist   = 302,
    SFieldDefEarlier   = 303,
    SVariantDefEarlier = 304,
    
}



#[derive(Debug, PartialEq)]
pub struct Error {
    body: Vec<ErrorOption>
}


impl Error {
    pub fn new(body: Vec<ErrorOption>) -> Self { Self { body } }

    pub fn build(self, files: &HashMap<SymbolIndex, FileData>, symbol_map: &SymbolMap) -> String {
        self.body.into_iter().map(|x| x.build(files, symbol_map)).collect()
    }
}


pub trait CombineIntoError {
    fn combine_into_error(self) -> Error;
}


impl CombineIntoError for Vec<Error> {
    fn combine_into_error(self) -> Error {
        let mut body = Vec::with_capacity(self.iter().map(|x| x.body.len()).sum());
        self.into_iter().for_each(|mut x| {
            body.append(&mut x.body);
            if !body.last().map(|x| {
                match x {
                    ErrorOption::Text(v) => v.as_str() == "\n",
                    _ => false,
                }
            }).unwrap_or(false) {
                body.push(ErrorOption::Text(String::from("\n")))
            }
        });

        Error { body }
    }
}


#[derive(Debug, PartialEq)]
pub enum ErrorOption {
    Text(String),
    Highlight {
        range: SourceRange,
        note: Option<String>,
        colour: Color,

        file: SymbolIndex,
    }
}


pub trait ErrorBuilder {
    fn highlight(self, range: SourceRange) -> Highlight<Self> 
    where
        Self: Sized
    {
        Highlight {
            parent: self,
            range,
            note: None,
            colour: Color::BrightRed,
        }
    }



    fn text(self, text: String) -> Text<Self> 
    where
        Self: Sized
    {
        Text {
            parent: self,
            text
        }
    }


    fn empty_line(self) -> Text<Self> 
    where
        Self: Sized
    {
        Text {
            parent: self,
            text: String::from('\n')
        }
    }


    
    fn flatten(self, vec: &mut Vec<ErrorOption>);
    
    fn build(self) -> Error
    where 
        Self: Sized
    {
        let mut buffer = vec![];

        self.flatten(&mut buffer);
        
        Error::new(buffer)
    }


    fn file(&self) -> SymbolIndex;
}


impl ErrorOption {
    pub fn build(self, files: &HashMap<SymbolIndex, FileData>, symbol_map: &SymbolMap) -> String {
        match self {
            ErrorOption::Text(text) => text,


            ErrorOption::Highlight { range, note, colour, file } => {
                let mut string = String::new();

                let file = files.get(&file).unwrap();
                let source = file.read();
                let file_name = file.name();
                let file_name = symbol_map.get(file_name);

                let start_line = line_at_index(source, range.start()).unwrap().1;
                let end_line   = line_at_index(source, range.end()).unwrap().1;
                let line_size  = end_line.to_string().len();

                
                {
                    let _ = writeln!(string, 
                        "{}{} {}:{}:{}", " ".repeat(line_size), 
                        "-->".color(ORANGE), 
                        file_name, start_line+1, 
                        range.start() - start_of_line(source, start_line)
                    );

                    let _ = write!(string, "{} {}", " ".repeat(line_size), "|".color(ORANGE));
                }


                // println!("{}", source.as_bytes().len());
                // writeln!(string, "{}", &source[range.start..range.end]);
                
               for (line_number, line) in source.lines().enumerate().take(end_line + 1).skip(start_line) {
                    let _ = writeln!(string);

                    let _ = writeln!(string, "{:>w$} {} {}", (line_number + 1).to_string().color(ORANGE), "|".color(ORANGE), line, w = line_size);

                    let _ = write!(string, "{:>w$} {} ",
                        " ".repeat(line_number.to_string().len()),
                        "|".color(ORANGE),

                        w = line_size,
                    );

                    if line_number == start_line {
                        let start_of_line = start_of_line(source, line_number);

                        let _ = write!(string, "{}{}",
                            " ".repeat({
                                let mut count = 0;
                                for (index, i) in line.chars().enumerate() {
                                    if count >= range.start() - start_of_line {
                                        count = index;
                                        break
                                    }
                                    count += i.len_utf8();
                                }
                                count
                            }),
                            "^".repeat({
                                if end_line == line_number {
                                    (range.end() - range.start()) + 1
                                } else {
                                    dbg!(line, range, start_of_line, line_number);
                                    line.len() - (range.start() - start_of_line) + 1
                                }
                            }).color(colour),
                        );

                        
                    } else if line_number == end_line {
                        let _ = write!(string, "{}",
                            "^".repeat({
                                let start_of_end = start_of_line(source, end_line);
                                range.end() - start_of_end + 1
                            }).color(colour),
                        );

                       
                    } else {
                        let _ = write!(string, "{}",
                            "^".repeat(line.len()).color(colour),
                        );
                    }

                }

                
                if let Some(note) = note {
                    let _ = writeln!(string, " {note}");
                } else {
                    let _ = writeln!(string);
                }
        
                string
            },
        }
    }
}



pub struct Highlight<T: ErrorBuilder> {
    parent: T,
    
    range: SourceRange,
    note: Option<String>,
    colour: Color,
}


impl<T: ErrorBuilder> ErrorBuilder for Highlight<T> {
    fn flatten(self, vec: &mut Vec<ErrorOption>) {
        let file = self.file();
        self.parent.flatten(vec);

        vec.push(ErrorOption::Highlight { range: self.range, note: self.note, colour: self.colour, file })
    }


    fn file(&self) -> SymbolIndex {
        self.parent.file()
    }
}


impl<T: ErrorBuilder> Highlight<T> {
    pub fn note(mut self, note: String) -> Self {
        self.note = Some(note);
        self
    }

    pub fn colour(mut self, colour: Color) -> Self {
        self.colour = colour;
        self
    }
}


pub struct Text<T: ErrorBuilder> {
    parent: T,
    
    text: String,
}


impl<T: ErrorBuilder> ErrorBuilder for Text<T> {
    fn flatten(self, vec: &mut Vec<ErrorOption>) {
        self.parent.flatten(vec);

        vec.push(ErrorOption::Text(self.text))
    }


    fn file(&self) -> SymbolIndex {
        self.parent.file()
    }
}


pub struct CompilerError<'a>(ErrorCode, &'a str, SymbolIndex);


impl CompilerError<'_> {
    pub fn new(file: SymbolIndex, id: ErrorCode, text: &str) -> CompilerError {
        CompilerError(id, text, file)
    }
}


impl ErrorBuilder for CompilerError<'_> {
    fn flatten(self, vec: &mut Vec<ErrorOption>) {
        let mut string = String::new();

        let _ = write!(string, "error[{:>03}]", self.0 as usize);

        string = string.red().bold().to_string();
                
        let _ = writeln!(string, " {}", self.1.white().bold());
        
        vec.push(ErrorOption::Text(string))
    }

    
    fn file(&self) -> SymbolIndex {
        self.2
    }
}


const LINE_COUNT : usize = 1;

pub fn line_at_index(value: &str, index: usize) -> Option<(&str, usize)> {
    let mut index_counter = 0;
    for (i, line) in value.lines().enumerate() {
        index_counter += line.chars().map(|x| x.len_utf8()).sum::<usize>();
        index_counter += LINE_COUNT;

        if index_counter > index {
            return Some((line, i));
        }
    }
    
    Some(("", value.lines().count()))
}

pub fn start_of_line(value: &str, line_number: usize) -> usize {
    let mut counter = 0;

    for (i, line) in value.lines().enumerate() {
        if i == line_number {
            break
        }

        counter += line.chars().map(|x| x.len_utf8()).sum::<usize>();
        counter += LINE_COUNT;
    }

    counter
}