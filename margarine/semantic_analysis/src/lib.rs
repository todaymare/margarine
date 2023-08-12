use std::{cell::RefCell, collections::HashMap, convert::identity};

use common::{SymbolMap, SourceRange, SymbolIndex};
use errors::{Error, CombineIntoError, CompilerError, ErrorCode, ErrorBuilder};
use istd::index_vec;
use parser::{nodes::{Node, NodeKind, Declaration, StructKind, FunctionArgument}, DataType, Block, DataTypeKind};


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
}


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
struct ScopeId(usize);


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
    mappings: Vec<(SymbolIndex, DataType, SourceRange)>,
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


struct AnalysisReport {
    data_type: DataType,
    mutability: bool,
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
                file,
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
        match data_type.kind_mut() {
            DataTypeKind::Int   => Ok(()),
            DataTypeKind::Bool  => Ok(()),
            DataTypeKind::Float => Ok(()),
            DataTypeKind::Unit  => Ok(()),
            DataTypeKind::Any   => Ok(()),
            DataTypeKind::Unknown => Ok(()),
            DataTypeKind::Option(v) => self.validate_data_type(v),
            DataTypeKind::CustomType(v) => {
                if let Some(new) = self.metadata.symbols.get(v) {
                    let name = &self.global.inner.borrow().symbols[*new];
                    let name = name.full_name();
                    *v = name;
                    return Ok(())
                }


                if let Some(v) = self.parent {
                    return v.validate_data_type(data_type)
                }


                *data_type.kind_mut() = DataTypeKind::Unknown;
                Err(CompilerError::new(self.metadata.file, ErrorCode::STypeDoesntExist, "type doesn't exist or inaccessible")
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
}


impl Scope<'_, '_> {
    fn analyse_block(&mut self, block: &mut Block) -> Result<AnalysisReport, Error> {
        let mut subscope = self.subscope();
        let mut errors = vec![];


        subscope.register_declarations(block)?;

        // subscope.register_declarations(iterator);

        // let result = block
        //     .iter_mut()
        //     .map(|x| subscope.analyse_node(x))
        //     .filter_map(|x| match x {
        //         Ok(v) => Some(v),
        //         Err(e) => {
        //             errors.push(e);
        //             None
        //         },
        //     })
        //     .last()
        //     .unwrap();


        if !errors.is_empty() {
            return Err(errors.combine_into_error())
        }

        Ok(AnalysisReport { data_type: DataType::new(SourceRange::new(0, 0, self.metadata.file), DataTypeKind::Unit), mutability: true })
    }


    fn analyse_node(&mut self, node: &mut Node) -> Result<AnalysisReport, Error> {
        match node.kind() {
            NodeKind::Declaration(v) => todo!(),
            NodeKind::Statement(_) => todo!(),
            NodeKind::Expression(_) => todo!(),
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
        fn filter_declaration(x: &mut Node) -> Option<(&mut Declaration, SourceRange)> {
            let range = x.range();
            if let NodeKind::Declaration(v) = x.kind_mut() {
                Some((v, range))
            } else {
                None
            }
        }

        {
            let mut errors = vec![];
            let iterator = nodes
                .iter_mut()
                .filter_map(filter_declaration);

            for (decl, source) in iterator {
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


                    _ => todo!()
                }
            
            }


            if !errors.is_empty() {
                return Err(errors.combine_into_error())
            }
        }

        {
            let mut errors = vec![];
            let iterator = nodes
                .iter_mut()
                .filter_map(filter_declaration);

            for (decl, source) in iterator {
                match decl {
                    Declaration::Struct { kind, name, fields } => {
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
                            let f = mappings.get_mut(i).unwrap();
                            let result = self.validate_data_type(&mut f.1);

                            if let Err(e) = result {
                                errors.push(e);
                            }

                            let f = mappings.get(i).unwrap();
                            if let Some(v) = mappings[0..i].iter().find(|x| x.0 == f.0) {
                                errors.push(CompilerError::new(
                                    self.metadata.file, 
                                    ErrorCode::SVariantDefEarlier, 
                                    "variant is already defined"
                                )
                                .highlight(v.2)
                                    .note("variant is defined earlier here".to_string())
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
                                        Symbol::Enum(v) => Some(v),
                                        _ => None
                                    }
                                })
                                .find(|x| x.full_name == *name).unwrap();

                            // TODO: Might be able to remove the clone here 
                            //       depending on how the rest of this goes
                            id.mappings = mappings.clone();
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


                    _ => todo!()
                }
            
            }


            if !errors.is_empty() {
                return Err(errors.combine_into_error())
            }
        }

        Ok(())
    }
}


impl Drop for Scope<'_, '_> {
    fn drop(&mut self) {
        self.global.add_scope_metadata(std::mem::take(&mut self.metadata))
    }
}


