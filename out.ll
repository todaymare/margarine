; ModuleID = 'margarine'
source_filename = "margarine"

%str = type <{ i32, [1 x i8] }>
%str.2 = type <{ i32, [2 x i8] }>
%str.3 = type <{ i32, [11 x i8] }>
%str.4 = type <{ i32, [11 x i8] }>
%lexer_err_ty = type { i32, ptr }
%parser_err_ty = type { i32, ptr }
%funcRef = type { ptr, ptr }
%captures.0 = type { ptr }
%"(int)" = type { i64 }
%enumRef = type { ptr, i32 }
%captures = type { %funcRef, ptr }
%"std::iter::Iter<unit>" = type { %funcRef }
%"std::iter::Iter<int>" = type { %funcRef }
%anyType = type { ptr, i32 }
%Range = type { i64, i64 }
%captures.1 = type { ptr }

@str = global %str <{ i32 1, [1 x i8] c"\0A" }>
@str.6 = global %str.2 <{ i32 2, [2 x i8] c"hi" }>
@str.7 = global %str.3 <{ i32 11, [11 x i8] c"hello world" }>
@str.9 = global %str.4 <{ i32 11, [11 x i8] c"hello world" }>
@fileCount = global i32 3
@0 = global [0 x ptr] zeroinitializer
@1 = global [0 x ptr] zeroinitializer
@2 = global [0 x ptr] zeroinitializer
@lexerErrors = global [3 x %lexer_err_ty] [%lexer_err_ty { i32 0, ptr @0 }, %lexer_err_ty { i32 0, ptr @1 }, %lexer_err_ty { i32 0, ptr @2 }]
@3 = global [0 x ptr] zeroinitializer
@4 = global [0 x ptr] zeroinitializer
@5 = global [0 x ptr] zeroinitializer
@parserErrors = global [3 x %parser_err_ty] [%parser_err_ty { i32 0, ptr @3 }, %parser_err_ty { i32 0, ptr @4 }, %parser_err_ty { i32 0, ptr @5 }]
@semaErrors = global [0 x ptr] zeroinitializer
@semaErrorsLen = global i32 0

; Function Attrs: noreturn
declare void @margarineAbort() #0

; Function Attrs: noreturn
declare void @margarineError() #0

declare ptr @margarineAlloc(i64)

define i64 @"std::iter::Iter::sum"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %captures.0, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %funcRef, align 8
  %10 = alloca %"(int)", align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %name = call ptr @margarineAlloc(i64 8)
  %field_ptr = getelementptr inbounds nuw %"(int)", ptr %name, i32 0, i32 0
  store i64 0, ptr %field_ptr, align 4
  store ptr %name, ptr %4, align 8
  %load = load ptr, ptr %2, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  store ptr @"std::iter::Iter::for_each", ptr %field_ptr1, align 8
  %field_ptr2 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 1
  store ptr null, ptr %field_ptr2, align 8
  %load3 = load %funcRef, ptr %5, align 8
  %load4 = load ptr, ptr %4, align 8
  %field_ptr5 = getelementptr inbounds nuw %captures.0, ptr %6, i32 0, i32 0
  store ptr %load4, ptr %field_ptr5, align 8
  %load6 = load %captures.0, ptr %6, align 8
  %name7 = call ptr @margarineAlloc(i64 8)
  store %captures.0 %load6, ptr %name7, align 8
  %field_ptr8 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  store ptr @"<closure>.2", ptr %field_ptr8, align 8
  %field_ptr9 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  store ptr %name7, ptr %field_ptr9, align 8
  %load10 = load %funcRef, ptr %7, align 8
  store %funcRef %load3, ptr %8, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 0
  %load11 = load ptr, ptr %field_load, align 8
  store %funcRef %load3, ptr %9, align 8
  %field_load12 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 1
  %load13 = load ptr, ptr %field_load12, align 8
  %name14 = call {} %load11(ptr %load, %funcRef %load10, ptr %load13)
  %load15 = load ptr, ptr %4, align 8
  %load16 = load %"(int)", ptr %load15, align 4
  store %"(int)" %load16, ptr %10, align 4
  %field_load17 = getelementptr inbounds nuw %"(int)", ptr %10, i32 0, i32 0
  %load18 = load i64, ptr %field_load17, align 4
  ret i64 %load18
}

define {} @"std::iter::Iter::for_each"(ptr %0, %funcRef %1, ptr %2) {
prelude:
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %enumRef, align 8
  %10 = alloca %enumRef, align 8
  %11 = alloca {}, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %3, align 8
  store %funcRef %1, ptr %4, align 8
  store ptr %2, ptr %5, align 8
  %load = load ptr, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  store ptr @"std::iter::Iter::map", ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load2 = load %funcRef, ptr %6, align 8
  %load3 = load %funcRef, ptr %4, align 8
  store %funcRef %load2, ptr %7, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  %load4 = load ptr, ptr %field_load, align 8
  store %funcRef %load2, ptr %8, align 8
  %field_load5 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 1
  %load6 = load ptr, ptr %field_load5, align 8
  %name = call ptr %load4(ptr %load, %funcRef %load3, ptr %load6)
  br label %loop_body

loop_body:                                        ; preds = %cont, %entry
  %name7 = call %enumRef @"std::iter::Iter::__next__.1"(ptr %name, ptr null)
  store %enumRef %name7, ptr %9, align 8
  %field_load8 = getelementptr inbounds nuw %enumRef, ptr %9, i32 0, i32 1
  %load9 = load i32, ptr %field_load8, align 4
  %icmp = icmp eq i32 %load9, 1
  br i1 %icmp, label %then, label %else

loop_cont:                                        ; preds = %then
  ret {} zeroinitializer

then:                                             ; preds = %loop_body
  br label %loop_cont

else:                                             ; preds = %loop_body
  br label %cont

cont:                                             ; preds = %else, %12
  store %enumRef %name7, ptr %10, align 8
  %field_load10 = getelementptr inbounds nuw %enumRef, ptr %10, i32 0, i32 0
  %load11 = load ptr, ptr %field_load10, align 8
  %load12 = load {}, ptr %load11, align 1
  store {} %load12, ptr %11, align 1
  br label %loop_body

12:                                               ; No predecessors!
  br label %cont
}

define ptr @"std::iter::Iter::map"(ptr %0, %funcRef %1, ptr %2) {
prelude:
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %captures, align 8
  %7 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %3, align 8
  store %funcRef %1, ptr %4, align 8
  store ptr %2, ptr %5, align 8
  %load = load %funcRef, ptr %4, align 8
  %load1 = load ptr, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %captures, ptr %6, i32 0, i32 0
  store %funcRef %load, ptr %field_ptr, align 8
  %field_ptr2 = getelementptr inbounds nuw %captures, ptr %6, i32 0, i32 1
  store ptr %load1, ptr %field_ptr2, align 8
  %load3 = load %captures, ptr %6, align 8
  %name = call ptr @margarineAlloc(i64 24)
  store %captures %load3, ptr %name, align 8
  %field_ptr4 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  store ptr @"<closure>", ptr %field_ptr4, align 8
  %field_ptr5 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  store ptr %name, ptr %field_ptr5, align 8
  %load6 = load %funcRef, ptr %7, align 8
  %name7 = call ptr @margarineAlloc(i64 16)
  %field_ptr8 = getelementptr inbounds nuw %"std::iter::Iter<unit>", ptr %name7, i32 0, i32 0
  store %funcRef %load6, ptr %field_ptr8, align 8
  ret ptr %name7
}

define %enumRef @"<closure>"(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %captures, align 8
  %3 = alloca %funcRef, align 8
  %4 = alloca %captures, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %enumRef, align 8
  %10 = alloca %enumRef, align 8
  %11 = alloca i64, align 8
  %12 = alloca %funcRef, align 8
  %13 = alloca %funcRef, align 8
  %14 = alloca %funcRef, align 8
  %15 = alloca %funcRef, align 8
  %16 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %load = load ptr, ptr %1, align 8
  %load1 = load %captures, ptr %load, align 8
  store %captures %load1, ptr %2, align 8
  %field_load = getelementptr inbounds nuw %captures, ptr %2, i32 0, i32 0
  %load2 = load %funcRef, ptr %field_load, align 8
  store %funcRef %load2, ptr %3, align 8
  store %captures %load1, ptr %4, align 8
  %field_load3 = getelementptr inbounds nuw %captures, ptr %4, i32 0, i32 1
  %load4 = load ptr, ptr %field_load3, align 8
  store ptr %load4, ptr %5, align 8
  %load5 = load ptr, ptr %5, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  store ptr @"std::iter::Iter::__next__", ptr %field_ptr, align 8
  %field_ptr6 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  store ptr null, ptr %field_ptr6, align 8
  %load7 = load %funcRef, ptr %6, align 8
  store %funcRef %load7, ptr %7, align 8
  %field_load8 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  %load9 = load ptr, ptr %field_load8, align 8
  store %funcRef %load7, ptr %8, align 8
  %field_load10 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 1
  %load11 = load ptr, ptr %field_load10, align 8
  %name = call %enumRef %load9(ptr %load5, ptr %load11)
  store %enumRef %name, ptr %9, align 8
  %field_load12 = getelementptr inbounds nuw %enumRef, ptr %9, i32 0, i32 1
  %load13 = load i32, ptr %field_load12, align 4
  store %enumRef %name, ptr %10, align 8
  %field_load14 = getelementptr inbounds nuw %enumRef, ptr %10, i32 0, i32 0
  %load15 = load ptr, ptr %field_load14, align 8
  %icmp = icmp eq i32 %load13, 0
  br i1 %icmp, label %then, label %else

then:                                             ; preds = %entry
  br label %cont

else:                                             ; preds = %entry
  ret %enumRef %name

cont:                                             ; preds = %17, %then
  %load16 = load i64, ptr %load15, align 4
  store i64 %load16, ptr %11, align 4
  %field_ptr17 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 0
  store ptr @Option, ptr %field_ptr17, align 8
  %field_ptr18 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 1
  store ptr null, ptr %field_ptr18, align 8
  %load19 = load %funcRef, ptr %12, align 8
  %load20 = load %funcRef, ptr %3, align 8
  %load21 = load i64, ptr %11, align 4
  store %funcRef %load20, ptr %13, align 8
  %field_load22 = getelementptr inbounds nuw %funcRef, ptr %13, i32 0, i32 0
  %load23 = load ptr, ptr %field_load22, align 8
  store %funcRef %load20, ptr %14, align 8
  %field_load24 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 1
  %load25 = load ptr, ptr %field_load24, align 8
  %name26 = call {} %load23(i64 %load21, ptr %load25)
  store %funcRef %load19, ptr %15, align 8
  %field_load27 = getelementptr inbounds nuw %funcRef, ptr %15, i32 0, i32 0
  %load28 = load ptr, ptr %field_load27, align 8
  store %funcRef %load19, ptr %16, align 8
  %field_load29 = getelementptr inbounds nuw %funcRef, ptr %16, i32 0, i32 1
  %load30 = load ptr, ptr %field_load29, align 8
  %name31 = call %enumRef %load28({} %name26, ptr %load30)
  ret %enumRef %name31

17:                                               ; No predecessors!
  br label %cont
}

define %enumRef @"std::iter::Iter::__next__"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %"std::iter::Iter<int>", align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %load1 = load %"std::iter::Iter<int>", ptr %load, align 8
  store %"std::iter::Iter<int>" %load1, ptr %4, align 8
  %field_load = getelementptr inbounds nuw %"std::iter::Iter<int>", ptr %4, i32 0, i32 0
  %load2 = load %funcRef, ptr %field_load, align 8
  store %funcRef %load2, ptr %5, align 8
  %load3 = load %funcRef, ptr %5, align 8
  store %funcRef %load3, ptr %6, align 8
  %field_load4 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  %load5 = load ptr, ptr %field_load4, align 8
  store %funcRef %load3, ptr %7, align 8
  %field_load6 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  %load7 = load ptr, ptr %field_load6, align 8
  %name = call %enumRef %load5(ptr %load7)
  ret %enumRef %name
}

define %enumRef @Option({} %0) {
prelude:
  %1 = alloca {}, align 8
  %2 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store {} %0, ptr %1, align 1
  %name = call ptr @margarineAlloc(i64 8)
  store ptr null, ptr %name, align 8
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %2, i32 0, i32 0
  store ptr %name, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %enumRef, ptr %2, i32 0, i32 1
  store i32 0, ptr %field_ptr1, align 4
  %load = load %enumRef, ptr %2, align 8
  ret %enumRef %load
}

define %enumRef @"std::iter::Iter::__next__.1"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %"std::iter::Iter<unit>", align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %load1 = load %"std::iter::Iter<unit>", ptr %load, align 8
  store %"std::iter::Iter<unit>" %load1, ptr %4, align 8
  %field_load = getelementptr inbounds nuw %"std::iter::Iter<unit>", ptr %4, i32 0, i32 0
  %load2 = load %funcRef, ptr %field_load, align 8
  store %funcRef %load2, ptr %5, align 8
  %load3 = load %funcRef, ptr %5, align 8
  store %funcRef %load3, ptr %6, align 8
  %field_load4 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  %load5 = load ptr, ptr %field_load4, align 8
  store %funcRef %load3, ptr %7, align 8
  %field_load6 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  %load7 = load ptr, ptr %field_load6, align 8
  %name = call %enumRef %load5(ptr %load7)
  ret %enumRef %name
}

define {} @"<closure>.2"(i64 %0, ptr %1) {
prelude:
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %captures.0, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %"(int)", align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %2, align 4
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %3, align 8
  %load1 = load %captures.0, ptr %load, align 8
  store %captures.0 %load1, ptr %4, align 8
  %field_load = getelementptr inbounds nuw %captures.0, ptr %4, i32 0, i32 0
  %load2 = load ptr, ptr %field_load, align 8
  store ptr %load2, ptr %5, align 8
  %load3 = load ptr, ptr %5, align 8
  %load4 = load %"(int)", ptr %load3, align 4
  store %"(int)" %load4, ptr %6, align 4
  %field_load5 = getelementptr inbounds nuw %"(int)", ptr %6, i32 0, i32 0
  %load6 = load i64, ptr %field_load5, align 4
  %load7 = load i64, ptr %2, align 4
  %addi = add i64 %load6, %load7
  %load8 = load ptr, ptr %5, align 8
  %field_ptr = getelementptr inbounds nuw %"(int)", ptr %load8, i32 0, i32 0
  store i64 %addi, ptr %field_ptr, align 4
  ret {} zeroinitializer
}

; Function Attrs: noreturn
define ptr @"std::panic"(ptr %0, ptr %1) #0 {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @"std::println", ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %4, align 8
  %load2 = load ptr, ptr %2, align 8
  store %funcRef %load, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load3 = load ptr, ptr %field_load, align 8
  store %funcRef %load, ptr %6, align 8
  %field_load4 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load5 = load ptr, ptr %field_load4, align 8
  %name = call {} %load3(ptr %load2, ptr %load5)
  %field_ptr6 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  store ptr @margarineAbort, ptr %field_ptr6, align 8
  %field_ptr7 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  store ptr null, ptr %field_ptr7, align 8
  %load8 = load %funcRef, ptr %7, align 8
  store %funcRef %load8, ptr %8, align 8
  %field_load9 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 0
  %load10 = load ptr, ptr %field_load9, align 8
  store %funcRef %load8, ptr %9, align 8
  %field_load11 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 1
  %load12 = load ptr, ptr %field_load11, align 8
  %name13 = call ptr %load10(ptr %load12)
  unreachable
}

define {} @"std::println"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @"std::print", ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %4, align 8
  %load2 = load ptr, ptr %2, align 8
  store %funcRef %load, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load3 = load ptr, ptr %field_load, align 8
  store %funcRef %load, ptr %6, align 8
  %field_load4 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load5 = load ptr, ptr %field_load4, align 8
  %name = call {} %load3(ptr %load2, ptr %load5)
  %field_ptr6 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  store ptr @"std::print", ptr %field_ptr6, align 8
  %field_ptr7 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  store ptr null, ptr %field_ptr7, align 8
  %load8 = load %funcRef, ptr %7, align 8
  store %funcRef %load8, ptr %8, align 8
  %field_load9 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 0
  %load10 = load ptr, ptr %field_load9, align 8
  store %funcRef %load8, ptr %9, align 8
  %field_load11 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 1
  %load12 = load ptr, ptr %field_load11, align 8
  %name13 = call {} %load10(ptr @str, ptr %load12)
  ret {} zeroinitializer
}

define {} @"std::print"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @print_raw, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %4, align 8
  %field_ptr2 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  store ptr @"$any", ptr %field_ptr2, align 8
  %field_ptr3 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 1
  store ptr null, ptr %field_ptr3, align 8
  %load4 = load %funcRef, ptr %5, align 8
  %load5 = load ptr, ptr %2, align 8
  store %funcRef %load4, ptr %6, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  %load6 = load ptr, ptr %field_load, align 8
  store %funcRef %load4, ptr %7, align 8
  %field_load7 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  %load8 = load ptr, ptr %field_load7, align 8
  %name = call %anyType %load6(ptr %load5, ptr %load8)
  store %funcRef %load, ptr %8, align 8
  %field_load9 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 0
  %load10 = load ptr, ptr %field_load9, align 8
  store %funcRef %load, ptr %9, align 8
  %field_load11 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 1
  %load12 = load ptr, ptr %field_load11, align 8
  %name13 = call {} %load10(%anyType %name, ptr %load12)
  ret {} zeroinitializer
}

declare {} @print_raw(%anyType, ptr)

define %anyType @"$any"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %anyType, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %name = call ptr @margarineAlloc(i64 8)
  %load = load ptr, ptr %2, align 8
  store ptr %load, ptr %name, align 8
  %field_ptr = getelementptr inbounds nuw %anyType, ptr %4, i32 0, i32 0
  store ptr %name, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %anyType, ptr %4, i32 0, i32 1
  store i32 16, ptr %field_ptr1, align 4
  %load2 = load %anyType, ptr %4, align 8
  ret %anyType %load2
}

define {} @"std::assert"(%enumRef %0, ptr %1, ptr %2) {
prelude:
  %3 = alloca %enumRef, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %enumRef, align 8
  %7 = alloca %enumRef, align 8
  %8 = alloca {}, align 8
  %9 = alloca %enumRef, align 8
  %10 = alloca %funcRef, align 8
  %11 = alloca %funcRef, align 8
  %12 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store %enumRef %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  store ptr %2, ptr %5, align 8
  %load = load %enumRef, ptr %3, align 8
  store %enumRef %load, ptr %6, align 8
  %field_load = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 1
  %load1 = load i32, ptr %field_load, align 4
  %icast = trunc i32 %load1 to i1
  %bnot = xor i1 %icast, true
  store %enumRef %load, ptr %7, align 8
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %7, i32 0, i32 1
  store i1 %bnot, ptr %field_ptr, align 1
  %load2 = load %enumRef, ptr %field_ptr, align 8
  store %enumRef %load2, ptr %9, align 8
  %field_load3 = getelementptr inbounds nuw %enumRef, ptr %9, i32 0, i32 1
  %load4 = load i32, ptr %field_load3, align 4
  %icast5 = trunc i32 %load4 to i1
  br i1 %icast5, label %then, label %else

then:                                             ; preds = %entry
  %field_ptr6 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 0
  store ptr @"std::panic", ptr %field_ptr6, align 8
  %field_ptr7 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 1
  store ptr null, ptr %field_ptr7, align 8
  %load8 = load %funcRef, ptr %10, align 8
  %load9 = load ptr, ptr %4, align 8
  store %funcRef %load8, ptr %11, align 8
  %field_load10 = getelementptr inbounds nuw %funcRef, ptr %11, i32 0, i32 0
  %load11 = load ptr, ptr %field_load10, align 8
  store %funcRef %load8, ptr %12, align 8
  %field_load12 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 1
  %load13 = load ptr, ptr %field_load12, align 8
  %name = call ptr %load11(ptr %load9, ptr %load13)
  unreachable

else:                                             ; preds = %entry
  br label %cont

cont:                                             ; preds = %else, %13
  %load14 = load {}, ptr %8, align 1
  ret {} %load14

13:                                               ; No predecessors!
  store {} zeroinitializer, ptr %8, align 1
  br label %cont
}

define %enumRef @"Range::__next__"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %Range, align 8
  %5 = alloca %Range, align 8
  %6 = alloca %enumRef, align 8
  %7 = alloca %enumRef, align 8
  %8 = alloca %enumRef, align 8
  %9 = alloca %Range, align 8
  %10 = alloca %funcRef, align 8
  %11 = alloca %Range, align 8
  %12 = alloca %funcRef, align 8
  %13 = alloca %funcRef, align 8
  %14 = alloca %funcRef, align 8
  %15 = alloca %funcRef, align 8
  %16 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %load1 = load %Range, ptr %load, align 4
  store %Range %load1, ptr %4, align 4
  %field_load = getelementptr inbounds nuw %Range, ptr %4, i32 0, i32 0
  %load2 = load i64, ptr %field_load, align 4
  %load3 = load ptr, ptr %2, align 8
  %load4 = load %Range, ptr %load3, align 4
  store %Range %load4, ptr %5, align 4
  %field_load5 = getelementptr inbounds nuw %Range, ptr %5, i32 0, i32 1
  %load6 = load i64, ptr %field_load5, align 4
  %icmp = icmp slt i64 %load2, %load6
  %icast = zext i1 %icmp to i32
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 0
  store ptr null, ptr %field_ptr, align 8
  %field_ptr7 = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 1
  store i32 %icast, ptr %field_ptr7, align 4
  %load8 = load %enumRef, ptr %6, align 8
  store %enumRef %load8, ptr %8, align 8
  %field_load9 = getelementptr inbounds nuw %enumRef, ptr %8, i32 0, i32 1
  %load10 = load i32, ptr %field_load9, align 4
  %icast11 = trunc i32 %load10 to i1
  br i1 %icast11, label %then, label %else

then:                                             ; preds = %entry
  %load12 = load ptr, ptr %2, align 8
  %load13 = load %Range, ptr %load12, align 4
  store %Range %load13, ptr %9, align 4
  %field_load14 = getelementptr inbounds nuw %Range, ptr %9, i32 0, i32 0
  %load15 = load i64, ptr %field_load14, align 4
  %addi = add i64 %load15, 1
  %load16 = load ptr, ptr %2, align 8
  %field_ptr17 = getelementptr inbounds nuw %Range, ptr %load16, i32 0, i32 0
  store i64 %addi, ptr %field_ptr17, align 4
  %field_ptr18 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 0
  store ptr @Option.3, ptr %field_ptr18, align 8
  %field_ptr19 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 1
  store ptr null, ptr %field_ptr19, align 8
  %load20 = load %funcRef, ptr %10, align 8
  %load21 = load ptr, ptr %2, align 8
  %load22 = load %Range, ptr %load21, align 4
  store %Range %load22, ptr %11, align 4
  %field_load23 = getelementptr inbounds nuw %Range, ptr %11, i32 0, i32 0
  %load24 = load i64, ptr %field_load23, align 4
  %subi = sub i64 %load24, 1
  store %funcRef %load20, ptr %12, align 8
  %field_load25 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 0
  %load26 = load ptr, ptr %field_load25, align 8
  store %funcRef %load20, ptr %13, align 8
  %field_load27 = getelementptr inbounds nuw %funcRef, ptr %13, i32 0, i32 1
  %load28 = load ptr, ptr %field_load27, align 8
  %name = call %enumRef %load26(i64 %subi, ptr %load28)
  store %enumRef %name, ptr %7, align 8
  br label %cont

else:                                             ; preds = %entry
  %field_ptr29 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 0
  store ptr @Option.4, ptr %field_ptr29, align 8
  %field_ptr30 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 1
  store ptr null, ptr %field_ptr30, align 8
  %load31 = load %funcRef, ptr %14, align 8
  store %funcRef %load31, ptr %15, align 8
  %field_load32 = getelementptr inbounds nuw %funcRef, ptr %15, i32 0, i32 0
  %load33 = load ptr, ptr %field_load32, align 8
  store %funcRef %load31, ptr %16, align 8
  %field_load34 = getelementptr inbounds nuw %funcRef, ptr %16, i32 0, i32 1
  %load35 = load ptr, ptr %field_load34, align 8
  %name36 = call %enumRef %load33(ptr %load35)
  store %enumRef %name36, ptr %7, align 8
  br label %cont

cont:                                             ; preds = %else, %then
  %load37 = load %enumRef, ptr %7, align 8
  ret %enumRef %load37
}

define %enumRef @Option.3(i64 %0, ptr %1) {
prelude:
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %2, align 4
  store ptr %1, ptr %3, align 8
  %load = load i64, ptr %2, align 4
  %name = call ptr @margarineAlloc(i64 8)
  store i64 %load, ptr %name, align 4
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %4, i32 0, i32 0
  store ptr %name, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %enumRef, ptr %4, i32 0, i32 1
  store i32 0, ptr %field_ptr1, align 4
  %load2 = load %enumRef, ptr %4, align 8
  ret %enumRef %load2
}

define %enumRef @Option.4(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %name = call ptr @margarineAlloc(i64 8)
  store ptr null, ptr %name, align 8
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %2, i32 0, i32 0
  store ptr %name, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %enumRef, ptr %2, i32 0, i32 1
  store i32 1, ptr %field_ptr1, align 4
  %load = load %enumRef, ptr %2, align 8
  ret %enumRef %load
}

define ptr @"Range::iter"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %captures.1, align 8
  %5 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %field_ptr = getelementptr inbounds nuw %captures.1, ptr %4, i32 0, i32 0
  store ptr %load, ptr %field_ptr, align 8
  %load1 = load %captures.1, ptr %4, align 8
  %name = call ptr @margarineAlloc(i64 8)
  store %captures.1 %load1, ptr %name, align 8
  %field_ptr2 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  store ptr @"<closure>.5", ptr %field_ptr2, align 8
  %field_ptr3 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 1
  store ptr %name, ptr %field_ptr3, align 8
  %load4 = load %funcRef, ptr %5, align 8
  %name5 = call ptr @margarineAlloc(i64 16)
  %field_ptr6 = getelementptr inbounds nuw %"std::iter::Iter<int>", ptr %name5, i32 0, i32 0
  store %funcRef %load4, ptr %field_ptr6, align 8
  ret ptr %name5
}

define %enumRef @"<closure>.5"(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %captures.1, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %load = load ptr, ptr %1, align 8
  %load1 = load %captures.1, ptr %load, align 8
  store %captures.1 %load1, ptr %2, align 8
  %field_load = getelementptr inbounds nuw %captures.1, ptr %2, i32 0, i32 0
  %load2 = load ptr, ptr %field_load, align 8
  store ptr %load2, ptr %3, align 8
  %load3 = load ptr, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @"Range::__next__", ptr %field_ptr, align 8
  %field_ptr4 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr4, align 8
  %load5 = load %funcRef, ptr %4, align 8
  store %funcRef %load5, ptr %5, align 8
  %field_load6 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load7 = load ptr, ptr %field_load6, align 8
  store %funcRef %load5, ptr %6, align 8
  %field_load8 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load9 = load ptr, ptr %field_load8, align 8
  %name = call %enumRef %load7(ptr %load3, ptr %load9)
  ret %enumRef %name
}

define i64 @"int::abs"(i64 %0, ptr %1) {
prelude:
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %2, align 4
  store ptr %1, ptr %3, align 8
  %load = load i64, ptr %2, align 4
  %ashr = ashr i64 %load, 63
  store i64 %ashr, ptr %4, align 4
  %load1 = load i64, ptr %2, align 4
  %load2 = load i64, ptr %4, align 4
  %xor = xor i64 %load1, %load2
  %load3 = load i64, ptr %4, align 4
  %subi = sub i64 %xor, %load3
  ret i64 %subi
}

define ptr @"int::to_str"(i64 %0, ptr %1) {
prelude:
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %2, align 4
  store ptr %1, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @int_to_str, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %4, align 8
  %load2 = load i64, ptr %2, align 4
  store %funcRef %load, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load3 = load ptr, ptr %field_load, align 8
  store %funcRef %load, ptr %6, align 8
  %field_load4 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load5 = load ptr, ptr %field_load4, align 8
  %name = call ptr %load3(i64 %load2, ptr %load5)
  ret ptr %name
}

declare ptr @int_to_str(i64, ptr)

define i64 @"int::max"(i64 %0, i64 %1, ptr %2) {
prelude:
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %enumRef, align 8
  %7 = alloca i64, align 8
  %8 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %3, align 4
  store i64 %1, ptr %4, align 4
  store ptr %2, ptr %5, align 8
  %load = load i64, ptr %3, align 4
  %load1 = load i64, ptr %4, align 4
  %icmp = icmp sgt i64 %load, %load1
  %icast = zext i1 %icmp to i32
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 0
  store ptr null, ptr %field_ptr, align 8
  %field_ptr2 = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 1
  store i32 %icast, ptr %field_ptr2, align 4
  %load3 = load %enumRef, ptr %6, align 8
  store %enumRef %load3, ptr %8, align 8
  %field_load = getelementptr inbounds nuw %enumRef, ptr %8, i32 0, i32 1
  %load4 = load i32, ptr %field_load, align 4
  %icast5 = trunc i32 %load4 to i1
  br i1 %icast5, label %then, label %else

then:                                             ; preds = %entry
  %load6 = load i64, ptr %3, align 4
  store i64 %load6, ptr %7, align 4
  br label %cont

else:                                             ; preds = %entry
  %load7 = load i64, ptr %4, align 4
  store i64 %load7, ptr %7, align 4
  br label %cont

cont:                                             ; preds = %else, %then
  %load8 = load i64, ptr %7, align 4
  ret i64 %load8
}

define i64 @"int::min"(i64 %0, i64 %1, ptr %2) {
prelude:
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %enumRef, align 8
  %7 = alloca i64, align 8
  %8 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %3, align 4
  store i64 %1, ptr %4, align 4
  store ptr %2, ptr %5, align 8
  %load = load i64, ptr %3, align 4
  %load1 = load i64, ptr %4, align 4
  %icmp = icmp slt i64 %load, %load1
  %icast = zext i1 %icmp to i32
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 0
  store ptr null, ptr %field_ptr, align 8
  %field_ptr2 = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 1
  store i32 %icast, ptr %field_ptr2, align 4
  %load3 = load %enumRef, ptr %6, align 8
  store %enumRef %load3, ptr %8, align 8
  %field_load = getelementptr inbounds nuw %enumRef, ptr %8, i32 0, i32 1
  %load4 = load i32, ptr %field_load, align 4
  %icast5 = trunc i32 %load4 to i1
  br i1 %icast5, label %then, label %else

then:                                             ; preds = %entry
  %load6 = load i64, ptr %3, align 4
  store i64 %load6, ptr %7, align 4
  br label %cont

else:                                             ; preds = %entry
  %load7 = load i64, ptr %4, align 4
  store i64 %load7, ptr %7, align 4
  br label %cont

cont:                                             ; preds = %else, %then
  %load8 = load i64, ptr %7, align 4
  ret i64 %load8
}

define i64 @"int::pow"(i64 %0, i64 %1, ptr %2) {
prelude:
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca %enumRef, align 8
  %8 = alloca %enumRef, align 8
  %9 = alloca i64, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %3, align 4
  store i64 %1, ptr %4, align 4
  store ptr %2, ptr %5, align 8
  store i64 1, ptr %6, align 4
  %load = load i64, ptr %4, align 4
  %name = call ptr @margarineAlloc(i64 16)
  %field_ptr = getelementptr inbounds nuw %Range, ptr %name, i32 0, i32 0
  store i64 0, ptr %field_ptr, align 4
  %field_ptr1 = getelementptr inbounds nuw %Range, ptr %name, i32 0, i32 1
  store i64 %load, ptr %field_ptr1, align 4
  br label %loop_body

loop_body:                                        ; preds = %cont, %entry
  %name2 = call %enumRef @"Range::__next__"(ptr %name, ptr null)
  store %enumRef %name2, ptr %7, align 8
  %field_load = getelementptr inbounds nuw %enumRef, ptr %7, i32 0, i32 1
  %load3 = load i32, ptr %field_load, align 4
  %icmp = icmp eq i32 %load3, 1
  br i1 %icmp, label %then, label %else

loop_cont:                                        ; preds = %then
  %load9 = load i64, ptr %6, align 4
  ret i64 %load9

then:                                             ; preds = %loop_body
  br label %loop_cont

else:                                             ; preds = %loop_body
  br label %cont

cont:                                             ; preds = %else, %10
  store %enumRef %name2, ptr %8, align 8
  %field_load4 = getelementptr inbounds nuw %enumRef, ptr %8, i32 0, i32 0
  %load5 = load ptr, ptr %field_load4, align 8
  %load6 = load i64, ptr %load5, align 4
  store i64 %load6, ptr %9, align 4
  %load7 = load i64, ptr %6, align 4
  %load8 = load i64, ptr %3, align 4
  %muli = mul i64 %load7, %load8
  store i64 %muli, ptr %6, align 4
  br label %loop_body

10:                                               ; No predecessors!
  br label %cont
}

define ptr @"float::to_str"(double %0, ptr %1) {
prelude:
  %2 = alloca double, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store double %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @float_to_str, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %4, align 8
  %load2 = load double, ptr %2, align 8
  store %funcRef %load, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load3 = load ptr, ptr %field_load, align 8
  store %funcRef %load, ptr %6, align 8
  %field_load4 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load5 = load ptr, ptr %field_load4, align 8
  %name = call ptr %load3(double %load2, ptr %load5)
  ret ptr %name
}

declare ptr @float_to_str(double, ptr)

define {} @main(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %funcRef, align 8
  %3 = alloca %funcRef, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %2, i32 0, i32 0
  store ptr @"std::println", ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %2, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %2, align 8
  %field_ptr2 = getelementptr inbounds nuw %funcRef, ptr %3, i32 0, i32 0
  store ptr @"Foo::hey", ptr %field_ptr2, align 8
  %field_ptr3 = getelementptr inbounds nuw %funcRef, ptr %3, i32 0, i32 1
  store ptr null, ptr %field_ptr3, align 8
  %load4 = load %funcRef, ptr %3, align 8
  store %funcRef %load4, ptr %4, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  %load5 = load ptr, ptr %field_load, align 8
  store %funcRef %load4, ptr %5, align 8
  %field_load6 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 1
  %load7 = load ptr, ptr %field_load6, align 8
  %name = call ptr %load5(ptr @str.6, ptr %load7)
  store %funcRef %load, ptr %6, align 8
  %field_load8 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  %load9 = load ptr, ptr %field_load8, align 8
  store %funcRef %load, ptr %7, align 8
  %field_load10 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  %load11 = load ptr, ptr %field_load10, align 8
  %name12 = call {} %load9(ptr %name, ptr %load11)
  ret {} zeroinitializer
}

define ptr @"Foo::hey"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  ret ptr @str.7
}

define ptr @"Foo::hey.8"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  ret ptr @str.9
}

attributes #0 = { noreturn }
