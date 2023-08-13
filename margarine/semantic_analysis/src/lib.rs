#![feature(if_let_guard)]
#![allow(clippy::map_entry)]
use std::{cell::{RefCell, Ref}, collections::HashMap, convert::identity, fmt::Write, ops::Deref, sync::Arc};

use common::{SymbolMap, SourceRange, SymbolIndex};
use errors::{Error, CombineIntoError, CompilerError, ErrorCode, ErrorBuilder};
use istd::index_vec;
use lexer::Literal;
use parser::{nodes::{Node, NodeKind, Declaration, StructKind, FunctionArgument, Statement, Expression, UnaryOperator, EnumMapping}, DataType, Block, DataTypeKind};


index_vec!(SymbolList, SymbolId, Symbol);


pub fn semantic_analysis<'a>(symbol_map: &'a mut SymbolMap, nodes: &mut Block) -> Result<GlobalState<'a>, Error> {
    let state = GlobalState {
        inner: RefCell::new(InnerGlobalState {
            symbol_map,
            scope_list: vec![],
            scope_amount: 0,
            symbols: SymbolList::new(),
        }),
    };

    let mut scope = Scope::new(None, nodes.range().file(), nodes.range().file(), &state);
    scope.analyse_block(nodes)?;
    drop(scope);

    state.inner.borrow_mut().scope_list.sort_unstable_by_key(|x| x.id.0);

    Ok(state)
}


#[derive(Debug)]
pub struct GlobalState<'a> {
    inner: RefCell<InnerGlobalState<'a>>,
}


impl GlobalState<'_> {
    fn new_scope_id(&self) -> ScopeId {
        let mut i = self.inner.borrow_mut();
        i.scope_amount += 1;
        ScopeId(i.scope_amount-1)
    }


    fn add_scope_metadata(&self, metadata: ScopeMetadata) {
        self.inner.borrow_mut().scope_list.push(metadata)
    }

    
    fn insert(&self, str: String) -> SymbolIndex { 
        self.inner.borrow_mut().symbol_map.insert(str)
    }


    fn add_symbol(&self, symbol: Symbol) -> SymbolId {
        self.inner.borrow_mut().symbols.push(symbol)
    }
    

    fn find_symbol(&self, name: SymbolIndex) -> Ref<'_, Symbol>{
        let borrow = self.inner.borrow();
        Ref::<'_, InnerGlobalState<'_>>::map(borrow, |x| x.symbols
            .as_slice()
            .iter()
            .find(|x| x.full_name() == name)
            .unwrap_or(&Symbol::Invalid)
        )
    }

}


#[derive(Debug)]
struct InnerGlobalState<'a> {
    symbol_map: &'a mut SymbolMap,

    scope_list: Vec<ScopeMetadata>,
    scope_amount: usize,

    symbols: SymbolList,
}


struct Scope<'a, 'b> {
    parent: Option<&'a Scope<'a, 'b>>,
    global: &'a GlobalState<'b>,
    metadata: ScopeMetadata,
}


#[derive(Clone, Default, Debug)]
struct ScopeMetadata {
    parent_id: Option<ScopeId>,
    id: ScopeId,
    path: SymbolIndex,
    depth: usize,
    file: SymbolIndex,

    symbols: HashMap<SymbolIndex, SymbolId>,
    namespaces: HashMap<SymbolIndex, ScopeId>,

    variables: Vec<Variable>,
}


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
struct ScopeId(usize);


impl Drop for Scope<'_, '_> {
    fn drop(&mut self) {
        self.global.add_scope_metadata(std::mem::take(&mut self.metadata))
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum Symbol {
    Structure(Structure),
    Enum(Enum),
    Function(Function),
    Invalid,
}


impl Symbol {
    pub fn full_name(&self) -> SymbolIndex {
        match self {
            Symbol::Structure(v) => v.full_name,
            Symbol::Enum(v) => v.full_name,
            Symbol::Function(v) => v.full_name,
            Symbol::Invalid => panic!(),
        }
    }


    pub fn type_name(&self) -> &'static str {
        match self {
            Symbol::Structure(_) => "structure",
            Symbol::Enum(_) => "enum",
            Symbol::Function(_) => "function",
            Symbol::Invalid => panic!(),
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct Structure {
    name: SymbolIndex,
    full_name: SymbolIndex,
    fields: Vec<(SymbolIndex, DataType, SourceRange)>,
    kind: StructKind,
}


#[derive(Clone, Debug, PartialEq)]
pub struct Enum {
    name: SymbolIndex,
    full_name: SymbolIndex,
    mappings: Vec<EnumMapping>,
}


#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    name: SymbolIndex,
    full_name: SymbolIndex,
    args: Vec<FunctionArgument>,
    is_system: bool,
    is_anonymous: bool,
    return_type: DataType,
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


struct AnalysisReport {
    data_type: DataType,
    mutability: bool,
}

impl AnalysisReport {
    fn new(data_type: DataType, mutability: bool) -> Self { Self { data_type, mutability } }
}


impl<'a, 'b> Scope<'a, 'b> {
    fn new(parent: Option<&'a Self>, file: SymbolIndex, path: SymbolIndex, global: &'a GlobalState<'b>) -> Self {
        Self {
            parent,
            global,
            metadata: ScopeMetadata { 
                parent_id: parent.map(|x| x.metadata.id), 
                id: global.new_scope_id(),
                path,
                depth: parent.map(|x| x.metadata.depth + 1).unwrap_or(0),
                symbols: HashMap::new(),
                namespaces: HashMap::new(),
                file,
                variables: Vec::new(),
            },
        }
    }

    
    fn subscope(&'a self) -> Self {
        Scope::new(
            Some(self),
            self.metadata.file,
            self.metadata.path,
            self.global,
        )
    }


    fn mangle(&self, symbol: SymbolIndex, source: SourceRange) -> SymbolIndex {
        let r = self.global.inner.borrow();
        let str = format!(
            "{}::{}({})", 
            r.symbol_map.get(self.metadata.path), 
            r.symbol_map.get(symbol), 
            source.start(),
        );
        drop(r);

        self.global.insert(str)
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
            DataTypeKind::CustomType(v) => {
                let symbol = self.find_symbol_id(*v, range)?;
                let borrow = self.global.inner.borrow();
                let symbol = borrow.symbols.get(symbol).unwrap();
                match symbol {
                    | Symbol::Enum(_)
                    | Symbol::Structure(_) => {
                        *v = symbol.full_name();
                        return Ok(())
                    },

                    _ => (),
                };


                *data_type.kind_mut() = DataTypeKind::Unknown;
                Err(CompilerError::new(self.metadata.file, ErrorCode::SSymbolIsntType, "symbol isn't a type")
                    .highlight(data_type.range())
                    .build())
            },
        }
    }


    ///
    /// Adds a `Symbol` to the global namespace
    ///
    /// It will **NOT** add the symbol to the local
    /// namespace if the symbol is an anonymous function
    ///
    /// # Errors
    /// This function will return a "name already defined"
    /// error if the identifier is defined in the current
    /// scope.
    ///
    /// # Important
    /// This function will add the symbol to the global
    /// namespace regardless of whether it errored or not
    ///
    /// But it will **NOT** add it to the local namespace if
    /// it returns an error
    ///
    fn create_symbol(&mut self, symbol: Symbol, identifier: SymbolIndex, source: SourceRange) -> Result<(), Error> {
        let is_anonymous_function = matches!(symbol, Symbol::Function(Function { is_anonymous: true, .. }));
        let id = self.global.add_symbol(symbol);
        
        // Anonymous functions are meant to be
        // unreachable and thus there's no need
        // to add them to the local namespace
        if is_anonymous_function {
            return Ok(())
        }
        
        if self.metadata.symbols.contains_key(&identifier) {
            return Err(CompilerError::new(
                    self.metadata.file, 
                    ErrorCode::SNameAlrDefined, 
                    "name is already defined in the namespace"
                )
                .highlight(source)
                .build()
            );
        }
        
        assert!(self.metadata.symbols.insert(identifier, id).is_none());
        Ok(())
    }


    fn register_variable(&mut self, variable: Variable) {
        self.metadata.variables.push(variable)
    }


    ///
    /// Searches for a variable from the end of the
    /// variables list, if it fails to find one it
    /// tries to recursively search on the parent scope.
    ///
    /// # Errors
    /// Returns a "variable not found" error if there
    /// are no more parents left && the variable doesn't exist
    ///
    fn find_variable(&self, name: SymbolIndex, source: SourceRange) -> Result<&Variable, Error> {
        if let Some(v) = self.metadata.variables.iter().rev().find(|x| x.name == name) {
            return Ok(v)
        }


        if let Some(parent) = self.parent {
            return parent.find_variable(name, source);
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
    /// Searches for a symbol in the current scope,
    /// if the symbol is unable to be found it will
    /// recursively search through the parent aswell.
    ///
    /// # Errors
    /// Returns a "symbol not found" error if there
    /// are no more parents left && the symbol doesn't
    /// exist
    ///
    fn find_symbol_id(&self, name: SymbolIndex, source: SourceRange) -> Result<SymbolId, Error> {
        println!("looking for {name:?} which is {}", self.global.inner.borrow().symbol_map.get(name));
        if let Some(v) = self.metadata.symbols.get(&name) {
            return Ok(*v)
        }


        if let Some(parent) = self.parent {
            return parent.find_symbol_id(name, source);
        }

        Err(CompilerError::new(
                source.file(), 
                ErrorCode::SSymbolUnreachable, 
                "symbol not defined"
            )
            .highlight(source)
            .build()
        )
    }


    ///
    /// Searches for a namespace in the current
    /// scope, if the namespace is unable to be
    /// found it will recursively search through
    /// the parent.
    ///
    /// # Errors
    /// Returns a "namespace not found" error if there
    /// are no more parents left && the namespace doesn't
    /// exist
    ///
    fn find_namespace(&self, name: SymbolIndex, source: SourceRange) -> Result<ScopeId, Error> {
        if let Some(v) = self.metadata.namespaces.get(&name) {
            return Ok(*v)
        }


        if let Some(parent) = self.parent {
            return parent.find_namespace(name, source);
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


    fn display_type(&self, typ: &DataTypeKind) -> String {
        let mut string = String::with_capacity(8);
        self.display_type_in(typ, &mut string);
        string
    }


    fn display_type_in(&self, typ: &DataTypeKind, str: &mut String) {
        let _ = write!(str, "{}", 
            match typ {
                DataTypeKind::Int => "int",
                DataTypeKind::Bool => "bool",
                DataTypeKind::Float => "float",
                DataTypeKind::Unit => "unit",
                DataTypeKind::Any => "any",
                DataTypeKind::Unknown => "unknown",

                
                DataTypeKind::Option(v) => {
                    self.display_type_in(v.kind(), str);
                    let _ = write!(str, "?");
                    return
                },

                
                DataTypeKind::CustomType(v) => {
                    let borrow = self.global.inner.borrow();
                    let name = borrow.symbol_map.get(*v);
                    let _ = write!(str, "{name}");
                    return
                }
            }
        );
        
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
                        self.display_type(expect),
                        self.display_type(value.kind())
                    ))
                .build()
        )
    }
}


impl Scope<'_, '_> {
    fn analyse_block(&mut self, block: &mut Block) -> Result<AnalysisReport, Error> {
        let mut subscope = self.subscope();
        let mut errors = vec![];


        subscope.register_declarations(block)?;


        let result = block
            .iter_mut()
            .map(|x| subscope.analyse_node(x))
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
                AnalysisReport::new(DataType::new(range, DataTypeKind::Unknown), true)
            });
        

        if !errors.is_empty() {
            return Err(errors.combine_into_error())
        }

        Ok(result)
    }


    fn analyse_node(&mut self, node: &mut Node) -> Result<AnalysisReport, Error> {
        let source = node.range();
        match node.kind_mut() {
            NodeKind::Declaration(v) => {
                self.analyse_declaration(v, source)?;
                Ok(AnalysisReport { data_type: DataType::new(source, DataTypeKind::Unit), mutability: true })
            },

            
            NodeKind::Statement(v) => {
                self.analyse_statement(v, source)?;
                Ok(AnalysisReport { data_type: DataType::new(source, DataTypeKind::Unit), mutability: true })
            },

            
            NodeKind::Expression(e) => {
                self.analyse_expression(e, source)
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
        {
            let mut errors = vec![];
            for node in nodes.iter_mut() {
                let source = node.range();
                let NodeKind::Declaration(decl) = node.kind_mut() else { continue };

                match decl {
                    Declaration::Struct { kind, name, .. } => {
                        let identifier = *name;
                        *name = self.mangle(*name, source);

                        let structure = Structure {
                            name: identifier,
                            full_name: *name,
                            fields: vec![], // will be initialised later
                            kind: *kind,
                        };

                        let result = self.create_symbol(Symbol::Structure(structure), identifier, source);
                        if let Err(e) = result {
                            errors.push(e);
                        }


                        if !self.metadata.namespaces.contains_key(&identifier) {
                            let subscope = self.subscope();
                            let id = subscope.metadata.id;
                            drop(subscope);
                            self.metadata.namespaces.insert(identifier, id);
                        }
                    },


                    Declaration::Enum { name, .. } => {
                        let identifier = *name;
                        *name = self.mangle(*name, source);

                        let enum_val = Enum {
                            name: identifier,
                            full_name: *name,
                            mappings: vec![], // will be initialised later
                        };

                        
                        let result = self.create_symbol(Symbol::Enum(enum_val), identifier, source);
                        if let Err(e) = result {
                            errors.push(e);
                        }
                    },


                    Declaration::Function { is_system, is_anonymous, name, arguments: _, return_type, body: _ } => {
                        let identifier = *name;
                        *name = self.mangle(*name, source);

                        let function = Function {
                            name: identifier,
                            full_name: *name,
                            args: vec![], // will be initialised later
                            is_system: *is_system,
                            is_anonymous: *is_anonymous,
                            // IMPORTANT: This should be updated before any block of body
                            //            is ran.
                            return_type: return_type.clone(),
                        };

                        
                        let result = self.create_symbol(Symbol::Function(function), identifier, source);
                        if let Err(e) = result {
                            errors.push(e);
                        }
                    }

                    
                    Declaration::Impl { data_type, body } => todo!(),
                    Declaration::Using { file } => todo!(),
                    Declaration::Module { name, body } => todo!(),
                    Declaration::Extern { file, functions } => todo!(),

                }
            
            }


            if !errors.is_empty() {
                return Err(errors.combine_into_error())
            }
        }


        // type updating
        {
            let mut errors = vec![];
            for node in nodes.iter_mut() {
                let _source = node.range();
                let NodeKind::Declaration(decl) = node.kind_mut() else { continue };

                match decl {
                    Declaration::Struct { kind: _, name, fields } => {
                        for i in 0..fields.len() {
                            let f = fields.get_mut(i).unwrap();
                            let result = self.validate_data_type(&mut f.1);

                            if let Err(e) = result {
                                errors.push(e);
                            }

                            let f = fields.get(i).unwrap();
                            if let Some(v) = fields[0..i].iter().find(|x| x.0 == f.0) {
                                errors.push(CompilerError::new(
                                    self.metadata.file, 
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
                            let mut borrow = self.global.inner.borrow_mut();
                            let id = borrow
                                .symbols.vec.iter_mut()
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
                            let result = self.validate_data_type(&mut m.data_type_mut());

                            if let Err(e) = result {
                                errors.push(e);
                            }

                            let m = mappings.get(i).unwrap();
                            if let Some(v) = mappings[0..i].iter().find(|x| x.name() == m.name()) {
                                errors.push(CompilerError::new(
                                    self.metadata.file, 
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
                            let mut borrow = self.global.inner.borrow_mut();
                            let (structure, index)= borrow
                                .symbols.vec.iter_mut()
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
                            drop(borrow);

                            
                            let mut subscope = self.subscope();
                            subscope.metadata.path = *name;

                            for m in mappings.iter() {
                                let func = Function {
                                    name: m.name(),
                                    full_name: subscope.mangle(m.name(), m.range()),
                                    args: {
                                        if m.is_implicit_unit() {
                                            vec![]
                                        } else {
                                            vec![FunctionArgument::new(m.name(), m.data_type().clone(), false, m.range())]
                                        }
                                    },
                                    is_system: false,
                                    is_anonymous: false,
                                    return_type: DataType::new(m.range(), DataTypeKind::CustomType(*name)),
                                };

                                subscope.create_symbol(Symbol::Function(func), m.name(), m.range())?;
                            }

                            let id = subscope.metadata.id;
                            let mut borrow = self.global.inner.borrow_mut();
                            let Symbol::Enum(structure) = &mut borrow.symbols[SymbolId(index)] else { unreachable!() };
                            let name = structure.name;
                            drop(borrow);
                            drop(subscope);

                            assert!(self.metadata.namespaces.insert(name, id).is_none());
                        }
                    }


                    Declaration::Function { is_system: _, is_anonymous: _, name, arguments, return_type, body: _ } => {                        
                        for i in 0..arguments.len() {
                            let f = arguments.get_mut(i).unwrap();
                            let result = self.validate_data_type(f.data_type_mut());

                            if let Err(e) = result {
                                errors.push(e);
                            }

                            let f = arguments.get(i).unwrap();
                            if let Some(v) = arguments[0..i].iter().find(|x| x.name() == f.name()) {
                                errors.push(CompilerError::new(
                                    self.metadata.file, 
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
                            let result = self.validate_data_type(return_type);
                            if let Err(e) = result {
                                errors.push(e);
                            }
                        }


                        {
                            let mut borrow = self.global.inner.borrow_mut();
                            let id = borrow
                                .symbols.vec.iter_mut()
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

                    
                    Declaration::Impl { data_type, body } => todo!(),
                    Declaration::Using { file } => todo!(),
                    Declaration::Module { name, body } => todo!(),
                    Declaration::Extern { file, functions } => todo!(),


                }
            
            }


            if !errors.is_empty() {
                return Err(errors.combine_into_error())
            }
        }

        Ok(())
    }



    fn analyse_declaration(&mut self, decl: &mut Declaration, source: SourceRange) -> Result<(), Error> {
        match decl {
            Declaration::Struct { .. } => Ok(()),
            Declaration::Enum { .. } => Ok(()),
            Declaration::Function { is_system, is_anonymous, name, arguments, return_type, body } => {

                // evaluate body
                let block_return_type = {
                    let mut subscope = self.subscope();

                    for i in arguments {
                        let variable = Variable {
                            name: i.name(),
                            data_type: i.data_type().clone(),
                            is_mut: true,
                        };

                        subscope.register_variable(variable);
                    }
                    
                    subscope.analyse_block(body)?
                };


                if !block_return_type.data_type.kind().is(return_type.kind()) {
                    return Err(
                        CompilerError::new(
                            source.file(), 
                            ErrorCode::SFuncReturnDiff, 
                            "function return type differs from body"
                        )
                        .highlight(source)
                            .note(format!("function returns {} but the body returns {}",
                                    self.display_type(return_type.kind()),
                                    self.display_type(block_return_type.data_type.kind())
                                ))
                        .build()
                    )
                }


                Ok(())
            },
            Declaration::Impl { data_type, body } => todo!(),
            Declaration::Using { file } => todo!(),
            Declaration::Module { name, body } => todo!(),
            Declaration::Extern { file, functions } => todo!(),
        }
    }



    fn analyse_statement(&mut self, stmt: &mut Statement, source: SourceRange) -> Result<(), Error> {
        Ok(())
    }



    fn analyse_expression(&mut self, expr: &mut Expression, source: SourceRange) -> Result<AnalysisReport, Error> {
        let result = match expr {
            Expression::Unit => AnalysisReport::new(DataType::new(source, DataTypeKind::Unit), true),

            
            Expression::Literal(v) => {
                let kind = match v {
                    Literal::Float(_)   => DataTypeKind::Float,
                    Literal::Integer(_) => DataTypeKind::Int,
                    Literal::String(_)  => todo!(),
                    Literal::Bool(_)    => DataTypeKind::Bool,
                };

                AnalysisReport::new(DataType::new(source, kind), true)
            },


            Expression::Identifier(v) => {
                let variable = self.find_variable(*v, source)?;
                
                AnalysisReport::new(variable.data_type.clone(), variable.is_mut)
            },

            
            Expression::BinaryOp { operator, lhs, rhs } => {
                let lhs_typ = self.analyse_node(&mut *lhs)?.data_type;
                let rhs_typ = self.analyse_node(&mut *rhs)?.data_type;

                if lhs_typ.kind() == &DataTypeKind::Unknown
                    || rhs_typ.kind() == &DataTypeKind::Unknown {
                    return Ok(AnalysisReport::new(DataType::new(source, DataTypeKind::Unknown), true))
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
                                    self.display_type(lhs_typ.kind()), 
                                    self.display_type(rhs_typ.kind())
                                ))
                            .build()
                    )
                };

                AnalysisReport::new(DataType::new(source, kind), true)
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

                AnalysisReport::new(DataType::new(source, kind), true)
            },

            
            Expression::If { condition, body, else_block } => {
                let condition_typ = self.analyse_node(&mut *condition)?.data_type;

                self.expect_type(&DataTypeKind::Bool, &condition_typ)?;

                let mut body_typ = self.analyse_block(body)?;

                if let Some(else_block) = else_block {
                    let else_typ = self.analyse_node(&mut *else_block)?;
                    self.expect_type(body_typ.data_type.kind(), &else_typ.data_type)?;
                    if !else_typ.mutability {
                        body_typ.mutability = false
                    }

                } else if !body_typ.data_type.kind().is(&DataTypeKind::Unit) {
                    return Err(
                        CompilerError::new(source.file(), ErrorCode::SIfExprNoElse, "if expression has no else")
                        .highlight(source)
                            .note(format!(
                                "the body returns {} but there's no else block",
                                self.display_type(body_typ.data_type.kind())
                            ))
                        .build()
                    )
                }


                AnalysisReport::new(
                    DataType::new(source, body_typ.data_type.kind_owned()), 
                    body_typ.mutability,
                )
            },

            
            Expression::Match { value, mappings } => {
                let value_typ = self.analyse_node(&mut *value)?;

                let e = match value_typ.data_type.kind() {
                    DataTypeKind::CustomType(v) => {
                        let symbol = self.global.find_symbol(*v);

                        match symbol.deref() {
                            Symbol::Enum(_) => Ref::map(symbol, |x| match x {
                                Symbol::Enum(e) => e,

                                _ => unreachable!(),
                            }),
                            
                            _ => return Err(
                                CompilerError::new(source.file(), ErrorCode::SMatchValNotEnum, "match value is not an enum")
                                    .highlight(source)
                                        .note(format!("is of type '{}' which is not an enum", self.display_type(value_typ.data_type.kind())))
                                    .build()
                            )
                        }
                    },


                    DataTypeKind::Unknown => return Ok(AnalysisReport::new(DataType::new(source, DataTypeKind::Unknown), true)),
                    
                    
                    _ => return Err(
                        CompilerError::new(source.file(), ErrorCode::SMatchValNotEnum, "match value is not an enum")
                            .highlight(source)
                                .note(format!("is of type '{}' which is not an enum", self.display_type(value_typ.data_type.kind())))
                            .build()
                    )
                };


                {
                    let mut errors = vec![];
                    for i in 0..mappings.len() {                    
                        let f = mappings.get(i).unwrap();

                        if let Some(v) = mappings[0..i].iter().find(|x| x.name() == f.name()) {
                            errors.push(CompilerError::new(
                                self.metadata.file, 
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
                                self.metadata.file, 
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
                                self.metadata.file, 
                                ErrorCode::SMissingField, 
                                "missing field"
                            )
                            .highlight(source)
                                .note(format!("missing '{}'",
                                    self.global.inner.borrow().symbol_map.get(mapping.name()), 
                                ))
                            .build())
                        }
                    }

                    if !errors.is_empty() {
                        return Err(errors.combine_into_error())
                    }
                }


                let (return_type, mutability) = {
                    let mut expected_type = None;
                    let mut mutability = true;
                    let mut errors = vec![];

                    //  PERFORMANCE: avoid this clone
                    let enum_mappings = e.mappings.clone();
                    drop(e);

                    for mapping in mappings {
                        let enum_mapping = enum_mappings.iter().find(|x| x.name() == mapping.name()).unwrap();

                        let mut mapping_scope = self.subscope();
                        mapping_scope.metadata.variables.push(
                            Variable::new(
                                mapping.binding(), 
                                enum_mapping.data_type().clone(), 
                                value_typ.mutability,
                            )
                        );


                        let result = mapping_scope.analyse_node(mapping.node_mut());
                        let result = match result {
                            Ok(v) => v,
                            Err(v) => {
                                errors.push(v);
                                continue
                            },
                        };


                        if !result.mutability {
                            mutability = false;
                        }


                        if expected_type.as_ref().is_none() {
                            expected_type = Some(result.data_type.kind_owned());
                            continue
                        }


                        if !expected_type.as_ref().unwrap().is(result.data_type.kind()) {
                            errors.push(CompilerError::new(
                                self.metadata.file, 
                                ErrorCode::SMatchBranchDiffTy, 
                                "match branch returns a different type"
                            )
                            .highlight(mapping.node().range())
                                .note(format!("expected '{}' found '{}'",
                                    self.display_type(expected_type.as_ref().unwrap()),
                                    self.display_type(result.data_type.kind())
                                ))
                            .build())
                        }
                    }

                    if !errors.is_empty() {
                        return Err(errors.combine_into_error())
                    }

                    (expected_type.unwrap_or(DataTypeKind::Unit), mutability)
                };
                

                AnalysisReport::new(DataType::new(source, return_type), mutability)

            },

            
            Expression::Block { block } => self.analyse_block(block)?,

            
            Expression::CreateStruct { data_type, fields } => {
                self.validate_data_type(data_type)?;


                let structure = match data_type.kind() {
                    | DataTypeKind::Int
                    | DataTypeKind::Bool
                    | DataTypeKind::Float
                    | DataTypeKind::Unit
                    | DataTypeKind::Any
                    | DataTypeKind::Option(_)
                     => {
                        return Err(
                            CompilerError::new(source.file(), ErrorCode::SCantInitPrimitive, "can't initialise primitive types with structure creation syntax")
                                .highlight(source)
                                .build()
                        )
                    }

                    DataTypeKind::Unknown => return Ok(AnalysisReport::new(DataType::new(source, DataTypeKind::Unknown), true)),

                    DataTypeKind::CustomType(e) => e,
                };


                let structure = self.global.find_symbol(*structure);
                let Symbol::Structure(structure) = structure.deref()
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
                                self.metadata.file, 
                                ErrorCode::SFieldDefEarlier, 
                                "field is already defined"
                            )
                            .highlight(v.1)
                                .note("field is defined earlier here".to_string())
                            .highlight(f.1)
                                .note("..but it's defined again here".to_string())
                            .build())
                        }


                        let field = structure.fields.iter().find(|x| x.0 == f.0);
                        if field.is_none() {
                            errors.push(CompilerError::new(
                                self.metadata.file, 
                                ErrorCode::SUnknownField, 
                                "unknown field"
                            )
                            .highlight(f.1)
                                .note(format!("there's no field named {} in {}",
                                    self.global.inner.borrow().symbol_map.get(f.0), 
                                    self.global.inner.borrow().symbol_map.get(structure.full_name), 
                                ))
                            .build());

                            continue
                        }

                        let field = field.unwrap();
                        let Some(result) = result else { continue };

                        if !field.1.kind().is(result.data_type.kind()) {                            
                            errors.push(CompilerError::new(
                                self.metadata.file, 
                                ErrorCode::SFieldTypeMismatch, 
                                "field type mismatch"
                            )
                            .highlight(field.2)
                                .note(format!("the field {} is defined as {} here",
                                    self.global.inner.borrow().symbol_map.get(f.0), 
                                    self.display_type(field.1.kind()),
                                ))
                            .empty_line()
                            .highlight(f.2.range())
                                .note(format!("..but the value here is of type {}",
                                    self.display_type(result.data_type.kind())
                                ))
                            .build());
                        }
                    }


                    for field in &structure.fields {
                        if !fields.iter().any(|x| x.0 == field.0) {
                            errors.push(CompilerError::new(
                                self.metadata.file, 
                                ErrorCode::SMissingField, 
                                "missing field"
                            )
                            .highlight(source)
                                .note(format!("missing '{}'",
                                    self.global.inner.borrow().symbol_map.get(field.0), 
                                ))
                            .build())
                        }
                    }
                    

                    if !errors.is_empty() {
                        return Err(errors.combine_into_error())
                    }
                }

                AnalysisReport::new(data_type.clone(), true)
            },

            
            Expression::AccessField { val, field: field_name } => {
                let val_typ = self.analyse_node(&mut *val)?;

                let structure = match val_typ.data_type.kind() {
                    DataTypeKind::CustomType(v) => v,

                    DataTypeKind::Unknown => return Ok(AnalysisReport::new(DataType::new(source, DataTypeKind::Unknown), true)),

                    _ => {
                        return Err(
                            CompilerError::new(source.file(), ErrorCode::SAccFieldOnPrim, "can't access fields on a primitive type")
                                .highlight(source)
                                .build()
                        )
                    }
                };

                let structure = self.global.find_symbol(*structure);
                let Symbol::Structure(structure) = structure.deref()
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


                let field = structure.fields.iter().find(|x| x.0 == *field_name);
                let Some(field) = field
                else {                    
                    return Err(
                        CompilerError::new(
                            source.file(), 
                            ErrorCode::SFieldDoesntExist, 
                            "field doesn't exist"
                        )
                        .highlight(source)
                            .note(format!("there's no field named {} on '{}'",
                                self.global.inner.borrow().symbol_map.get(*field_name),
                                self.global.inner.borrow().symbol_map.get(structure.full_name),
                            ))
                        .build()
                    )
                };

                AnalysisReport::new(DataType::new(source, field.1.kind().clone()), val_typ.mutability)
            },

            
            Expression::CallFunction { name, args, is_accessor } => {
                let args_analysis = {
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

                println!("{}", self.global.inner.borrow().symbol_map.get(*name));

                let function = self.find_symbol_id(*name, source)?;
                let borrow = self.global.inner.borrow();
                let function = borrow.symbols.get(function).unwrap();
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


                {
                    let mut errors = vec![];
                    for i in 0..args.len() {
                        let arg = &mut args[i];
                        let arg_analysis = &args_analysis[i];
                        let expected_arg = &function.args[i];

                        if !arg_analysis.data_type.kind().is(expected_arg.data_type().kind()) {
                            errors.push(
                                CompilerError::new(source.file(), ErrorCode::SArgTypeMismatch, "argument type mismatch")
                                    .highlight(expected_arg.range())
                                        .note(format!("argument is defined as '{}'", self.display_type(expected_arg.data_type().kind())))
                                    .highlight(arg.0.range())
                                        .note(format!("..but the value is of type '{}'", self.display_type(arg_analysis.data_type.kind())))
                                    .build()
                            )
                        }


                        if arg.1 != expected_arg.is_inout() {
                            errors.push(
                                CompilerError::new(source.file(), ErrorCode::SArgDiffInOut, "arguments differ in in-outness")
                                    .highlight(arg.0.range())
                                        .note({
                                            match (arg.1, expected_arg.is_inout()) {
                                                (true, false) => "consider removing this '&'",
                                                (false, true) => "consider adding a '&' before the argument",

                                                _ => unreachable!(),
                                            }.to_string()
                                        })
                                    .build()
                            )
                        }
                    }


                    if !errors.is_empty() {
                        return Err(errors.combine_into_error())
                    }
                }
                
                
                AnalysisReport::new(DataType::new(source, function.return_type.kind().clone()), true)
            },

            
            Expression::WithinNamespace { namespace, action } => {
                let namespace = self.find_namespace(*namespace, source)?;
                let mut borrow = self.global.inner.borrow_mut();
                let (index, namespace) = borrow.scope_list.iter_mut().enumerate().find(|x| x.1.id == namespace).unwrap();

                let mut mock_scope = Scope {
                    parent: None,
                    global: self.global,
                    metadata: std::mem::take(namespace),
                };

                drop(borrow);
                let result = mock_scope.analyse_node(&mut *action)?;

                {
                    let mut borrow = self.global.inner.borrow_mut();
                    borrow.scope_list[index] = std::mem::take(&mut mock_scope.metadata);
                }


                AnalysisReport::new(DataType::new(source, result.data_type.kind_owned()), result.mutability)
            },
        };

        Ok(result)
    }
}


