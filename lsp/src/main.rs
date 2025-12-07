#![feature(backtrace_frames)]
mod json;

use std::{backtrace::Backtrace, cell::RefCell, collections::HashMap, fs::File, io::{Read, Write}, str::FromStr, sync::{Mutex, RwLock}, time::{Duration, Instant}};

use chrono::Local;
use color_eyre::owo_colors::colored;
use common::string_map::StringIndex;
use dashmap::DashMap;
use margarine::{Arena, FileData, SourceRange, StringMap};
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



/*

struct Backend {
    client: tower_lsp::Client,
    docs: DashMap<Url, LspFile>,
}


struct LspFile {
    rope: Rope,
    file: FileData,
}


#[tower_lsp::async_trait]
impl tower_lsp::LanguageServer for Backend {
    async fn initialize(&self, _: tower_lsp::lsp_types::InitializeParams) -> tower_lsp::jsonrpc::Result<tower_lsp::lsp_types::InitializeResult> {
        tracing::info!("initializing margarine server");
        self.client.show_message(tower_lsp::lsp_types::MessageType::INFO, "hey").await;
        Ok(tower_lsp::lsp_types::InitializeResult {
            capabilities: tower_lsp::lsp_types::ServerCapabilities {
                text_document_sync: Some(tower_lsp::lsp_types::TextDocumentSyncCapability::Kind(
                    tower_lsp::lsp_types::TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            server_info: None,
            offset_encoding: None,
        })
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        Ok(())
    }


    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!("file opened");
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: params.text_document.text,
            version: params.text_document.version,
            language_id: String::from("mar"),
        })
        .await
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.on_change(TextDocumentItem {
            text: params.content_changes[0].text.clone(),
            uri: params.text_document.uri,
            version: params.text_document.version,
            language_id: String::from("mar"),
        })
        .await
    }


    async fn hover(&self, params: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
        error!("Got a textDocument/hover request, but it is not implemented");
        todo!()
    }

}


impl Backend {
    pub async fn on_hover(&self, params: HoverParams) {
        let rope = self.docs.get(&params.text_document_position_params.text_document.uri).unwrap();
        rope.to_string();
    }
    

    pub async fn on_change(&self, params: TextDocumentItem) {
        info!("uri: {}", params.uri);
        info!("version: {}", params.version);

        let rope = ropey::Rope::from_str(&params.text);
        self.docs.insert(params.uri.clone(), rope);

        let rope = self.docs.get(&params.uri).unwrap();

        let arena = Arena::new();
        let mut sm = StringMap::new(&arena);

        let file = FileData::new(params.text, sm.insert(params.uri.as_str()), margarine::Extension::Mar);
        let result = margarine::lex(&file, &mut sm, 0);

        let diagnostics = result.1.iter()
            .filter_map(|e| {
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
                        SourceRange::new(source_range.start(), source_range.end() + 1))
                    },


                    lexer::errors::Error::NumberTooLarge(source_range) => {
                        (format!("Syntax: Error: constant number is too large to represent in an i64"),
                         SourceRange::new(source_range.start(), source_range.end()))
                    },


                    lexer::errors::Error::TooManyDots(source_range) => {
                        (format!("Syntax Error: number has too many dots"),
                        SourceRange::new(source_range.start(), source_range.end() + 1))
                    },
                };


                Some(Diagnostic::new_simple(
                    to_range(&rope, span),
                    message
                ))
            })
            .collect();

        self.client.publish_diagnostics(params.uri, diagnostics, Some(params.version)).await;
    }
}


fn to_range(rope: &Rope, src: SourceRange) -> Range {
    Range::new(
        offset_to_position(src.start(), rope).unwrap(),
        offset_to_position(src.end(), rope).unwrap(),
    )
}


fn offset_to_position(offset: u32, rope: &Rope) -> Option<Position> {
    let offset = offset as usize;
    let line = rope.try_byte_to_line(offset).ok()?;
    let first_char_of_line = rope.try_line_to_byte(line).ok()?;
    let column = offset - first_char_of_line;
    Some(Position::new(line as u32, column as u32))
}


#[tokio::main]
async fn main() {
    let _guard = tracing_init();
    set_panic_hook();

    info!("starting LSP server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = tower_lsp::LspService::new(|client| Backend { client, docs: HashMap::new() });
    tower_lsp::Server::new(stdin, stdout, socket)
        .serve(service)
        .await;
}

*/


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
    string_map: StringMap<'static>,

    files: HashMap<StringIndex, LspFile>,
}


struct LspFile {
    root: StringIndex,
    module_name: StringIndex,
    data: FileData,
    version: u32,
    rope: Rope,
    modules: Vec<StringIndex>,
}


impl Lsp {
    fn resolve_file(&mut self, path_index: StringIndex) -> &mut LspFile {
        if !self.files.contains_key(&path_index) {
            self.resolve_file_ex(path_index)
        } else {
            self.files.get_mut(&path_index).unwrap()
        }
    }

    fn resolve_file_ex(&mut self, path_index: StringIndex) -> &mut LspFile {
        let path = self.string_map.get(path_index);

        trace!("margarine-lsp/resolve-file: path = '{path}'");

        let parent = path.bytes().enumerate().rev().find(|b| b.1 == b'/');

        let mut file = LspFile {
            root: path_index,
            data: FileData::new(String::new(), path_index, margarine::Extension::Mar),
            version: 0,
            rope: Rope::new(),
            modules: vec![],
            module_name: StringIndex::MAX,
        };



        if let Some(parent) = parent {
            let parent_path = &path[..parent.0];
            let url_parent_path = format!("{parent_path}.mar");
            trace!("path {parent_path}");
            let parent_path = Url::from_str(&url_parent_path).unwrap();
            let parent_path = parent_path.to_file_path().unwrap();
            let parent_path = parent_path.to_string_lossy();

            let module_name = &path[(parent.0+1)..path.len()-".mar".len()];
            let module_name = self.string_map.insert(module_name);

            file.module_name = module_name;


            trace!("has parent {parent_path}");
            let parent_path_index = self.string_map.insert(&*url_parent_path);

            if std::fs::exists(&*parent_path).unwrap() {
                trace!("exists");
                let parent = self.resolve_file(parent_path_index);
                let parent = self.files.get(&parent_path_index).unwrap();


                if parent.modules.contains(&module_name) {
                    trace!("is included in {parent_path}");
                    file.root = parent.root;
                }

                trace!("looking for {}", self.string_map.get(module_name));
                for module in &parent.modules {
                    trace!("includes: {}", self.string_map.get(*module));
                }
            } else {
                trace!("parent path doesn't exist");
            }
        } else {
            let module_name = &path[..path.len()-".mar".len()];
            let module_name = self.string_map.insert(module_name);

            file.module_name = module_name;
        }


        // try to figure out modules
        'b: {
            let path = Url::from_str(&path).unwrap();
            let path = path.to_file_path().unwrap();
            let Ok(data) = std::fs::read_to_string(path)
            else { break 'b };

            let fd = FileData::new(data, path_index, margarine::Extension::Mar);

            let (tokens, _) = margarine::lex(&fd, &mut self.string_map, 0);
            
            let arena = Arena::new();
            let mut ast = AST::new(&arena);
            let (_, modules, _) = margarine::parse(tokens, 0, &arena, &mut self.string_map, &mut ast);

            for module in modules {
                let Decl::ImportFile { name, .. } = ast.decl(module.1)
                else { continue };

                file.modules.push(name);
            }
        }


        self.files.insert(path_index, file);
        self.files.get_mut(&path_index).unwrap()
    }


    pub fn on_change(&mut self, version: u32, path: &str, text: &str) {
        trace!("margarine-lsp/on-change: version = {version}, path = {path}");


        let path = self.string_map.insert(path);

        let fd = FileData::new(text.to_string(), path, margarine::Extension::Mar);

        // fanks rust. very cool
        let _ = self.resolve_file(path);
        let file = self.files.get_mut(&path).unwrap();

        if file.version < version {
            file.data = fd;
            file.rope = text.into();
            file.version = version;
        }


        trace!("root of '{}' is {}", self.string_map.get(path), self.string_map.get(file.root));


        let (tokens, result) = margarine::lex(&file.data, &mut self.string_map, 0);


        let mut diagnostics = vec![];
        result
            .iter()
            .filter_map(|e| {
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


                Some(Diagnostic::new_simple(
                    to_range(&file.rope, span),
                    message,
                ))
            })

            .for_each(|d| diagnostics.push(d));


        let arena = Arena::new();
        let mut ast = AST::new(&arena);
        let (_, _, result) = margarine::parse(tokens, 0, &arena, &mut self.string_map, &mut ast);


        result
            .iter()
            .filter_map(|e| {
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
                        let path = self.string_map.get(*path);
                        (format!("file '{path}' doesn't exist"), source)
                    },
                };

                Some(Diagnostic::new_simple(
                    to_range(&file.rope, span),
                    message,
                ))
            })

        .for_each(|d| diagnostics.push(d));


        let uri = self.string_map.get(path);
        self.send_diagnostics(uri, &diagnostics);


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

                self.on_change(version, path, text);
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

                self.on_change(version, path, text.as_string());
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


    

    fn send_diagnostics(&mut self, uri: &str, diag: &[Diagnostic]) {
        trace!("margarine-lsp/send_diagnostics: uri = {uri}, len(diag) = {}", diag.len());

        let mut result = String::with_capacity(diag.len() * 64);

        sti::write!(&mut result, "{{\"uri\":\"{uri}\",\"diagnostics\":[");

        for (i, d) in diag.iter().enumerate() {
            if i != 0 { _ = sti::write!(&mut result, ","); }
            sti::write!(&mut result, "{{");
            sti::write!(&mut result, "\"range\":{{\"start\":{{\"line\":{},\"character\":{}}},\"end\":{{\"line\":{},\"character\":{}}}}}",
                d.range.start.line, d.range.start.character, d.range.end.line, d.range.end.character);
            sti::write!(&mut result, ",\"message\":{:?}", d.message);
            sti::write!(&mut result, "}}");
        }

        sti::write!(&mut result, "]}}");

        self.send_notification("textDocument/publishDiagnostics", json::Value::Encoded(&result));
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


    fn send_notification(&mut self, method: &str, params: json::Value) {
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
        string_map: StringMap::new(arena),
        arena,
        files: HashMap::new(),
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
