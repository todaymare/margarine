use std::fmt::Write;

use colourful::{Colour, ColourBrush};
use common::{source::{SourceRange, FileData}, string_map::StringMap};


const ORANGE: Colour = Colour::rgb(
    255,
    160,
    100,
);

// Error Creation
#[repr(usize)]
#[derive(Clone, Copy)]
pub enum ErrorCode {
    LInvalidCharacter  = 101,
    LUnterminatedStr   = 102,
    LCorruptUnicodeEsc = 103,
    LUnicodeNotBase16  = 104,
    LInvalidUnicodeChr = 105,
    LUnterminatedUni   = 106,
    LTooManyDots       = 107,
    LNumTooLarge       = 108,


    PUnexpectedToken   = 201,


    SNameAlrDefined    = 301,
    STypeDoesntExist   = 302,
    SFieldDefEarlier   = 303,
    SVariantDefEarlier = 304,
    SArgDefEarlier     = 305,
    SVariableNotDef    = 306,
    SFuncReturnDiff    = 307,
    SInvalidBinOp      = 308,
    SUnexpectedType    = 309,
    SIfExprNoElse      = 310,
    SMatchValNotEnum   = 311,
    SSymbolUnreachable = 312,
    SSymbolIsntFunc    = 313,
    SFuncArgcMismatch  = 314,
    SArgTypeMismatch   = 315,
    SArgDiffInOut      = 316,
    SCantInitPrimitive = 317,
    SSymbolIsntType    = 318,
    SSymbolIsntStruct  = 319,
    SUnknownField      = 320,
    SMissingField      = 321,
    SFieldTypeMismatch = 322,
    SAccFieldOnPrim    = 323,
    SFieldDoesntExist  = 324,
    SNspaceUnreachable = 325,
    SMatchUnkownVar    = 326,
    SMatchVariantMiss  = 327,
    SMatchBranchDiffTy = 328,
    SVarHintTypeDiff   = 329,
    SInOutArgIsntMut   = 330,
    SAssignValNotLHS   = 331,
    SAssignValNotMut   = 332,
    SAssignValDiffTy   = 333,
    SReturnOutsideFunc = 334,
    SBreakOutsideLoop  = 335,
    SContOutsideLoop   = 336,
    SCantUnwrapType    = 337,
    STryOpOptionRetVal = 338,
    STryOpResultRetVal = 339,
    SBlockOnlyAllowDec = 340,
}


impl ErrorCode {
    pub fn msg(self) -> &'static str {
        match self {
            ErrorCode::LInvalidCharacter => "invalid character",
            ErrorCode::LUnterminatedStr => "unterminated string",
            ErrorCode::LCorruptUnicodeEsc => "corrupt unicode escape",
            ErrorCode::LUnicodeNotBase16 => "unicode value is not base-16",
            ErrorCode::LInvalidUnicodeChr => "invalid unicode character",
            ErrorCode::LUnterminatedUni => "unterminated unicode escape",
            ErrorCode::LTooManyDots => "too many dots",
            ErrorCode::LNumTooLarge => "number too large",
            ErrorCode::PUnexpectedToken => "unexpected token",
            ErrorCode::SNameAlrDefined => "name is already defined",
            ErrorCode::STypeDoesntExist => "type doesn't exist",
            ErrorCode::SFieldDefEarlier => "field is defined earlier",
            ErrorCode::SVariantDefEarlier => "variant is defined earlier",
            ErrorCode::SArgDefEarlier => "argument is defined earlier",
            ErrorCode::SVariableNotDef => "variable is not defined",
            ErrorCode::SFuncReturnDiff => "function returns a different type than the body",
            ErrorCode::SInvalidBinOp => "invalid binary operation",
            ErrorCode::SUnexpectedType => "unexpected type",
            ErrorCode::SIfExprNoElse => "if expression returns a type but has no else block",
            ErrorCode::SMatchValNotEnum => "the value being matched is not an enum",
            ErrorCode::SSymbolUnreachable => "symbol exists but it's unreachable",
            ErrorCode::SSymbolIsntFunc => "symbol exists but it's not a function",
            ErrorCode::SFuncArgcMismatch => "function expects a different number of arguments than given",
            ErrorCode::SArgTypeMismatch => "function argument is of a different type than given",
            ErrorCode::SArgDiffInOut => "function argument is of a different in-outness than given",
            ErrorCode::SCantInitPrimitive => "can't initialise a primitive type with structure creation syntax",
            ErrorCode::SSymbolIsntType => "symbol exists but it's not a type",
            ErrorCode::SSymbolIsntStruct => "symbol exists but it's not a structure",
            ErrorCode::SUnknownField => "unknown field",
            ErrorCode::SMissingField => "missing field",
            ErrorCode::SFieldTypeMismatch => "type field us if a dufferebt tyoe than given",
            ErrorCode::SAccFieldOnPrim => "can't access fields on primitives as they have no fields",
            ErrorCode::SFieldDoesntExist => "field doesn't exist",
            ErrorCode::SNspaceUnreachable => "namespace exists but it's unreachable",
            ErrorCode::SMatchUnkownVar => "unknown variant in match",
            ErrorCode::SMatchVariantMiss => "missing variant in match",
            ErrorCode::SMatchBranchDiffTy => "the match branch differs in the return type compared to others",
            ErrorCode::SVarHintTypeDiff => "the type's hint differs from the given value",
            ErrorCode::SInOutArgIsntMut => "in-out argument isn't mutable",
            ErrorCode::SAssignValNotLHS => "can't assign on this expression",
            ErrorCode::SAssignValNotMut => "can't assign to a value that is immutable",
            ErrorCode::SAssignValDiffTy => "the given value differs from the expected type of the assignment target",
            ErrorCode::SReturnOutsideFunc => "return outside of a function",
            ErrorCode::SBreakOutsideLoop => "break outside of a loop",
            ErrorCode::SContOutsideLoop => "continue outside of a loop",
            ErrorCode::SCantUnwrapType => "this type can't be unwrapped",
            ErrorCode::STryOpOptionRetVal => "the try-operator on an option type only works if the function's return type is an option too",
            ErrorCode::STryOpResultRetVal => "the try-operator on a result type only works if the function's return type is a result with the same error type",
            ErrorCode::SBlockOnlyAllowDec => "this block only allows declarations",
        }
    }
}



#[derive(Debug, PartialEq)]
pub struct Error {
    body: Vec<ErrorOption>
}


impl Error {
    pub fn new(body: Vec<ErrorOption>) -> Self { Self { body } }

    pub fn build(self, files: &[FileData], symbol_map: &StringMap) -> String {
        self.body.into_iter().map(|x| x.build(files, symbol_map)).collect()
    }
}


pub trait CombineIntoError {
    fn combine_into_error(self) -> Error;
}


impl CombineIntoError for &mut [Error] {
    fn combine_into_error(self) -> Error {
        let mut body = Vec::with_capacity(self.iter().map(|x| x.body.len()).sum());
        self.into_iter().for_each(|x| {
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
        colour: Colour,
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
            colour: Colour::rgb(255, 0, 0),
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
}


impl ErrorOption {
    pub fn build(self, files: &[FileData], symbol_map: &StringMap) -> String {
        match self {
            ErrorOption::Text(text) => {
                println!("{text}");
                text
            },


            ErrorOption::Highlight { range, note, colour } => {
                println!("{note:?}");
                let mut string = String::new();

                let file = range.file(files);
                let source = file.read();
                let file_name = file.name();
                let file_name = symbol_map.get(file_name);

                let start_line = line_at_index(source, range.start() as usize).unwrap().1;
                let end_line   = line_at_index(source, range.end() as usize).unwrap().1;
                let line_size  = (end_line + 1).to_string().len();

                
                {
                    let _ = writeln!(string, 
                        "{}{} {}:{}:{}", " ".repeat(line_size), 
                        "-->".colour(ORANGE), 
                        file_name, start_line+1, 
                        range.start() as usize - start_of_line(source, start_line),
                    );

                    let _ = write!(string, "{} {}", " ".repeat(line_size), "|".colour(ORANGE));
                }


                // println!("{}", source.as_bytes().len());
                // writeln!(string, "{}", &source[range.start..range.end]);
                
               for (line_number, line) in source.lines().enumerate().take(end_line + 1).skip(start_line) {
                    let _ = writeln!(string);

                    let _ = writeln!(string, "{:>w$} {} {}", (line_number + 1).colour(ORANGE), "|".colour(ORANGE), line, w = line_size);

                    let _ = write!(string, "{:>w$} {} ",
                        " ".repeat(line_number.to_string().len()),
                        "|".colour(ORANGE),

                        w = line_size,
                    );

                    if line_number == start_line {
                        let start_of_line = start_of_line(source, line_number);

                        let _ = write!(string, "{}{}",
                            " ".repeat({
                                let mut count = 0;
                                for (index, i) in line.chars().enumerate() {
                                    if count >= range.start() as usize - start_of_line {
                                        count = index;
                                        break
                                    }
                                    count += i.len_utf8();
                                }
                                count
                            }),
                            "^".repeat({
                                if end_line == line_number {
                                    (range.end() - range.start()) as usize + 1
                                } else {
                                    dbg!(line, range, start_of_line, line_number);
                                    line.len() - (range.start() as usize - start_of_line) + 1
                                }
                            }).colour(colour),
                        );

                        
                    } else if line_number == end_line {
                        let _ = write!(string, "{}",
                            "^".repeat({
                                let start_of_end = start_of_line(source, end_line);
                                range.end() as usize - start_of_end + 1
                            }).colour(colour),
                        );

                       
                    } else {
                        let _ = write!(string, "{}",
                            "^".repeat(line.len()).colour(colour),
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
    colour: Colour,
}


impl<T: ErrorBuilder> ErrorBuilder for Highlight<T> {
    fn flatten(self, vec: &mut Vec<ErrorOption>) {
        self.parent.flatten(vec);

        vec.push(ErrorOption::Highlight { range: self.range, note: self.note, colour: self.colour })
    }
}


impl<T: ErrorBuilder> Highlight<T> {
    pub fn note(mut self, note: String) -> Self {
        self.note = Some(note);
        self
    }

    pub fn colour(mut self, colour: Colour) -> Self {
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
}


pub struct CompilerError(ErrorCode);


impl CompilerError {
    pub fn new(id: ErrorCode) -> CompilerError {
        CompilerError(id)
    }
}


impl ErrorBuilder for CompilerError {
    fn flatten(self, vec: &mut Vec<ErrorOption>) {
        let mut string = String::new();

        let _ = write!(string, "error[{:>03}]", self.0 as usize);

        string = string.red().bold().to_string();
                
        let _ = writeln!(string, " {}", self.0.msg().white().bold());
        
        vec.push(ErrorOption::Text(string))
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