#![feature(if_let_guard)]
#![allow(clippy::map_entry)]
use std::{collections::HashMap, fmt::Write};

use common::{SymbolMap, SourceRange, SymbolIndex, FuckMap};
use errors::{Error, CombineIntoError, CompilerError, ErrorCode, ErrorBuilder};
use istd::index_vec;
use lexer::Literal;
use parser::{nodes::{Node, NodeKind, Declaration, StructKind, FunctionArgument, Statement, Expression, UnaryOperator, EnumMapping, MatchMapping}, DataType, Block, DataTypeKind};

index_vec!(Symbols, SymbolId, Symbol);


const GLOBAL_NAMESPACE : &str = "<global>";
const ANY_CAST_ERROR_STRUCT : &str = "any::CastError";


pub fn semantic_analysis<'a>(symbol_map: &'a mut SymbolMap, nodes: &mut Block) -> Result<Infer<'a>, Error> {
    let mut infer = Infer::new(
        symbol_map,
        Symbols::new(),
        Context::new(Scope::new(nodes.range().file(), nodes.range().file())),
        nodes.range().file(),
    );

    infer.analyse_block(nodes, true)?;

    Ok(infer)
}


#[derive(Debug)]
pub struct Infer<'a> {
    pub symbol_map: &'a mut SymbolMap,
    name_to_symbol: FuckMap<SymbolIndex, SymbolId>,
    pub symbols: Symbols,
    namespaces: FuckMap<SymbolIndex, Scope>,
    ctx: Context,
    root_file: SymbolIndex,
    global_scope: Scope,
}


#[derive(Debug)]
pub struct Context { 
    scopes: Vec<Scope>,
    // current scope
    cs: Scope,
    file: SymbolIndex,
}


#[derive(Debug, Default, Clone)]
pub struct Scope {
    symbols: FuckMap<SymbolIndex, (SymbolId, SourceRange)>,
    namespaces: FuckMap<SymbolIndex, (SymbolIndex, SourceRange)>,
    variables: Vec<Variable>,

    mangle_name: SymbolIndex,
    file: SymbolIndex,

    is_function_scope: Option<DataType>,
    // the bool is whether or not it has `break` or not
    is_loop_scope: Option<bool>,
}


#[derive(Clone, Debug, PartialEq)]
pub enum Symbol {
    Structure(Structure),
    Enum(Enum),
    Function(Function),
}


impl Symbol {
    pub fn full_name(&self) -> SymbolIndex {
        match self {
            Symbol::Structure(v) => v.full_name,
            Symbol::Enum(v) => v.full_name,
            Symbol::Function(v) => v.full_name,
        }
    }


    pub fn type_name(&self) -> &'static str {
        match self {
            Symbol::Structure(_) => "structure",
            Symbol::Enum(_) => "enum",
            Symbol::Function(_) => "function",
        }
    }


    pub fn range(&self) -> SourceRange {
        match self {
            Symbol::Structure(v) => v.source,
            Symbol::Enum(v) => v.source,
            Symbol::Function(v) => v.source,
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct Structure {
    name: SymbolIndex,
    full_name: SymbolIndex,
    fields: Vec<(SymbolIndex, DataType, SourceRange)>,
    kind: StructKind,
    source: SourceRange
}


#[derive(Clone, Debug, PartialEq)]
pub struct Enum {
    name: SymbolIndex,
    full_name: SymbolIndex,
    mappings: Vec<EnumMapping>,
    source: SourceRange,
}


#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    name: SymbolIndex,
    full_name: SymbolIndex,
    pub args: Vec<FunctionArgument>,
    pub is_extern: Option<(SymbolIndex, SymbolIndex)>,
    pub is_system: bool,
    pub is_anonymous: bool,
    pub is_enum_variant_function: Option<u16>,
    pub return_type: DataType,
    source: SourceRange,

    pub body: Vec<Node>,
}


#[derive(Clone, Debug)]
pub struct Variable {
    name: SymbolIndex,
    data_type: DataType,
    is_mut: bool,
}

impl Variable {
    pub fn new(name: SymbolIndex, data_type: DataType, is_mut: bool) -> Self { Self { name, data_type, is_mut } }
}


struct AnalysisResult {
    data_type: DataType,
    mutability: bool,

    accessor_field_is_an_enum: bool,
}

impl AnalysisResult {
    fn new(data_type: DataType, mutability: bool) -> Self { Self { data_type, mutability, accessor_field_is_an_enum: false } }
}


impl Scope {    
    pub fn new(mangle_name: SymbolIndex, file: SymbolIndex) -> Self { 
        Self { 
            symbols: FuckMap::new(), 
            mangle_name, 
            namespaces: FuckMap::new(), 
            file, 
            variables: Vec::new(),
            is_function_scope: None,
            is_loop_scope: None, 
        } 
    }

    
    ///
    /// Mangles a name based off of the current
    /// active scope of the current context.
    ///
    fn mangle(&self, symbol_map: &mut SymbolMap, symbol: SymbolIndex, source: SourceRange) -> SymbolIndex {
        #[cfg(debug_assertions)]
        {
            if symbol_map.get(symbol).contains("::") {
                panic!("name already mangled")
            }
        }
        let str = format!(
            "{}::{}({})", 
            symbol_map.get(self.mangle_name), 
            symbol_map.get(symbol), 
            source.start(),
        );

        symbol_map.insert(str.as_str())
    }


    fn put_namespace(&mut self, identifier: SymbolIndex, path: SymbolIndex, source: SourceRange) -> Result<(), Error> {
        if self.namespaces.contains_key(&identifier) {
            return Err(CompilerError::new(
                    self.file, 
                    ErrorCode::SNameAlrDefined,
                    "name is already defined in the namespace"
                )
                .highlight(source)
                .build()
            );
        }
        
        assert!(self.namespaces.insert(identifier, (path, source)).is_none());
        Ok(())
    }


    fn register_variable(&mut self, variable: Variable) {
        self.variables.push(variable)
    }


    ///
    /// Pushes a symbol into the symbols list
    ///
    /// # Errors
    /// This function will return a "name is
    /// already defined error" if the `identifier`
    /// is already defined in the current scope
    /// and the given symbol isn't an anonymous
    /// symbol
    ///
    fn create_symbol(
        &mut self, 
        symbols: &mut Symbols, 
        mappings: &mut FuckMap<SymbolIndex, SymbolId>,
        symbol: Symbol, 
        identifier: SymbolIndex, 
        source: SourceRange
    ) -> Result<SymbolId, Error> {
        let is_anonymous_function = matches!(symbol, Symbol::Function(Function { is_anonymous: true, .. }));
        let name = symbol.full_name();
        let id = symbols.push(symbol);
        mappings.insert(name, id);
        
        
        // Anonymous functions are meant to be
        // unreachable and thus there's no need
        // to add them to the local namespace
        if is_anonymous_function {
            return Ok(id)
        }

        self.put_symbol(id, identifier, source).unwrap();
        Ok(id)
    }

    
    fn put_symbol(&mut self, symbol_id: SymbolId, identifier: SymbolIndex, source: SourceRange) -> Result<(), Error> {
        if self.symbols.contains_key(&identifier) {
            return Err(CompilerError::new(
                    self.file, 
                    ErrorCode::SNameAlrDefined, 
                    "name is already defined in the namespace"
                )
                .highlight(source)
                .build()
            );
        }
        
        assert!(self.symbols.insert(identifier, (symbol_id, source)).is_none());
        Ok(())
    }
}

impl Context {
    pub fn new(cs: Scope) -> Self { 
        Self {
            scopes: vec![],
            file: cs.file,
            cs,
        } 
    }

    
    ///
    /// Replaces current_scope with the
    /// given `Scope` and puts the old value of
    /// current scope to the end of the
    /// scopes list
    ///
    fn subscope(&mut self, scope: Scope) {
        let old_scope = std::mem::replace(&mut self.cs, scope);
        self.scopes.push(old_scope);
    }


    ///
    /// Pops a scope off of the scopes list and
    /// sets it as the current scope, returning
    /// the old current scope
    ///
    fn pop_scope(&mut self) -> Scope {
        let popped_scope = self.scopes.pop().unwrap();

        std::mem::replace(&mut self.cs, popped_scope)
    }

    
    ///
    /// Searches for a symbol in the current scope,
    /// if the symbol is unable to be found it will
    /// recursively search through the parent aswell.
    ///
    /// # Errors
    /// Returns a "symbol not found" error if there
    /// are no more parents left && the symbol doesn't
    /// exist
    ///
    fn find_namespace(&self, name: SymbolIndex, source: SourceRange) -> Result<SymbolIndex, Error> {
        let mut current_scope = &self.cs;

        for i in (0..self.scopes.len()+1).rev() {
            if let Some(v) = current_scope.namespaces.get(&name) {
                return Ok(v.0)
            }

            if i == 0 { break }

            current_scope = &self.scopes[i-1];
            
        }

        Err(CompilerError::new(
                source.file(), 
                ErrorCode::SNspaceUnreachable, 
                "namespace not defined or inaccessible"
            )
            .highlight(source)
            .build()
        )
    }

    
    ///
    /// Searches for a namespace in the current scope,
    /// if the namespace is unable to be found it will
    /// recursively search through the parent aswell.
    ///
    /// # Errors
    /// Returns a "symbol not found" error if there
    /// are no more parents left && the symbol doesn't
    /// exist
    ///
    fn find_symbol_id(&self, symbol_map: &SymbolMap, global_scope: Option<&Scope>, name: SymbolIndex, source: SourceRange) -> Result<SymbolId, Error> {
        let mut current_scope = &self.cs;

        for i in (0..self.scopes.len()+1).rev() {
            if let Some(v) = current_scope.symbols.get(&name) {
                return Ok(v.0)
            }

            if i == 0 { break }
            
            current_scope = &self.scopes[i-1];
            
        }

        if let Some(v) = global_scope {
            if let Some(v) = v.symbols.get(&name) {
                return Ok(v.0)
            }
        }

        Err(CompilerError::new(
                source.file(), 
                ErrorCode::SSymbolUnreachable, 
                "symbol not defined"
            )
            .highlight(source)
                .note(format!(
                    "there's no symbol named '{}' within '{}'",
                    symbol_map.get(name),
                    symbol_map.get(self.cs.mangle_name),
                ))
            .build()
        )
    }



    ///
    /// Searches for a variable from the end of the
    /// variables list, if it fails to find one it
    /// tries to recursively search on the parent scope.
    ///
    /// If encountered a scope with `is_function_scope`
    /// true, it will behave as if there are no more
    /// parents left
    ///
    /// # Errors
    /// Returns a "variable not found" error if there
    /// are no more parents left && the variable doesn't exist
    ///
    fn find_variable(&self, name: SymbolIndex, source: SourceRange) -> Result<&Variable, Error> {
        let mut current_scope = &self.cs;

        for i in (0..self.scopes.len()+1).rev() {
            if let Some(v) = current_scope.variables.iter().rev().find(|x| x.name == name) {
                return Ok(v)
            }

            if current_scope.is_function_scope.is_some() {
                break
            }

            if i == 0 { break }

            current_scope = &self.scopes[i-1];
        }

        Err(CompilerError::new(
                source.file(), 
                ErrorCode::SVariableNotDef, 
                "variable not defined"
            )
            .highlight(source)
            .build()
        )
    }
    /// 
    /// Starting from the current scope, search
    /// up the scopes vec, if any of the scopes
    /// are loop-scopes return the `break` marker
    /// otherwise none. 
    ///
    fn is_in_loop_body(&mut self) -> Option<&mut bool> {
        self.cs.is_loop_scope.as_mut().or_else(|| self.scopes.iter_mut().rev().find_map(|x| x.is_loop_scope.as_mut())) 
    }


    /// 
    /// Starting from the current scope, search
    /// up the scopes vec, if any of the scopes
    /// are func-scopes return the function type
    /// otherwise return None
    ///
    fn is_func_body(&self) -> Option<&DataType> {
        self.cs.is_function_scope.as_ref().or_else(|| self.scopes.iter().rev().find_map(|x| x.is_function_scope.as_ref())) 
    }
}


impl<'a> Infer<'a> {
    pub fn new(symbol_map: &'a mut SymbolMap, mut symbols: Symbols, ctx: Context, root_file: SymbolIndex) -> Self {
        let namespaces = FuckMap::new();
        let mut name_to_symbol = FuckMap::new();
        
        let global_namespace = symbol_map.insert(GLOBAL_NAMESPACE);
        let mut global_scope = Scope::new(symbol_map.insert(""), root_file);
        {
            let name = symbol_map.insert("CastError");
            let source = SourceRange::new(0, 0, root_file);
            let symbol = Symbol::Structure(Structure {
                name,
                full_name: symbol_map.insert(ANY_CAST_ERROR_STRUCT),
                fields: vec![],
                kind: StructKind::Normal,
                source,
            });

            let id = global_scope.create_symbol(&mut symbols, &mut name_to_symbol, symbol, name, source).unwrap();
        }
        
        Self {
            symbol_map,
            symbols,
            ctx,
            namespaces,
            root_file,
            global_scope,
            name_to_symbol,
        }
    }

    
    fn expect_type(&self, expect: &DataTypeKind, value: &DataType) -> Result<(), Error> {
        if value.kind().is(expect) {
            return Ok(())
        }

        
        Err(
            CompilerError::new(value.range().file(), ErrorCode::SUnexpectedType, "unexpected type")
                .highlight(value.range())
                    .note(format!(
                        "this expression expects a '{}' but found a '{}'", 
                        display_type(self.symbol_map, expect),
                        display_type(self.symbol_map, value.kind())
                    ))
                .build()
        )
    }

    
    fn find_symbol(&self, name: SymbolIndex) -> Option<&Symbol> {
        let id = self.name_to_symbol.get(&name)?;
        self.symbols.get(*id)
    }

    
    fn find_symbol_mut(&mut self, name: SymbolIndex) -> Option<&mut Symbol> {
        let id = self.name_to_symbol.get(&name)?;
        self.symbols.get_mut(*id)
    }


    fn type_namespace(&mut self, data_type: &DataType) -> SymbolIndex {
        let name = display_type(self.symbol_map, data_type.kind());
        let name = self.symbol_map.insert(name.as_str());

        if self.namespaces.contains_key(&name) {
            return name;
        }


        let source = data_type.range();
        self.namespaces.insert(name, Scope::new(name, source.file()));
        let _ = self.global_scope.put_namespace(name, name, source);

        
        if let DataTypeKind::Result(v1, v2) = data_type.kind() {
            // enum
            {
                let full_name = name;

                let enum_val = Enum {
                    name,
                    full_name,
                    mappings: vec![
                        EnumMapping::new(
                            self.symbol_map.insert("err"),
                            0,
                            DataType::new(source, v2.kind().clone()),
                            source,
                            false,
                        ),
                        EnumMapping::new(
                            self.symbol_map.insert("ok"),
                            1,
                            DataType::new(source, v1.kind().clone()),
                            source,
                            false,
                        ),
                    ],
                    source,
                };

                let id = self.global_scope.create_symbol(&mut self.symbols, &mut self.name_to_symbol, Symbol::Enum(enum_val), name, source).unwrap();
                self.register_enum_methods(id, source, data_type.kind().clone()).unwrap();
            }
        } else if let DataTypeKind::Option(v) = data_type.kind() {            
            let some = self.symbol_map.insert("some");
            let none = self.symbol_map.insert("none");
            // enum
            {
                let full_name = name;

                let enum_val = Enum {
                    name,
                    full_name,
                    mappings: vec![
                        EnumMapping::new(
                            none,
                            0,
                            DataType::new(source, DataTypeKind::Unit),
                            source,
                            true,
                        ),
                        EnumMapping::new(
                            some,
                            1,
                            DataType::new(source, v.kind().clone()),
                            source,
                            false,
                        ),
                    ],
                    source,
                };

                let id = self.global_scope.create_symbol(&mut self.symbols, &mut self.name_to_symbol, Symbol::Enum(enum_val), name, source).unwrap();
                self.register_enum_methods(id, source, data_type.kind().clone()).unwrap();
            }
            // is_some
            {
                let namespace = self.namespaces.get_mut(&name).unwrap();
                let function_name = self.symbol_map.insert("is_some");
                let self_var = self.symbol_map.insert("self");

                let function = Function {
                    name: function_name,
                    full_name: namespace.mangle(self.symbol_map, function_name, source),
                    args: Vec::from([FunctionArgument::new(self_var, data_type.clone(), false, source)]),
                    is_extern: None,
                    is_system: false,
                    is_anonymous: false,
                    is_enum_variant_function: None,
                    return_type: DataType::new(source, DataTypeKind::Bool),
                    source,
                    body: Vec::from([Node::new(
                        NodeKind::Expression(Expression::Match { 
                            value: Box::new(Node::new(
                                NodeKind::Expression(Expression::Identifier(self_var)),
                                source,
                            )),
                            mappings: Vec::from([
                                MatchMapping::new(
                                    some,
                                    self_var, 
                                    source, 
                                    Node::new(
                                        NodeKind::Expression(Expression::Literal(Literal::Bool(true))),
                                        source,
                                    )
                                ),
                                MatchMapping::new(
                                    none,
                                    self_var, 
                                    source, 
                                    Node::new(
                                        NodeKind::Expression(Expression::Literal(Literal::Bool(false))),
                                        source,
                                    )
                                )
                            ]) 
                        }),
                        source
                    )]),
                };

                namespace.create_symbol(&mut self.symbols, &mut self.name_to_symbol, Symbol::Function(function), function_name, source).unwrap();
                return name;
            }
        }

        

        name
    }
    
    ///
    /// Validates that a type exists and then
    /// replaces the type with its fully-qualified
    /// version. If the type doesn't exist it will
    /// replace the type with `DataTypeKind::Unknown`
    ///
    /// This function will do nothing for built-in
    /// types.
    ///
    /// # Errors
    /// If the type doesn't exist in the current scope
    /// or any of the parent scopes it will return a
    /// "type doesn't exist or inaccessible" error.
    ///
    fn validate_data_type(&self, data_type: &mut DataType) -> Result<(), Error> {
        let range = data_type.range();
        match data_type.kind_mut() {
            DataTypeKind::Int   => Ok(()),
            DataTypeKind::Bool  => Ok(()),
            DataTypeKind::Float => Ok(()),
            DataTypeKind::Unit  => Ok(()),
            DataTypeKind::Any   => Ok(()),
            DataTypeKind::Unknown => Ok(()),
            DataTypeKind::Option(v) => self.validate_data_type(v),
            DataTypeKind::Result(v1, v2) => {
                self.validate_data_type(v1)?;
                self.validate_data_type(v2)
            },
            DataTypeKind::Never => Ok(()),
            DataTypeKind::CustomType(v) => {
                let symbol = self.ctx.find_symbol_id(self.symbol_map, Some(&self.global_scope), *v, range)?;
                let symbol = self.symbols.get(symbol).unwrap();
                match symbol {
                    | Symbol::Enum(_)
                    | Symbol::Structure(_) => {
                        *v = symbol.full_name();
                        return Ok(())
                    },

                    _ => (),
                };


                *data_type.kind_mut() = DataTypeKind::Unknown;
                Err(CompilerError::new(self.ctx.file, ErrorCode::SSymbolIsntType, "symbol isn't a type")
                    .highlight(data_type.range())
                    .build())
            },
        }
    }


    fn register_enum_methods(&mut self, symbol: SymbolId, source: SourceRange, kind: DataTypeKind) -> Result<(), Error> {
        let structure = &mut self.symbols[symbol];
        let Symbol::Enum(structure) = structure else { panic!() };
        let scope = self.namespaces.entry(structure.full_name).or_insert(Scope::new(structure.full_name, source.file()));

        let mappings = std::mem::take(&mut structure.mappings);
        let full_name = structure.full_name;

        let mut errors = vec![];
        for m in mappings.iter() {
            let func = Function {
                name: m.name(),
                full_name: scope.mangle(self.symbol_map, m.name(), m.range()),
                args: {
                    if m.is_implicit_unit() {
                        vec![]
                    } else {
                        vec![FunctionArgument::new(m.name(), m.data_type().clone(), false, m.range())]
                    }
                },
                is_extern: None,
                is_system: false,
                is_anonymous: false,
                is_enum_variant_function: Some(m.number()),
                return_type: DataType::new(m.range(), kind.clone()),
                source,

                body: vec![], // body cant be represented
            };

            let result = scope.create_symbol(&mut self.symbols, &mut self.name_to_symbol, Symbol::Function(func), m.name(), m.range());
            if let Err(e) = result {
                errors.push(e)
            }
        }

        let structure = &mut self.symbols[symbol];
        let Symbol::Enum(structure) = structure else { panic!() };

        structure.mappings = mappings;

        if !errors.is_empty() {
            return Err(errors.combine_into_error())
        }
        
        Ok(())
    }
}



impl<'a> Infer<'a> {
    fn analyse_block(&mut self, block: &mut Block, only_declarations: bool) -> Result<AnalysisResult, Error> {
        self.ctx.subscope(Scope::new(self.ctx.cs.mangle_name, self.ctx.file));

        let result = self.register_declarations(block);
        let scope = self.ctx.pop_scope();

        result?;

        let result = self.analyse_block_with_scope(block, scope, only_declarations)?;
        Ok(result.0)
    }


    fn analyse_block_with_scope(&mut self, block: &mut Block, scope: Scope, only_declarations: bool) -> Result<(AnalysisResult, Scope), Error> {
        if only_declarations && !matches!(block[0].kind(), NodeKind::Expression(Expression::Unit)) {
            let errors = block.iter().filter_map(|x| {
                if !matches!(x.kind(), NodeKind::Declaration(_)) {
                    Some(CompilerError::new(self.ctx.cs.file, ErrorCode::SBlockOnlyAllowDec, "this block only allows declarations")
                        .highlight(x.range())
                        .build())
                } else {
                    None
                }
            }).collect::<Vec<_>>();
    

            if !errors.is_empty() {
                return Err(errors.combine_into_error())
            }
        }

        
        let mut errors = vec![];
        self.ctx.subscope(scope);
        
        let result = block
            .iter_mut()
            .map(|x| self.analyse_node(x))
            .filter_map(|x| match x {
                Ok(v) => Some(v),
                Err(e) => {
                    errors.push(e);
                    None
                },
            })
            .last()
            .unwrap_or_else(|| {
                let range = block.last().unwrap().range();
                AnalysisResult::new(DataType::new(range, DataTypeKind::Unknown), true)
            });
        
        let scope = self.ctx.pop_scope();

        if !errors.is_empty() {
            return Err(errors.combine_into_error())
        }

        Ok((result, scope))
    }


    fn analyse_declaration(&mut self, decl: &mut Declaration, source: SourceRange) -> Result<(), Error> {
        match decl {
            Declaration::Struct { .. } => Ok(()),
            Declaration::Enum { .. } => Ok(()),


            Declaration::Function { name, arguments, return_type, body, .. } => {

                // evaluate body
                let block_return_type = {
                    let mut subscope = Scope::new(*name, source.file());
                    subscope.is_function_scope = Some(return_type.clone());

                    for i in arguments {
                        let variable = Variable {
                            name: i.name(),
                            data_type: i.data_type().clone(),
                            is_mut: true,
                        };

                        subscope.register_variable(variable);
                    }

                    self.ctx.subscope(subscope);
                    let result = self.analyse_block(body, false);
                    self.ctx.pop_scope();

                    result?
                };


                if !block_return_type.data_type.kind().is(return_type.kind()) {
                    return Err(
                        CompilerError::new(
                            source.file(), 
                            ErrorCode::SFuncReturnDiff, 
                            "function return type differs from body"
                        )
                        .highlight(source)
                            .note(format!("function returns '{}' but the body returns '{}'",
                                    display_type(self.symbol_map, return_type.kind()),
                                    display_type(self.symbol_map, block_return_type.data_type.kind())
                                ))
                        .build()
                    )
                }


                let Symbol::Function(symbol) = self.find_symbol_mut(*name).unwrap() else { panic!() };
                symbol.body = body.take_vec();
                

                Ok(())
            },


            Declaration::Impl { data_type, body } => {
                let name = self.type_namespace(data_type);

                let namespace = self.namespaces.get(&name).unwrap().clone();
                let (_, namespace) = self.analyse_block_with_scope(body, namespace, true)?;

                *self.namespaces.get_mut(&name).unwrap() = namespace;

                Ok(())
            },


            Declaration::Using { .. } => todo!(),

            
            Declaration::Module { name, body } => {
                let namespace = self.namespaces.get(name).unwrap().clone();
                let (_, namespace) = self.analyse_block_with_scope(body, namespace, true)?;

                *self.namespaces.get_mut(name).unwrap() = namespace;

                Ok(())
                
            },


            Declaration::Extern { .. } => Ok(()),
        }
    }


    fn analyse_expression(&mut self, expr: &mut Expression, source: SourceRange) -> Result<AnalysisResult, Error> {
        let result = match expr {
            Expression::Unit => AnalysisResult::new(DataType::new(source, DataTypeKind::Unit), true),

            
            Expression::Literal(v) => {
                let kind = match v {
                    Literal::Float(_)   => DataTypeKind::Float,
                    Literal::Integer(_) => DataTypeKind::Int,
                    Literal::String(_)  => todo!(),
                    Literal::Bool(_)    => DataTypeKind::Bool,
                };

                AnalysisResult::new(DataType::new(source, kind), true)
            },


            Expression::Identifier(v) => {
                let variable = self.ctx.find_variable(*v, source)?;
                
                AnalysisResult::new(variable.data_type.clone(), variable.is_mut)
            },

            
            Expression::BinaryOp { operator, lhs, rhs } => {
                let lhs_typ = self.analyse_node(&mut *lhs)?.data_type;
                let rhs_typ = self.analyse_node(&mut *rhs)?.data_type;

                if lhs_typ.kind() == &DataTypeKind::Unknown
                    || rhs_typ.kind() == &DataTypeKind::Unknown {
                    return Ok(AnalysisResult::new(DataType::new(source, DataTypeKind::Unknown), true))
                }

                let kind = match (operator, lhs_typ.kind()) {
                    | (parser::nodes::BinaryOperator::Add, DataTypeKind::Int)
                    | (parser::nodes::BinaryOperator::Add, DataTypeKind::Float)
                    | (parser::nodes::BinaryOperator::Sub, DataTypeKind::Int)
                    | (parser::nodes::BinaryOperator::Sub, DataTypeKind::Float)
                    | (parser::nodes::BinaryOperator::Mul, DataTypeKind::Int)
                    | (parser::nodes::BinaryOperator::Mul, DataTypeKind::Float)
                    | (parser::nodes::BinaryOperator::Div, DataTypeKind::Int)
                    | (parser::nodes::BinaryOperator::Div, DataTypeKind::Float)
                    | (parser::nodes::BinaryOperator::Rem, DataTypeKind::Int)
                    | (parser::nodes::BinaryOperator::Rem, DataTypeKind::Float)
                    | (parser::nodes::BinaryOperator::BitshiftLeft, DataTypeKind::Int)
                    | (parser::nodes::BinaryOperator::BitshiftRight, DataTypeKind::Int)
                    | (parser::nodes::BinaryOperator::BitwiseAnd, DataTypeKind::Int)
                    | (parser::nodes::BinaryOperator::BitwiseOr, DataTypeKind::Int)
                    | (parser::nodes::BinaryOperator::BitwiseXor, DataTypeKind::Int)
                     if lhs_typ.kind().is(rhs_typ.kind()) => {
                        lhs_typ.kind_owned()
                    }

                    | (parser::nodes::BinaryOperator::Eq, _ )
                    | (parser::nodes::BinaryOperator::Ne, _ )
                    | (parser::nodes::BinaryOperator::Gt, DataTypeKind::Int)
                    | (parser::nodes::BinaryOperator::Gt, DataTypeKind::Float)
                    | (parser::nodes::BinaryOperator::Ge, DataTypeKind::Int)
                    | (parser::nodes::BinaryOperator::Ge, DataTypeKind::Float)
                    | (parser::nodes::BinaryOperator::Lt, DataTypeKind::Int)
                    | (parser::nodes::BinaryOperator::Lt, DataTypeKind::Float)
                    | (parser::nodes::BinaryOperator::Le, DataTypeKind::Int)
                    | (parser::nodes::BinaryOperator::Le, DataTypeKind::Float)
                     if lhs_typ.kind().is(rhs_typ.kind()) => {
                        DataTypeKind::Bool
                    }


                    _ => return Err(
                        CompilerError::new(source.file(), ErrorCode::SInvalidBinOp, "invalid binary operation")
                            .highlight(source)
                                .note(format!(
                                    "left side is '{}' while the right side is '{}'", 
                                    display_type(self.symbol_map, lhs_typ.kind()), 
                                    display_type(self.symbol_map, rhs_typ.kind())
                                ))
                            .build()
                    )
                };

                AnalysisResult::new(DataType::new(source, kind), true)
            },
            

            Expression::UnaryOp { operator, rhs } => {
                let rhs = self.analyse_node(&mut *rhs)?.data_type;
                let rhs_typ = rhs.kind();

                let kind = match (&operator, rhs_typ) {
                    | (UnaryOperator::Not, DataTypeKind::Bool)
                    | (UnaryOperator::Neg, DataTypeKind::Int)
                    | (UnaryOperator::Neg, DataTypeKind::Float)
                     => {
                        rhs.kind_owned()
                    }

                    _ => return Err(
                        CompilerError::new(source.file(), ErrorCode::SInvalidBinOp, "invalid unary operation")
                            .highlight(source)
                                .note(format!(
                                    "the '{}' operator only works on values of type {}", 
                                    match &operator {
                                        UnaryOperator::Not => "not",
                                        UnaryOperator::Neg => "negate",
                                    },

                                    match &operator {
                                        UnaryOperator::Not => "'bool'",
                                        UnaryOperator::Neg => "'int' or 'float'",
                                    },
                                
                                ))
                            .build()
                    )
                };

                AnalysisResult::new(DataType::new(source, kind), true)
            },

            
            Expression::If { condition, body, else_block } => {
                let condition_typ = self.analyse_node(&mut *condition)?.data_type;

                self.expect_type(&DataTypeKind::Bool, &condition_typ)?;

                let mut body_typ = self.analyse_block(body, false)?;

                if let Some(else_block) = else_block {
                    let else_typ = self.analyse_node(&mut *else_block)?;
                    if body_typ.data_type.kind() == &DataTypeKind::Never {
                        body_typ.data_type = else_typ.data_type
                    } else {
                        self.expect_type(body_typ.data_type.kind(), &else_typ.data_type)?;
                    }

                } else if !body_typ.data_type.kind().is(&DataTypeKind::Unit) {
                    return Err(
                        CompilerError::new(source.file(), ErrorCode::SIfExprNoElse, "if expression has no else")
                        .highlight(source)
                            .note(format!(
                                "the body returns {} but there's no else block",
                                display_type(self.symbol_map, body_typ.data_type.kind())
                            ))
                        .build()
                    )
                }


                AnalysisResult::new(
                    DataType::new(source, body_typ.data_type.kind_owned()), 
                    true,
                )
            },

            
            Expression::Match { value, mappings } => {
                let value_typ = self.analyse_node(&mut *value)?;

                if value_typ.data_type.kind() == &DataTypeKind::Unknown {
                    return Ok(AnalysisResult::new(DataType::new(source, DataTypeKind::Unknown), true))
                }

                let e = self.type_namespace(&value_typ.data_type);
                let e = self.find_symbol(e).unwrap();
                let Symbol::Enum(e) = e
                else {
                    return Err(
                        CompilerError::new(source.file(), ErrorCode::SMatchValNotEnum, "match value is not an enum")
                            .highlight(source)
                                .note(format!("is of type '{}' which is not an enum", display_type(self.symbol_map, value_typ.data_type.kind())))
                            .build()
                    )

                };



                {
                    let mut errors = vec![];
                    for i in 0..mappings.len() {                    
                        let f = mappings.get(i).unwrap();

                        if let Some(v) = mappings[0..i].iter().find(|x| x.name() == f.name()) {
                            errors.push(CompilerError::new(
                                self.ctx.file, 
                                ErrorCode::SVariantDefEarlier, 
                                "variant is already defined"
                            )
                            .highlight(v.node().range())
                                .note("variant is defined earlier here".to_string())
                            .highlight(f.node().range())
                                .note("..but it's defined again here".to_string())
                            .build())
                        }


                        if !e.mappings.iter().any(|x| x.name() == f.name()) {                            
                            errors.push(CompilerError::new(
                                self.ctx.file, 
                                ErrorCode::SMatchUnkownVar, 
                                "unknown variant"
                            )
                            .highlight(f.range())
                            .build())
                        }
                    }

                    
                    for mapping in &e.mappings {
                        if !mappings.iter().any(|x| x.name() == mapping.name()) {
                            errors.push(CompilerError::new(
                                self.ctx.file, 
                                ErrorCode::SMissingField, 
                                "missing field"
                            )
                            .highlight(source)
                                .note(format!("missing '{}'",
                                    self.symbol_map.get(mapping.name()), 
                                ))
                            .build())
                        }
                    }

                    if !errors.is_empty() {
                        return Err(errors.combine_into_error())
                    }
                }


                let return_type = {
                    let mut expected_type : Option<DataTypeKind> = None;
                    let mut errors = vec![];

                    //  PERFORMANCE: avoid this clone
                    let enum_mappings = e.mappings.clone();
                    let enum_name = e.name;

                    let mut numbers = Vec::with_capacity(mappings.len());
                    for mapping in mappings.iter_mut() {
                        let enum_mapping = enum_mappings.iter().find(|x| x.name() == mapping.name()).unwrap();
                        numbers.push(enum_mapping.number());

                        let mut mapping_scope = Scope::new(enum_name, source.file());
                        mapping_scope.register_variable(
                            Variable::new(
                                mapping.binding(), 
                                enum_mapping.data_type().clone(), 
                                value_typ.mutability,
                            )
                        );

                        self.ctx.subscope(mapping_scope);

                        let result = self.analyse_node(mapping.node_mut());

                        self.ctx.pop_scope();

                        let result = match result {
                            Ok(v) => v,
                            Err(v) => {
                                errors.push(v);
                                continue
                            },
                        };


                        if expected_type.as_ref().is_none() || expected_type.as_ref().unwrap() == &DataTypeKind::Never {
                            expected_type = Some(result.data_type.kind_owned());
                            continue
                        }


                        if !result.data_type.kind().is(expected_type.as_ref().unwrap()) {
                            errors.push(CompilerError::new(
                                self.ctx.file, 
                                ErrorCode::SMatchBranchDiffTy, 
                                "match branch returns a different type"
                            )
                            .highlight(mapping.node().range())
                                .note(format!("expected '{}' found '{}'",
                                    display_type(self.symbol_map, expected_type.as_ref().unwrap()),
                                    display_type(self.symbol_map, result.data_type.kind())
                                ))
                            .build())
                        }
                    }


                    if !errors.is_empty() {
                        return Err(errors.combine_into_error())
                    }

                    let mut counter = 0;
                    mappings.sort_unstable_by_key(|x| {
                        counter += 1;
                        numbers[counter-1]
                    });
                    expected_type.unwrap_or(DataTypeKind::Unit)
                };
                

                AnalysisResult::new(DataType::new(source, return_type), true)

            },

            
            Expression::Block { block } => self.analyse_block(block, false)?,

            
            Expression::CreateStruct { data_type, fields } => {
                self.validate_data_type(data_type)?;


                let structure = match data_type.kind() {
                    | DataTypeKind::Int
                    | DataTypeKind::Bool
                    | DataTypeKind::Float
                    | DataTypeKind::Unit
                    | DataTypeKind::Any
                    | DataTypeKind::Option(_)
                    | DataTypeKind::Result(_, _)
                    | DataTypeKind::Never
                     => {
                        return Err(
                            CompilerError::new(source.file(), ErrorCode::SCantInitPrimitive, "can't initialise primitive types with structure creation syntax")
                                .highlight(source)
                                .build()
                        )
                    }

                    DataTypeKind::Unknown => return Ok(AnalysisResult::new(DataType::new(source, DataTypeKind::Unknown), true)),

                    DataTypeKind::CustomType(e) => e,
                };


                let structure = self.find_symbol(*structure).unwrap();
                let Symbol::Structure(structure) = structure
                else {
                    return Err(
                        CompilerError::new(
                            source.file(), 
                            ErrorCode::SSymbolIsntStruct, 
                            "symbol exists but it's not a structure"
                        )
                        .highlight(source)
                            .note(format!("the symbol is a '{}'", structure.type_name()))
                        .build()
                    )
                };
                

                {
                    // PERFORAMANCE: Remove clone
                    let structure_fields = structure.fields.clone();
                    let structure_name = structure.full_name;

                    let mut errors = vec![];
                    for i in 0..fields.len() {
                        let f = fields.get_mut(i).unwrap();
                        let result = self.analyse_node(&mut f.2);
                        let result = match result {
                            Ok(v) => Some(v),
                            Err(v) => {
                                errors.push(v);
                                None
                            }
                        };
                        
                        
                        let f = fields.get(i).unwrap();
                        if let Some(v) = fields[0..i].iter().find(|x| x.0 == f.0) {
                            errors.push(CompilerError::new(
                                self.ctx.file, 
                                ErrorCode::SFieldDefEarlier, 
                                "field is already defined"
                            )
                            .highlight(v.1)
                                .note("field is defined earlier here".to_string())
                            .highlight(f.1)
                                .note("..but it's defined again here".to_string())
                            .build())
                        }


                        let field = structure_fields.iter().find(|x| x.0 == f.0);
                        if field.is_none() {
                            errors.push(CompilerError::new(
                                self.ctx.file, 
                                ErrorCode::SUnknownField, 
                                "unknown field"
                            )
                            .highlight(f.1)
                                .note(format!("there's no field named {} in {}",
                                    self.symbol_map.get(f.0), 
                                    self.symbol_map.get(structure_name), 
                                ))
                            .build());

                            continue
                        }

                        let field = field.unwrap();
                        let Some(result) = result else { continue };

                        if !result.data_type.kind().is(field.1.kind()) {                            
                            errors.push(CompilerError::new(
                                self.ctx.file, 
                                ErrorCode::SFieldTypeMismatch, 
                                "field type mismatch"
                            )
                            .highlight(field.2)
                                .note(format!("the field {} is defined as {} here",
                                    self.symbol_map.get(f.0), 
                                    display_type(self.symbol_map, field.1.kind()),
                                ))
                            .empty_line()
                            .highlight(f.2.range())
                                .note(format!("..but the value here is of type {}",
                                    display_type(self.symbol_map, result.data_type.kind())
                                ))
                            .build());
                        }
                    }


                    for field in &structure_fields {
                        if !fields.iter().any(|x| x.0 == field.0) {
                            errors.push(CompilerError::new(
                                self.ctx.file, 
                                ErrorCode::SMissingField, 
                                "missing field"
                            )
                            .highlight(source)
                                .note(format!("missing '{}'",
                                    self.symbol_map.get(field.0), 
                                ))
                            .build())
                        }
                    }
                    

                    if !errors.is_empty() {
                        return Err(errors.combine_into_error())
                    }
                }

                AnalysisResult::new(data_type.clone(), true)
            },

            
            Expression::AccessField { val, field: field_name, field_meta } => {
                let val_typ = self.analyse_node(&mut *val)?;

                if val_typ.data_type.kind() == &DataTypeKind::Unknown {
                    return Ok(AnalysisResult::new(DataType::new(source, DataTypeKind::Unknown), true))
                }
                
                let structure = self.type_namespace(&val_typ.data_type);
                let Some(structure) = self.find_symbol(structure)
                else {
                    return Err(
                        CompilerError::new(
                            source.file(), 
                            ErrorCode::SAccFieldOnPrim, 
                            "can't access fields on a primitive"
                        )
                        .highlight(source)
                        .build()
                    )
                };

                match structure {
                    Symbol::Structure(structure) => {
                        let field = structure.fields.iter().enumerate().find(|x| x.1.0 == *field_name);
                        let Some((index, field)) = field
                        else {
                            return Err(
                                CompilerError::new(
                                    source.file(), 
                                    ErrorCode::SFieldDoesntExist, 
                                    "field doesn't exist"
                                )
                                .highlight(source)
                                    .note(format!("there's no field named {} on '{}'",
                                        self.symbol_map.get(*field_name),
                                        self.symbol_map.get(structure.full_name),
                                    ))
                                .build()
                            )
                        };

                        *field_meta = (index as u16, false);

                        AnalysisResult::new(DataType::new(source, field.1.kind().clone()), val_typ.mutability)
                    }


                    Symbol::Enum(e) => {
                        let variant = e.mappings.iter().enumerate().find(|x| x.1.name() == *field_name);
                        let Some((index, variant)) = variant
                        else {
                            return Err(
                                CompilerError::new(
                                    source.file(), 
                                    ErrorCode::SFieldDoesntExist, 
                                    "field doesn't exist"
                                )
                                .highlight(source)
                                    .note(format!("there's no variant named {} on '{}'",
                                        self.symbol_map.get(*field_name),
                                        self.symbol_map.get(e.full_name),
                                    ))
                                .build()
                            )
                        };

                        *field_meta = (index as u16, true);

                        let mut analysis = AnalysisResult::new(DataType::new(source, DataTypeKind::Option(Box::new(variant.data_type().clone()))), true);
                        analysis.accessor_field_is_an_enum = true;
                        analysis
                    }
                    
                    
                    _ => {
                        return Err(
                            CompilerError::new(
                                source.file(), 
                                ErrorCode::SSymbolIsntStruct, 
                                "symbol exists but it's not a structure"
                            )
                            .highlight(source)
                                .note(format!("the symbol is a '{}'", structure.type_name()))
                            .build()
                        )
                    }
                }
            },

            
            Expression::CallFunction { name, args, is_accessor } => {
                let mut args_analysis = {
                    let mut errors = vec![];
                    let mut arg_analysis = Vec::with_capacity(args.len());
                    for a in args.iter_mut() {
                        let a_typ = self.analyse_node(&mut a.0);

                        match a_typ {
                            Ok(e) => arg_analysis.push(e),
                            Err(e) => errors.push(e),
                        }
                    }

                    if !errors.is_empty() {
                        return Err(errors.combine_into_error())
                    }

                    arg_analysis
                };
                assert_eq!(args_analysis.len(), args.len());


                let has_accessor = is_accessor.is_some();
                let function = if let Some(v) = is_accessor {
                    let accessor_analysis = self.analyse_node(&mut *v)?;
                    let path = self.type_namespace(&accessor_analysis.data_type);

                    let scope = self.namespaces.get(&path).unwrap();

                    {
                        args.insert(0, (*is_accessor.take().unwrap(), true));
                        args_analysis.insert(0, accessor_analysis);
                        
                    }

                    Context::new(scope.clone()).find_symbol_id(self.symbol_map, Some(&self.global_scope), *name, source)

                } else {
                    self.ctx.find_symbol_id(self.symbol_map, Some(&self.global_scope), *name, source)
                };

                let function = function?;
                let function = self.symbols.get(function).unwrap();
                let Symbol::Function(function) = function
                else {
                    return Err(
                        CompilerError::new(
                            source.file(), 
                            ErrorCode::SSymbolIsntFunc, 
                            "symbol exists but it's not a function"
                        )
                        .highlight(source)
                            .note(format!("the symbol is a '{}'", function.type_name()))
                        .build()
                    )
                };


                *name = function.full_name;



                if args.len() != function.args.len() {
                    return Err(
                        CompilerError::new(
                            source.file(), 
                            ErrorCode::SFuncArgcMismatch, 
                            "argument count mismatch"
                        )
                        .highlight(source)
                            .note(format!("the function requires {} arguments but you've provided {}", function.args.len(), args.len()))
                        .build()
                    )
                }

                if has_accessor {
                    args[0].1 = function.args[0].is_inout();
                }

                {
                    let mut errors = vec![];
                    for i in 0..args.len() {
                        let arg = &mut args[i];
                        let arg_analysis = &args_analysis[i];
                        let expected_arg = &function.args[i];

                        if !arg_analysis.data_type.kind().is(expected_arg.data_type().kind()) {
                            errors.push(
                                CompilerError::new(source.file(), ErrorCode::SArgTypeMismatch, "argument type mismatch")
                                    .highlight(arg.0.range())
                                        .note(format!(
                                            "argument is defined as {} but the value is of type '{}'", 
                                            display_type(self.symbol_map, expected_arg.data_type().kind()),
                                            display_type(self.symbol_map, arg_analysis.data_type.kind()),
                                        ))
                                    .build()
                            );
                        }


                        if arg.1 != expected_arg.is_inout() {
                            errors.push(
                                CompilerError::new(source.file(), ErrorCode::SArgDiffInOut, "argument differ in in-outness")
                                    .highlight(arg.0.range())
                                        .note({
                                            match (arg.1, expected_arg.is_inout()) {
                                                (true, false) => "consider removing the '&'",
                                                (false, true) => "consider adding a '&' before the argument",

                                                _ => unreachable!(),
                                            }.to_string()
                                        })
                                    .build()
                            );
                            continue;
                        }


                        if expected_arg.is_inout() && !arg_analysis.mutability {                            
                            errors.push(
                                CompilerError::new(source.file(), ErrorCode::SInOutArgIsntMut, "argument is in-out but the value isn't mutable")
                                    .highlight(arg.0.range())
                                        .note("..isn't a mutable value".to_string())
                                    .build()
                            );
                        }


                    }


                    if !errors.is_empty() {
                        return Err(errors.combine_into_error())
                    }
                }
                
                
                AnalysisResult::new(DataType::new(source, function.return_type.kind().clone()), true)
            },

            
            Expression::WithinNamespace { namespace, action, namespace_source } => {
                let namespace = self.ctx.find_namespace(*namespace, *namespace_source)?;
                // PERFORMANCE: Maybe? Remove the clone
                let namespace = self.namespaces.get(&namespace).unwrap();
                let namespace = namespace.clone();

                self.ctx.subscope(namespace);

                let result = self.analyse_node(&mut *action);

                self.ctx.pop_scope();

                AnalysisResult::new(DataType::new(source, result?.data_type.kind_owned()), true)
            },

            
            Expression::WithinTypeNamespace { namespace, action } => {
                self.validate_data_type(namespace)?;
                
                let namespace = self.type_namespace(namespace);
                // PERFORMANCE: Maybe? Remove the clone
                let namespace = self.namespaces.get(&namespace).unwrap();
                let namespace = namespace.clone();

                self.ctx.subscope(namespace);

                let result = self.analyse_node(&mut *action);

                self.ctx.pop_scope();

                AnalysisResult::new(DataType::new(source, result?.data_type.kind_owned()), true)
            },

            
            Expression::Loop { body } => {
                let mut scope = Scope::new(self.ctx.cs.mangle_name, self.ctx.cs.file);
                scope.is_loop_scope = Some(false);

                self.ctx.subscope(scope);
                let result = self.analyse_block(body, false);
                let scope = self.ctx.pop_scope();

                result?;

                let data_type = if scope.is_loop_scope.unwrap() { DataTypeKind::Unit }
                                else { DataTypeKind::Never };

                AnalysisResult::new(DataType::new(source, data_type), true)
            },

            
            Expression::Return(v) => {
                let return_type = self.analyse_node(&mut *v)?;

                let function_return = self.ctx.is_func_body();
                let Some(function_return) = function_return
                else {
                    return Err(
                        CompilerError::new(source.file(), ErrorCode::SReturnOutsideFunc, "can't return outside of a function")
                            .highlight(source)
                            .build()
                    )
                };


                if !return_type.data_type.kind().is(function_return.kind()) {
                    return Err(
                        CompilerError::new(source.file(), ErrorCode::SFuncReturnDiff, "function return type differs from body")
                            .highlight(function_return.range())
                                .note(format!("function returns '{}'", display_type(self.symbol_map, function_return.kind())))
                            .highlight(source)
                                .note(format!("..but this returns '{}'", display_type(self.symbol_map, return_type.data_type.kind())))
                            .build()
                    )
                }


                AnalysisResult::new(DataType::new(source, DataTypeKind::Never), true)
            },

            
            Expression::Break => {
                if !self.ctx.is_in_loop_body().map(|x| { *x = true; *x } ).unwrap_or(false) {
                    return Err(
                        CompilerError::new(source.file(), ErrorCode::SBreakOutsideLoop, "break outside of loop")
                            .highlight(source)
                            .build()
                    )
                }

                AnalysisResult::new(DataType::new(source, DataTypeKind::Never), true)
            },

            
            Expression::Continue => {
                if self.ctx.is_in_loop_body().is_none() {
                    return Err(
                        CompilerError::new(source.file(), ErrorCode::SContOutsideLoop, "continue outside of loop")
                            .highlight(source)
                            .build()
                    )
                }

                AnalysisResult::new(DataType::new(source, DataTypeKind::Never), true)
            },

            
            Expression::CastAny { lhs, data_type } => {
                self.analyse_node(&mut *lhs)?;

                let kind = DataTypeKind::Result(
                    Box::new(data_type.clone()), 
                    Box::new(DataType::new(
                        source, 
                        DataTypeKind::CustomType(self.symbol_map.insert(ANY_CAST_ERROR_STRUCT))
                    )));

                AnalysisResult::new(DataType::new(source, kind), true)
            },


            Expression::Unwrap(v) => {
                let val_typ = self.analyse_node(&mut *v)?;

                match val_typ.data_type.kind() {
                    DataTypeKind::Unknown => AnalysisResult::new(DataType::new(source, DataTypeKind::Unknown), true),

                    | DataTypeKind::Option(v)
                    | DataTypeKind::Result(v, _)
                     => AnalysisResult::new(DataType::new(source, v.kind().clone()), true),

                    _ => {
                        return Err(
                            CompilerError::new(source.file(), ErrorCode::SCantUnwrapType, "can't unwrap type")
                                .highlight(source)
                                    .note(format!("can't unwrap a value of type {}", display_type(self.symbol_map, val_typ.data_type.kind())))
                                .build()
                        )
                    }
                }
            }


            Expression::OrReturn(v) => {
                let val_typ = self.analyse_node(&mut *v)?;
                let Some(func_return) = self.ctx.is_func_body()
                else {
                    return Err(
                        CompilerError::new(source.file(), ErrorCode::SReturnOutsideFunc, "can't return outside of a function")
                            .highlight(source)
                            .build()
                    )
                };

                match val_typ.data_type.kind() {
                    DataTypeKind::Unknown => return Ok(AnalysisResult::new(DataType::new(source, DataTypeKind::Unknown), true)),

                    DataTypeKind::Option(val) => {
                        if !matches!(func_return.kind(), DataTypeKind::Option(_)) {
                            return Err(CompilerError::new(source.file(), ErrorCode::STryOpOptionRetVal, "try operator on an option type only works if the function returns an option")
                                .highlight(source)
                                .build())
                        }

                        AnalysisResult::new(DataType::new(source, val.kind().clone()), true)

                    }

                    DataTypeKind::Result(val, err) => {
                        if !matches!(func_return.kind(), DataTypeKind::Result(_, err2) if err == err2) {
                            return Err(CompilerError::new(source.file(), ErrorCode::STryOpOptionRetVal, "try operator on a result type only works if the function returns a result with the same error type")
                                .highlight(source)
                                .build())
                        }

                        AnalysisResult::new(DataType::new(source, val.kind().clone()), true)

                    }

                    _ => {
                        return Err(
                            CompilerError::new(source.file(), ErrorCode::SCantUnwrapType, "can't use the try operator on this type")
                                .highlight(source)
                                    .note(format!("..is of type {}", display_type(self.symbol_map, val_typ.data_type.kind())))
                                .build()
                        )
                    }
                }
            }

        };

        Ok(result)
    }


    fn analyse_node(&mut self, node: &mut Node) -> Result<AnalysisResult, Error> {
        let source = node.range();
        let result = match node.kind_mut() {
            NodeKind::Declaration(v) => {
                self.analyse_declaration(v, source)?;
                AnalysisResult::new(DataType::new(source, DataTypeKind::Unit), true)
            },

            
            NodeKind::Statement(v) => {
                self.analyse_statement(v, source)?;
                AnalysisResult::new(DataType::new(source, DataTypeKind::Unit), true)
            },

            
            NodeKind::Expression(e) => {
                self.analyse_expression(e, source)?
            },
        };

        node.data_kind = result.data_type.kind().clone();
        Ok(result)
    }


    fn analyse_statement(&mut self, stmt: &mut Statement, source: SourceRange) -> Result<(), Error> {
        match stmt {
            Statement::Variable { name, hint, is_mut, rhs } => {
                let rhs_typ = self.analyse_node(&mut *rhs);

                // Decoy value in the case of the variable
                // throwing an error
                let index = self.ctx.cs.variables.len();
                self.ctx.cs.register_variable(Variable::new(*name, DataType::new(source, DataTypeKind::Unknown), *is_mut));

                let rhs_typ = match rhs_typ {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };

                if let Some(hint) = hint {
                    if !rhs_typ.data_type.kind().is(hint.kind()) {
                        return Err(
                            CompilerError::new(
                                source.file(), 
                                ErrorCode::SVarHintTypeDiff, 
                                "variable value differs from type hint"
                            )
                            .highlight(source)
                                .note(format!("variable has the type hint {} but the value is {}",
                                        display_type(self.symbol_map, hint.kind()),
                                        display_type(self.symbol_map, rhs_typ.data_type.kind())
                                    ))
                            .build()
                        )
                    }
                }
                

                let variable = Variable::new(*name, DataType::new(source, rhs_typ.data_type.kind_owned()), *is_mut);
                self.ctx.cs.variables[index] = variable;
                

                Ok(())
            },

            
            Statement::UpdateValue { lhs, rhs } => {
                let lhs_typ = self.analyse_node(&mut *lhs)?;
                let rhs_typ = self.analyse_node(&mut *rhs)?;

                if lhs_typ.accessor_field_is_an_enum || !is_l_val(lhs.kind()) {
                    return Err(
                        CompilerError::new(source.file(), ErrorCode::SAssignValNotLHS, "invalid assignment target")
                            .highlight(lhs.range())
                            .build()
                    )
                }


                if !lhs_typ.mutability {
                    return Err(
                        CompilerError::new(source.file(), ErrorCode::SAssignValNotMut, "assignment target isn't mutable")
                            .highlight(lhs.range())
                            .build()
                    )
                }


                if !lhs_typ.data_type.kind().is(rhs_typ.data_type.kind()) {
                    return Err(
                        CompilerError::new(source.file(), ErrorCode::SAssignValDiffTy, "assignment target's type and the value's types mismatch")
                            .highlight(source)
                                .note(format!(
                                    "assignment target is of type '{}' but the value is '{}'",
                                    display_type(self.symbol_map, lhs_typ.data_type.kind()),
                                    display_type(self.symbol_map, rhs_typ.data_type.kind()),
                                ))
                            .build()
                    )
                    
                }

                Ok(())
            },
        }
    }


    fn register_declarations(
        &mut self, 
        // We can't just take an iterator of
        // `Declaration` as rust doesn't allow
        // the iteration of iterators more than once
        // so this function should always filter for
        // declarations before doing anything
        nodes: &mut [Node],
    ) -> Result<(), Error> {
        // - name renaming
        // - base registeration
        fn register_stage_1(slf: &mut Infer, nodes: &mut [Node]) -> Result<(), Error> {
            let mut errors = vec![];
            for node in nodes.iter_mut() {
                let source = node.range();
                let NodeKind::Declaration(decl) = node.kind_mut() else { continue };

                match decl {
                    Declaration::Struct { kind, name, .. } => {
                        let identifier = *name;
                        *name = slf.ctx.cs.mangle(slf.symbol_map, *name, source);

                        let structure = Structure {
                            name: identifier,
                            full_name: *name,
                            fields: vec![], // will be initialised later
                            kind: *kind,
                            source,
                        };

                        let result = slf.ctx.cs.create_symbol(&mut slf.symbols, &mut slf.name_to_symbol, Symbol::Structure(structure), identifier, source);
                        if let Err(e) = result {
                            errors.push(e);
                        }


                        assert!(slf.namespaces.insert(*name, Scope::new(identifier, slf.ctx.file)).is_none());
                    },


                    Declaration::Enum { name, .. } => {
                        let identifier = *name;
                        *name = slf.ctx.cs.mangle(slf.symbol_map, *name, source);

                        let enum_val = Enum {
                            name: identifier,
                            full_name: *name,
                            mappings: vec![], // will be initialised later
                            source,
                        };

                        
                        let result = slf.ctx.cs.create_symbol(&mut slf.symbols, &mut slf.name_to_symbol, Symbol::Enum(enum_val), identifier, source);
                        slf.ctx.cs.put_namespace(identifier, *name, source)?;
                        if let Err(e) = result {
                            errors.push(e);
                        }
                    },


                    Declaration::Function { is_system, is_anonymous, name, arguments: _, return_type, body: _ } => {
                        let identifier = *name;
                        *name = slf.ctx.cs.mangle(slf.symbol_map, *name, source);

                        let function = Function {
                            name: identifier,
                            full_name: *name,
                            args: vec![], // will be initialised later
                            is_extern: None,
                            is_system: *is_system,
                            is_anonymous: *is_anonymous,
                            is_enum_variant_function: None,
                            // IMPORTANT: This should be updated before any block of body
                            //            is ran.
                            return_type: return_type.clone(),
                            source,

                            body: vec![],
                        };

                        
                        let result = slf.ctx.cs.create_symbol(&mut slf.symbols, &mut slf.name_to_symbol, Symbol::Function(function), identifier, source);
                        if let Err(e) = result {
                            errors.push(e);
                        }
                    }

                    
                    Declaration::Module { name, body } => {
                        let identifier = *name;
                        *name = slf.ctx.cs.mangle(slf.symbol_map, *name, source);
                        slf.ctx.cs.put_namespace(identifier, *name, source)?;

                        let scope = Scope::new(*name, source.file());
                        slf.namespaces.insert(*name, scope.clone());

                        slf.ctx.subscope(scope);

                        let result = register_stage_1(slf, body);
                        
                        let scope = slf.ctx.pop_scope();

                        result?;

                        if !slf.namespaces.contains_key(name) {
                            slf.namespaces.insert(*name, scope);
                            continue;
                        }
                        
                        let namespace = slf.namespaces.get_mut(name).unwrap();

                        let mut errors = vec![];
                        {
                            namespace.symbols.reserve(scope.symbols.len());
                            for s in scope.symbols {
                                namespace.put_symbol(s.1.0, s.0, s.1.1).unwrap();
                                // if let Err(e) =  {
                                    // errors.push(e)
                                // }
                            }
                        }
                        {
                            namespace.namespaces.reserve(scope.namespaces.len());
                            for s in scope.namespaces {
                                if let Err(e) = namespace.put_namespace(s.0, s.1.0, s.1.1) {
                                    errors.push(e)
                                }
                            }
                        }

                        if !errors.is_empty() {
                            return Err(errors.combine_into_error())
                        }
                    },

                    
                    Declaration::Extern { file, functions } => {
                        let mut errors = vec![];

                        for f in functions.iter() {
                            let function = Function { 
                                name: f.name(), 
                                full_name: slf.ctx.cs.mangle(slf.symbol_map, f.name(), f.range()), 
                                args: f.args().to_vec(), 
                                is_extern: Some((*file, f.path())), 
                                is_system: false,
                                is_anonymous: false,
                                is_enum_variant_function: None,
                                return_type: f.return_type().clone(),
                                source,

                                body: vec![],
                            };
                            
                            let result = slf.ctx.cs.create_symbol(&mut slf.symbols, &mut slf.name_to_symbol, Symbol::Function(function), f.name(), f.range());

                            if let Err(e) = result {
                                errors.push(e);
                            }
                        }
                    },


                    Declaration::Impl { .. } => (),
                    Declaration::Using { .. } => todo!(),
                }
            
            }


            for node in nodes.iter_mut() {
                let source = node.range();
                let NodeKind::Declaration(decl) = node.kind_mut() else { continue };


                match decl {
                    Declaration::Impl { data_type, body } => {
                        slf.validate_data_type(data_type)?;

                        let name = slf.type_namespace(data_type);

                        let scope = Scope::new(name, source.file());

                        slf.ctx.subscope(scope);

                        let result = register_stage_1(slf, body);
                        
                        let scope = slf.ctx.pop_scope();

                        result?;

                        let namespace = slf.namespaces.get_mut(&name).unwrap();

                        let mut errors = vec![];
                        {
                            namespace.symbols.reserve(scope.symbols.len());
                            for s in scope.symbols {
                                if let Err(e) = namespace.put_symbol(s.1.0, s.0, s.1.1) {
                                    errors.push(e)
                                }
                            }
                        }
                        {
                            namespace.namespaces.reserve(scope.namespaces.len());
                            for s in scope.namespaces {
                                if let Err(e) = namespace.put_namespace(s.0, s.1.0, s.1.1) {
                                    errors.push(e)
                                }
                            }
                        }

                        if !errors.is_empty() {
                            return Err(errors.combine_into_error())
                        }

                    },

                    
                    Declaration::Using { .. } => todo!(),
                    Declaration::Module { .. } => (),
                    Declaration::Extern { .. } => (),

                    _ => ()
                }
                
            }


            if !errors.is_empty() {
                return Err(errors.combine_into_error())
            }

            Ok(())
        }


        // type updating
        fn register_stage_2(slf: &mut Infer, nodes: &mut [Node]) -> Result<(), Error> {
            let mut errors = vec![];
            for node in nodes.iter_mut() {
                let source = node.range();
                let NodeKind::Declaration(decl) = node.kind_mut() else { continue };

                match decl {
                    Declaration::Struct { kind: _, name, fields } => {
                        for i in 0..fields.len() {
                            let f = fields.get_mut(i).unwrap();
                            let result = slf.validate_data_type(&mut f.1);

                            if let Err(e) = result {
                                errors.push(e);
                            }

                            let f = fields.get(i).unwrap();
                            if let Some(v) = fields[0..i].iter().find(|x| x.0 == f.0) {
                                errors.push(CompilerError::new(
                                    slf.ctx.file, 
                                    ErrorCode::SFieldDefEarlier, 
                                    "field is already defined"
                                )
                                .highlight(v.2)
                                    .note("field is defined earlier here".to_string())
                                .highlight(f.2)
                                    .note("..but it's defined again here".to_string())
                                .build())
                            }
                        }


                        {
                            let id = slf.symbols.vec
                                .iter_mut()
                                .rev()
                                .filter_map(|x| {
                                    match x {
                                        Symbol::Structure(v) => Some(v),
                                        _ => None
                                    }
                                })
                                .find(|x| x.full_name == *name).unwrap();

                            // TODO: Might be able to remove the clone here 
                            //       depending on how the rest of this goes
                            id.fields = fields.clone();
                        }
                        
                    },


                    Declaration::Enum { name, mappings } => {
                        for i in 0..mappings.len() {
                            let m = mappings.get_mut(i).unwrap();
                            let result = slf.validate_data_type(m.data_type_mut());

                            if let Err(e) = result {
                                errors.push(e);
                            }

                            let m = mappings.get(i).unwrap();
                            if let Some(v) = mappings[0..i].iter().find(|x| x.name() == m.name()) {
                                errors.push(CompilerError::new(
                                    slf.ctx.file, 
                                    ErrorCode::SVariantDefEarlier, 
                                    "variant is already defined"
                                )
                                .highlight(v.range())
                                    .note("variant is defined earlier here".to_string())
                                .highlight(m.range())
                                    .note("..but it's defined again here".to_string())
                                .build())
                            }
                        }


                        {
                            let (structure, index) = slf.symbols.vec
                                .iter_mut()
                                .enumerate()
                                .rev()
                                .filter_map(|x| {
                                    match x.1 {
                                        Symbol::Enum(v) => Some((v, x.0)),
                                        _ => None
                                    }
                                })
                                .find(|x| x.0.full_name == *name).unwrap();

                            // TODO: Might be able to remove the clone here 
                            //       depending on how the rest of this goes
                            structure.mappings = mappings.clone();
                            let name = structure.full_name;

                            slf.register_enum_methods(SymbolId(index), source, DataTypeKind::CustomType(name))?;

                            let structure = &slf.symbols[SymbolId(index)];
                            let Symbol::Enum(structure) = structure else { panic!() };
                            slf.ctx.cs.namespaces.insert(structure.name, (structure.full_name, source));
                        }
                    }


                    Declaration::Function { is_system: _, is_anonymous: _, name, arguments, return_type, body: _ } => {                        
                        for i in 0..arguments.len() {
                            let f = arguments.get_mut(i).unwrap();
                            let result = slf.validate_data_type(f.data_type_mut());

                            if let Err(e) = result {
                                errors.push(e);
                            }

                            let f = arguments.get(i).unwrap();
                            if let Some(v) = arguments[0..i].iter().find(|x| x.name() == f.name()) {
                                errors.push(CompilerError::new(
                                    slf.ctx.file, 
                                    ErrorCode::SArgDefEarlier, 
                                    "argument is already defined"
                                )
                                .highlight(v.range())
                                    .note("argument is defined earlier here".to_string())
                                .highlight(f.range())
                                    .note("..but it's defined again here".to_string())
                                .build())
                            }
                        }


                        {
                            let result = slf.validate_data_type(return_type);
                            if let Err(e) = result {
                                errors.push(e);
                            }
                        }


                        {
                            let id = slf.symbols.vec
                                .iter_mut()
                                .rev()
                                .filter_map(|x| {
                                    match x {
                                        Symbol::Function(v) => Some(v),
                                        _ => None
                                    }
                                })
                                .find(|x| x.full_name == *name).unwrap();

                            // TODO: Might be able to remove the clone here 
                            //       depending on how the rest of this goes
                            id.args = arguments.clone();
                            id.return_type = return_type.clone();
                        }
                    }

                    
                    Declaration::Impl { data_type, body } => {
                        let name = slf.type_namespace(data_type);


                        let namespace = slf.namespaces.get(&name).unwrap().clone();
                        slf.ctx.subscope(namespace);
                        let result = register_stage_2(slf, body);
                        let namespace = slf.ctx.pop_scope();

                        result?;

                        *slf.namespaces.get_mut(&name).unwrap() = namespace;

                    },

                    
                    Declaration::Using { .. } => todo!(),

                    
                    Declaration::Module { name, body } => {
                        let namespace = slf.namespaces.get(name).unwrap().clone();
                        slf.ctx.subscope(namespace);
                        let result = register_stage_2(slf, body);
                        let namespace = slf.ctx.pop_scope();

                        result?;

                        *slf.namespaces.get_mut(name).unwrap() = namespace;

                        
                    },

                    
                    Declaration::Extern { .. } => (),


                }
            
            }

            if !errors.is_empty() {
                return Err(errors.combine_into_error())
            }

            Ok(())
        }

        
        register_stage_1(self, nodes)?;
        register_stage_2(self, nodes)?;

        
        Ok(())
    }
}



fn display_type(symbol_map: &SymbolMap, typ: &DataTypeKind) -> String {
    let mut string = String::with_capacity(8);
    display_type_in(symbol_map, typ, &mut string);
    string
}


fn display_type_in(symbol_map: &SymbolMap, typ: &DataTypeKind, str: &mut String) {
    let _ = write!(str, "{}", 
        match typ {
            DataTypeKind::Int => "int",
            DataTypeKind::Bool => "bool",
            DataTypeKind::Float => "float",
            DataTypeKind::Unit => "unit",
            DataTypeKind::Any => "any",
            DataTypeKind::Unknown => "unknown",
            DataTypeKind::Never => "!",

            DataTypeKind::Result(v1, v2) => {
                if matches!(v1.kind(), DataTypeKind::Result(_, _)) { let _ = write!(str, "("); }
                display_type_in(symbol_map, v1.kind(), str);
                if matches!(v1.kind(), DataTypeKind::Result(_, _)) { let _ = write!(str, ")"); }

                let _ = write!(str, " ~ ");
                
                if matches!(v2.kind(), DataTypeKind::Result(_, _)) { let _ = write!(str, "("); }
                display_type_in(symbol_map, v2.kind(), str);
                if matches!(v2.kind(), DataTypeKind::Result(_, _)) { let _ = write!(str, ")"); }

                return
            },
            
            DataTypeKind::Option(v) => {
                display_type_in(symbol_map, v.kind(), str);
                let _ = write!(str, "?");
                return
            },

            
            DataTypeKind::CustomType(v) => {
                let name = symbol_map.get(*v);
                let _ = write!(str, "{name}");
                return
            }
        }
    );
}


pub fn is_l_val(node: &NodeKind) -> bool {
    match node {
        NodeKind::Expression(Expression::Identifier(_)) => true,
        NodeKind::Expression(Expression::AccessField { val, .. }) => is_l_val(val.kind()),
        NodeKind::Expression(Expression::Unwrap(val)) => is_l_val(val.kind()),
        NodeKind::Expression(Expression::OrReturn(val)) => is_l_val(val.kind()),

        _ => false,
    }
}
