use common::{string_map::{StringMap, StringIndex}, fuck_map::FuckMap, source::SourceRange};
use errors::{SemaError, ErrorId};
use parser::{nodes::{NodeKind, Node}, DataType};
use sti::{define_key, prelude::Arena, keyed::KVec, packed_option::PackedOption, arena_pool::ArenaPool, vec::Vec};

use crate::{Type, errors::Error, TypeId, FuncId, ir::terms::{Reg, IR}, State, TypeSymbol, StructureKind, Function, LocalAnalyser};

define_key!(u32, pub NamespaceId);
define_key!(u32, pub ScopeId);

#[derive(Debug)]
pub struct InferState<'ns> {
    pub(crate) arena_nasp: &'ns Arena,

    pub(crate) namespaces: KVec<NamespaceId, Namespace<'ns>>,
    pub(crate) scopes: KVec<ScopeId, Scope>,

    pub(crate) option_table: FuckMap<Type, TypeId>,
    pub(crate) result_table: FuckMap<(Type, Type), TypeId>,
    pub(crate) namespace_table: FuckMap<Type, NamespaceId>,
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
    Variable((StringIndex, Type, bool, Reg)),
    None,
}


#[derive(Debug)]
pub struct Namespace<'arena> {
    types: FuckMap<StringIndex, TypeId, &'arena Arena>,
    funcs: FuckMap<StringIndex, FuncId, &'arena Arena>
}


impl Scope {
    pub fn new(parent: PackedOption<ScopeId>, kind: ScopeKind) -> Self {
        Self {
            parent,
            kind,
        }
    }


    #[inline(always)]
    pub fn find_var(
        &self,
        name: StringIndex,
        scopes: &KVec<ScopeId, Scope>,
    ) -> Option<(StringIndex, Type, bool, Reg)> {

        let mut current = self;
        loop {
            'ns: {
                let ScopeKind::Variable(index) = self.kind
                else { break 'ns };

                if name == index.0 {
                    return Some(index);
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


    #[inline(always)]
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
    ) -> Result<(), Error> {

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
    ) -> Result<(), Error> {

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


    pub fn move_into<'b>(self, arena: &'b Arena) -> Namespace<'b> {
        Namespace {
            types: self.types.move_into(arena),
            funcs: self.funcs.move_into(arena),
        }
    }
}


impl<'an> InferState<'an> {
    #[inline(always)]
    pub fn create_ns(&mut self, ns: Namespace<'an>) -> NamespaceId {
        self.namespaces.push(ns)
    }
}


impl<'me, 'at, 'af, 'an> State<'me, 'at, 'af, 'an> {
    pub fn update_data_type(
        &mut self, 
        dt: &DataType,
        scope: &Scope,
    ) -> Result<Type, SemaError> {
        match dt.kind() {
            parser::DataTypeKind::Int => Ok(Type::Int),
            parser::DataTypeKind::Bool => Ok(Type::Bool),
            parser::DataTypeKind::Float => Ok(Type::Float),
            parser::DataTypeKind::Unit => Ok(Type::Unit),
            parser::DataTypeKind::Any => Ok(Type::Any),
            parser::DataTypeKind::Never => Ok(Type::Never),

            parser::DataTypeKind::Option(v) => {
                let v = self.update_data_type(v, scope)?;
                if let Some(v) = self.sema.option_table.get(&v) { return Ok(Type::UserType(*v)) };

                let index_some = self.string_map.insert("some");
                let index_none = self.string_map.insert("none");

                let index = self.declare_type();
                let final_type = Type::UserType(index);

                self.update_type(index, TypeSymbol::Enum { 
                    mappings: self.arena_type.alloc_new([
                        (index_some, v, SourceRange::new(u32::MAX, u32::MAX), false),
                        (index_none, Type::Unit, SourceRange::new(u32::MAX, u32::MAX), true),
                    ]),
                });

                self.sema.option_table.insert(v, index);

                Ok(final_type)
            },

            
            parser::DataTypeKind::Result(v1, v2) => {                
                let v1 = self.update_data_type(v1, scope)?;
                let v2 = self.update_data_type(v2, scope)?;
                if let Some(v) = self.sema.result_table.get(&(v1, v2))
                { return Ok(Type::UserType(*v)) };

                let index_ok   = self.string_map.insert("ok");
                let index_err  = self.string_map.insert("err");

                let index = self.declare_type();
                let final_type = Type::UserType(index);

                self.update_type(index, TypeSymbol::Enum {
                    mappings: self.arena_type.alloc_new([
                        (index_ok, v1, SourceRange::new(u32::MAX, u32::MAX), false),
                        (index_err, v2, SourceRange::new(u32::MAX, u32::MAX), false),
                    ]),
                });

                self.sema.result_table.insert((v1, v2), index);

                Ok(final_type)
            },


            parser::DataTypeKind::CustomType(index) => {
                let Some(type_index) = scope.find_type(
                    *index, 
                    &self.sema.scopes, 
                    &self.sema.namespaces
                )
                else { return Err(self.errors.push(Error::UnknownType(*index, dt.range()))) };
        
                Ok(Type::UserType(type_index))
            }
        }


    }
}


impl<'me, 'at, 'af, 'an> State<'me, 'at, 'af, 'an> {
   pub fn collect_names(
        &mut self, 
        anal: &mut LocalAnalyser,
        ns: &mut Namespace, 
        body: &[Node]
    ) -> Option<()> {
        let errors_len = self.errors.len();
        
        for node in body {
            let NodeKind::Declaration(decl) = node.kind()
            else { continue };


            match decl {
                parser::nodes::Declaration::Struct { header, name, fields, .. } => {
                    let index = self.declare_type();
                    let result = ns.add_type(*name, index, *header);
                    if let Err(e) = result {
                        anal.current.push(IR::Error(ErrorId::Sema(self.errors.push(e))))
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
                            anal.current.push(IR::Error(ErrorId::Sema(self.errors.push(
                                Error::DuplicateField { 
                                    declared_at: i.1.2, 
                                    error_point: i.0.2,
                                }
                            ))));
                        }
                    }
                },

                
                parser::nodes::Declaration::Enum { header, name, mappings, .. } => {
                    let index = self.declare_type();
                    let result = ns.add_type(*name, index, *header);
                    if let Err(e) = result {
                        anal.current.push(IR::Error(ErrorId::Sema(self.errors.push(e))))
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
                            anal.current.push(IR::Error(ErrorId::Sema(self.errors.push(
                                Error::DuplicateField { 
                                    declared_at: i.1.range(), 
                                    error_point: i.0.range(),
                                }
                            ))));
                        }
                    }
                },


                parser::nodes::Declaration::Function { name, header, arguments, .. } => {
                    let index = self.declare_func();
                    let result = ns.add_func(*name, index, *header);
                    if let Err(e) = result {
                        anal.current.push(IR::Error(ErrorId::Sema(self.errors.push(e))))
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
                            anal.current.push(IR::Error(ErrorId::Sema(self.errors.push(
                                Error::DuplicateArg { 
                                    declared_at: i.1.range(), 
                                    error_point: i.0.range(),
                                }
                            ))));
                        }
                    }
                }
                
                
                _ => todo!(),
            }
        }

        if self.errors.len() != errors_len { return None }
        Some(())
    }


    pub fn update_types(
        &mut self, 
        anal: &mut LocalAnalyser,
        scope: &Scope, 
        body: &[Node]
    ) -> Option<()> {
        
        let errors_len = self.errors.len();
        for node in body {
            let NodeKind::Declaration(decl) = node.kind()
            else { continue };


            match decl {
                parser::nodes::Declaration::Struct { kind, name, fields, .. } => {
                    let fields = {
                        let mut vec = Vec::with_cap_in(self.arena_type, fields.len());

                        for f in fields.iter() {
                            let updated = self.update_data_type(
                                &f.1,
                                scope,
                            );

                            let updated = match updated {
                                Ok(v) => v,
                                Err(e) => {
                                    anal.current.push(IR::Error(ErrorId::Sema(e)));
                                    continue
                                },
                            };

                            vec.push(updated)
                        }

                        vec.leak()
                    };


                    let index = scope.find_type(
                        *name, 
                        &self.sema.scopes, 
                        &self.sema.namespaces
                    ).unwrap();
                    
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
                        let mut vec = Vec::with_cap_in(self.arena_type, mappings.len());

                        for m in mappings.iter() {
                            let updated = self.update_data_type(
                                m.data_type(),
                                scope,
                            );

                            let updated = match updated {
                                Ok(v) => v,
                                Err(e) => {
                                    anal.current.push(IR::Error(ErrorId::Sema(e)));
                                    continue
                                },
                            };

                            vec.push((m.name(), updated, m.range(), m.is_implicit_unit()))
                        }

                        vec.leak()
                    };
                    
                    let index = scope.find_type(
                        *name, 
                        &self.sema.scopes, 
                        &self.sema.namespaces
                    ).unwrap();
                    self.update_type(index, TypeSymbol::Enum { mappings })
                },

                
                parser::nodes::Declaration::Function { name, arguments, return_type, .. } => {
                    let args = {
                        let mut vec = Vec::with_cap_in(self.arena_func, arguments.len());

                        for m in arguments.iter() {
                            let updated = self.update_data_type(
                                m.data_type(),
                                scope,
                            );

                            let updated = match updated {
                                Ok(v) => v,
                                Err(e) => {
                                    anal.current.push(IR::Error(ErrorId::Sema(e)));
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
                            anal.current.push(IR::Error(ErrorId::Sema(e)));
                            continue
                        },
                    };


                    // an error occurred while updating arguments
                    if args.len() != arguments.len() {
                        continue
                    }


                    let index = scope.find_func(
                        *name, 
                        &self.sema.scopes, 
                        &self.sema.namespaces
                    ).unwrap();
                    let func = self.funcs.get_mut(index).unwrap();
                    func.replace(Function { 
                        args, 
                        body: Vec::new_in(self.arena_func),
                        return_type,
                    });
                },

                
                parser::nodes::Declaration::Impl { data_type, body } => todo!(),
                parser::nodes::Declaration::Using { file } => todo!(),
                parser::nodes::Declaration::Module { name, body } => todo!(),
                parser::nodes::Declaration::Extern { file, functions } => todo!(),
            }
        }


        if self.errors.len() != errors_len { return None }
        Some(())
    }
}