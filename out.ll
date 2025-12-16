; ModuleID = 'margarine'
source_filename = "margarine"

%str = type <{ i32, [2 x i8] }>
%lexer_err_ty = type { i32, ptr }
%parser_err_ty = type { i32, ptr }
%funcRef = type { ptr, ptr }

@str = global %str <{ i32 2, [2 x i8] c"hi" }>
@fileCount = global i32 1
@0 = global [0 x ptr] zeroinitializer
@lexerErrors = global [1 x %lexer_err_ty] [%lexer_err_ty { i32 0, ptr @0 }]
@1 = global [0 x ptr] zeroinitializer
@parserErrors = global [1 x %parser_err_ty] [%parser_err_ty { i32 0, ptr @1 }]
@semaErrors = global [0 x ptr] zeroinitializer
@semaErrorsLen = global i32 0

; Function Attrs: noreturn
declare void @margarineAbort() #0

; Function Attrs: noreturn
declare void @margarineError() #0

declare ptr @margarineAlloc(i64)

define {} @main(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %funcRef, align 8
  %3 = alloca %funcRef, align 8
  %4 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %2, i32 0, i32 0
  store ptr @"str::hey", ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %2, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %2, align 8
  store %funcRef %load, ptr %3, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %3, i32 0, i32 0
  %load2 = load ptr, ptr %field_load, align 8
  store %funcRef %load, ptr %4, align 8
  %field_load3 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  %load4 = load ptr, ptr %field_load3, align 8
  %name = call {} %load2(ptr @str, ptr %load4)
  ret {} %name
}

define {} @"str::hey"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  ret {} zeroinitializer
}

attributes #0 = { noreturn }
