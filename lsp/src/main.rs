#![feature(backtrace_frames)]
mod json;

use std::{backtrace::Backtrace, cell::RefCell, collections::HashMap, fs::File, io::{Read, Write}, path::Path, str::FromStr, sync::{Mutex, RwLock}, time::{Duration, Instant}};

use chrono::Local;
use color_eyre::owo_colors::colored;
use common::string_map::{self, StringIndex};
use dashmap::DashMap;
use margarine::{Arena, Compiler, FileData, SourceRange, StringMap};
use parser::nodes::{decl::Decl, AST};
use ropey::Rope;
use sti::{ext::FromIn, key::Key};
use tower_lsp::lsp_types::{CodeLensParams, Diagnostic, DidChangeTextDocumentParams, DidOpenTextDocumentParams, Hover, HoverParams, MessageType, Position, Range, TextDocumentItem, Url};
use tracing::{debug, error, info, level_filters::LevelFilter, trace, warn};
use tracing_appender::{non_blocking, rolling};
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_appender::non_blocking::WorkerGuard;

thread_local! {
    static BUFFER : RefCell<String> = RefCell::new(String::new());
}



#[macro_export]
macro_rules! send {
    ($($arg:tt)*) => {{
        BUFFER.with_borrow_mut(|buf| {
            buf.clear();
            _ = ::core::fmt::Write::write_fmt(buf, ::core::format_args!($($arg)*));
            print!("Content-Length: {}\r\n\r\n{}", buf.len(), buf);
            std::io::stdout().flush().unwrap();
        });
    }};
}


fn to_range(rope: &Rope, src: SourceRange) -> Range {
    Range::new(
        offset_to_position(src.start(), rope).unwrap(),
        {
            let mut pos = offset_to_position(src.end(), rope).unwrap_or_else(|| offset_to_position(src.end(), rope).unwrap());
            pos.character += 1;
            pos
        },
    )
}


fn offset_to_position(offset: u32, rope: &Rope) -> Option<Position> {
    let offset = (offset as usize).min(rope.len_bytes() - 1);
    let line = rope.try_byte_to_line(offset).ok()?;
    let first_char_of_line = rope.try_line_to_byte(line).ok()?;
    
    // Slice the line up to the offset
    let line_slice = rope.byte_slice(first_char_of_line..offset);
    
    // Count UTF-16 code units
    let utf16_col : usize = line_slice.chars().map(|c| c.len_utf16()).sum();
    
    Some(Position::new(line as u32, utf16_col as u32))
}


const URI_PREFIX : &str = "file://";


struct Lsp {
    message: Vec<u8>,
    initialized: bool,

    next_request_id: u32,
    active_requests: HashMap<u32, ()>,

    arena: &'static Arena,

    files: HashMap<Url, LspFile>,
    compiler: Compiler<'static>,
}


struct LspFile {
    root: StringIndex,
    module_name: StringIndex,
    file_path: StringIndex,
    version: u32,
    rope: Rope,
    modules: Vec<StringIndex>,
}


impl Lsp {
    /// Path index is in normal file system
    fn resolve_file(&mut self, path: &Url) -> &mut LspFile {
        if !self.files.contains_key(&path) {
            self.resolve_file_ex(path)
        } else {
            self.files.get_mut(&path).unwrap()
        }
    }


    /// Path index is in normal file system
    fn resolve_file_ex(&mut self, uri_path: &Url) -> &mut LspFile {
        trace!("margarine-lsp/resolve-file: path = '{uri_path}'");

        let path = uri_path.to_file_path().unwrap();
        let path = path.to_string_lossy();
        let path = &*path;
        let module_path = &path[..path.len()-".mar".len()];
        let path_index = self.compiler.string_map.insert(module_path);
        let parent = path.bytes().enumerate().rev().find(|b| b.1 == b'/');

        let data = std::fs::read_to_string(path).unwrap();

        let mut file = LspFile {
            root: path_index,
            version: 0,
            rope: Rope::from_str(&data),
            modules: vec![],
            module_name: StringIndex::MAX,
            file_path: path_index,
        };



        if let Some(parent) = parent {
            let parent_path = &path[..parent.0];
            let parent_path = format!("{parent_path}.mar");
            trace!("path {parent_path}");

            let module_name = &path[(parent.0+1)..path.len()-".mar".len()];
            let module_name = self.compiler.string_map.insert(module_name);

            file.module_name = module_name;


            trace!("has parent {parent_path}");

            if std::fs::exists(&*parent_path).unwrap() {
                trace!("exists");
                let parent_uri_path = Url::from_file_path(&parent_path).unwrap();
                let _ = self.resolve_file(&parent_uri_path);
                let parent = self.files.get(&parent_uri_path).unwrap();


                if parent.modules.contains(&module_name) {
                    trace!("is included in {parent_path}");
                    file.root = parent.root;
                }

                trace!("looking for {}", self.compiler.string_map.get(module_name));
                for module in &parent.modules {
                    trace!("includes: {}", self.compiler.string_map.get(*module));
                }
            } else {
                trace!("parent path doesn't exist");
            }
        } else {
            let module_name = &path[..path.len()-".mar".len()];
            let module_name = self.compiler.string_map.insert(module_name);

            file.module_name = module_name;
        }


        // try to figure out modules
        {
            let fd = FileData::new(data, path_index, margarine::Extension::Mar);

            let (tokens, _) = margarine::lex(&fd, &mut self.compiler.string_map, 0);
            
            let arena = Arena::new();
            let mut ast = AST::new(&arena);
            let (_, modules, _) = margarine::parse(tokens, 0, &arena, &mut self.compiler.string_map, &mut ast);

            for module in modules {
                let Decl::ImportFile { name, .. } = ast.decl(module.1)
                else { continue };

                file.modules.push(name);
            }

            trace!("registering {}", self.compiler.string_map.get(fd.name()));
            self.compiler.files.register(fd);
        }


        self.files.insert(uri_path.clone(), file);
        self.files.get_mut(&uri_path).unwrap()
    }


    pub fn on_change(&mut self, version: u32, uri_path: Url, text: &str) {
        trace!("margarine-lsp/on-change: version = {version}, uri_path = {uri_path}");


        // fanks rust. very cool
        let _ = self.resolve_file(&uri_path);
        let file = self.files.get_mut(&uri_path).unwrap();

        if file.version < version {
            file.rope = text.into();
            file.version = version;
        }


        trace!("root of '{}' is {}", self.compiler.string_map.get(file.file_path), self.compiler.string_map.get(file.root));
        let fd = FileData::new(text.to_string(), file.file_path, margarine::Extension::Mar);

        let compiler = &mut self.compiler;
        compiler.files.register(fd);

        let arena = Arena::new();
        let mut result = compiler.run(&arena, file.root);

        trace!("{:#?}", &result.errors);

        let mut diagnostics = vec![];

        result
            .errors
            .lexer_errors
            .iter()
            .flatten()
            .filter_map(|(_, e)| {
                let (message, span) = match e {
                    lexer::errors::Error::InvalidCharacter { character, position } => {
                        (format!("Syntax Error: invalid character '{character}'"), *position)
                    },


                    lexer::errors::Error::UnterminatedString(source_range) => {
                        (format!("Syntax Error: unterminated string"), *source_range)
                    },


                    lexer::errors::Error::CorruptUnicodeEscape(source_range) => {
                        (format!("Syntax Error: corrupt unicode escape"), *source_range)
                    },


                    lexer::errors::Error::InvalidUnicodeCharacter(source_range) => {
                        (format!("Syntax Error: invalid unicode escape"),
                        SourceRange::new(source_range.start(), source_range.end()))
                    },


                    lexer::errors::Error::NumberTooLarge(source_range) => {
                        (format!("Syntax: Error: constant number is too large to represent in an i64"),
                         SourceRange::new(source_range.start(), source_range.end()))
                    },


                    lexer::errors::Error::TooManyDots(source_range) => {
                        (format!("Syntax Error: number has too many dots"),
                        SourceRange::new(source_range.start(), source_range.end()))
                    },
                };


                Some((
                    span,
                    message,
                ))
            })

            .for_each(|d| diagnostics.push(d));


        result
            .errors
            .parser_errors
            .iter()
            .flatten()
            .filter_map(|(_, e)| {
                let (message, &span) = match e {
                    parser::errors::Error::ExpectedLiteralString { source, token } => {
                        (format!("expected a string literal, found '{token:?}'"), source)
                    },


                    parser::errors::Error::ExpectedLiteralBool { source, token } => {
                        (format!("expected a boolean literal, found '{token:?}'"), source)
                    },


                    parser::errors::Error::ExpectedIdentifier { source, token } => {
                        (format!("expected an identifier, found '{token:?}'"), source)
                    },


                    parser::errors::Error::UnexpectedToken(source_range) => {
                        (format!("unexpected token"), source_range)
                    },


                    parser::errors::Error::ExpectedXFoundY { source, found, expected } => {
                        (format!("expected '{expected:?}' found '{found:?}';"), source)

                    },


                    parser::errors::Error::ExpectedXFoundYMulti { source, found, expected } => {
                        let mut str = String::new();
                        str.push_str("expected ");

                        for (i, e) in expected.iter().enumerate() {
                            if i == expected.len() - 1 {
                                str.push_str(" or ");
                            } else if i != 0 {
                                str.push_str(", ");
                            }

                            sti::write!(&mut str, "'{:?}'", e);
                        }

                        sti::write!(&mut str, " but found '{found:?}'");

                        (str, source)
                    },


                    parser::errors::Error::DeclarationOnlyBlock { source } => {
                        (format!("this block only supports declarations"), source)
                    },


                    parser::errors::Error::FileDoesntExist { source, path } => {
                        let path = self.compiler.string_map.get(*path);
                        (format!("file '{path}' doesn't exist"), source)
                    },
                };




                Some((
                    span,
                    message,
                ))
            })

            .for_each(|d| diagnostics.push(d));

        let sm = &self.compiler.string_map;
        let syms = &mut result.syms;

        result
            .errors
            .sema_errors
            .iter()
            .filter_map(|e| {
                let (message, &span) = match e {
                    semantic_analysis::errors::Error::IteratorFunctionInvalidSig(source_range) => {
                        (
                            format!(
                                "signature must match 'fn __next__(&self): Option<[type]>'",
                            ),
                            source_range,
                        )

                    },


                    semantic_analysis::errors::Error::InvalidCast { range, from_ty, to_ty } => {
                        (
                            format!(
                                "invalid cast from '{}' to '{}'",
                                from_ty.display(sm, syms),
                                to_ty.display(sm, syms),
                            ),
                            range,
                        )
                    },


                    semantic_analysis::errors::Error::InvalidValueForAttr { attr, value, expected } => {
                        (
                            format!("attribute '{}' expected '{expected}'", sm.get(attr.1)),
                            value
                        )
                    },


                    semantic_analysis::errors::Error::UnknownAttr(source_range, string_index) => {
                        (
                            format!(
                                "'{}' is not a valid attribute",
                                sm.get(*string_index),
                            ),
                            source_range,
                        )
                    },


                    semantic_analysis::errors::Error::NameIsAlreadyDefined { source, name } => {
                        (
                            format!(
                                "name '{}' is already defined",
                                sm.get(*name),
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::UnknownType(string_index, source_range) => {
                        (
                            format!(
                                "unknown type '{}'",
                                sm.get(*string_index),
                            ),
                            source_range,
                        )
                    },


                    semantic_analysis::errors::Error::FunctionBodyAndReturnMismatch { header, item, return_type, body_type } => {
                        (
                            format!(
                                "function body returns '{}'",
                                return_type.display(sm, syms),
                            ),
                            item,
                        )
                    },


                    semantic_analysis::errors::Error::OutsideOfAFunction { source } => {
                        (
                            format!(
                                "this can't be outside of a function",
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::InvalidType { source, found, expected } => {
                        (
                            format!(
                                "expected '{}' found '{}'",
                                expected.display(sm, syms),
                                found.display(sm, syms),
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::DuplicateField { declared_at, error_point } => {
                        (
                            format!(
                                "duplicate field",
                            ),
                            error_point,
                        )
                    },


                    semantic_analysis::errors::Error::DuplicateArg { declared_at, error_point } => {
                        (
                            format!(
                                "duplicate argument",
                            ),
                            error_point,
                        )
                    },


                    semantic_analysis::errors::Error::VariableValueAndHintDiffer { value_type, hint_type, source } => {
                        (
                            format!(
                                "..is of type '{}' but the hint expects '{}'",
                                value_type.display(sm, syms),
                                hint_type.display(sm, syms),
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::VariableValueNotTuple(source_range) => {
                        (
                            format!(
                                "expected a tuple",
                            ),
                            source_range,
                        )
                    },


                    semantic_analysis::errors::Error::VariableTupleAndHintTupleSizeMismatch(source_range, found, expected) => {
                        (
                            format!(
                                "expected a tuple of size '{expected}' but found a tuple of size '{found}'",
                            ),
                            source_range,
                        )
                    },


                    semantic_analysis::errors::Error::VariableNotFound { name, source } => {
                        (
                            format!(
                                "variable '{}' not found",
                                sm.get(*name),
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::InvalidBinaryOp { operator, lhs, rhs, source } => {
                        (
                            format!(
                                "can't apply binary op '{}' between '{}' and '{}'",
                                operator,
                                lhs.display(sm, syms),
                                rhs.display(sm, syms),
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::InvalidUnaryOp { operator, rhs, source } => {
                        (
                            format!(
                                "can't apply unary op '{}' on '{}'",
                                operator,
                                rhs.display(sm, syms),
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::IfMissingElse { body } => {
                        (
                            format!(
                                "main branch returns '{}' but there's no else branch",
                                body.1.display(sm, syms),
                            ),
                            &body.0,
                        )
                    },


                    semantic_analysis::errors::Error::IfBodyAndElseMismatch { body, else_block } => {
                        (
                            format!(
                                "main branch returns '{}' but this returns '{}'",
                                body.1.display(sm, syms),
                                else_block.1.display(sm, syms),
                            ),
                            &else_block.0,
                        )
                    },


                    semantic_analysis::errors::Error::MatchValueIsntEnum { source, typ } => {
                        (
                            format!(
                                "'{}' is not an enum",
                                typ.display(sm, syms),
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::MatchBranchesDifferInReturnType { initial_source, initial_typ, branch_source, branch_typ } => {
                        (
                            format!(
                                "previously returned '{}' but this returns '{}'",
                                initial_typ.display(sm, syms),
                                branch_typ.display(sm, syms),
                            ),
                            branch_source,
                        )
                    },


                    semantic_analysis::errors::Error::DuplicateMatch { declared_at, error_point } => {
                        (
                            format!(
                                "..is already handled",
                            ),
                            error_point,
                        )
                    },


                    semantic_analysis::errors::Error::InvalidMatch { name, range, value } => {
                        (
                            format!(
                                "there's no variant named '{}' in '{}'",
                                sm.get(*name),
                                value.display(sm, syms),
                            ),
                            range,
                        )
                    },


                    semantic_analysis::errors::Error::MissingMatch { name, range } => {
                        let mut msg = format!("missing variants: ");
                        let mut is_first = true;
                        for n in name.iter() {
                            if !is_first {
                                sti::write!(&mut msg, ", ");
                            }

                            is_first = false;
                            sti::write!(&mut msg, "{}", sm.get(*n));
                        }


                        (
                            msg,
                            range,
                        )
                    },


                    semantic_analysis::errors::Error::ValueIsntAnIterator { ty, range } => {
                        (
                            format!(
                                "'{}' isn't an iterator",
                                ty.display(sm, syms),
                            ),
                            range,
                        )

                    },


                    semantic_analysis::errors::Error::StructCreationOnNonStruct { source, typ } => {
                        (
                            format!(
                                "'{}' isn't a struct type",
                                typ.display(sm, syms),
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::FieldAccessOnNonEnumOrStruct { source, typ } => {
                        (
                            format!(
                                "'{}' is an opaque type",
                                typ.display(sm, syms)
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::FieldDoesntExist { source, field, typ } => {
                        (
                            format!(
                                "'{}' doesn't have a field named '{}'",
                                typ.display(sm, syms),
                                sm.get(*field),
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::MissingFields { source, fields } => {
                        let mut msg = format!("missing fields: ");
                        let mut is_first = true;
                        for (_, n) in fields {
                            if !is_first {
                                sti::write!(&mut msg, ", ");
                            }

                            is_first = false;
                            sti::write!(&mut msg, "{}", sm.get(*n));
                        }

                        (
                            msg,
                            source,
                        )


                    },

                    semantic_analysis::errors::Error::FunctionArgsMismatch { source, sig_len, call_len } => {
                        (
                            format!(
                                "expected '{}' args but found '{}",
                                sig_len,
                                call_len,
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::NamespaceNotFound { source, namespace } => {
                        (
                            format!(
                                "namespace '{}' not found",
                                sm.get(*namespace)
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::ValueUpdateTypeMismatch { lhs, rhs, source } => {
                        (
                            format!(
                                "lhs is '{}' while the rhs is '{}'",
                                lhs.display(sm, syms),
                                rhs.display(sm, syms),
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::ContinueOutsideOfLoop(source_range) => {
                        (
                            format!(
                                "continue outside of a loop",
                            ),
                            source_range,
                        )
                    },


                    semantic_analysis::errors::Error::BreakOutsideOfLoop(source_range) => {
                        (
                            format!(
                                "break outside of a loop",
                            ),
                            source_range,
                        )
                    },


                    semantic_analysis::errors::Error::CantUnwrapOnGivenType(source_range, sym) => {
                        (
                            format!(
                                "can't unwrap on '{}'",
                                sym.display(sm, syms),
                            ),
                            source_range,
                        )
                    },


                    semantic_analysis::errors::Error::CantTryOnGivenType(source_range, sym) => {
                        (
                            format!(
                                "can't try on '{}'",
                                sym.display(sm, syms),
                            ),
                            source_range,
                        )
                    },


                    semantic_analysis::errors::Error::FunctionDoesntReturnAnOption { source, func_typ } => {
                        (
                            format!(
                                "function returns '{}' but an option is required",
                                func_typ.display(sm, syms),
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::FunctionDoesntReturnAResult { source, func_typ } => {
                        (
                            format!(
                                "function returns '{}' but a result is required",
                                func_typ.display(sm, syms),
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::FunctionReturnsAResultButTheErrIsntTheSame { source, func_source, func_err_typ, err_typ } => {
                        (
                            format!(
                                "function returns a result with '{}' as an error but '{}' is required",
                                func_err_typ.display(sm, syms),
                                err_typ.display(sm, syms),
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::ReturnAndFuncTypDiffer { source, func_source, typ, func_typ } => {
                        (
                            format!(
                                "function returns '{}', but tried to return '{}'",
                                func_typ.display(sm, syms),
                                typ.display(sm, syms)
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::AssignIsNotLHSValue { source } => {
                        (
                            format!(
                                "..is not a valid LHS value",
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::UnableToInfer(source_range) => {
                        (
                            format!(
                                "unable to infer type",
                            ),
                            source_range,
                        )
                    },


                    semantic_analysis::errors::Error::InvalidRange { source, ty } => {
                        (
                            format!(
                                "can't create a range out of '{}'",
                                ty.display(sm, syms),
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::ImplOnGeneric(source_range) => {
                        (
                            format!(
                                "impl on a complete generic is not supported",
                            ),
                            source_range,
                        )
                    },


                    semantic_analysis::errors::Error::GenericLenMismatch { source, found, expected } => {
                        (
                            format!(
                                "expected '{expected}' generics but found '{found}'",
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::CantUseHoleHere { source } => {
                        (
                            format!(
                                "a hole isn't supported here",
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::NameIsReservedForFunctions { source } => {
                        (
                            format!(
                                "this name is reserved for special functions",
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::IndexOnNonList(source_range, sym) => {
                        (
                            format!(
                                "can't inedx '{}'",
                                sym.display(sm, syms),
                            ),
                            source_range,
                        )
                    },


                    semantic_analysis::errors::Error::CallOnNonFunction { source } => {
                        (
                            format!(
                                "..is not a function",
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::CallOnField { source, .. } => {
                        (
                            format!(
                                "..is a field. add parenthesis around it",
                            ),
                            source,
                        )
                    },


                    semantic_analysis::errors::Error::Bypass => return None,
                };




                Some((
                    span,
                    message,
                ))
            })

            .for_each(|d| diagnostics.push(d));

        self.send_diagnostics(diagnostics);


    }
}



impl Lsp {
    fn process_bytes(&mut self, bytes: &[u8]) -> bool {
        trace!("margarine-lsp/process_bytes: processing {} bytes", bytes.len());
        self.message.extend_from_slice(bytes);

        let arena = Arena::new();
        
        // Try to parse messages
        loop {
            // Look for \r\n\r\n (header separator)
            let header_end = self.message.windows(4).position(|w| w == b"\r\n\r\n");
            
            if header_end.is_none() {
                // Not enough data yet
                return true;
            }
            
            let header_end = header_end.unwrap();
            let header_bytes = &self.message[..header_end];
            let body_start = header_end + 4;
            
            // Parse headers
            let header_str = match std::str::from_utf8(header_bytes) {
                Ok(s) => s,
                Err(_) => {
                    error!("Invalid UTF-8 in headers");
                    self.message.clear();
                    return true;
                }
            };

            trace!("margarine-lsp/process_bytes: header_str={header_str}");
            
            let mut content_length = None;
            for line in header_str.lines() {
                if let Some(value) = line.strip_prefix("Content-Length: ") {
                    if let Ok(len) = value.parse::<usize>() {
                        content_length = Some(len);
                    }
                }
            }
            
            let content_length = match content_length {
                Some(len) => len,
                None => {
                    error!("No Content-Length header found");
                    self.message.clear();
                    return true;
                }
            };
            
            // Check if we have the full body
            if self.message.len() < body_start + content_length {
                // Not enough data yet
                return true;
            }
            
            // Extract the body
            let body_bytes = &self.message[body_start..body_start + content_length];
            let body_bytes = sti::vec::Vec::from_slice_in(&arena, body_bytes);

            let content = json::parse(&arena, &body_bytes);
            debug!("Received message: {:?}", content);

            let time = Instant::now();
            let content = content.unwrap();
            self.handle_message(content);

            info!("margarine-lsp: took {:?} to respond", time.elapsed());
            
            // Remove processed message from buffer
            self.message.drain(..body_start + content_length);
        }
    }

    
    fn handle_message<'a>(&mut self, msg: json::Value<'a>) {
        trace!("margarine-lsp/handle_message");

        assert_eq!(msg["jsonrpc"], "2.0".into());

        if let Some(method) = msg.get("method") {
            let method = method.as_string();

            let params = msg.get("params").unwrap_or(json::Value::Null);
            assert!(params.is_object() || params.is_array() || params.is_null());

            if let Some(id) = msg.get("id") {
                let id = id.as_number();
                assert_eq!(id as u32 as f64, id);
                let id = id as u32;

                self.handle_request(method, id, params);
            } else {
                self.handle_notif(method, params);
            }

        } else {
            let id = msg["id"].as_number();
            assert_eq!(id as u32 as f64, id);
            let id = id as u32;

            let result =
            if let Some(result) = msg.get("result") { Ok(result) }
            else { Err(msg["error"]) };

            self.handle_response(id, result);
        }
    }

    
    fn handle_request(&mut self, method: &str, id: u32, params: json::Value) -> bool {
        trace!("margarine-lsp/handle_notif: method = '{method}', id = {id}");
        if !self.initialized {
            if method != "initialize" {
                error!("received '{method}' before 'initialize'");
                return true;
            }

            self.send_response(id, Ok(json::Value::Object(&[
                ("capabilities", json::Value::Object(&[
                    ("positionEncoding", "utf-16".into()),

                    ("textDocumentSync", 1.0.into()),
                    ("semanticTokensProvider", json::Value::Object(&[
                        ("legend", json::Value::Object(&[
                            ("tokenTypes", json::Value::Array(&[
                                "error".into(),
                                "comment".into(),
                                "keyword".into(),
                                "punctuation".into(),
                                "operator".into(),
                                "string".into(),
                                "number".into(),
                                "type".into(),
                                "parameter".into(),
                                "variable".into(),
                                "property".into(),
                                "function".into(),
                                "method".into(),
                            ])),
                            ("tokenModifiers", json::Value::Array(&[])),
                        ])),
                        ("full", json::Value::Object(&[
                            ("delta", false.into()),
                        ])),
                    ])),

                    ("hoverProvider", true.into()),
                ])),
            ])));

            self.initialized = true;
            return true;
        }


        match method {
            "shutdown" => {
                self.send_response(id, Ok(json::Value::Null));
            }


            _ => {
                warn!("request not supported. method = '{method}'");
            }
        }


        true
    }


    fn handle_notif(&mut self, method: &str, params: json::Value) -> bool {
        trace!("margarine-lsp/handle_notif: method = '{method}'");


        if !self.initialized {
            error!("received notification for '{method}' before 'initialize'");
            return true;
        }


        match method {
            "exit" => {
                debug!("exit received");
                return false;
            }


            "textDocument/didOpen" => {
                let doc = params["textDocument"];

                let lang = doc["languageId"];
                trace!("languageId = '{lang}'");

                if lang != "margarine".into() {
                    return true;
                }


                let path = doc["uri"].as_string();
                let text = doc["text"].as_string();
                let version = doc["version"].as_number();
                assert_eq!(version as u32 as f64, version);
                let version = version as u32;

                trace!("uri = '{path}', version = {version}");

                self.on_change(version, Url::from_str(path).unwrap(), text);
            }



            "textDocument/didChange" => {
                let doc = params["textDocument"];
                let changes = params["contentChanges"].as_array();

                let path = doc["uri"].as_string();
                let version = doc["version"].as_number();
                assert_eq!(version as u32 as f64, version);
                let version = version as u32;


                let text = changes[0]["text"];
                trace!("uri = '{path}', version = {version}, text = '{text}'");

                self.on_change(version, Url::from_str(path).unwrap(), text.as_string());
            }



            _ => {
                warn!("notification not supported. method = '{method}'");
            }
        }


        true
    }


    fn handle_response(&mut self, id: u32, params: Result<json::Value, json::Value>) -> bool {
        trace!("margarine-lsp/handle_response: id = {id}");


        if !self.initialized {
            error!("received response before 'initialize'");
            return true;
        }


        warn!("responded to '{id}' with {params:?}. unfortunately, i do not care");
        true
    }


    

    fn send_diagnostics(&mut self, diags: Vec<(SourceRange, String)>) {
        let mut file_diags = HashMap::new();


        let mut result = String::with_capacity(diags.len() * 64);

        for (span, msg) in diags {
            let (file, offset) = span.file(self.compiler.files.files());
            let span = span.base(offset);
            let path = Path::new(self.compiler.string_map.get(file.name()));
            let path = path.with_extension("mar");
            let uri = Url::from_file_path(path).unwrap();

            if !file_diags.contains_key(&uri) {
                file_diags.insert(uri.clone(), vec![]);
            }

            let vec = file_diags.get_mut(&uri).unwrap();

            trace!("file {}", uri);
            let rope = &self.resolve_file(&uri).rope;
            vec.push(Diagnostic::new_simple(to_range(rope, span), msg));
        }


        for (uri, _) in &self.files {
            let diags = 
            if let Some(diags) = file_diags.remove(uri) { diags }
            else { vec![] };
            trace!("margarine-lsp/send_diagnostics: uri = {uri}, len(diag) = {}", diags.len());


            sti::write!(&mut result, "{{\"uri\":\"{uri}\",\"diagnostics\":[");

            for (i, d) in diags.iter().enumerate() {
                if i != 0 { _ = sti::write!(&mut result, ","); }
                sti::write!(&mut result, "{{");
                sti::write!(&mut result, "\"range\":{{\"start\":{{\"line\":{},\"character\":{}}},\"end\":{{\"line\":{},\"character\":{}}}}}",
                    d.range.start.line, d.range.start.character, d.range.end.line, d.range.end.character);
                sti::write!(&mut result, ",\"message\":{:?}", d.message);
                sti::write!(&mut result, "}}");
            }

            sti::write!(&mut result, "]}}");

            self.send_notification("textDocument/publishDiagnostics", json::Value::Encoded(&result));

            result.clear();

        }

    }


    fn send_request(&mut self, method: &str, params: json::Value) {
        let id = self.next_request_id;
        self.next_request_id += 1;
        self.active_requests.insert(id, ());

        trace!("margarine-lsp/send_request: method = '{method}', request_id = {id}");

        send!("{}", json::Value::Object(&[
            ("jsonrpc", "2.0".into()),
            ("id", (id as f64).into()),
            ("method", method.into()),
            ("params", params),
        ]));
    }


    fn send_notification(&self, method: &str, params: json::Value) {
        trace!("margarine-lsp/send_notification: method = '{}'", method);

        send!("{}", json::Value::Object(&[
            ("jsonrpc", "2.0".into()),
            ("method", method.into()),
            ("params", params),
        ]));
    }


    fn send_response(&mut self, id: u32, result: Result<json::Value, json::Value>) {
        trace!("margarine-lsp/send_response: id = {}", id);

        send!("{}", json::Value::Object(&[
            ("jsonrpc", "2.0".into()),
            ("id", (id as f64).into()),
            match result {
                Ok(result) => ("result", result),
                Err(error) => ("error",  error),
            }
        ]));
    }
}



fn main() {
    let _a = tracing_init();
    set_panic_hook();

    let arena = Box::leak(Box::new(Arena::new()));
    let mut lsp = Lsp {
        message: vec![],
        initialized: false,
        next_request_id: 1,
        active_requests: HashMap::new(),
        arena,
        files: HashMap::new(),
        compiler: Compiler::new(arena),
    };

    
    if std::env::args().len() == 2 {
        let path = std::env::args().nth(1).unwrap();
        let code = std::fs::read(&path).unwrap();
        lsp.process_bytes(&code);
        return;
    }

    trace!("hey bish");

    let mut buffer = [0; 128*1024];
    loop {
        match std::io::stdin().lock().read(&mut buffer) {
            Ok(n) => {
                if !lsp.process_bytes(&buffer[..n]) {
                    debug!("exiting");
                    return;
                }

                std::io::stdout().flush().unwrap();
            }

            Err(e) => {
                error!("reading stdin failed. exiting. {e}");
                return;
            }
        }

        // @todo: block
        std::thread::sleep(Duration::from_millis(5));
    }
}


pub fn tracing_init() -> WorkerGuard {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace"));

    let formatting_layer = fmt::layer().pretty().with_writer(std::io::stderr);

    let log_file_name = Local::now().format("%Y-%m-%d").to_string() + ".log";
    let file_appender = rolling::daily("logs", &log_file_name);
    let (non_blocking_appender, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking_appender);

    Registry::default()
        .with(LevelFilter::TRACE)
        .with(ErrorLayer::default())
        .with(formatting_layer)
        .with(file_layer)
        .init();


    guard
}


fn set_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let payload = panic_info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| panic_info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "Unknown panic payload".to_string());

        // Get location
        let location = panic_info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown location".to_string());

        // Capture a backtrace (stable since std::backtrace)
        let bt = Backtrace::force_capture();

        // Log to tracing (will end up in your file layer)
        error!(
            message = %payload,
            location = %location,
            "thread panicked"
        );

        for b in bt.frames() {
            error!("{:?}", b);
        }


        // Also print to stderr (if available). This won't touch stdout.
    }));
}
