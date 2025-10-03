use std::fmt::Write;

use common::{source::SourceRange, string_map::StringIndex};
use errors::{ErrorId, ErrorType};
use parser::nodes::expr::{BinaryOperator, UnaryOperator};
use sti::vec::Vec;

use crate::syms::{ty::Sym, sym_map::SymbolMap};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Error {
    IteratorFunctionInvalidSig(SourceRange),

    InvalidCast {
        range: SourceRange,
        from_ty: Sym,
        to_ty: Sym,
    },

    DerefOnNonPtr(SourceRange),

    InvalidValueForAttr {
        attr: (SourceRange, StringIndex),
        value: SourceRange,
        expected: &'static str,
    },

    UnknownAttr(SourceRange, StringIndex),

    NameIsAlreadyDefined {
        source: SourceRange,
        name: StringIndex,
    },

    UnknownType(StringIndex, SourceRange),

    FunctionBodyAndReturnMismatch {
        header: SourceRange,
        item: SourceRange,
        return_type: Sym,
        body_type: Sym,
    },

    OutsideOfAFunction {
        source: SourceRange,
    },

    InvalidType {
        source: SourceRange,
        found: Sym,
        expected: Sym,
    },

    DuplicateField {
        declared_at: SourceRange,
        error_point: SourceRange,
    },

    DuplicateArg {
        declared_at: SourceRange,
        error_point: SourceRange,
    },

    VariableValueAndHintDiffer {
        value_type: Sym,
        hint_type: Sym,
        source: SourceRange,
    },

    VariableValueNotTuple(SourceRange),

    VariableTupleAndHintTupleSizeMismatch(SourceRange, usize, usize),

    VariableNotFound {
        name: StringIndex,
        source: SourceRange,
    },

    InvalidBinaryOp {
        operator: BinaryOperator,
        lhs: Sym,
        rhs: Sym,
        source: SourceRange,
    },

    InvalidUnaryOp {
        operator: UnaryOperator,
        rhs: Sym,
        source: SourceRange,
    },

    IfMissingElse {
        body: (SourceRange, Sym),
    },

    IfBodyAndElseMismatch {
        body: (SourceRange, Sym),
        else_block: (SourceRange, Sym),
    },
    
    MatchValueIsntEnum {
        source: SourceRange,
        typ: Sym,
    },
    
    MatchBranchesDifferInReturnType {
        initial_source: SourceRange,
        initial_typ: Sym,
        branch_source: SourceRange,
        branch_typ: Sym,
    },

    DuplicateMatch {
        declared_at: SourceRange,
        error_point: SourceRange,
    },
    
    InvalidMatch {
        name: StringIndex,
        range: SourceRange,
        value: Sym,
    },
     
    MissingMatch {
        name: Vec<StringIndex>,
        range: SourceRange,
    },

    ValueIsntAnIterator {
        ty: Sym,
        range: SourceRange,
    },

    StructCreationOnNonStruct {
        source: SourceRange,
        typ: Sym,
    },
    
    FieldAccessOnNonEnumOrStruct {
        source: SourceRange,
        typ: Sym,
    },

    FieldDoesntExist {
        source: SourceRange,
        field: StringIndex,
        typ: Sym,
    },

    MissingFields {
        source: SourceRange,
        fields: sti::vec::Vec<StringIndex>,
    },

    FunctionNotFound {
        source: SourceRange,
        name: StringIndex,
    },

    BindedFunctionNotFound {
        source: SourceRange,
        name: StringIndex,
        bind: Sym,
    },

    FunctionArgsMismatch {
        source: SourceRange,
        sig_len: usize,
        call_len: usize,
    },

    NamespaceNotFound {
        source: SourceRange,
        namespace: StringIndex,
    },

    ValueUpdateTypeMismatch {
        lhs: Sym,
        rhs: Sym,
        source: SourceRange,
    },

    ContinueOutsideOfLoop(SourceRange),

    BreakOutsideOfLoop(SourceRange),

    CantUnwrapOnGivenType(SourceRange, Sym),

    CantTryOnGivenType(SourceRange, Sym),

    FunctionDoesntReturnAnOption {
        source: SourceRange,
        func_typ: Sym,
    },

    FunctionDoesntReturnAResult {
        source: SourceRange,
        func_typ: Sym,
    },
    
    FunctionReturnsAResultButTheErrIsntTheSame {
        source: SourceRange,
        func_source: SourceRange,
        func_err_typ: Sym,
        err_typ: Sym,
    },

    ReturnAndFuncTypDiffer {
        source: SourceRange,
        func_source: SourceRange,
        typ: Sym,
        func_typ: Sym,
    },

    AssignIsNotLHSValue {
        source: SourceRange,
    },

    UnableToInfer(SourceRange),

    InvalidRange {
        source: SourceRange,
        ty: Sym,
    },

    ImplOnGeneric(SourceRange),

    GenericLenMismatch { source: SourceRange, found: usize, expected: usize },

    CantUseHoleHere { source: SourceRange },

    NameIsReservedForFunctions { source: SourceRange },

    InvalidSystem(SourceRange),

    IndexOnNonList(SourceRange, Sym),

    Bypass,

    CallOnNonFunction { source: SourceRange, name: StringIndex },
}


impl<'a> ErrorType<SymbolMap<'_>> for Error {
    fn display(&self, fmt: &mut errors::fmt::ErrorFormatter, types: &mut SymbolMap) {
        match self {
            Error::NameIsAlreadyDefined { source, name } => {
                let name = fmt.string(*name).to_string();
                fmt.error("name is already defined")
                    .highlight_with_note(
                        *source,
                        &format!("there's already a symbol with the name '{name}'"),
                    )
            },

            
            Error::UnknownType(name, pos) => {
                let name = fmt.string(*name).to_string();
                fmt.error("unknown type")
                    .highlight_with_note(
                        *pos,
                        &format!("there's no type named '{name}'"),
                    )
            },

            
            Error::InvalidType { source, found, expected } => {
                let msg = format!("expected a value of type '{}' but found '{}'",
                    expected.display(fmt.string_map(), types),
                    found.display(fmt.string_map(), types),
                );
                
                fmt.error("invalid type")
                    .highlight_with_note(
                        *source,
                        &msg,
                    )
            },

            
            Error::FunctionBodyAndReturnMismatch { header, item, return_type, body_type } => {
                let msg = format!("the function returns '{}'",
                    return_type.display(fmt.string_map(), types),
                );
                
                let msg2 = format!("but the body returns '{}'",
                    body_type.display(fmt.string_map(), types),
                );

                let mut err = fmt.error("function's return type and the body mismatch");
                err.highlight_with_note(*header, &msg);
                err.highlight_with_note(*item, &msg2);
            },

            
            Error::OutsideOfAFunction { source } => {
                fmt.error("this operation can't be performed outside of a function")
                    .highlight(*source);
            },

           
            Error::DuplicateField { declared_at, error_point } => {
                let mut error = fmt.error("duplicate field");
                error
                    .highlight_with_note(*declared_at, "the field is declared here");
                error.highlight_with_note(*error_point, "..but it's redeclared here");
            
            },

            
            Error::DuplicateArg { declared_at, error_point } => {
                let mut error = fmt.error("duplicate argument");
                error
                    .highlight_with_note(*declared_at, "the argument is declared here");
                error.highlight_with_note(*error_point, "..but it's redeclared here");
            
            },

            
            Error::DuplicateMatch { declared_at, error_point } => {
                let mut error = fmt.error("duplicate match variant");
                error
                    .highlight_with_note(*declared_at, "the variant is first declared here");
                error.highlight_with_note(*error_point, "..but it's redeclared here");
            
            },

            
            Error::VariableValueAndHintDiffer { value_type, hint_type, source } => {
                let msg = format!("the value is '{}' but the hint is '{}'",
                    value_type.display(fmt.string_map(), types),
                    hint_type.display(fmt.string_map(), types),
                );
                
                fmt
                    .error("variable type & hint differ in types")
                    .highlight_with_note(*source, &msg)
            },
            
            
            Error::VariableNotFound { name, source } => {
                let msg = format!("no variable named '{}'", fmt.string_map().get(*name));
                fmt.error("variable not found")
                    .highlight_with_note(*source, &msg)
            },

            
            Error::InvalidBinaryOp { operator, lhs, rhs, source } => {
                let msg = format!("can't apply the binary op '{}' between the types '{}' and '{}'",
                    operator,
                    lhs.display(fmt.string_map(), types),
                    rhs.display(fmt.string_map(), types),
                );

                fmt.error("invalid binary operation")
                    .highlight_with_note(*source, &msg)
            },

            
            Error::InvalidUnaryOp { operator, rhs, source } => {                
                let msg = format!("can't apply the unary op '{}' on type '{}'",
                    operator,
                    rhs.display(fmt.string_map(), types),
                );

                fmt.error("invalid binary operation")
                    .highlight_with_note(*source, &msg)
            },
            
             
            Error::IfMissingElse { body } => {
                let msg = format!("the main branch returns '{}' but there's no else branch", 
                    body.1.display(fmt.string_map(), types));

                let mut err = fmt.error("if is missing an else case");
                err.highlight_with_note(body.0, &msg);
            },

            Error::IfBodyAndElseMismatch { body, else_block } => {
                let msg = format!("the main branch returns '{}'", 
                    body.1.display(fmt.string_map(), types));
                
                let msg2 = format!("but the else branch returns '{}'", 
                    else_block.1.display(fmt.string_map(), types));

                let mut err = fmt.error("if branches differ in types");
                err.highlight_with_note(body.0, &msg);
                err.highlight_with_note(else_block.0, &msg2);
            },
            
            
            Error::MatchValueIsntEnum { source, typ } => {
                let msg = format!("is of type '{}' which is not an enum", 
                    typ.display(fmt.string_map(), types));

                fmt.error("match value isn't an enum")
                    .highlight_with_note(*source, &msg);
            },
            
            Error::StructCreationOnNonStruct { source, typ } => {
                let msg = format!("is of type '{}'", 
                    typ.display(fmt.string_map(), types));

                fmt.error("struct creation on a type which is not a struct")
                    .highlight_with_note(*source, &msg);
            }
             
            Error::FunctionNotFound { name, source } => {
                let msg = format!("there's no function named '{}' in the current scope",
                    fmt.string(*name),
                );

                fmt.error("function not found")
                    .highlight_with_note(*source, &msg)
            },

            Error::CallOnNonFunction { name, source } => {
                let msg = format!("the symbol named '{}' isn't a function",
                    fmt.string(*name),
                );

                fmt.error("call on non-function")
                    .highlight_with_note(*source, &msg)
            },
            
            Error::BindedFunctionNotFound { name, bind, source } => {                
                let msg = format!("there's no function named '{}' in the namespace of '{}'",
                    fmt.string(*name),
                    bind.display(fmt.string_map(), types)
                );

                fmt.error("associated function not found")
                    .highlight_with_note(*source, &msg)
            },
            
            Error::FunctionArgsMismatch { source, sig_len, call_len } => {
                let msg = format!("function has {} argument(s) but you've provided {} argument(s)",
                    sig_len,
                    call_len,
                );

                fmt.error("function argument count mismatch")
                    .highlight_with_note(*source, &msg)
            },
            
            Error::NamespaceNotFound { source, namespace } => {
                let msg = format!("there's no namespace named '{}' in the current scope",
                    fmt.string(*namespace),
                );

                fmt.error("namespace not found")
                    .highlight_with_note(*source, &msg)
            },

            
            Error::FieldAccessOnNonEnumOrStruct { source, typ } => {                
                let msg = format!("..is of type '{}' which is neither a struct or an enum",
                    typ.display(fmt.string_map(), types),
                );

                fmt.error("can't access fields on this type")
                    .highlight_with_note(*source, &msg)
            },

            
            Error::FieldDoesntExist { source, field, typ } => {                
                let msg = format!("the type '{}' doesn't have a field named '{}'",
                    typ.display(fmt.string_map(), types),
                    fmt.string(*field),
                );

                fmt.error("field doesn't exist")
                    .highlight_with_note(*source, &msg)
            },
            
            
            Error::MatchBranchesDifferInReturnType { 
                initial_source, initial_typ, 
                branch_source, branch_typ 
            } => {
                let msg1 = format!("..returns '{}'",
                    initial_typ.display(fmt.string_map(), types),
                );

                let msg2 = format!("..but this returns '{}'",
                    branch_typ.display(fmt.string_map(), types),
                );

                let mut err = fmt.error("match branches differ in return types");
                err
                    .highlight_with_note(*initial_source, &msg1);
                err
                    .highlight_with_note(*branch_source, &msg2);
            },
            
            
            Error::InvalidMatch { name, range, value } => {
                let msg = format!("there's no variant named '{}' in '{}'",
                    fmt.string(*name),
                    value.display(fmt.string_map(), types),
                );

                fmt.error("invalid match variant")
                    .highlight_with_note(*range, &msg)
            },

            
            Error::MissingMatch { name, range } => {
                let mut msg = format!("missing variants: ");
                let mut is_first = true;
                for n in name.iter() {
                    if !is_first {
                        let _ = write!(msg, ", ");
                    }

                    is_first = false;
                    let _ = write!(msg, "{}", fmt.string(*n));
                }

                fmt.error("non-exhaustive match")
                    .highlight_with_note(*range, &msg)
            },
            
            
            Error::MissingFields { source, fields } => {
                let mut msg = format!("missing fields: ");
                let mut is_first = true;
                for n in fields {
                    if !is_first {
                        let _ = write!(msg, ", ");
                    }

                    is_first = false;
                    let _ = write!(msg, "{}", fmt.string(*n));
                }

                fmt.error("missing fields")
                    .highlight_with_note(*source, &msg)
                
            },
            
                       
            Error::ValueUpdateTypeMismatch { lhs, rhs, source } => {
                let msg = format!("lhs is '{}' while the rhs is '{}'",
                    lhs.display(fmt.string_map(), types),
                    rhs.display(fmt.string_map(), types),
                );

                fmt.error("can't update a value with a different type")
                    .highlight_with_note(*source, &msg)
            },

                       
            Error::ContinueOutsideOfLoop(v) => {
                fmt.error("continue outside of loop")
                    .highlight(*v);
            },
            
            
            Error::BreakOutsideOfLoop(v) => {
                fmt.error("break outside of loop")
                    .highlight(*v);
            },
            
            
            Error::CantUnwrapOnGivenType(s, t) => {
                let typ_name = t.display(fmt.string_map(), types);
                let msg = format!("..is of type '{typ_name}'");
                
                fmt.error("can't unwrap on given type")
                    .highlight_with_note(*s, &msg)
            },
            
            
            Error::CantTryOnGivenType(s, t) => {
                let typ_name = t.display(fmt.string_map(), types);
                let msg = format!("..is of type '{typ_name}'");
                
                fmt.error("can't try on given type")
                    .highlight_with_note(*s, &msg)
            },
            
            
            Error::FunctionDoesntReturnAnOption { source, func_typ } => {
                let msg = format!(
                    "..because of this expected the function to return an option but the function returns '{}'",
                    func_typ.display(fmt.string_map(), types)
                );

                fmt.error("function doesn't return an option")
                    .highlight_with_note(*source, &msg)
            },
            
            
            Error::FunctionDoesntReturnAResult { source, func_typ } => {
                let msg = format!(
                    "..because of this expected the function to return a result but the function returns '{}'",
                    func_typ.display(fmt.string_map(), types)
                );

                fmt.error("function doesn't return a result ")
                    .highlight_with_note(*source, &msg)
            },
            
            
            Error::FunctionReturnsAResultButTheErrIsntTheSame { source, func_err_typ, err_typ, func_source } => {
                let msg = format!(
                    "the error type is '{}'",
                    err_typ.display(fmt.string_map(), types)
                );
                
                let msg2 = format!(
                    "..but the error type of the function is '{}'",
                    func_err_typ.display(fmt.string_map(), types)
                );

                let mut err = fmt.error("result error types differ");

                err.highlight_with_note(*source, &msg);
                err.highlight_with_note(*func_source, &msg2);
            },
            
            
            Error::ReturnAndFuncTypDiffer { source, func_source, typ, func_typ } => {
                let msg = format!(
                    "..is of type '{}'",
                    typ.display(fmt.string_map(), types)
                );
                
                let msg2 = format!(
                    "..but the function returns '{}'",
                    func_typ.display(fmt.string_map(), types)
                );

                let mut err = fmt.error("return and function return type differ");

                err.highlight_with_note(*source, &msg);
                err.highlight_with_note(*func_source, &msg2);
            }
            

            Error::AssignIsNotLHSValue { source } => {
                fmt.error("this is not a valid lhs expression")
                    .highlight(*source);
            },


            Error::VariableValueNotTuple(s) => {
                fmt.error("variable value is not a tuple")
                    .highlight(*s);
            },

            
            Error::UnknownAttr(source, _) => {
                fmt.error("unknown attribute")
                    .highlight(*source);
            },

            Error::InvalidValueForAttr { attr, value, expected } => {
                let msg = format!("is an invalid value for attribute '{}' which expects {expected}", fmt.string(attr.1));
                fmt.error("invalid value for attribute")
                    .highlight_with_note(*value, &msg);
            },

            Error::InvalidCast { range, from_ty, to_ty } => {
                let msg = format!("can't cast '{}' to '{}'", 
                                  from_ty.display(fmt.string_map(), types),
                                  to_ty.display(fmt.string_map(), types));
                fmt.error("invalid as cast")
                    .highlight_with_note(*range, &msg);
            },

            Error::ValueIsntAnIterator { ty, range } => {
                let msg = format!("'{}' is not an iterator",
                                  ty.display(fmt.string_map(), types));

                fmt.error("expression isn't an iterator")
                    .highlight_with_note(*range, &msg);
            },

            Error::IteratorFunctionInvalidSig(v) => {
                fmt.error("invalid iterator function signature")
                    .highlight_with_note(*v, "signature must match 'fn __next__(&self): Option<[type]>`");
            },

            Error::DerefOnNonPtr(v) => {
                fmt.error("deref on non pointer")
                    .highlight(*v);
            },

            Error::CantUseHoleHere { source } => {
                fmt.error("can't use the hole ('_') here")
                    .highlight(*source);
            },

            Error::UnableToInfer(v) => {
                fmt.error("unable to infer type")
                    .highlight_with_note(*v, "try specifying it's generics");
            },

            Error::InvalidRange { source, ty } => {
                let msg = format!("range bounds can only be integers but you provided '{}'", ty.display(fmt.string_map(), types));

                fmt.error("invalid range bound")
                    .highlight_with_note(*source, &*msg);
            },

            Error::ImplOnGeneric(s) => {
                fmt.error("can't impl on a generic")
                    .highlight(*s)
            },

            Error::GenericLenMismatch { source, found, expected } => {
                let msg = format!("the type has {} generics but you've provided {}", expected, found);
                fmt.error("generic length mismatch")
                    .highlight_with_note(*source, &msg)
            },

            Error::NameIsReservedForFunctions { source } => {
                fmt.error("this name is reserved for an overwritable function")
                    .highlight(*source);
            },

            Error::InvalidSystem(v) => {
                fmt.error("system functions must be outside of an impl block & not have any generics")
                    .highlight(*v)
            },

            Error::IndexOnNonList(range, sym) => {
                let msg = format!("indexing is not available on '{}'", sym.display(fmt.string_map(), types));
                fmt.error("indexing on non-list value")
                    .highlight_with_note(*range, &msg)
            },

            Error::VariableTupleAndHintTupleSizeMismatch(range, exp, given) => {
                todo!()
            },


            Error::Bypass => (),
        }
    }
}
