use common::{buffer::Buffer, source::SourceRange, string_map::{StringIndex, StringMap}, Once};
use lexer::Literal;
use errors::ErrorId;
use parser::{dt::{DataType, DataTypeKind}, nodes::{decl::{AttributeValue, Decl, DeclId, FunctionSignature, UseItem, UseItemKind, Visibility}, expr::{BinaryOperator, Expr, ExprId, UnaryOperator}, stmt::{Stmt, StmtId}, NodeId, Pattern, PatternKind}};
use sti::{alloc::GlobalAlloc, key::Key, vec::{KVec, Vec}};

use crate::{errors::Error, namespace::{Namespace, NamespaceId, SymbolGetResult}, scope::{FunctionScope, GenericsScope, Scope, ScopeId, ScopeKind, VariableScope}, syms::{containers::{Container, ContainerKind}, func::{FunctionArgument, FunctionKind, FunctionTy}, sym_map::{BoundedGeneric, Generic, GenericKind, SymbolId, SymbolMap, TraitImplEntry}, ty::Type, Symbol, SymbolKind, Trait}, AnalysisResult, TyChecker};
pub mod blocks;
mod phases;
mod collect;
mod types;
mod eval;
enum TraitMethodRejection {
    BoundError(ErrorId),
    BoundFailure(Type, SymbolId),
}
