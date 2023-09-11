#![deny(unused_must_use)]

use common::{string_map::{StringIndex, StringMap}, fuck_map::FuckMap, source::SourceRange, OptionalPlus, find_duplicate};
use errors::Error;
use parser::{nodes::{Node, NodeKind, Declaration, Statement}, DataType};
use sti::{define_key, keyed::KVec, arena_pool::ArenaPool, prelude::{Arena, Alloc, GlobalAlloc}, packed_option::PackedOption, vec::Vec};
use typed_ast::{Type, TypedBlock, TypedNode, TypedStatement, TypedNodeKind};

pub mod symbol_vec;
pub mod typed_ast;
pub mod errors;


pub fn semantic_analysis<'me, 'ns, 'typ, 'func>(
    ns_arena: &'ns Arena, 
    type_arena: &'typ Arena, 
    func_arena: &'func Arena, 
    string_map: &'me mut StringMap, 
    body: &[Node]
) -> Result<State<'me, 'ns, 'typ, 'func>, Errors> {
    let mut state = State {
        ns_arena,
        type_arena,
        func_arena,

        string_map,

        types: KVec::new(),
        funcs: KVec::new(),
        namespaces: KVec::new(),
        scopes: KVec::new(),

        option_table: FuckMap::new(),
        result_table: FuckMap::new(),
        namespace_table: FuckMap::new(),
    };

    let root_scope = Scope::new(PackedOption::NONE, ScopeKind::None);
    let root_scope = state.scopes.push(root_scope);
    let new_scope = state.analyse_body(root_scope, body)?;
    assert!(new_scope.1.is_empty());

    Ok(state)
}


// ik. naming is fucking hard.
type Errors = ::errors::Errors<errors::Error>;

define_key!(u32, pub NamespaceId);
define_key!(u32, pub TypeId);
define_key!(u32, pub FuncId);
define_key!(u32, pub ScopeId);

#[derive(Debug)]
pub struct State<'me, 'ns, 'type_arena, 'func_arena> {
    ns_arena: &'ns Arena,
    type_arena: &'type_arena Arena,
    func_arena: &'func_arena Arena,

    string_map: &'me mut StringMap,

    types: KVec<TypeId, Option<TypeSymbol<'type_arena>>>,
    funcs: KVec<FuncId, Option<Function<'func_arena>>>,
    namespaces: KVec<NamespaceId, Namespace<'ns>>,
    scopes: KVec<ScopeId, Scope>,

    option_table: FuckMap<Type, TypeId>,
    result_table: FuckMap<(Type, Type), TypeId>,
    namespace_table: FuckMap<Type, NamespaceId>,
}


#[derive(Debug, Clone, Copy)]
pub struct Scope {
    parent: PackedOption<ScopeId>,
    kind: ScopeKind,
}


#[derive(Debug, Clone, Copy)]
pub enum ScopeKind {
    NamedNamespace((StringIndex, NamespaceId)),
    Namespace(NamespaceId),
    Function,
    Variable((StringIndex, Type)),
    None,
}


#[derive(Debug)]
pub struct Namespace<'arena> {
    types: FuckMap<StringIndex, TypeId, &'arena Arena>,
    funcs: FuckMap<StringIndex, FuncId, &'arena Arena>
}


#[derive(Debug)]
enum TypeSymbol<'a> {
    Structure {
        kind: StructureKind,
        fields: &'a [Type],
    },

    Enum {
        mappings: &'a [(StringIndex, Type, SourceRange, bool)],
    },
}


#[derive(Debug)]
enum StructureKind {
    Normal,
    Component,
    Resource,
}


#[derive(Debug)]
struct Function<'a> {
    args: &'a [(StringIndex, Type, bool, SourceRange)],
    kind: FunctionKind<'a>,
    return_type: Type,
}


#[derive(Debug)]
enum FunctionKind<'a> {
    Normal {
        body: TypedBlock<'a>
    },

    Enum {
        variant: u16,
    },
}


impl Scope {
    pub fn new(parent: PackedOption<ScopeId>, kind: ScopeKind) -> Self {
        Self {
            parent,
            kind,
        }
    }


    #[inline(always)]
    pub fn find_type(
        &self,
        name: StringIndex,
        scopes: &KVec<ScopeId, Scope>,
        namespaces: &KVec<NamespaceId, Namespace>,
    ) -> Option<TypeId> {
        self.over_namespaces(
            |ns| ns.find_type(name), 
            scopes, 
            namespaces,
        )
    }


    pub fn find_func(
        &self,
        name: StringIndex,
        scopes: &KVec<ScopeId, Scope>,
        namespaces: &KVec<NamespaceId, Namespace>,
    ) -> Option<FuncId> {
        self.over_namespaces(
            |ns| ns.find_func(name), 
            scopes, 
            namespaces,
        )
    }


    pub fn over_namespaces<T>(
        &self,
        f: impl Fn(&Namespace) -> Option<T>,
        scopes: &KVec<ScopeId, Scope>,
        namespaces: &KVec<NamespaceId, Namespace>,
    ) -> Option<T> {
        let mut current = self;

        loop {
            'ns: {
                let ScopeKind::Namespace(index) = current.kind
                else { break 'ns };

                let ns = namespaces.get(index).unwrap();
                if let Some(v) = f(ns) {
                    return Some(v);
                }
            }

            if let Some(parent) = current.parent.into() {
                current = scopes.get(parent).unwrap();
                continue
            }

            break
        }

        None
    }
}


impl<'arena> Namespace<'arena> {
    pub fn new(arena: &'arena Arena) -> Self {
        Self {
            types: FuckMap::new_in(arena),
            funcs: FuckMap::new_in(arena),
        }
    }


    pub fn add_type(
        &mut self, 
        name: StringIndex, 
        typ: TypeId, 
        source: SourceRange
    ) -> Result<(), Errors> {

        let result = self.types.insert(name, typ);
        if result.is_some() {
            return Err(Error::NameIsAlreadyDefined { source, name }.into())
        }

        Ok(())
    }


    pub fn add_func(
        &mut self, 
        name: StringIndex, 
        typ: FuncId, 
        source: SourceRange
    ) -> Result<(), Errors> {

        let result = self.funcs.insert(name, typ);
        if result.is_some() {
            return Err(Error::NameIsAlreadyDefined { source, name }.into())
        }

        Ok(())
    }


    pub fn find_type(&self, name: StringIndex) -> Option<TypeId> {
        self.types.get(&name).copied()
    }


    pub fn find_func(&self, name: StringIndex) -> Option<FuncId> {
        self.funcs.get(&name).copied()
    }


    pub fn move_into<'b>(self, arena: &'b Arena<GlobalAlloc>) -> Namespace<'b> {
        Namespace {
            types: self.types.move_into(arena),
            funcs: self.funcs.move_into(arena),
        }
    }
}


impl<'me, 'ns, 'type_arena, 'func_arena> State<'me, 'ns, 'type_arena, 'func_arena> {
    fn create_ns(&mut self, ns: Namespace<'ns>) -> NamespaceId {
        self.namespaces.push(ns)
    }


    fn create_type(&mut self, type_symbol: TypeSymbol<'type_arena>) -> TypeId {
        let index = self.declare_type();
        self.update_type(index, type_symbol);
        index
    }


    fn create_func(&mut self, func: Function<'func_arena>) -> FuncId {
        // self.funcs.push(None)
        todo!()
    }


    fn declare_type(&mut self) -> TypeId {
        self.types.push(None)
    }


    fn declare_func(&mut self) -> FuncId {
        self.funcs.push(None)
    }


    fn update_type(&mut self, index: TypeId, symbol: TypeSymbol<'type_arena>) {
        let type_symbol = self.types.get_mut(index).unwrap();
        type_symbol.replace(symbol);
    }


    fn type_namespace(&mut self, typ: Type) -> NamespaceId {
        if let Some(v) = self.namespace_table.get(&typ) { return *v }

        let index = match typ {
            | Type::Int
            | Type::Bool
            | Type::Float
            | Type::Unit
            | Type::Any
            | Type::Never
            | Type::Unknown => {
                let ns = Namespace::new(self.ns_arena);
                let ns_index = self.create_ns(ns);
                self.namespace_table.insert(typ, ns_index);
                return ns_index
            },

            Type::UserType(v) => v,
        };
        let mut ns = Namespace::new(self.ns_arena);
        let type_symbol = &*self.types.get(index).unwrap();

        match type_symbol.unwrap_ref() {
            TypeSymbol::Enum { mappings } => {
                for (i, m) in mappings.iter().enumerate() {
                    let args : &[_] = if m.3 {
                        assert_eq!(m.1, Type::Unit);
                        self.func_arena.alloc_new([])
                    } else {
                        self.func_arena.alloc_new([(m.0, m.1, false, m.2)])
                    };

                    let id = self.create_func(Function {
                        args, 
                        kind: FunctionKind::Enum { variant: i as u16 },
                        return_type: typ,
                    });
                    
                    ns.add_func(m.0, id, m.2).unwrap();
                }
            },

            _ => (),
        }
        
        let ns_index = self.create_ns(ns);
        self.namespace_table.insert(typ, ns_index);
        ns_index
        
    }


    pub fn update_data_type(
        &mut self, 
        dt: &DataType,
        scope: &Scope,
    ) -> Result<Type, Error> {
        match dt.kind() {
            parser::DataTypeKind::Int => Ok(Type::Int),
            parser::DataTypeKind::Bool => Ok(Type::Bool),
            parser::DataTypeKind::Float => Ok(Type::Float),
            parser::DataTypeKind::Unit => Ok(Type::Unit),
            parser::DataTypeKind::Any => Ok(Type::Any),
            parser::DataTypeKind::Never => Ok(Type::Never),

            parser::DataTypeKind::Unknown => Err(Error::Bypass),

            parser::DataTypeKind::Option(v) => {
                let v = self.update_data_type(v, scope)?;
                if let Some(v) = self.option_table.get(&v) { return Ok(Type::UserType(*v)) };

                let index_some = self.string_map.insert("some");
                let index_none = self.string_map.insert("none");

                let index = self.declare_type();
                let final_type = Type::UserType(index);

                self.update_type(index, TypeSymbol::Enum { 
                    mappings: self.type_arena.alloc_new([
                        (index_some, v, SourceRange::new(u32::MAX, u32::MAX), false),
                        (index_none, Type::Unit, SourceRange::new(u32::MAX, u32::MAX), true),
                    ]),
                });

                self.option_table.insert(v, index);

                Ok(final_type)
            },

            
            parser::DataTypeKind::Result(v1, v2) => {                
                let v1 = self.update_data_type(v1, scope)?;
                let v2 = self.update_data_type(v2, scope)?;
                if let Some(v) = self.result_table.get(&(v1, v2)) { return Ok(Type::UserType(*v)) };

                let index_ok   = self.string_map.insert("ok");
                let index_err  = self.string_map.insert("err");

                let index = self.declare_type();
                let final_type = Type::UserType(index);

                self.update_type(index, TypeSymbol::Enum {
                    mappings: self.type_arena.alloc_new([
                        (index_ok, v1, SourceRange::new(u32::MAX, u32::MAX), false),
                        (index_err, v2, SourceRange::new(u32::MAX, u32::MAX), false),
                    ]),
                });

                self.result_table.insert((v1, v2), index);

                Ok(final_type)
            },


            parser::DataTypeKind::CustomType(index) => {
                let Some(type_index) = scope.find_type(*index, &self.scopes, &self.namespaces)
                else { return Err(Error::UnknownType(*index, dt.range())) };
        
                Ok(Type::UserType(type_index))
            }
        }


    }
}


impl<'me, 'nsa, 'ta, 'fa> State<'me, 'nsa, 'ta, 'fa> {
    fn analyse_body(
        &mut self, 
        parent: ScopeId, 
        body: &[Node]
    ) -> Result<(ScopeId, TypedBlock<'fa>), Errors> {

        let scope = {
            let pool = ArenaPool::tls_get_rec();
            let mut namespace = Namespace::new(&pool);

            self.collect_names(&mut namespace, body)?;

            let ns = namespace.move_into(self.ns_arena);
            let ns = self.create_ns(ns);

            let scope = Scope::new(parent.some(), ScopeKind::Namespace(ns));
            self.update_types(&scope, body)?;
            scope
        };

        let scope = self.scopes.push(scope);

        let mut errors = Errors::new();

        for node in body {
            match node.kind() {
                NodeKind::Declaration(d) => {
                    let result = self.analyse_declaration(scope, d);
                    if let Err(e) = result {
                        errors.with(e);
                    }
                },

                NodeKind::Statement(_) => (),
                NodeKind::Expression(_) => (),
            }
        }

        
        Ok((scope, self.func_arena.alloc_new([])))

    }


    fn collect_names(&mut self, ns: &mut Namespace, body: &[Node]) -> Result<(), Errors> {
        let mut errors = Errors::new();
        for node in body {
            let NodeKind::Declaration(decl) = node.kind()
            else { continue };


            match decl {
                parser::nodes::Declaration::Struct { header, name, fields, .. } => {
                    let index = self.declare_type();
                    let result = ns.add_type(*name, index, *header);
                    if let Err(e) = result {
                        errors.with(e)
                    }


                    {
                        let pool = ArenaPool::tls_get_temp();
                        let mut vec = Vec::with_cap_in(
                            &*pool,
                            fields.len(),
                        );

                        {
                            for i in 0..fields.len() {
                                for j in 0..i {
                                    if fields[i].0 == fields[j].0 {
                                        vec.push((&fields[i], &fields[j]))
                                    }
                                }
                            }
                        };

                        for i in vec.iter() {
                            errors.push(Error::DuplicateField { 
                                declared_at: i.1.2, 
                                error_point: i.0.2,
                            });
                        }
                    }
                },

                
                parser::nodes::Declaration::Enum { header, name, mappings, .. } => {
                    let index = self.declare_type();
                    let result = ns.add_type(*name, index, *header);
                    if let Err(e) = result {
                        errors.with(e)
                    }


                    {
                        let pool = ArenaPool::tls_get_temp();
                        let mut vec = Vec::with_cap_in(
                            &*pool,
                            mappings.len(),
                        );

                        {
                            for i in 0..mappings.len() {
                                for j in 0..i {
                                    if mappings[i].name() == mappings[j].name() {
                                        vec.push((&mappings[i], &mappings[j]))
                                    }
                                }
                            }
                        };

                        for i in vec.iter() {
                            errors.push(Error::DuplicateField { 
                                declared_at: i.1.range(), 
                                error_point: i.0.range(),
                            });
                        }
                    }
                },


                parser::nodes::Declaration::Function { name, header, arguments, .. } => {
                    let index = self.declare_func();
                    let result = ns.add_func(*name, index, *header);
                    if let Err(e) = result {
                        errors.with(e)
                    }

                    {
                        let pool = ArenaPool::tls_get_temp();
                        let mut vec = Vec::with_cap_in(
                            &*pool,
                            arguments.len(),
                        );

                        {
                            for i in 0..arguments.len() {
                                for j in 0..i {
                                    if arguments[i].name() == arguments[j].name() {
                                        vec.push((&arguments[i], &arguments[j]))
                                    }
                                }
                            }
                        };

                        for i in vec.iter() {
                            errors.push(Error::DuplicateArg { 
                                declared_at: i.1.range(), 
                                error_point: i.0.range(),
                            });
                        }
                    }
                }
                
                
                _ => todo!(),
            }
        }

        if !errors.is_empty() { return Err(errors) }

        Ok(())
    }


    fn update_types(&mut self, scope: &Scope, body: &[Node]) -> Result<(), Errors> {
        let mut errors = Errors::new();
        for node in body {
            let NodeKind::Declaration(decl) = node.kind()
            else { continue };


            match decl {
                parser::nodes::Declaration::Struct { kind, name, fields, .. } => {
                    let fields = {
                        let mut vec = Vec::with_cap_in(self.type_arena, fields.len());

                        for f in fields.iter() {
                            let updated = self.update_data_type(
                                &f.1,
                                scope,
                            );

                            let updated = match updated {
                                Ok(v) => v,
                                Err(e) => {
                                    errors.push(e);
                                    continue
                                },
                            };

                            vec.push(updated)
                        }

                        vec.leak()
                    };


                    let index = scope.find_type(*name, &self.scopes, &self.namespaces).unwrap();
                    self.update_type(index, TypeSymbol::Structure { 
                        fields, 
                        kind: match kind {
                            parser::nodes::StructKind::Component => StructureKind::Component,
                            parser::nodes::StructKind::Resource => StructureKind::Resource,
                            parser::nodes::StructKind::Normal => StructureKind::Normal,
                        }
                    });
                },

                
                parser::nodes::Declaration::Enum { name, mappings, .. } => {
                    let mappings = {                        
                        let mut vec = Vec::with_cap_in(self.type_arena, mappings.len());

                        for m in mappings.iter() {
                            let updated = self.update_data_type(
                                m.data_type(),
                                scope,
                            );

                            let updated = match updated {
                                Ok(v) => v,
                                Err(e) => {
                                    errors.push(e);
                                    continue
                                },
                            };

                            vec.push((m.name(), updated, m.range(), m.is_implicit_unit()))
                        }

                        vec.leak()
                    };
                    
                    let index = scope.find_type(*name, &self.scopes, &self.namespaces).unwrap();
                    self.update_type(index, TypeSymbol::Enum { mappings })
                },

                
                parser::nodes::Declaration::Function { name, arguments, return_type, .. } => {
                    let args = {
                        let mut vec = Vec::with_cap_in(self.func_arena, arguments.len());

                        for m in arguments.iter() {
                            let updated = self.update_data_type(
                                m.data_type(),
                                scope,
                            );

                            let updated = match updated {
                                Ok(v) => v,
                                Err(e) => {
                                    errors.push(e);
                                    continue
                                },
                            };

                            vec.push((m.name(), updated, m.is_inout(), m.range()))
                        }

                        vec.leak()
                    };

                    let return_type = self.update_data_type(return_type, scope);
                    let return_type = match return_type {
                        Ok(e) => e,
                        Err(e) => {
                            errors.push(e);
                            continue
                        },
                    };


                    // an error occurred while updating arguments
                    if args.len() != arguments.len() {
                        continue
                    }


                    let index = scope.find_func(*name, &self.scopes, &self.namespaces).unwrap();
                    let func = self.funcs.get_mut(index).unwrap();
                    func.replace(Function { 
                        args, 
                        kind: FunctionKind::Normal { body: self.func_arena.alloc_new([]) }, 
                        return_type,
                    });
                },

                
                parser::nodes::Declaration::Impl { data_type, body } => todo!(),
                parser::nodes::Declaration::Using { file } => todo!(),
                parser::nodes::Declaration::Module { name, body } => todo!(),
                parser::nodes::Declaration::Extern { file, functions } => todo!(),
            }
        }


        if !errors.is_empty() { return Err(errors) }

        Ok(())
    }


    fn analyse_node(&mut self, current: ScopeId, node: &Node) -> Result<TypedNode, Errors> {
        let kind = match node.kind() {
            NodeKind::Declaration(_) => unreachable!(),
            NodeKind::Statement(v) => TypedNodeKind::Statement(self.analyse_statement(current, v)?),
            NodeKind::Expression(_) => todo!(),
        };

        Ok(TypedNode {
            kind,
        })
    }


    fn analyse_declaration(&mut self, current: ScopeId, decl: &Declaration) -> Result<(), Errors>{
        match decl {
            Declaration::Struct { kind, name, header, fields } => Ok(()),
            
            Declaration::Enum { name, header, mappings } => Ok(()),
            
            Declaration::Function { is_system, name, header, arguments, return_type, body } => {
                let scope = Scope::new(current.some(), ScopeKind::Function);
                let scope = self.scopes.push(scope);
                let body_analysis = self.analyse_body(scope, &body)?;



                let current = self.scopes.get(current).unwrap();
                let index = current.find_func(*name, &self.scopes, &self.namespaces).unwrap();
                let func = self.funcs.get_mut(index).unwrap();
                func.as_mut().unwrap().kind = FunctionKind::Normal { body: body_analysis.1 };

                Ok(())
            },

            Declaration::Impl { data_type, body } => todo!(),
            Declaration::Using { file } => todo!(),
            Declaration::Module { name, body } => todo!(),
            Declaration::Extern { file, functions } => todo!(),
        }
    }


    fn analyse_statement(
        &mut self, 
        current: ScopeId, 
        stmt: &Statement
    ) -> Result<TypedStatement, Errors> {
        
        match stmt {
            Statement::Variable { name, hint, is_mut, rhs } => {
                todo!()
            },
            Statement::UpdateValue { lhs, rhs } => todo!(),
        }
    }
}