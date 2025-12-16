; ModuleID = 'margarine'
source_filename = "margarine"

%str = type <{ i32, [0 x i8] }>
%str.6 = type <{ i32, [1 x i8] }>
%str.8 = type <{ i32, [1 x i8] }>
%str.9 = type <{ i32, [11 x i8] }>
%str.10 = type <{ i32, [10 x i8] }>
%str.11 = type <{ i32, [9 x i8] }>
%str.12 = type <{ i32, [11 x i8] }>
%str.13 = type <{ i32, [0 x i8] }>
%str.14 = type <{ i32, [11 x i8] }>
%str.15 = type <{ i32, [1 x i8] }>
%str.16 = type <{ i32, [0 x i8] }>
%str.17 = type <{ i32, [3 x i8] }>
%str.18 = type <{ i32, [11 x i8] }>
%str.19 = type <{ i32, [2 x i8] }>
%str.21 = type <{ i32, [3 x i8] }>
%str.22 = type <{ i32, [1 x i8] }>
%str.23 = type <{ i32, [4 x i8] }>
%str.25 = type <{ i32, [7 x i8] }>
%lexer_err_ty = type { i32, ptr }
%parser_err_ty = type { i32, ptr }
%funcRef = type { ptr, ptr }
%captures.0 = type { ptr }
%"(int)" = type { i64 }
%enumRef = type { ptr, i32 }
%captures = type { ptr, %funcRef }
%"std::iter::Iter<unit>" = type { %funcRef }
%"std::iter::Iter<int>" = type { %funcRef }
%"std::duration::Duration" = type { i64, i64 }
%captures.1 = type { ptr }
%"std::iter::Iter<str>" = type { %funcRef }
%captures.3 = type { ptr, ptr }
%"std::string::SplitState" = type { %enumRef, ptr }
%"(str, str).4" = type { ptr, ptr }
%Range = type { i64, i64 }
%captures.5 = type { ptr }
%"std::string::Chars" = type { ptr }
%"(str, str)" = type { ptr, ptr }
%anyType = type { ptr, i32 }
%captures.7 = type { ptr }
%listType = type { i32, i32, ptr }
%captures.20 = type {}
%captures.24 = type {}

@str = global %str zeroinitializer
@str.8 = global %str.6 <{ i32 1, [1 x i8] c"\0A" }>
@str.15 = global %str.8 <{ i32 1, [1 x i8] c"\0A" }>
@str.16 = global %str.9 <{ i32 11, [11 x i8] c"hello world" }>
@str.17 = global %str.10 <{ i32 10, [10 x i8] c"Cargo.toml" }>
@str.19 = global %str.11 <{ i32 9, [9 x i8] c"---------" }>
@str.20 = global %str.12 <{ i32 11, [11 x i8] c"hello world" }>
@str.21 = global %str.13 zeroinitializer
@str.22 = global %str.14 <{ i32 11, [11 x i8] c"hello world" }>
@str.23 = global %str.15 <{ i32 1, [1 x i8] c" " }>
@str.24 = global %str.16 zeroinitializer
@str.25 = global %str.17 <{ i32 3, [3 x i8] c"hey" }>
@str.26 = global %str.18 <{ i32 11, [11 x i8] c"hello world" }>
@str.27 = global %str.19 <{ i32 2, [2 x i8] c"67" }>
@str.30 = global %str.21 <{ i32 3, [3 x i8] c"hey" }>
@str.34 = global %str.22 <{ i32 1, [1 x i8] c"\0A" }>
@str.35 = global %str.23 <{ i32 4, [4 x i8] c"4.20" }>
@str.43 = global %str.25 <{ i32 7, [7 x i8] c"paniced" }>
@fileCount = global i32 7
@0 = global [0 x ptr] zeroinitializer
@1 = global [0 x ptr] zeroinitializer
@2 = global [0 x ptr] zeroinitializer
@3 = global [0 x ptr] zeroinitializer
@4 = global [0 x ptr] zeroinitializer
@5 = global [0 x ptr] zeroinitializer
@6 = global [0 x ptr] zeroinitializer
@lexerErrors = global [7 x %lexer_err_ty] [%lexer_err_ty { i32 0, ptr @0 }, %lexer_err_ty { i32 0, ptr @1 }, %lexer_err_ty { i32 0, ptr @2 }, %lexer_err_ty { i32 0, ptr @3 }, %lexer_err_ty { i32 0, ptr @4 }, %lexer_err_ty { i32 0, ptr @5 }, %lexer_err_ty { i32 0, ptr @6 }]
@7 = global [0 x ptr] zeroinitializer
@8 = global [0 x ptr] zeroinitializer
@9 = global [0 x ptr] zeroinitializer
@10 = global [0 x ptr] zeroinitializer
@11 = global [0 x ptr] zeroinitializer
@12 = global [0 x ptr] zeroinitializer
@13 = global [0 x ptr] zeroinitializer
@parserErrors = global [7 x %parser_err_ty] [%parser_err_ty { i32 0, ptr @7 }, %parser_err_ty { i32 0, ptr @8 }, %parser_err_ty { i32 0, ptr @9 }, %parser_err_ty { i32 0, ptr @10 }, %parser_err_ty { i32 0, ptr @11 }, %parser_err_ty { i32 0, ptr @12 }, %parser_err_ty { i32 0, ptr @13 }]
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
  %load = load ptr, ptr %3, align 8
  %load1 = load %funcRef, ptr %4, align 8
  %field_ptr = getelementptr inbounds nuw %captures, ptr %6, i32 0, i32 0
  store ptr %load, ptr %field_ptr, align 8
  %field_ptr2 = getelementptr inbounds nuw %captures, ptr %6, i32 0, i32 1
  store %funcRef %load1, ptr %field_ptr2, align 8
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
  %3 = alloca ptr, align 8
  %4 = alloca %captures, align 8
  %5 = alloca %funcRef, align 8
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
  %load2 = load ptr, ptr %field_load, align 8
  store ptr %load2, ptr %3, align 8
  store %captures %load1, ptr %4, align 8
  %field_load3 = getelementptr inbounds nuw %captures, ptr %4, i32 0, i32 1
  %load4 = load %funcRef, ptr %field_load3, align 8
  store %funcRef %load4, ptr %5, align 8
  %load5 = load ptr, ptr %3, align 8
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
  %load20 = load %funcRef, ptr %5, align 8
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

define ptr @"std::duration::Duration::now"(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %funcRef, align 8
  %3 = alloca %funcRef, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %funcRef, align 8
  %10 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %2, i32 0, i32 0
  store ptr @"std::duration::Duration::new", ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %2, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %2, align 8
  %field_ptr2 = getelementptr inbounds nuw %funcRef, ptr %3, i32 0, i32 0
  store ptr @now_secs, ptr %field_ptr2, align 8
  %field_ptr3 = getelementptr inbounds nuw %funcRef, ptr %3, i32 0, i32 1
  store ptr null, ptr %field_ptr3, align 8
  %load4 = load %funcRef, ptr %3, align 8
  store %funcRef %load4, ptr %4, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  %load5 = load ptr, ptr %field_load, align 8
  store %funcRef %load4, ptr %5, align 8
  %field_load6 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 1
  %load7 = load ptr, ptr %field_load6, align 8
  %name = call i64 %load5(ptr %load7)
  %field_ptr8 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  store ptr @now_nanos, ptr %field_ptr8, align 8
  %field_ptr9 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  store ptr null, ptr %field_ptr9, align 8
  %load10 = load %funcRef, ptr %6, align 8
  store %funcRef %load10, ptr %7, align 8
  %field_load11 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  %load12 = load ptr, ptr %field_load11, align 8
  store %funcRef %load10, ptr %8, align 8
  %field_load13 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 1
  %load14 = load ptr, ptr %field_load13, align 8
  %name15 = call i64 %load12(ptr %load14)
  store %funcRef %load, ptr %9, align 8
  %field_load16 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 0
  %load17 = load ptr, ptr %field_load16, align 8
  store %funcRef %load, ptr %10, align 8
  %field_load18 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 1
  %load19 = load ptr, ptr %field_load18, align 8
  %name20 = call ptr %load17(i64 %name, i64 %name15, ptr %load19)
  ret ptr %name20
}

define ptr @"std::duration::Duration::new"(i64 %0, i64 %1, ptr %2) {
prelude:
  %3 = alloca i64, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca i64, align 8
  %7 = alloca i64, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %3, align 4
  store i64 %1, ptr %4, align 4
  store ptr %2, ptr %5, align 8
  %load = load i64, ptr %3, align 4
  %load1 = load i64, ptr %4, align 4
  %divs = sdiv i64 %load1, 1000000000
  %addi = add i64 %load, %divs
  store i64 %addi, ptr %6, align 4
  %load2 = load i64, ptr %4, align 4
  %rems = srem i64 %load2, 1000000000
  store i64 %rems, ptr %7, align 4
  %load3 = load i64, ptr %6, align 4
  %load4 = load i64, ptr %7, align 4
  %name = call ptr @margarineAlloc(i64 16)
  %field_ptr = getelementptr inbounds nuw %"std::duration::Duration", ptr %name, i32 0, i32 0
  store i64 %load3, ptr %field_ptr, align 4
  %field_ptr5 = getelementptr inbounds nuw %"std::duration::Duration", ptr %name, i32 0, i32 1
  store i64 %load4, ptr %field_ptr5, align 4
  ret ptr %name
}

declare i64 @now_secs(ptr)

declare i64 @now_nanos(ptr)

define ptr @"std::duration::Duration::elapsed"(ptr %0, ptr %1) {
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
  store ptr @"std::duration::Duration::now", ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %4, align 8
  store %funcRef %load, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load2 = load ptr, ptr %field_load, align 8
  store %funcRef %load, ptr %6, align 8
  %field_load3 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load4 = load ptr, ptr %field_load3, align 8
  %name = call ptr %load2(ptr %load4)
  %field_ptr5 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  store ptr @"std::duration::Duration::sub", ptr %field_ptr5, align 8
  %field_ptr6 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  store ptr null, ptr %field_ptr6, align 8
  %load7 = load %funcRef, ptr %7, align 8
  %load8 = load ptr, ptr %2, align 8
  store %funcRef %load7, ptr %8, align 8
  %field_load9 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 0
  %load10 = load ptr, ptr %field_load9, align 8
  store %funcRef %load7, ptr %9, align 8
  %field_load11 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 1
  %load12 = load ptr, ptr %field_load11, align 8
  %name13 = call ptr %load10(ptr %name, ptr %load8, ptr %load12)
  ret ptr %name13
}

define ptr @"std::duration::Duration::sub"(ptr %0, ptr %1, ptr %2) {
prelude:
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %"std::duration::Duration", align 8
  %7 = alloca %"std::duration::Duration", align 8
  %8 = alloca i64, align 8
  %9 = alloca %"std::duration::Duration", align 8
  %10 = alloca %"std::duration::Duration", align 8
  %11 = alloca i64, align 8
  %12 = alloca %enumRef, align 8
  %13 = alloca {}, align 8
  %14 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  store ptr %2, ptr %5, align 8
  %load = load ptr, ptr %3, align 8
  %load1 = load %"std::duration::Duration", ptr %load, align 4
  store %"std::duration::Duration" %load1, ptr %6, align 4
  %field_load = getelementptr inbounds nuw %"std::duration::Duration", ptr %6, i32 0, i32 1
  %load2 = load i64, ptr %field_load, align 4
  %load3 = load ptr, ptr %4, align 8
  %load4 = load %"std::duration::Duration", ptr %load3, align 4
  store %"std::duration::Duration" %load4, ptr %7, align 4
  %field_load5 = getelementptr inbounds nuw %"std::duration::Duration", ptr %7, i32 0, i32 1
  %load6 = load i64, ptr %field_load5, align 4
  %subi = sub i64 %load2, %load6
  store i64 %subi, ptr %8, align 4
  %load7 = load ptr, ptr %3, align 8
  %load8 = load %"std::duration::Duration", ptr %load7, align 4
  store %"std::duration::Duration" %load8, ptr %9, align 4
  %field_load9 = getelementptr inbounds nuw %"std::duration::Duration", ptr %9, i32 0, i32 0
  %load10 = load i64, ptr %field_load9, align 4
  %load11 = load ptr, ptr %4, align 8
  %load12 = load %"std::duration::Duration", ptr %load11, align 4
  store %"std::duration::Duration" %load12, ptr %10, align 4
  %field_load13 = getelementptr inbounds nuw %"std::duration::Duration", ptr %10, i32 0, i32 0
  %load14 = load i64, ptr %field_load13, align 4
  %subi15 = sub i64 %load10, %load14
  store i64 %subi15, ptr %11, align 4
  %load16 = load i64, ptr %8, align 4
  %icmp = icmp slt i64 %load16, 0
  %icast = zext i1 %icmp to i32
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %12, i32 0, i32 0
  store ptr null, ptr %field_ptr, align 8
  %field_ptr17 = getelementptr inbounds nuw %enumRef, ptr %12, i32 0, i32 1
  store i32 %icast, ptr %field_ptr17, align 4
  %load18 = load %enumRef, ptr %12, align 8
  store %enumRef %load18, ptr %14, align 8
  %field_load19 = getelementptr inbounds nuw %enumRef, ptr %14, i32 0, i32 1
  %load20 = load i32, ptr %field_load19, align 4
  %icast21 = trunc i32 %load20 to i1
  br i1 %icast21, label %then, label %else

then:                                             ; preds = %entry
  %load22 = load i64, ptr %11, align 4
  %subi23 = sub i64 %load22, 1
  store i64 %subi23, ptr %11, align 4
  %load24 = load i64, ptr %8, align 4
  %addi = add i64 1000000000, %load24
  store i64 %addi, ptr %8, align 4
  store {} zeroinitializer, ptr %13, align 1
  br label %cont

else:                                             ; preds = %entry
  br label %cont

cont:                                             ; preds = %else, %then
  %load25 = load {}, ptr %13, align 1
  %load26 = load i64, ptr %11, align 4
  %load27 = load i64, ptr %8, align 4
  %name = call ptr @margarineAlloc(i64 16)
  %field_ptr28 = getelementptr inbounds nuw %"std::duration::Duration", ptr %name, i32 0, i32 0
  store i64 %load26, ptr %field_ptr28, align 4
  %field_ptr29 = getelementptr inbounds nuw %"std::duration::Duration", ptr %name, i32 0, i32 1
  store i64 %load27, ptr %field_ptr29, align 4
  ret ptr %name
}

define ptr @"std::duration::Duration::from_secs"(i64 %0, ptr %1) {
prelude:
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %2, align 4
  store ptr %1, ptr %3, align 8
  %load = load i64, ptr %2, align 4
  %name = call ptr @margarineAlloc(i64 16)
  %field_ptr = getelementptr inbounds nuw %"std::duration::Duration", ptr %name, i32 0, i32 0
  store i64 %load, ptr %field_ptr, align 4
  %field_ptr1 = getelementptr inbounds nuw %"std::duration::Duration", ptr %name, i32 0, i32 1
  store i64 0, ptr %field_ptr1, align 4
  ret ptr %name
}

define ptr @"std::duration::Duration::from_secs_float"(double %0, ptr %1) {
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
  store ptr @"std::duration::Duration::from_nanos", ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %4, align 8
  %load2 = load double, ptr %2, align 8
  %mulfp = fmul double %load2, 1.000000e+09
  %fp_to_si = fptosi double %mulfp to i64
  store %funcRef %load, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load3 = load ptr, ptr %field_load, align 8
  store %funcRef %load, ptr %6, align 8
  %field_load4 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load5 = load ptr, ptr %field_load4, align 8
  %name = call ptr %load3(i64 %fp_to_si, ptr %load5)
  ret ptr %name
}

define ptr @"std::duration::Duration::from_nanos"(i64 %0, ptr %1) {
prelude:
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %2, align 4
  store ptr %1, ptr %3, align 8
  %load = load i64, ptr %2, align 4
  %divs = sdiv i64 %load, 1000000000
  store i64 %divs, ptr %4, align 4
  %load1 = load i64, ptr %2, align 4
  %rems = srem i64 %load1, 1000000000
  store i64 %rems, ptr %5, align 4
  %load2 = load i64, ptr %4, align 4
  %load3 = load i64, ptr %5, align 4
  %name = call ptr @margarineAlloc(i64 16)
  %field_ptr = getelementptr inbounds nuw %"std::duration::Duration", ptr %name, i32 0, i32 0
  store i64 %load2, ptr %field_ptr, align 4
  %field_ptr4 = getelementptr inbounds nuw %"std::duration::Duration", ptr %name, i32 0, i32 1
  store i64 %load3, ptr %field_ptr4, align 4
  ret ptr %name
}

define ptr @"std::duration::Duration::from_millis"(i64 %0, ptr %1) {
prelude:
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %2, align 4
  store ptr %1, ptr %3, align 8
  %load = load i64, ptr %2, align 4
  %divs = sdiv i64 %load, 1000
  store i64 %divs, ptr %4, align 4
  %load1 = load i64, ptr %2, align 4
  %rems = srem i64 %load1, 1000
  %muli = mul i64 %rems, 1000000
  store i64 %muli, ptr %5, align 4
  %load2 = load i64, ptr %4, align 4
  %load3 = load i64, ptr %5, align 4
  %name = call ptr @margarineAlloc(i64 16)
  %field_ptr = getelementptr inbounds nuw %"std::duration::Duration", ptr %name, i32 0, i32 0
  store i64 %load2, ptr %field_ptr, align 4
  %field_ptr4 = getelementptr inbounds nuw %"std::duration::Duration", ptr %name, i32 0, i32 1
  store i64 %load3, ptr %field_ptr4, align 4
  ret ptr %name
}

define ptr @"std::duration::Duration::from_micros"(i64 %0, ptr %1) {
prelude:
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %2, align 4
  store ptr %1, ptr %3, align 8
  %load = load i64, ptr %2, align 4
  %divs = sdiv i64 %load, 1000000
  store i64 %divs, ptr %4, align 4
  %load1 = load i64, ptr %2, align 4
  %rems = srem i64 %load1, 1000000
  %muli = mul i64 %rems, 1000
  store i64 %muli, ptr %5, align 4
  %load2 = load i64, ptr %4, align 4
  %load3 = load i64, ptr %5, align 4
  %name = call ptr @margarineAlloc(i64 16)
  %field_ptr = getelementptr inbounds nuw %"std::duration::Duration", ptr %name, i32 0, i32 0
  store i64 %load2, ptr %field_ptr, align 4
  %field_ptr4 = getelementptr inbounds nuw %"std::duration::Duration", ptr %name, i32 0, i32 1
  store i64 %load3, ptr %field_ptr4, align 4
  ret ptr %name
}

define %enumRef @"std::duration::Duration::is_zero"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %"std::duration::Duration", align 8
  %5 = alloca i1, align 1
  %6 = alloca %enumRef, align 8
  %7 = alloca %enumRef, align 8
  %8 = alloca %enumRef, align 8
  %9 = alloca %"std::duration::Duration", align 8
  %10 = alloca i1, align 1
  %11 = alloca %enumRef, align 8
  %12 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %load1 = load %"std::duration::Duration", ptr %load, align 4
  store %"std::duration::Duration" %load1, ptr %4, align 4
  %field_load = getelementptr inbounds nuw %"std::duration::Duration", ptr %4, i32 0, i32 0
  %load2 = load i64, ptr %field_load, align 4
  store i1 true, ptr %5, align 1
  %load3 = load i1, ptr %5, align 1
  %icmp = icmp eq i64 %load2, 0
  %and = and i1 %load3, %icmp
  store i1 %and, ptr %5, align 1
  %load4 = load i1, ptr %5, align 1
  %icast = zext i1 %load4 to i32
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 0
  store ptr null, ptr %field_ptr, align 8
  %field_ptr5 = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 1
  store i32 %icast, ptr %field_ptr5, align 4
  %load6 = load %enumRef, ptr %6, align 8
  store %enumRef %load6, ptr %8, align 8
  %field_load7 = getelementptr inbounds nuw %enumRef, ptr %8, i32 0, i32 1
  %load8 = load i32, ptr %field_load7, align 4
  %icast9 = trunc i32 %load8 to i1
  br i1 %icast9, label %then, label %else

then:                                             ; preds = %entry
  %load10 = load ptr, ptr %2, align 8
  %load11 = load %"std::duration::Duration", ptr %load10, align 4
  store %"std::duration::Duration" %load11, ptr %9, align 4
  %field_load12 = getelementptr inbounds nuw %"std::duration::Duration", ptr %9, i32 0, i32 1
  %load13 = load i64, ptr %field_load12, align 4
  store i1 true, ptr %10, align 1
  %load14 = load i1, ptr %10, align 1
  %icmp15 = icmp eq i64 %load13, 0
  %and16 = and i1 %load14, %icmp15
  store i1 %and16, ptr %10, align 1
  %load17 = load i1, ptr %10, align 1
  %icast18 = zext i1 %load17 to i32
  %field_ptr19 = getelementptr inbounds nuw %enumRef, ptr %11, i32 0, i32 0
  store ptr null, ptr %field_ptr19, align 8
  %field_ptr20 = getelementptr inbounds nuw %enumRef, ptr %11, i32 0, i32 1
  store i32 %icast18, ptr %field_ptr20, align 4
  %load21 = load %enumRef, ptr %11, align 8
  store %enumRef %load21, ptr %7, align 8
  br label %cont

else:                                             ; preds = %entry
  %field_ptr22 = getelementptr inbounds nuw %enumRef, ptr %12, i32 0, i32 0
  store ptr null, ptr %field_ptr22, align 8
  %field_ptr23 = getelementptr inbounds nuw %enumRef, ptr %12, i32 0, i32 1
  store i32 0, ptr %field_ptr23, align 4
  %load24 = load %enumRef, ptr %12, align 8
  store %enumRef %load24, ptr %7, align 8
  br label %cont

cont:                                             ; preds = %else, %then
  %load25 = load %enumRef, ptr %7, align 8
  ret %enumRef %load25
}

define i64 @"std::duration::Duration::as_secs"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %"std::duration::Duration", align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %load1 = load %"std::duration::Duration", ptr %load, align 4
  store %"std::duration::Duration" %load1, ptr %4, align 4
  %field_load = getelementptr inbounds nuw %"std::duration::Duration", ptr %4, i32 0, i32 0
  %load2 = load i64, ptr %field_load, align 4
  ret i64 %load2
}

define double @"std::duration::Duration::as_secs_float"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %"std::duration::Duration", align 8
  %5 = alloca %"std::duration::Duration", align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %load1 = load %"std::duration::Duration", ptr %load, align 4
  store %"std::duration::Duration" %load1, ptr %4, align 4
  %field_load = getelementptr inbounds nuw %"std::duration::Duration", ptr %4, i32 0, i32 0
  %load2 = load i64, ptr %field_load, align 4
  %icast = sitofp i64 %load2 to double
  %load3 = load ptr, ptr %2, align 8
  %load4 = load %"std::duration::Duration", ptr %load3, align 4
  store %"std::duration::Duration" %load4, ptr %5, align 4
  %field_load5 = getelementptr inbounds nuw %"std::duration::Duration", ptr %5, i32 0, i32 1
  %load6 = load i64, ptr %field_load5, align 4
  %icast7 = sitofp i64 %load6 to double
  %divfp = fdiv double %icast7, 1.000000e+09
  %addfp = fadd double %icast, %divfp
  ret double %addfp
}

define i64 @"std::duration::Duration::as_millis"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %"std::duration::Duration", align 8
  %5 = alloca %"std::duration::Duration", align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %load1 = load %"std::duration::Duration", ptr %load, align 4
  store %"std::duration::Duration" %load1, ptr %4, align 4
  %field_load = getelementptr inbounds nuw %"std::duration::Duration", ptr %4, i32 0, i32 0
  %load2 = load i64, ptr %field_load, align 4
  %muli = mul i64 %load2, 1000
  %load3 = load ptr, ptr %2, align 8
  %load4 = load %"std::duration::Duration", ptr %load3, align 4
  store %"std::duration::Duration" %load4, ptr %5, align 4
  %field_load5 = getelementptr inbounds nuw %"std::duration::Duration", ptr %5, i32 0, i32 1
  %load6 = load i64, ptr %field_load5, align 4
  %divs = sdiv i64 %load6, 1000000
  %addi = add i64 %muli, %divs
  ret i64 %addi
}

define i64 @"std::duration::Duration::as_micros"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %"std::duration::Duration", align 8
  %5 = alloca %"std::duration::Duration", align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %load1 = load %"std::duration::Duration", ptr %load, align 4
  store %"std::duration::Duration" %load1, ptr %4, align 4
  %field_load = getelementptr inbounds nuw %"std::duration::Duration", ptr %4, i32 0, i32 0
  %load2 = load i64, ptr %field_load, align 4
  %muli = mul i64 %load2, 1000000
  %load3 = load ptr, ptr %2, align 8
  %load4 = load %"std::duration::Duration", ptr %load3, align 4
  store %"std::duration::Duration" %load4, ptr %5, align 4
  %field_load5 = getelementptr inbounds nuw %"std::duration::Duration", ptr %5, i32 0, i32 1
  %load6 = load i64, ptr %field_load5, align 4
  %divs = sdiv i64 %load6, 1000
  %addi = add i64 %muli, %divs
  ret i64 %addi
}

define i64 @"std::duration::Duration::as_nanos"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %"std::duration::Duration", align 8
  %5 = alloca %"std::duration::Duration", align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %load1 = load %"std::duration::Duration", ptr %load, align 4
  store %"std::duration::Duration" %load1, ptr %4, align 4
  %field_load = getelementptr inbounds nuw %"std::duration::Duration", ptr %4, i32 0, i32 0
  %load2 = load i64, ptr %field_load, align 4
  %muli = mul i64 %load2, 1000000000
  %load3 = load ptr, ptr %2, align 8
  %load4 = load %"std::duration::Duration", ptr %load3, align 4
  store %"std::duration::Duration" %load4, ptr %5, align 4
  %field_load5 = getelementptr inbounds nuw %"std::duration::Duration", ptr %5, i32 0, i32 1
  %load6 = load i64, ptr %field_load5, align 4
  %addi = add i64 %muli, %load6
  ret i64 %addi
}

define i64 @"std::duration::Duration::subsec_millis"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %"std::duration::Duration", align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %load1 = load %"std::duration::Duration", ptr %load, align 4
  store %"std::duration::Duration" %load1, ptr %4, align 4
  %field_load = getelementptr inbounds nuw %"std::duration::Duration", ptr %4, i32 0, i32 1
  %load2 = load i64, ptr %field_load, align 4
  %divs = sdiv i64 %load2, 1000000
  ret i64 %divs
}

define i64 @"std::duration::Duration::subsec_micros"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %"std::duration::Duration", align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %load1 = load %"std::duration::Duration", ptr %load, align 4
  store %"std::duration::Duration" %load1, ptr %4, align 4
  %field_load = getelementptr inbounds nuw %"std::duration::Duration", ptr %4, i32 0, i32 1
  %load2 = load i64, ptr %field_load, align 4
  %divs = sdiv i64 %load2, 1000
  ret i64 %divs
}

define i64 @"std::duration::Duration::subsec_nanos"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %"std::duration::Duration", align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %load1 = load %"std::duration::Duration", ptr %load, align 4
  store %"std::duration::Duration" %load1, ptr %4, align 4
  %field_load = getelementptr inbounds nuw %"std::duration::Duration", ptr %4, i32 0, i32 1
  %load2 = load i64, ptr %field_load, align 4
  ret i64 %load2
}

define ptr @"std::duration::Duration::add"(ptr %0, ptr %1, ptr %2) {
prelude:
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %"std::duration::Duration", align 8
  %7 = alloca %"std::duration::Duration", align 8
  %8 = alloca i64, align 8
  %9 = alloca %"std::duration::Duration", align 8
  %10 = alloca %"std::duration::Duration", align 8
  %11 = alloca i64, align 8
  %12 = alloca %enumRef, align 8
  %13 = alloca {}, align 8
  %14 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  store ptr %2, ptr %5, align 8
  %load = load ptr, ptr %3, align 8
  %load1 = load %"std::duration::Duration", ptr %load, align 4
  store %"std::duration::Duration" %load1, ptr %6, align 4
  %field_load = getelementptr inbounds nuw %"std::duration::Duration", ptr %6, i32 0, i32 1
  %load2 = load i64, ptr %field_load, align 4
  %load3 = load ptr, ptr %4, align 8
  %load4 = load %"std::duration::Duration", ptr %load3, align 4
  store %"std::duration::Duration" %load4, ptr %7, align 4
  %field_load5 = getelementptr inbounds nuw %"std::duration::Duration", ptr %7, i32 0, i32 1
  %load6 = load i64, ptr %field_load5, align 4
  %addi = add i64 %load2, %load6
  store i64 %addi, ptr %8, align 4
  %load7 = load ptr, ptr %3, align 8
  %load8 = load %"std::duration::Duration", ptr %load7, align 4
  store %"std::duration::Duration" %load8, ptr %9, align 4
  %field_load9 = getelementptr inbounds nuw %"std::duration::Duration", ptr %9, i32 0, i32 0
  %load10 = load i64, ptr %field_load9, align 4
  %load11 = load ptr, ptr %4, align 8
  %load12 = load %"std::duration::Duration", ptr %load11, align 4
  store %"std::duration::Duration" %load12, ptr %10, align 4
  %field_load13 = getelementptr inbounds nuw %"std::duration::Duration", ptr %10, i32 0, i32 0
  %load14 = load i64, ptr %field_load13, align 4
  %addi15 = add i64 %load10, %load14
  store i64 %addi15, ptr %11, align 4
  %load16 = load i64, ptr %8, align 4
  %icmp = icmp sge i64 %load16, 1000000000
  %icast = zext i1 %icmp to i32
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %12, i32 0, i32 0
  store ptr null, ptr %field_ptr, align 8
  %field_ptr17 = getelementptr inbounds nuw %enumRef, ptr %12, i32 0, i32 1
  store i32 %icast, ptr %field_ptr17, align 4
  %load18 = load %enumRef, ptr %12, align 8
  store %enumRef %load18, ptr %14, align 8
  %field_load19 = getelementptr inbounds nuw %enumRef, ptr %14, i32 0, i32 1
  %load20 = load i32, ptr %field_load19, align 4
  %icast21 = trunc i32 %load20 to i1
  br i1 %icast21, label %then, label %else

then:                                             ; preds = %entry
  %load22 = load i64, ptr %11, align 4
  %addi23 = add i64 %load22, 1
  store i64 %addi23, ptr %11, align 4
  %load24 = load i64, ptr %8, align 4
  %subi = sub i64 %load24, 1000000000
  store i64 %subi, ptr %8, align 4
  store {} zeroinitializer, ptr %13, align 1
  br label %cont

else:                                             ; preds = %entry
  br label %cont

cont:                                             ; preds = %else, %then
  %load25 = load {}, ptr %13, align 1
  %load26 = load i64, ptr %11, align 4
  %load27 = load i64, ptr %8, align 4
  %name = call ptr @margarineAlloc(i64 16)
  %field_ptr28 = getelementptr inbounds nuw %"std::duration::Duration", ptr %name, i32 0, i32 0
  store i64 %load26, ptr %field_ptr28, align 4
  %field_ptr29 = getelementptr inbounds nuw %"std::duration::Duration", ptr %name, i32 0, i32 1
  store i64 %load27, ptr %field_ptr29, align 4
  ret ptr %name
}

define ptr @"str::lines"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca ptr, align 8
  %8 = alloca %captures.1, align 8
  %9 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @str_lines_iter, ptr %field_ptr, align 8
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
  %name = call ptr %load3(ptr %load2, ptr %load5)
  store ptr %name, ptr %7, align 8
  %load6 = load ptr, ptr %7, align 8
  %field_ptr7 = getelementptr inbounds nuw %captures.1, ptr %8, i32 0, i32 0
  store ptr %load6, ptr %field_ptr7, align 8
  %load8 = load %captures.1, ptr %8, align 8
  %name9 = call ptr @margarineAlloc(i64 8)
  store %captures.1 %load8, ptr %name9, align 8
  %field_ptr10 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 0
  store ptr @"<closure>.3", ptr %field_ptr10, align 8
  %field_ptr11 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 1
  store ptr %name9, ptr %field_ptr11, align 8
  %load12 = load %funcRef, ptr %9, align 8
  %name13 = call ptr @margarineAlloc(i64 16)
  %field_ptr14 = getelementptr inbounds nuw %"std::iter::Iter<str>", ptr %name13, i32 0, i32 0
  store %funcRef %load12, ptr %field_ptr14, align 8
  ret ptr %name13
}

declare ptr @str_lines_iter(ptr, ptr)

define %enumRef @"<closure>.3"(ptr %0) {
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
  store ptr @"std::string::Lines::__next__", ptr %field_ptr, align 8
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

define %enumRef @"std::string::Lines::__next__"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @str_lines_iter_next, ptr %field_ptr, align 8
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
  %name = call %enumRef %load3(ptr %load2, ptr %load5)
  ret %enumRef %name
}

declare %enumRef @str_lines_iter_next(ptr, ptr)

define ptr @"str::split_at"(ptr %0, i64 %1, ptr %2) {
prelude:
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %3, align 8
  store i64 %1, ptr %4, align 4
  store ptr %2, ptr %5, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  store ptr @str_split_at, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %6, align 8
  %load2 = load ptr, ptr %3, align 8
  %load3 = load i64, ptr %4, align 4
  store %funcRef %load, ptr %7, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  %load4 = load ptr, ptr %field_load, align 8
  store %funcRef %load, ptr %8, align 8
  %field_load5 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 1
  %load6 = load ptr, ptr %field_load5, align 8
  %name = call ptr %load4(ptr %load2, i64 %load3, ptr %load6)
  ret ptr %name
}

declare ptr @str_split_at(ptr, i64, ptr)

define %enumRef @"str::is_empty"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca i1, align 1
  %8 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @"str::len", ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load2 = load %funcRef, ptr %4, align 8
  store %funcRef %load2, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load3 = load ptr, ptr %field_load, align 8
  store %funcRef %load2, ptr %6, align 8
  %field_load4 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load5 = load ptr, ptr %field_load4, align 8
  %name = call i64 %load3(ptr %load, ptr %load5)
  store i1 true, ptr %7, align 1
  %load6 = load i1, ptr %7, align 1
  %icmp = icmp eq i64 %name, 0
  %and = and i1 %load6, %icmp
  store i1 %and, ptr %7, align 1
  %load7 = load i1, ptr %7, align 1
  %icast = zext i1 %load7 to i32
  %field_ptr8 = getelementptr inbounds nuw %enumRef, ptr %8, i32 0, i32 0
  store ptr null, ptr %field_ptr8, align 8
  %field_ptr9 = getelementptr inbounds nuw %enumRef, ptr %8, i32 0, i32 1
  store i32 %icast, ptr %field_ptr9, align 4
  %load10 = load %enumRef, ptr %8, align 8
  ret %enumRef %load10
}

define i64 @"str::len"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @str_len, ptr %field_ptr, align 8
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
  %name = call i64 %load3(ptr %load2, ptr %load5)
  ret i64 %name
}

declare i64 @str_len(ptr, ptr)

define ptr @"str::split"(ptr %0, ptr %1, ptr %2) {
prelude:
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca ptr, align 8
  %10 = alloca %captures.3, align 8
  %11 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  store ptr %2, ptr %5, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  store ptr @Option.4, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %6, align 8
  %load2 = load ptr, ptr %3, align 8
  store %funcRef %load, ptr %7, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  %load3 = load ptr, ptr %field_load, align 8
  store %funcRef %load, ptr %8, align 8
  %field_load4 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 1
  %load5 = load ptr, ptr %field_load4, align 8
  %name = call %enumRef %load3(ptr %load2, ptr %load5)
  %load6 = load ptr, ptr %4, align 8
  %name7 = call ptr @margarineAlloc(i64 24)
  %field_ptr8 = getelementptr inbounds nuw %"std::string::SplitState", ptr %name7, i32 0, i32 0
  store %enumRef %name, ptr %field_ptr8, align 8
  %field_ptr9 = getelementptr inbounds nuw %"std::string::SplitState", ptr %name7, i32 0, i32 1
  store ptr %load6, ptr %field_ptr9, align 8
  store ptr %name7, ptr %9, align 8
  %load10 = load ptr, ptr %9, align 8
  %load11 = load ptr, ptr %4, align 8
  %field_ptr12 = getelementptr inbounds nuw %captures.3, ptr %10, i32 0, i32 0
  store ptr %load10, ptr %field_ptr12, align 8
  %field_ptr13 = getelementptr inbounds nuw %captures.3, ptr %10, i32 0, i32 1
  store ptr %load11, ptr %field_ptr13, align 8
  %load14 = load %captures.3, ptr %10, align 8
  %name15 = call ptr @margarineAlloc(i64 16)
  store %captures.3 %load14, ptr %name15, align 8
  %field_ptr16 = getelementptr inbounds nuw %funcRef, ptr %11, i32 0, i32 0
  store ptr @"<closure>.5", ptr %field_ptr16, align 8
  %field_ptr17 = getelementptr inbounds nuw %funcRef, ptr %11, i32 0, i32 1
  store ptr %name15, ptr %field_ptr17, align 8
  %load18 = load %funcRef, ptr %11, align 8
  %name19 = call ptr @margarineAlloc(i64 16)
  %field_ptr20 = getelementptr inbounds nuw %"std::iter::Iter<str>", ptr %name19, i32 0, i32 0
  store %funcRef %load18, ptr %field_ptr20, align 8
  ret ptr %name19
}

define %enumRef @Option.4(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %name = call ptr @margarineAlloc(i64 8)
  store ptr %load, ptr %name, align 8
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %4, i32 0, i32 0
  store ptr %name, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %enumRef, ptr %4, i32 0, i32 1
  store i32 0, ptr %field_ptr1, align 4
  %load2 = load %enumRef, ptr %4, align 8
  ret %enumRef %load2
}

define %enumRef @"<closure>.5"(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %captures.3, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %captures.3, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %"std::string::SplitState", align 8
  %7 = alloca %enumRef, align 8
  %8 = alloca %enumRef, align 8
  %9 = alloca ptr, align 8
  %10 = alloca %enumRef, align 8
  %11 = alloca %"std::string::SplitState", align 8
  %12 = alloca %enumRef, align 8
  %13 = alloca %enumRef, align 8
  %14 = alloca %funcRef, align 8
  %15 = alloca %funcRef, align 8
  %16 = alloca %funcRef, align 8
  %17 = alloca %enumRef, align 8
  %18 = alloca %funcRef, align 8
  %19 = alloca %funcRef, align 8
  %20 = alloca %funcRef, align 8
  %21 = alloca %enumRef, align 8
  %22 = alloca %enumRef, align 8
  %23 = alloca %enumRef, align 8
  %24 = alloca %enumRef, align 8
  %25 = alloca ptr, align 8
  %26 = alloca %funcRef, align 8
  %27 = alloca %"(str, str).4", align 8
  %28 = alloca %funcRef, align 8
  %29 = alloca %funcRef, align 8
  %30 = alloca %funcRef, align 8
  %31 = alloca %"(str, str).4", align 8
  %32 = alloca %funcRef, align 8
  %33 = alloca %funcRef, align 8
  %34 = alloca %funcRef, align 8
  %35 = alloca %funcRef, align 8
  %36 = alloca %funcRef, align 8
  %37 = alloca %funcRef, align 8
  %38 = alloca %funcRef, align 8
  %39 = alloca %funcRef, align 8
  %40 = alloca {}, align 8
  %41 = alloca %enumRef, align 8
  %42 = alloca %funcRef, align 8
  %43 = alloca %funcRef, align 8
  %44 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %load = load ptr, ptr %1, align 8
  %load1 = load %captures.3, ptr %load, align 8
  store %captures.3 %load1, ptr %2, align 8
  %field_load = getelementptr inbounds nuw %captures.3, ptr %2, i32 0, i32 0
  %load2 = load ptr, ptr %field_load, align 8
  store ptr %load2, ptr %3, align 8
  store %captures.3 %load1, ptr %4, align 8
  %field_load3 = getelementptr inbounds nuw %captures.3, ptr %4, i32 0, i32 1
  %load4 = load ptr, ptr %field_load3, align 8
  store ptr %load4, ptr %5, align 8
  %load5 = load ptr, ptr %3, align 8
  %load6 = load %"std::string::SplitState", ptr %load5, align 8
  store %"std::string::SplitState" %load6, ptr %6, align 8
  %field_load7 = getelementptr inbounds nuw %"std::string::SplitState", ptr %6, i32 0, i32 0
  %load8 = load %enumRef, ptr %field_load7, align 8
  store %enumRef %load8, ptr %7, align 8
  %field_load9 = getelementptr inbounds nuw %enumRef, ptr %7, i32 0, i32 1
  %load10 = load i32, ptr %field_load9, align 4
  switch i32 %load10, label %switch_end [
    i32 0, label %switch_br
    i32 1, label %switch_br100
  ]

switch_end:                                       ; preds = %switch_br100, %cont43, %entry
  %load112 = load %enumRef, ptr %8, align 8
  ret %enumRef %load112

switch_br:                                        ; preds = %entry
  store %enumRef %load8, ptr %10, align 8
  %field_load11 = getelementptr inbounds nuw %enumRef, ptr %10, i32 0, i32 0
  %load12 = load ptr, ptr %field_load11, align 8
  %load13 = load ptr, ptr %load12, align 8
  store ptr %load13, ptr %9, align 8
  %load14 = load ptr, ptr %3, align 8
  %load15 = load %"std::string::SplitState", ptr %load14, align 8
  store %"std::string::SplitState" %load15, ptr %11, align 8
  %field_load16 = getelementptr inbounds nuw %"std::string::SplitState", ptr %11, i32 0, i32 0
  %load17 = load %enumRef, ptr %field_load16, align 8
  store %enumRef %load17, ptr %12, align 8
  %field_load18 = getelementptr inbounds nuw %enumRef, ptr %12, i32 0, i32 1
  %load19 = load i32, ptr %field_load18, align 4
  store %enumRef %load17, ptr %13, align 8
  %field_load20 = getelementptr inbounds nuw %enumRef, ptr %13, i32 0, i32 0
  %load21 = load ptr, ptr %field_load20, align 8
  %icmp = icmp eq i32 %load19, 0
  br i1 %icmp, label %then, label %else

then:                                             ; preds = %switch_br
  br label %cont

else:                                             ; preds = %switch_br
  ret %enumRef %load17

cont:                                             ; preds = %45, %then
  %load22 = load ptr, ptr %load21, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 0
  store ptr @"str::split_once", ptr %field_ptr, align 8
  %field_ptr23 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 1
  store ptr null, ptr %field_ptr23, align 8
  %load24 = load %funcRef, ptr %14, align 8
  %load25 = load ptr, ptr %5, align 8
  store %funcRef %load24, ptr %15, align 8
  %field_load26 = getelementptr inbounds nuw %funcRef, ptr %15, i32 0, i32 0
  %load27 = load ptr, ptr %field_load26, align 8
  store %funcRef %load24, ptr %16, align 8
  %field_load28 = getelementptr inbounds nuw %funcRef, ptr %16, i32 0, i32 1
  %load29 = load ptr, ptr %field_load28, align 8
  %name = call %enumRef %load27(ptr %load22, ptr %load25, ptr %load29)
  store %enumRef %name, ptr %17, align 8
  %load30 = load %enumRef, ptr %17, align 8
  %field_ptr31 = getelementptr inbounds nuw %funcRef, ptr %18, i32 0, i32 0
  store ptr @"Option::is_some", ptr %field_ptr31, align 8
  %field_ptr32 = getelementptr inbounds nuw %funcRef, ptr %18, i32 0, i32 1
  store ptr null, ptr %field_ptr32, align 8
  %load33 = load %funcRef, ptr %18, align 8
  store %funcRef %load33, ptr %19, align 8
  %field_load34 = getelementptr inbounds nuw %funcRef, ptr %19, i32 0, i32 0
  %load35 = load ptr, ptr %field_load34, align 8
  store %funcRef %load33, ptr %20, align 8
  %field_load36 = getelementptr inbounds nuw %funcRef, ptr %20, i32 0, i32 1
  %load37 = load ptr, ptr %field_load36, align 8
  %name38 = call %enumRef %load35(%enumRef %load30, ptr %load37)
  store %enumRef %name38, ptr %22, align 8
  %field_load39 = getelementptr inbounds nuw %enumRef, ptr %22, i32 0, i32 1
  %load40 = load i32, ptr %field_load39, align 4
  %icast = trunc i32 %load40 to i1
  br i1 %icast, label %then41, label %else42

45:                                               ; No predecessors!
  br label %cont

then41:                                           ; preds = %cont
  %load44 = load %enumRef, ptr %17, align 8
  store %enumRef %load44, ptr %23, align 8
  %field_load45 = getelementptr inbounds nuw %enumRef, ptr %23, i32 0, i32 1
  %load46 = load i32, ptr %field_load45, align 4
  %icmp47 = icmp eq i32 %load46, 0
  br i1 %icmp47, label %then48, label %else49

else42:                                           ; preds = %cont
  %field_ptr80 = getelementptr inbounds nuw %funcRef, ptr %34, i32 0, i32 0
  store ptr @Option.6, ptr %field_ptr80, align 8
  %field_ptr81 = getelementptr inbounds nuw %funcRef, ptr %34, i32 0, i32 1
  store ptr null, ptr %field_ptr81, align 8
  %load82 = load %funcRef, ptr %34, align 8
  store %funcRef %load82, ptr %35, align 8
  %field_load83 = getelementptr inbounds nuw %funcRef, ptr %35, i32 0, i32 0
  %load84 = load ptr, ptr %field_load83, align 8
  store %funcRef %load82, ptr %36, align 8
  %field_load85 = getelementptr inbounds nuw %funcRef, ptr %36, i32 0, i32 1
  %load86 = load ptr, ptr %field_load85, align 8
  %name87 = call %enumRef %load84(ptr %load86)
  %load88 = load ptr, ptr %3, align 8
  %field_ptr89 = getelementptr inbounds nuw %"std::string::SplitState", ptr %load88, i32 0, i32 0
  store %enumRef %name87, ptr %field_ptr89, align 8
  %field_ptr90 = getelementptr inbounds nuw %funcRef, ptr %37, i32 0, i32 0
  store ptr @Option.4, ptr %field_ptr90, align 8
  %field_ptr91 = getelementptr inbounds nuw %funcRef, ptr %37, i32 0, i32 1
  store ptr null, ptr %field_ptr91, align 8
  %load92 = load %funcRef, ptr %37, align 8
  %load93 = load ptr, ptr %9, align 8
  store %funcRef %load92, ptr %38, align 8
  %field_load94 = getelementptr inbounds nuw %funcRef, ptr %38, i32 0, i32 0
  %load95 = load ptr, ptr %field_load94, align 8
  store %funcRef %load92, ptr %39, align 8
  %field_load96 = getelementptr inbounds nuw %funcRef, ptr %39, i32 0, i32 1
  %load97 = load ptr, ptr %field_load96, align 8
  %name98 = call %enumRef %load95(ptr %load93, ptr %load97)
  store %enumRef %name98, ptr %21, align 8
  br label %cont43

cont43:                                           ; preds = %else42, %cont50
  %load99 = load %enumRef, ptr %21, align 8
  store %enumRef %load99, ptr %8, align 8
  br label %switch_end

then48:                                           ; preds = %then41
  br label %cont50

else49:                                           ; preds = %then41
  call void @margarineAbort()
  br label %cont50

cont50:                                           ; preds = %else49, %then48
  store %enumRef %load44, ptr %24, align 8
  %field_load51 = getelementptr inbounds nuw %enumRef, ptr %24, i32 0, i32 0
  %load52 = load ptr, ptr %field_load51, align 8
  %load53 = load ptr, ptr %load52, align 8
  store ptr %load53, ptr %25, align 8
  %field_ptr54 = getelementptr inbounds nuw %funcRef, ptr %26, i32 0, i32 0
  store ptr @Option.4, ptr %field_ptr54, align 8
  %field_ptr55 = getelementptr inbounds nuw %funcRef, ptr %26, i32 0, i32 1
  store ptr null, ptr %field_ptr55, align 8
  %load56 = load %funcRef, ptr %26, align 8
  %load57 = load ptr, ptr %25, align 8
  %load58 = load %"(str, str).4", ptr %load57, align 8
  store %"(str, str).4" %load58, ptr %27, align 8
  %field_load59 = getelementptr inbounds nuw %"(str, str).4", ptr %27, i32 0, i32 1
  %load60 = load ptr, ptr %field_load59, align 8
  store %funcRef %load56, ptr %28, align 8
  %field_load61 = getelementptr inbounds nuw %funcRef, ptr %28, i32 0, i32 0
  %load62 = load ptr, ptr %field_load61, align 8
  store %funcRef %load56, ptr %29, align 8
  %field_load63 = getelementptr inbounds nuw %funcRef, ptr %29, i32 0, i32 1
  %load64 = load ptr, ptr %field_load63, align 8
  %name65 = call %enumRef %load62(ptr %load60, ptr %load64)
  %load66 = load ptr, ptr %3, align 8
  %field_ptr67 = getelementptr inbounds nuw %"std::string::SplitState", ptr %load66, i32 0, i32 0
  store %enumRef %name65, ptr %field_ptr67, align 8
  %field_ptr68 = getelementptr inbounds nuw %funcRef, ptr %30, i32 0, i32 0
  store ptr @Option.4, ptr %field_ptr68, align 8
  %field_ptr69 = getelementptr inbounds nuw %funcRef, ptr %30, i32 0, i32 1
  store ptr null, ptr %field_ptr69, align 8
  %load70 = load %funcRef, ptr %30, align 8
  %load71 = load ptr, ptr %25, align 8
  %load72 = load %"(str, str).4", ptr %load71, align 8
  store %"(str, str).4" %load72, ptr %31, align 8
  %field_load73 = getelementptr inbounds nuw %"(str, str).4", ptr %31, i32 0, i32 0
  %load74 = load ptr, ptr %field_load73, align 8
  store %funcRef %load70, ptr %32, align 8
  %field_load75 = getelementptr inbounds nuw %funcRef, ptr %32, i32 0, i32 0
  %load76 = load ptr, ptr %field_load75, align 8
  store %funcRef %load70, ptr %33, align 8
  %field_load77 = getelementptr inbounds nuw %funcRef, ptr %33, i32 0, i32 1
  %load78 = load ptr, ptr %field_load77, align 8
  %name79 = call %enumRef %load76(ptr %load74, ptr %load78)
  store %enumRef %name79, ptr %21, align 8
  br label %cont43

switch_br100:                                     ; preds = %entry
  store %enumRef %load8, ptr %41, align 8
  %field_load101 = getelementptr inbounds nuw %enumRef, ptr %41, i32 0, i32 0
  %load102 = load ptr, ptr %field_load101, align 8
  %load103 = load {}, ptr %load102, align 1
  store {} %load103, ptr %40, align 1
  %field_ptr104 = getelementptr inbounds nuw %funcRef, ptr %42, i32 0, i32 0
  store ptr @Option.6, ptr %field_ptr104, align 8
  %field_ptr105 = getelementptr inbounds nuw %funcRef, ptr %42, i32 0, i32 1
  store ptr null, ptr %field_ptr105, align 8
  %load106 = load %funcRef, ptr %42, align 8
  store %funcRef %load106, ptr %43, align 8
  %field_load107 = getelementptr inbounds nuw %funcRef, ptr %43, i32 0, i32 0
  %load108 = load ptr, ptr %field_load107, align 8
  store %funcRef %load106, ptr %44, align 8
  %field_load109 = getelementptr inbounds nuw %funcRef, ptr %44, i32 0, i32 1
  %load110 = load ptr, ptr %field_load109, align 8
  %name111 = call %enumRef %load108(ptr %load110)
  store %enumRef %name111, ptr %8, align 8
  br label %switch_end
}

define %enumRef @"str::split_once"(ptr %0, ptr %1, ptr %2) {
prelude:
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  store ptr %2, ptr %5, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  store ptr @str_split_once, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %6, align 8
  %load2 = load ptr, ptr %3, align 8
  %load3 = load ptr, ptr %4, align 8
  store %funcRef %load, ptr %7, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  %load4 = load ptr, ptr %field_load, align 8
  store %funcRef %load, ptr %8, align 8
  %field_load5 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 1
  %load6 = load ptr, ptr %field_load5, align 8
  %name = call %enumRef %load4(ptr %load2, ptr %load3, ptr %load6)
  ret %enumRef %name
}

declare %enumRef @str_split_once(ptr, ptr, ptr)

define %enumRef @"Option::is_some"(%enumRef %0, ptr %1) {
prelude:
  %2 = alloca %enumRef, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %enumRef, align 8
  %5 = alloca %enumRef, align 8
  %6 = alloca ptr, align 8
  %7 = alloca %enumRef, align 8
  %8 = alloca %enumRef, align 8
  %9 = alloca {}, align 8
  %10 = alloca %enumRef, align 8
  %11 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store %enumRef %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load %enumRef, ptr %2, align 8
  store %enumRef %load, ptr %4, align 8
  %field_load = getelementptr inbounds nuw %enumRef, ptr %4, i32 0, i32 1
  %load1 = load i32, ptr %field_load, align 4
  switch i32 %load1, label %switch_end [
    i32 0, label %switch_br
    i32 1, label %switch_br7
  ]

switch_end:                                       ; preds = %switch_br7, %switch_br, %entry
  %load14 = load %enumRef, ptr %5, align 8
  ret %enumRef %load14

switch_br:                                        ; preds = %entry
  store %enumRef %load, ptr %7, align 8
  %field_load2 = getelementptr inbounds nuw %enumRef, ptr %7, i32 0, i32 0
  %load3 = load ptr, ptr %field_load2, align 8
  %load4 = load ptr, ptr %load3, align 8
  store ptr %load4, ptr %6, align 8
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %8, i32 0, i32 0
  store ptr null, ptr %field_ptr, align 8
  %field_ptr5 = getelementptr inbounds nuw %enumRef, ptr %8, i32 0, i32 1
  store i32 1, ptr %field_ptr5, align 4
  %load6 = load %enumRef, ptr %8, align 8
  store %enumRef %load6, ptr %5, align 8
  br label %switch_end

switch_br7:                                       ; preds = %entry
  store %enumRef %load, ptr %10, align 8
  %field_load8 = getelementptr inbounds nuw %enumRef, ptr %10, i32 0, i32 0
  %load9 = load ptr, ptr %field_load8, align 8
  %load10 = load {}, ptr %load9, align 1
  store {} %load10, ptr %9, align 1
  %field_ptr11 = getelementptr inbounds nuw %enumRef, ptr %11, i32 0, i32 0
  store ptr null, ptr %field_ptr11, align 8
  %field_ptr12 = getelementptr inbounds nuw %enumRef, ptr %11, i32 0, i32 1
  store i32 0, ptr %field_ptr12, align 4
  %load13 = load %enumRef, ptr %11, align 8
  store %enumRef %load13, ptr %5, align 8
  br label %switch_end
}

define %enumRef @Option.6(ptr %0) {
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

define ptr @"str::slice"(ptr %0, ptr %1, ptr %2) {
prelude:
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %Range, align 8
  %8 = alloca %Range, align 8
  %9 = alloca %funcRef, align 8
  %10 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  store ptr %2, ptr %5, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  store ptr @str_slice, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %6, align 8
  %load2 = load ptr, ptr %3, align 8
  %load3 = load ptr, ptr %4, align 8
  %load4 = load %Range, ptr %load3, align 4
  store %Range %load4, ptr %7, align 4
  %field_load = getelementptr inbounds nuw %Range, ptr %7, i32 0, i32 0
  %load5 = load i64, ptr %field_load, align 4
  %load6 = load ptr, ptr %4, align 8
  %load7 = load %Range, ptr %load6, align 4
  store %Range %load7, ptr %8, align 4
  %field_load8 = getelementptr inbounds nuw %Range, ptr %8, i32 0, i32 1
  %load9 = load i64, ptr %field_load8, align 4
  store %funcRef %load, ptr %9, align 8
  %field_load10 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 0
  %load11 = load ptr, ptr %field_load10, align 8
  store %funcRef %load, ptr %10, align 8
  %field_load12 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 1
  %load13 = load ptr, ptr %field_load12, align 8
  %name = call ptr %load11(ptr %load2, i64 %load5, i64 %load9, ptr %load13)
  ret ptr %name
}

declare ptr @str_slice(ptr, i64, i64, ptr)

define ptr @"str::nth"(ptr %0, i64 %1, ptr %2) {
prelude:
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %3, align 8
  store i64 %1, ptr %4, align 4
  store ptr %2, ptr %5, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  store ptr @str_nth, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %6, align 8
  %load2 = load ptr, ptr %3, align 8
  %load3 = load i64, ptr %4, align 4
  store %funcRef %load, ptr %7, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  %load4 = load ptr, ptr %field_load, align 8
  store %funcRef %load, ptr %8, align 8
  %field_load5 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 1
  %load6 = load ptr, ptr %field_load5, align 8
  %name = call ptr %load4(ptr %load2, i64 %load3, ptr %load6)
  ret ptr %name
}

declare ptr @str_nth(ptr, i64, ptr)

define ptr @"str::chars"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca %captures.5, align 8
  %6 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %name = call ptr @margarineAlloc(i64 8)
  %field_ptr = getelementptr inbounds nuw %"std::string::Chars", ptr %name, i32 0, i32 0
  store ptr %load, ptr %field_ptr, align 8
  store ptr %name, ptr %4, align 8
  %load1 = load ptr, ptr %4, align 8
  %field_ptr2 = getelementptr inbounds nuw %captures.5, ptr %5, i32 0, i32 0
  store ptr %load1, ptr %field_ptr2, align 8
  %load3 = load %captures.5, ptr %5, align 8
  %name4 = call ptr @margarineAlloc(i64 8)
  store %captures.5 %load3, ptr %name4, align 8
  %field_ptr5 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  store ptr @"<closure>.7", ptr %field_ptr5, align 8
  %field_ptr6 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  store ptr %name4, ptr %field_ptr6, align 8
  %load7 = load %funcRef, ptr %6, align 8
  %name8 = call ptr @margarineAlloc(i64 16)
  %field_ptr9 = getelementptr inbounds nuw %"std::iter::Iter<str>", ptr %name8, i32 0, i32 0
  store %funcRef %load7, ptr %field_ptr9, align 8
  ret ptr %name8
}

define %enumRef @"<closure>.7"(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %captures.5, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %"std::string::Chars", align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca {}, align 8
  %9 = alloca %enumRef, align 8
  %10 = alloca %funcRef, align 8
  %11 = alloca %funcRef, align 8
  %12 = alloca %funcRef, align 8
  %13 = alloca %"std::string::Chars", align 8
  %14 = alloca %funcRef, align 8
  %15 = alloca %funcRef, align 8
  %16 = alloca %funcRef, align 8
  %17 = alloca i1, align 1
  %18 = alloca %enumRef, align 8
  %19 = alloca {}, align 8
  %20 = alloca %enumRef, align 8
  %21 = alloca %"std::string::Chars", align 8
  %22 = alloca ptr, align 8
  %23 = alloca %funcRef, align 8
  %24 = alloca %funcRef, align 8
  %25 = alloca %funcRef, align 8
  %26 = alloca %"std::string::Chars", align 8
  %27 = alloca %funcRef, align 8
  %28 = alloca %funcRef, align 8
  %29 = alloca %funcRef, align 8
  %30 = alloca %"(str, str)", align 8
  %31 = alloca ptr, align 8
  %32 = alloca %"(str, str)", align 8
  %33 = alloca ptr, align 8
  %34 = alloca %funcRef, align 8
  %35 = alloca %funcRef, align 8
  %36 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %load = load ptr, ptr %1, align 8
  %load1 = load %captures.5, ptr %load, align 8
  store %captures.5 %load1, ptr %2, align 8
  %field_load = getelementptr inbounds nuw %captures.5, ptr %2, i32 0, i32 0
  %load2 = load ptr, ptr %field_load, align 8
  store ptr %load2, ptr %3, align 8
  %load3 = load ptr, ptr %3, align 8
  %load4 = load %"std::string::Chars", ptr %load3, align 8
  store %"std::string::Chars" %load4, ptr %4, align 8
  %field_load5 = getelementptr inbounds nuw %"std::string::Chars", ptr %4, i32 0, i32 0
  %load6 = load ptr, ptr %field_load5, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  store ptr @"str::is_empty", ptr %field_ptr, align 8
  %field_ptr7 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 1
  store ptr null, ptr %field_ptr7, align 8
  %load8 = load %funcRef, ptr %5, align 8
  store %funcRef %load8, ptr %6, align 8
  %field_load9 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  %load10 = load ptr, ptr %field_load9, align 8
  store %funcRef %load8, ptr %7, align 8
  %field_load11 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  %load12 = load ptr, ptr %field_load11, align 8
  %name = call %enumRef %load10(ptr %load6, ptr %load12)
  store %enumRef %name, ptr %9, align 8
  %field_load13 = getelementptr inbounds nuw %enumRef, ptr %9, i32 0, i32 1
  %load14 = load i32, ptr %field_load13, align 4
  %icast = trunc i32 %load14 to i1
  br i1 %icast, label %then, label %else

then:                                             ; preds = %entry
  %field_ptr15 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 0
  store ptr @Option.6, ptr %field_ptr15, align 8
  %field_ptr16 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 1
  store ptr null, ptr %field_ptr16, align 8
  %load17 = load %funcRef, ptr %10, align 8
  store %funcRef %load17, ptr %11, align 8
  %field_load18 = getelementptr inbounds nuw %funcRef, ptr %11, i32 0, i32 0
  %load19 = load ptr, ptr %field_load18, align 8
  store %funcRef %load17, ptr %12, align 8
  %field_load20 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 1
  %load21 = load ptr, ptr %field_load20, align 8
  %name22 = call %enumRef %load19(ptr %load21)
  ret %enumRef %name22

else:                                             ; preds = %entry
  br label %cont

cont:                                             ; preds = %else, %38
  %load23 = load {}, ptr %8, align 1
  %load24 = load ptr, ptr %3, align 8
  %load25 = load %"std::string::Chars", ptr %load24, align 8
  store %"std::string::Chars" %load25, ptr %13, align 8
  %field_load26 = getelementptr inbounds nuw %"std::string::Chars", ptr %13, i32 0, i32 0
  %load27 = load ptr, ptr %field_load26, align 8
  %field_ptr28 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 0
  store ptr @"str::len", ptr %field_ptr28, align 8
  %field_ptr29 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 1
  store ptr null, ptr %field_ptr29, align 8
  %load30 = load %funcRef, ptr %14, align 8
  store %funcRef %load30, ptr %15, align 8
  %field_load31 = getelementptr inbounds nuw %funcRef, ptr %15, i32 0, i32 0
  %load32 = load ptr, ptr %field_load31, align 8
  store %funcRef %load30, ptr %16, align 8
  %field_load33 = getelementptr inbounds nuw %funcRef, ptr %16, i32 0, i32 1
  %load34 = load ptr, ptr %field_load33, align 8
  %name35 = call i64 %load32(ptr %load27, ptr %load34)
  store i1 true, ptr %17, align 1
  %load36 = load i1, ptr %17, align 1
  %icmp = icmp eq i64 %name35, 1
  %and = and i1 %load36, %icmp
  store i1 %and, ptr %17, align 1
  %load37 = load i1, ptr %17, align 1
  %icast38 = zext i1 %load37 to i32
  %field_ptr39 = getelementptr inbounds nuw %enumRef, ptr %18, i32 0, i32 0
  store ptr null, ptr %field_ptr39, align 8
  %field_ptr40 = getelementptr inbounds nuw %enumRef, ptr %18, i32 0, i32 1
  store i32 %icast38, ptr %field_ptr40, align 4
  %load41 = load %enumRef, ptr %18, align 8
  store %enumRef %load41, ptr %20, align 8
  %field_load42 = getelementptr inbounds nuw %enumRef, ptr %20, i32 0, i32 1
  %load43 = load i32, ptr %field_load42, align 4
  %icast44 = trunc i32 %load43 to i1
  br i1 %icast44, label %then45, label %else46

37:                                               ; No predecessors!
  unreachable

38:                                               ; No predecessors!
  store {} zeroinitializer, ptr %8, align 1
  br label %cont

then45:                                           ; preds = %cont
  %load48 = load ptr, ptr %3, align 8
  %load49 = load %"std::string::Chars", ptr %load48, align 8
  store %"std::string::Chars" %load49, ptr %21, align 8
  %field_load50 = getelementptr inbounds nuw %"std::string::Chars", ptr %21, i32 0, i32 0
  %load51 = load ptr, ptr %field_load50, align 8
  store ptr %load51, ptr %22, align 8
  %load52 = load ptr, ptr %3, align 8
  %field_ptr53 = getelementptr inbounds nuw %"std::string::Chars", ptr %load52, i32 0, i32 0
  store ptr @str, ptr %field_ptr53, align 8
  %field_ptr54 = getelementptr inbounds nuw %funcRef, ptr %23, i32 0, i32 0
  store ptr @Option.4, ptr %field_ptr54, align 8
  %field_ptr55 = getelementptr inbounds nuw %funcRef, ptr %23, i32 0, i32 1
  store ptr null, ptr %field_ptr55, align 8
  %load56 = load %funcRef, ptr %23, align 8
  %load57 = load ptr, ptr %22, align 8
  store %funcRef %load56, ptr %24, align 8
  %field_load58 = getelementptr inbounds nuw %funcRef, ptr %24, i32 0, i32 0
  %load59 = load ptr, ptr %field_load58, align 8
  store %funcRef %load56, ptr %25, align 8
  %field_load60 = getelementptr inbounds nuw %funcRef, ptr %25, i32 0, i32 1
  %load61 = load ptr, ptr %field_load60, align 8
  %name62 = call %enumRef %load59(ptr %load57, ptr %load61)
  ret %enumRef %name62

else46:                                           ; preds = %cont
  br label %cont47

cont47:                                           ; preds = %else46, %40
  %load63 = load {}, ptr %19, align 1
  %load64 = load ptr, ptr %3, align 8
  %load65 = load %"std::string::Chars", ptr %load64, align 8
  store %"std::string::Chars" %load65, ptr %26, align 8
  %field_load66 = getelementptr inbounds nuw %"std::string::Chars", ptr %26, i32 0, i32 0
  %load67 = load ptr, ptr %field_load66, align 8
  %field_ptr68 = getelementptr inbounds nuw %funcRef, ptr %27, i32 0, i32 0
  store ptr @"str::split_at", ptr %field_ptr68, align 8
  %field_ptr69 = getelementptr inbounds nuw %funcRef, ptr %27, i32 0, i32 1
  store ptr null, ptr %field_ptr69, align 8
  %load70 = load %funcRef, ptr %27, align 8
  store %funcRef %load70, ptr %28, align 8
  %field_load71 = getelementptr inbounds nuw %funcRef, ptr %28, i32 0, i32 0
  %load72 = load ptr, ptr %field_load71, align 8
  store %funcRef %load70, ptr %29, align 8
  %field_load73 = getelementptr inbounds nuw %funcRef, ptr %29, i32 0, i32 1
  %load74 = load ptr, ptr %field_load73, align 8
  %name75 = call ptr %load72(ptr %load67, i64 1, ptr %load74)
  %load76 = load %"(str, str)", ptr %name75, align 8
  store %"(str, str)" %load76, ptr %30, align 8
  %field_load77 = getelementptr inbounds nuw %"(str, str)", ptr %30, i32 0, i32 0
  %load78 = load ptr, ptr %field_load77, align 8
  store ptr %load78, ptr %31, align 8
  store %"(str, str)" %load76, ptr %32, align 8
  %field_load79 = getelementptr inbounds nuw %"(str, str)", ptr %32, i32 0, i32 1
  %load80 = load ptr, ptr %field_load79, align 8
  store ptr %load80, ptr %33, align 8
  %load81 = load ptr, ptr %33, align 8
  %load82 = load ptr, ptr %3, align 8
  %field_ptr83 = getelementptr inbounds nuw %"std::string::Chars", ptr %load82, i32 0, i32 0
  store ptr %load81, ptr %field_ptr83, align 8
  %field_ptr84 = getelementptr inbounds nuw %funcRef, ptr %34, i32 0, i32 0
  store ptr @Option.4, ptr %field_ptr84, align 8
  %field_ptr85 = getelementptr inbounds nuw %funcRef, ptr %34, i32 0, i32 1
  store ptr null, ptr %field_ptr85, align 8
  %load86 = load %funcRef, ptr %34, align 8
  %load87 = load ptr, ptr %31, align 8
  store %funcRef %load86, ptr %35, align 8
  %field_load88 = getelementptr inbounds nuw %funcRef, ptr %35, i32 0, i32 0
  %load89 = load ptr, ptr %field_load88, align 8
  store %funcRef %load86, ptr %36, align 8
  %field_load90 = getelementptr inbounds nuw %funcRef, ptr %36, i32 0, i32 1
  %load91 = load ptr, ptr %field_load90, align 8
  %name92 = call %enumRef %load89(ptr %load87, ptr %load91)
  ret %enumRef %name92

39:                                               ; No predecessors!
  unreachable

40:                                               ; No predecessors!
  store {} zeroinitializer, ptr %19, align 1
  br label %cont47
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
  %name13 = call {} %load10(ptr @str.8, ptr %load12)
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
  store ptr @Option.9, ptr %field_ptr18, align 8
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
  store ptr @Option.10, ptr %field_ptr29, align 8
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

define %enumRef @Option.9(i64 %0, ptr %1) {
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

define %enumRef @Option.10(ptr %0) {
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
  %4 = alloca %captures.7, align 8
  %5 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %field_ptr = getelementptr inbounds nuw %captures.7, ptr %4, i32 0, i32 0
  store ptr %load, ptr %field_ptr, align 8
  %load1 = load %captures.7, ptr %4, align 8
  %name = call ptr @margarineAlloc(i64 8)
  store %captures.7 %load1, ptr %name, align 8
  %field_ptr2 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  store ptr @"<closure>.11", ptr %field_ptr2, align 8
  %field_ptr3 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 1
  store ptr %name, ptr %field_ptr3, align 8
  %load4 = load %funcRef, ptr %5, align 8
  %name5 = call ptr @margarineAlloc(i64 16)
  %field_ptr6 = getelementptr inbounds nuw %"std::iter::Iter<int>", ptr %name5, i32 0, i32 0
  store %funcRef %load4, ptr %field_ptr6, align 8
  ret ptr %name5
}

define %enumRef @"<closure>.11"(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %captures.7, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %load = load ptr, ptr %1, align 8
  %load1 = load %captures.7, ptr %load, align 8
  store %captures.7 %load1, ptr %2, align 8
  %field_load = getelementptr inbounds nuw %captures.7, ptr %2, i32 0, i32 0
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

define i64 @fib(i64 %0, ptr %1) {
prelude:
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %enumRef, align 8
  %5 = alloca i64, align 8
  %6 = alloca %enumRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %funcRef, align 8
  %10 = alloca %funcRef, align 8
  %11 = alloca %funcRef, align 8
  %12 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %2, align 4
  store ptr %1, ptr %3, align 8
  %load = load i64, ptr %2, align 4
  %icmp = icmp sle i64 %load, 1
  %icast = zext i1 %icmp to i32
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %4, i32 0, i32 0
  store ptr null, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %enumRef, ptr %4, i32 0, i32 1
  store i32 %icast, ptr %field_ptr1, align 4
  %load2 = load %enumRef, ptr %4, align 8
  store %enumRef %load2, ptr %6, align 8
  %field_load = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 1
  %load3 = load i32, ptr %field_load, align 4
  %icast4 = trunc i32 %load3 to i1
  br i1 %icast4, label %then, label %else

then:                                             ; preds = %entry
  %load5 = load i64, ptr %2, align 4
  store i64 %load5, ptr %5, align 4
  br label %cont

else:                                             ; preds = %entry
  %field_ptr6 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  store ptr @fib, ptr %field_ptr6, align 8
  %field_ptr7 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  store ptr null, ptr %field_ptr7, align 8
  %load8 = load %funcRef, ptr %7, align 8
  %load9 = load i64, ptr %2, align 4
  %subi = sub i64 %load9, 1
  store %funcRef %load8, ptr %8, align 8
  %field_load10 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 0
  %load11 = load ptr, ptr %field_load10, align 8
  store %funcRef %load8, ptr %9, align 8
  %field_load12 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 1
  %load13 = load ptr, ptr %field_load12, align 8
  %name = call i64 %load11(i64 %subi, ptr %load13)
  %field_ptr14 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 0
  store ptr @fib, ptr %field_ptr14, align 8
  %field_ptr15 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 1
  store ptr null, ptr %field_ptr15, align 8
  %load16 = load %funcRef, ptr %10, align 8
  %load17 = load i64, ptr %2, align 4
  %subi18 = sub i64 %load17, 2
  store %funcRef %load16, ptr %11, align 8
  %field_load19 = getelementptr inbounds nuw %funcRef, ptr %11, i32 0, i32 0
  %load20 = load ptr, ptr %field_load19, align 8
  store %funcRef %load16, ptr %12, align 8
  %field_load21 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 1
  %load22 = load ptr, ptr %field_load21, align 8
  %name23 = call i64 %load20(i64 %subi18, ptr %load22)
  %addi = add i64 %name, %name23
  store i64 %addi, ptr %5, align 4
  br label %cont

cont:                                             ; preds = %else, %then
  %load24 = load i64, ptr %5, align 4
  ret i64 %load24
}

define {} @main(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %funcRef, align 8
  %3 = alloca %funcRef, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %funcRef, align 8
  %10 = alloca %funcRef, align 8
  %11 = alloca %funcRef, align 8
  %12 = alloca %funcRef, align 8
  %13 = alloca %funcRef, align 8
  %14 = alloca %funcRef, align 8
  %15 = alloca %anyType, align 8
  %16 = alloca %funcRef, align 8
  %17 = alloca %funcRef, align 8
  %18 = alloca %funcRef, align 8
  %19 = alloca %funcRef, align 8
  %20 = alloca %enumRef, align 8
  %21 = alloca %enumRef, align 8
  %22 = alloca %funcRef, align 8
  %23 = alloca %funcRef, align 8
  %24 = alloca %funcRef, align 8
  %25 = alloca %funcRef, align 8
  %26 = alloca %funcRef, align 8
  %27 = alloca %funcRef, align 8
  %28 = alloca %funcRef, align 8
  %29 = alloca %funcRef, align 8
  %30 = alloca %funcRef, align 8
  %31 = alloca %funcRef, align 8
  %32 = alloca %funcRef, align 8
  %33 = alloca %funcRef, align 8
  %34 = alloca %funcRef, align 8
  %35 = alloca %funcRef, align 8
  %36 = alloca %funcRef, align 8
  %37 = alloca %funcRef, align 8
  %38 = alloca %funcRef, align 8
  %39 = alloca %enumRef, align 8
  %40 = alloca %enumRef, align 8
  %41 = alloca ptr, align 8
  %42 = alloca %funcRef, align 8
  %43 = alloca %funcRef, align 8
  %44 = alloca %funcRef, align 8
  %45 = alloca %enumRef, align 8
  %46 = alloca %enumRef, align 8
  %47 = alloca ptr, align 8
  %48 = alloca %funcRef, align 8
  %49 = alloca %funcRef, align 8
  %50 = alloca %funcRef, align 8
  %51 = alloca %funcRef, align 8
  %52 = alloca %funcRef, align 8
  %53 = alloca %funcRef, align 8
  %54 = alloca %funcRef, align 8
  %55 = alloca %funcRef, align 8
  %56 = alloca %funcRef, align 8
  %57 = alloca %"(str, str)", align 8
  %58 = alloca ptr, align 8
  %59 = alloca %"(str, str)", align 8
  %60 = alloca ptr, align 8
  %61 = alloca %funcRef, align 8
  %62 = alloca %funcRef, align 8
  %63 = alloca %funcRef, align 8
  %64 = alloca %funcRef, align 8
  %65 = alloca %funcRef, align 8
  %66 = alloca %funcRef, align 8
  %67 = alloca %funcRef, align 8
  %68 = alloca %funcRef, align 8
  %69 = alloca %funcRef, align 8
  %70 = alloca %funcRef, align 8
  %71 = alloca %funcRef, align 8
  %72 = alloca %funcRef, align 8
  %73 = alloca %enumRef, align 8
  %74 = alloca %enumRef, align 8
  %75 = alloca %"(str, str).4", align 8
  %76 = alloca ptr, align 8
  %77 = alloca %"(str, str).4", align 8
  %78 = alloca ptr, align 8
  %79 = alloca %funcRef, align 8
  %80 = alloca %funcRef, align 8
  %81 = alloca %funcRef, align 8
  %82 = alloca %funcRef, align 8
  %83 = alloca %funcRef, align 8
  %84 = alloca %funcRef, align 8
  %85 = alloca %funcRef, align 8
  %86 = alloca %funcRef, align 8
  %87 = alloca %funcRef, align 8
  %88 = alloca %funcRef, align 8
  %89 = alloca %funcRef, align 8
  %90 = alloca %funcRef, align 8
  %91 = alloca %funcRef, align 8
  %92 = alloca %funcRef, align 8
  %93 = alloca %funcRef, align 8
  %94 = alloca %funcRef, align 8
  %95 = alloca %funcRef, align 8
  %96 = alloca %funcRef, align 8
  %97 = alloca %funcRef, align 8
  %98 = alloca %funcRef, align 8
  %99 = alloca %funcRef, align 8
  %100 = alloca %funcRef, align 8
  %101 = alloca %funcRef, align 8
  %102 = alloca %funcRef, align 8
  %103 = alloca %enumRef, align 8
  %104 = alloca %enumRef, align 8
  %105 = alloca i64, align 8
  %106 = alloca %funcRef, align 8
  %107 = alloca %funcRef, align 8
  %108 = alloca %funcRef, align 8
  %109 = alloca %funcRef, align 8
  %110 = alloca %funcRef, align 8
  %111 = alloca %funcRef, align 8
  %112 = alloca %funcRef, align 8
  %113 = alloca %funcRef, align 8
  %114 = alloca %funcRef, align 8
  %115 = alloca %funcRef, align 8
  %116 = alloca %enumRef, align 8
  %117 = alloca %enumRef, align 8
  %118 = alloca %funcRef, align 8
  %119 = alloca %funcRef, align 8
  %120 = alloca %funcRef, align 8
  %121 = alloca %funcRef, align 8
  %122 = alloca %funcRef, align 8
  %123 = alloca %funcRef, align 8
  %124 = alloca %funcRef, align 8
  %125 = alloca %funcRef, align 8
  %126 = alloca %listType, align 8
  %127 = alloca ptr, align 8
  %128 = alloca %funcRef, align 8
  %129 = alloca %funcRef, align 8
  %130 = alloca %funcRef, align 8
  %131 = alloca %funcRef, align 8
  %132 = alloca %enumRef, align 8
  %133 = alloca %enumRef, align 8
  %134 = alloca %funcRef, align 8
  %135 = alloca %funcRef, align 8
  %136 = alloca %funcRef, align 8
  %137 = alloca %funcRef, align 8
  %138 = alloca %funcRef, align 8
  %139 = alloca %funcRef, align 8
  %140 = alloca %enumRef, align 8
  %141 = alloca %enumRef, align 8
  %142 = alloca %funcRef, align 8
  %143 = alloca %funcRef, align 8
  %144 = alloca %enumRef, align 8
  %145 = alloca %enumRef, align 8
  %146 = alloca i64, align 8
  %147 = alloca %funcRef, align 8
  %148 = alloca %funcRef, align 8
  %149 = alloca %funcRef, align 8
  %150 = alloca %funcRef, align 8
  %151 = alloca %funcRef, align 8
  %152 = alloca %funcRef, align 8
  %153 = alloca %funcRef, align 8
  %154 = alloca %enumRef, align 8
  %155 = alloca %enumRef, align 8
  %156 = alloca %funcRef, align 8
  %157 = alloca %funcRef, align 8
  %158 = alloca %funcRef, align 8
  %159 = alloca %funcRef, align 8
  %160 = alloca %funcRef, align 8
  %161 = alloca %funcRef, align 8
  %162 = alloca %enumRef, align 8
  %163 = alloca %enumRef, align 8
  %164 = alloca %funcRef, align 8
  %165 = alloca %funcRef, align 8
  %166 = alloca %funcRef, align 8
  %167 = alloca %funcRef, align 8
  %168 = alloca %funcRef, align 8
  %169 = alloca %funcRef, align 8
  %170 = alloca %funcRef, align 8
  %171 = alloca %funcRef, align 8
  %172 = alloca %funcRef, align 8
  %173 = alloca %funcRef, align 8
  %174 = alloca %funcRef, align 8
  %175 = alloca %funcRef, align 8
  %176 = alloca %funcRef, align 8
  %177 = alloca %funcRef, align 8
  %178 = alloca %funcRef, align 8
  %179 = alloca %funcRef, align 8
  %180 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %2, i32 0, i32 0
  store ptr @"std::duration::Duration::now", ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %2, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %2, align 8
  store %funcRef %load, ptr %3, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %3, i32 0, i32 0
  %load2 = load ptr, ptr %field_load, align 8
  store %funcRef %load, ptr %4, align 8
  %field_load3 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  %load4 = load ptr, ptr %field_load3, align 8
  %name = call ptr %load2(ptr %load4)
  store ptr %name, ptr %5, align 8
  %field_ptr5 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  store ptr @"std::println.12", ptr %field_ptr5, align 8
  %field_ptr6 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  store ptr null, ptr %field_ptr6, align 8
  %load7 = load %funcRef, ptr %6, align 8
  %field_ptr8 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  store ptr @fib, ptr %field_ptr8, align 8
  %field_ptr9 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  store ptr null, ptr %field_ptr9, align 8
  %load10 = load %funcRef, ptr %7, align 8
  store %funcRef %load10, ptr %8, align 8
  %field_load11 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 0
  %load12 = load ptr, ptr %field_load11, align 8
  store %funcRef %load10, ptr %9, align 8
  %field_load13 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 1
  %load14 = load ptr, ptr %field_load13, align 8
  %name15 = call i64 %load12(i64 40, ptr %load14)
  store %funcRef %load7, ptr %10, align 8
  %field_load16 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 0
  %load17 = load ptr, ptr %field_load16, align 8
  store %funcRef %load7, ptr %11, align 8
  %field_load18 = getelementptr inbounds nuw %funcRef, ptr %11, i32 0, i32 1
  %load19 = load ptr, ptr %field_load18, align 8
  %name20 = call {} %load17(i64 %name15, ptr %load19)
  %field_ptr21 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 0
  store ptr @"$any", ptr %field_ptr21, align 8
  %field_ptr22 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 1
  store ptr null, ptr %field_ptr22, align 8
  %load23 = load %funcRef, ptr %12, align 8
  store %funcRef %load23, ptr %13, align 8
  %field_load24 = getelementptr inbounds nuw %funcRef, ptr %13, i32 0, i32 0
  %load25 = load ptr, ptr %field_load24, align 8
  store %funcRef %load23, ptr %14, align 8
  %field_load26 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 1
  %load27 = load ptr, ptr %field_load26, align 8
  %name28 = call %anyType %load25(ptr @str.16, ptr %load27)
  store %anyType %name28, ptr %15, align 8
  %field_ptr29 = getelementptr inbounds nuw %funcRef, ptr %16, i32 0, i32 0
  store ptr @"std::println", ptr %field_ptr29, align 8
  %field_ptr30 = getelementptr inbounds nuw %funcRef, ptr %16, i32 0, i32 1
  store ptr null, ptr %field_ptr30, align 8
  %load31 = load %funcRef, ptr %16, align 8
  %field_ptr32 = getelementptr inbounds nuw %funcRef, ptr %17, i32 0, i32 0
  store ptr @"$downcast_any", ptr %field_ptr32, align 8
  %field_ptr33 = getelementptr inbounds nuw %funcRef, ptr %17, i32 0, i32 1
  store ptr null, ptr %field_ptr33, align 8
  %load34 = load %funcRef, ptr %17, align 8
  %load35 = load %anyType, ptr %15, align 8
  store %funcRef %load34, ptr %18, align 8
  %field_load36 = getelementptr inbounds nuw %funcRef, ptr %18, i32 0, i32 0
  %load37 = load ptr, ptr %field_load36, align 8
  store %funcRef %load34, ptr %19, align 8
  %field_load38 = getelementptr inbounds nuw %funcRef, ptr %19, i32 0, i32 1
  %load39 = load ptr, ptr %field_load38, align 8
  %name40 = call %enumRef %load37(%anyType %load35, ptr %load39)
  store %enumRef %name40, ptr %20, align 8
  %field_load41 = getelementptr inbounds nuw %enumRef, ptr %20, i32 0, i32 1
  %load42 = load i32, ptr %field_load41, align 4
  %icmp = icmp eq i32 %load42, 0
  br i1 %icmp, label %then, label %else

then:                                             ; preds = %entry
  br label %cont

else:                                             ; preds = %entry
  call void @margarineAbort()
  br label %cont

cont:                                             ; preds = %else, %then
  store %enumRef %name40, ptr %21, align 8
  %field_load43 = getelementptr inbounds nuw %enumRef, ptr %21, i32 0, i32 0
  %load44 = load ptr, ptr %field_load43, align 8
  %load45 = load ptr, ptr %load44, align 8
  store %funcRef %load31, ptr %22, align 8
  %field_load46 = getelementptr inbounds nuw %funcRef, ptr %22, i32 0, i32 0
  %load47 = load ptr, ptr %field_load46, align 8
  store %funcRef %load31, ptr %23, align 8
  %field_load48 = getelementptr inbounds nuw %funcRef, ptr %23, i32 0, i32 1
  %load49 = load ptr, ptr %field_load48, align 8
  %name50 = call {} %load47(ptr %load45, ptr %load49)
  %field_ptr51 = getelementptr inbounds nuw %funcRef, ptr %24, i32 0, i32 0
  store ptr @"std::println", ptr %field_ptr51, align 8
  %field_ptr52 = getelementptr inbounds nuw %funcRef, ptr %24, i32 0, i32 1
  store ptr null, ptr %field_ptr52, align 8
  %load53 = load %funcRef, ptr %24, align 8
  %field_ptr54 = getelementptr inbounds nuw %funcRef, ptr %25, i32 0, i32 0
  store ptr @"int::to_str", ptr %field_ptr54, align 8
  %field_ptr55 = getelementptr inbounds nuw %funcRef, ptr %25, i32 0, i32 1
  store ptr null, ptr %field_ptr55, align 8
  %load56 = load %funcRef, ptr %25, align 8
  store %funcRef %load56, ptr %26, align 8
  %field_load57 = getelementptr inbounds nuw %funcRef, ptr %26, i32 0, i32 0
  %load58 = load ptr, ptr %field_load57, align 8
  store %funcRef %load56, ptr %27, align 8
  %field_load59 = getelementptr inbounds nuw %funcRef, ptr %27, i32 0, i32 1
  %load60 = load ptr, ptr %field_load59, align 8
  %name61 = call ptr %load58(i64 69, ptr %load60)
  store %funcRef %load53, ptr %28, align 8
  %field_load62 = getelementptr inbounds nuw %funcRef, ptr %28, i32 0, i32 0
  %load63 = load ptr, ptr %field_load62, align 8
  store %funcRef %load53, ptr %29, align 8
  %field_load64 = getelementptr inbounds nuw %funcRef, ptr %29, i32 0, i32 1
  %load65 = load ptr, ptr %field_load64, align 8
  %name66 = call {} %load63(ptr %name61, ptr %load65)
  %field_ptr67 = getelementptr inbounds nuw %funcRef, ptr %30, i32 0, i32 0
  store ptr @"std::println", ptr %field_ptr67, align 8
  %field_ptr68 = getelementptr inbounds nuw %funcRef, ptr %30, i32 0, i32 1
  store ptr null, ptr %field_ptr68, align 8
  %load69 = load %funcRef, ptr %30, align 8
  %field_ptr70 = getelementptr inbounds nuw %funcRef, ptr %31, i32 0, i32 0
  store ptr @"float::to_str", ptr %field_ptr70, align 8
  %field_ptr71 = getelementptr inbounds nuw %funcRef, ptr %31, i32 0, i32 1
  store ptr null, ptr %field_ptr71, align 8
  %load72 = load %funcRef, ptr %31, align 8
  store %funcRef %load72, ptr %32, align 8
  %field_load73 = getelementptr inbounds nuw %funcRef, ptr %32, i32 0, i32 0
  %load74 = load ptr, ptr %field_load73, align 8
  store %funcRef %load72, ptr %33, align 8
  %field_load75 = getelementptr inbounds nuw %funcRef, ptr %33, i32 0, i32 1
  %load76 = load ptr, ptr %field_load75, align 8
  %name77 = call ptr %load74(double 6.967000e+01, ptr %load76)
  store %funcRef %load69, ptr %34, align 8
  %field_load78 = getelementptr inbounds nuw %funcRef, ptr %34, i32 0, i32 0
  %load79 = load ptr, ptr %field_load78, align 8
  store %funcRef %load69, ptr %35, align 8
  %field_load80 = getelementptr inbounds nuw %funcRef, ptr %35, i32 0, i32 1
  %load81 = load ptr, ptr %field_load80, align 8
  %name82 = call {} %load79(ptr %name77, ptr %load81)
  %field_ptr83 = getelementptr inbounds nuw %funcRef, ptr %36, i32 0, i32 0
  store ptr @io_read_file, ptr %field_ptr83, align 8
  %field_ptr84 = getelementptr inbounds nuw %funcRef, ptr %36, i32 0, i32 1
  store ptr null, ptr %field_ptr84, align 8
  %load85 = load %funcRef, ptr %36, align 8
  store %funcRef %load85, ptr %37, align 8
  %field_load86 = getelementptr inbounds nuw %funcRef, ptr %37, i32 0, i32 0
  %load87 = load ptr, ptr %field_load86, align 8
  store %funcRef %load85, ptr %38, align 8
  %field_load88 = getelementptr inbounds nuw %funcRef, ptr %38, i32 0, i32 1
  %load89 = load ptr, ptr %field_load88, align 8
  %name90 = call %enumRef %load87(ptr @str.17, ptr %load89)
  store %enumRef %name90, ptr %39, align 8
  %field_load91 = getelementptr inbounds nuw %enumRef, ptr %39, i32 0, i32 1
  %load92 = load i32, ptr %field_load91, align 4
  %icmp93 = icmp eq i32 %load92, 0
  br i1 %icmp93, label %then94, label %else95

then94:                                           ; preds = %cont
  br label %cont96

else95:                                           ; preds = %cont
  call void @margarineAbort()
  br label %cont96

cont96:                                           ; preds = %else95, %then94
  store %enumRef %name90, ptr %40, align 8
  %field_load97 = getelementptr inbounds nuw %enumRef, ptr %40, i32 0, i32 0
  %load98 = load ptr, ptr %field_load97, align 8
  %load99 = load ptr, ptr %load98, align 8
  store ptr %load99, ptr %41, align 8
  %load100 = load ptr, ptr %41, align 8
  %field_ptr101 = getelementptr inbounds nuw %funcRef, ptr %42, i32 0, i32 0
  store ptr @"str::lines", ptr %field_ptr101, align 8
  %field_ptr102 = getelementptr inbounds nuw %funcRef, ptr %42, i32 0, i32 1
  store ptr null, ptr %field_ptr102, align 8
  %load103 = load %funcRef, ptr %42, align 8
  store %funcRef %load103, ptr %43, align 8
  %field_load104 = getelementptr inbounds nuw %funcRef, ptr %43, i32 0, i32 0
  %load105 = load ptr, ptr %field_load104, align 8
  store %funcRef %load103, ptr %44, align 8
  %field_load106 = getelementptr inbounds nuw %funcRef, ptr %44, i32 0, i32 1
  %load107 = load ptr, ptr %field_load106, align 8
  %name108 = call ptr %load105(ptr %load100, ptr %load107)
  br label %loop_body

loop_body:                                        ; preds = %cont115, %cont96
  %name109 = call %enumRef @"std::iter::Iter::__next__.18"(ptr %name108, ptr null)
  store %enumRef %name109, ptr %45, align 8
  %field_load110 = getelementptr inbounds nuw %enumRef, ptr %45, i32 0, i32 1
  %load111 = load i32, ptr %field_load110, align 4
  %icmp112 = icmp eq i32 %load111, 1
  br i1 %icmp112, label %then113, label %else114

loop_cont:                                        ; preds = %then113
  %field_ptr136 = getelementptr inbounds nuw %funcRef, ptr %54, i32 0, i32 0
  store ptr @"str::split_at", ptr %field_ptr136, align 8
  %field_ptr137 = getelementptr inbounds nuw %funcRef, ptr %54, i32 0, i32 1
  store ptr null, ptr %field_ptr137, align 8
  %load138 = load %funcRef, ptr %54, align 8
  store %funcRef %load138, ptr %55, align 8
  %field_load139 = getelementptr inbounds nuw %funcRef, ptr %55, i32 0, i32 0
  %load140 = load ptr, ptr %field_load139, align 8
  store %funcRef %load138, ptr %56, align 8
  %field_load141 = getelementptr inbounds nuw %funcRef, ptr %56, i32 0, i32 1
  %load142 = load ptr, ptr %field_load141, align 8
  %name143 = call ptr %load140(ptr @str.20, i64 5, ptr %load142)
  %load144 = load %"(str, str)", ptr %name143, align 8
  store %"(str, str)" %load144, ptr %57, align 8
  %field_load145 = getelementptr inbounds nuw %"(str, str)", ptr %57, i32 0, i32 0
  %load146 = load ptr, ptr %field_load145, align 8
  store ptr %load146, ptr %58, align 8
  store %"(str, str)" %load144, ptr %59, align 8
  %field_load147 = getelementptr inbounds nuw %"(str, str)", ptr %59, i32 0, i32 1
  %load148 = load ptr, ptr %field_load147, align 8
  store ptr %load148, ptr %60, align 8
  %field_ptr149 = getelementptr inbounds nuw %funcRef, ptr %61, i32 0, i32 0
  store ptr @"std::print", ptr %field_ptr149, align 8
  %field_ptr150 = getelementptr inbounds nuw %funcRef, ptr %61, i32 0, i32 1
  store ptr null, ptr %field_ptr150, align 8
  %load151 = load %funcRef, ptr %61, align 8
  %load152 = load ptr, ptr %58, align 8
  store %funcRef %load151, ptr %62, align 8
  %field_load153 = getelementptr inbounds nuw %funcRef, ptr %62, i32 0, i32 0
  %load154 = load ptr, ptr %field_load153, align 8
  store %funcRef %load151, ptr %63, align 8
  %field_load155 = getelementptr inbounds nuw %funcRef, ptr %63, i32 0, i32 1
  %load156 = load ptr, ptr %field_load155, align 8
  %name157 = call {} %load154(ptr %load152, ptr %load156)
  %field_ptr158 = getelementptr inbounds nuw %funcRef, ptr %64, i32 0, i32 0
  store ptr @"std::print", ptr %field_ptr158, align 8
  %field_ptr159 = getelementptr inbounds nuw %funcRef, ptr %64, i32 0, i32 1
  store ptr null, ptr %field_ptr159, align 8
  %load160 = load %funcRef, ptr %64, align 8
  %load161 = load ptr, ptr %60, align 8
  store %funcRef %load160, ptr %65, align 8
  %field_load162 = getelementptr inbounds nuw %funcRef, ptr %65, i32 0, i32 0
  %load163 = load ptr, ptr %field_load162, align 8
  store %funcRef %load160, ptr %66, align 8
  %field_load164 = getelementptr inbounds nuw %funcRef, ptr %66, i32 0, i32 1
  %load165 = load ptr, ptr %field_load164, align 8
  %name166 = call {} %load163(ptr %load161, ptr %load165)
  %field_ptr167 = getelementptr inbounds nuw %funcRef, ptr %67, i32 0, i32 0
  store ptr @"std::println", ptr %field_ptr167, align 8
  %field_ptr168 = getelementptr inbounds nuw %funcRef, ptr %67, i32 0, i32 1
  store ptr null, ptr %field_ptr168, align 8
  %load169 = load %funcRef, ptr %67, align 8
  store %funcRef %load169, ptr %68, align 8
  %field_load170 = getelementptr inbounds nuw %funcRef, ptr %68, i32 0, i32 0
  %load171 = load ptr, ptr %field_load170, align 8
  store %funcRef %load169, ptr %69, align 8
  %field_load172 = getelementptr inbounds nuw %funcRef, ptr %69, i32 0, i32 1
  %load173 = load ptr, ptr %field_load172, align 8
  %name174 = call {} %load171(ptr @str.21, ptr %load173)
  %field_ptr175 = getelementptr inbounds nuw %funcRef, ptr %70, i32 0, i32 0
  store ptr @"str::split_once", ptr %field_ptr175, align 8
  %field_ptr176 = getelementptr inbounds nuw %funcRef, ptr %70, i32 0, i32 1
  store ptr null, ptr %field_ptr176, align 8
  %load177 = load %funcRef, ptr %70, align 8
  store %funcRef %load177, ptr %71, align 8
  %field_load178 = getelementptr inbounds nuw %funcRef, ptr %71, i32 0, i32 0
  %load179 = load ptr, ptr %field_load178, align 8
  store %funcRef %load177, ptr %72, align 8
  %field_load180 = getelementptr inbounds nuw %funcRef, ptr %72, i32 0, i32 1
  %load181 = load ptr, ptr %field_load180, align 8
  %name182 = call %enumRef %load179(ptr @str.22, ptr @str.23, ptr %load181)
  store %enumRef %name182, ptr %73, align 8
  %field_load183 = getelementptr inbounds nuw %enumRef, ptr %73, i32 0, i32 1
  %load184 = load i32, ptr %field_load183, align 4
  %icmp185 = icmp eq i32 %load184, 0
  br i1 %icmp185, label %then186, label %else187

then113:                                          ; preds = %loop_body
  br label %loop_cont

else114:                                          ; preds = %loop_body
  br label %cont115

cont115:                                          ; preds = %else114, %181
  store %enumRef %name109, ptr %46, align 8
  %field_load116 = getelementptr inbounds nuw %enumRef, ptr %46, i32 0, i32 0
  %load117 = load ptr, ptr %field_load116, align 8
  %load118 = load ptr, ptr %load117, align 8
  store ptr %load118, ptr %47, align 8
  %field_ptr119 = getelementptr inbounds nuw %funcRef, ptr %48, i32 0, i32 0
  store ptr @"std::println", ptr %field_ptr119, align 8
  %field_ptr120 = getelementptr inbounds nuw %funcRef, ptr %48, i32 0, i32 1
  store ptr null, ptr %field_ptr120, align 8
  %load121 = load %funcRef, ptr %48, align 8
  store %funcRef %load121, ptr %49, align 8
  %field_load122 = getelementptr inbounds nuw %funcRef, ptr %49, i32 0, i32 0
  %load123 = load ptr, ptr %field_load122, align 8
  store %funcRef %load121, ptr %50, align 8
  %field_load124 = getelementptr inbounds nuw %funcRef, ptr %50, i32 0, i32 1
  %load125 = load ptr, ptr %field_load124, align 8
  %name126 = call {} %load123(ptr @str.19, ptr %load125)
  %field_ptr127 = getelementptr inbounds nuw %funcRef, ptr %51, i32 0, i32 0
  store ptr @"std::println", ptr %field_ptr127, align 8
  %field_ptr128 = getelementptr inbounds nuw %funcRef, ptr %51, i32 0, i32 1
  store ptr null, ptr %field_ptr128, align 8
  %load129 = load %funcRef, ptr %51, align 8
  %load130 = load ptr, ptr %47, align 8
  store %funcRef %load129, ptr %52, align 8
  %field_load131 = getelementptr inbounds nuw %funcRef, ptr %52, i32 0, i32 0
  %load132 = load ptr, ptr %field_load131, align 8
  store %funcRef %load129, ptr %53, align 8
  %field_load133 = getelementptr inbounds nuw %funcRef, ptr %53, i32 0, i32 1
  %load134 = load ptr, ptr %field_load133, align 8
  %name135 = call {} %load132(ptr %load130, ptr %load134)
  br label %loop_body

181:                                              ; No predecessors!
  br label %cont115

then186:                                          ; preds = %loop_cont
  br label %cont188

else187:                                          ; preds = %loop_cont
  call void @margarineAbort()
  br label %cont188

cont188:                                          ; preds = %else187, %then186
  store %enumRef %name182, ptr %74, align 8
  %field_load189 = getelementptr inbounds nuw %enumRef, ptr %74, i32 0, i32 0
  %load190 = load ptr, ptr %field_load189, align 8
  %load191 = load ptr, ptr %load190, align 8
  %load192 = load %"(str, str).4", ptr %load191, align 8
  store %"(str, str).4" %load192, ptr %75, align 8
  %field_load193 = getelementptr inbounds nuw %"(str, str).4", ptr %75, i32 0, i32 0
  %load194 = load ptr, ptr %field_load193, align 8
  store ptr %load194, ptr %76, align 8
  store %"(str, str).4" %load192, ptr %77, align 8
  %field_load195 = getelementptr inbounds nuw %"(str, str).4", ptr %77, i32 0, i32 1
  %load196 = load ptr, ptr %field_load195, align 8
  store ptr %load196, ptr %78, align 8
  %field_ptr197 = getelementptr inbounds nuw %funcRef, ptr %79, i32 0, i32 0
  store ptr @"std::print", ptr %field_ptr197, align 8
  %field_ptr198 = getelementptr inbounds nuw %funcRef, ptr %79, i32 0, i32 1
  store ptr null, ptr %field_ptr198, align 8
  %load199 = load %funcRef, ptr %79, align 8
  %load200 = load ptr, ptr %76, align 8
  store %funcRef %load199, ptr %80, align 8
  %field_load201 = getelementptr inbounds nuw %funcRef, ptr %80, i32 0, i32 0
  %load202 = load ptr, ptr %field_load201, align 8
  store %funcRef %load199, ptr %81, align 8
  %field_load203 = getelementptr inbounds nuw %funcRef, ptr %81, i32 0, i32 1
  %load204 = load ptr, ptr %field_load203, align 8
  %name205 = call {} %load202(ptr %load200, ptr %load204)
  %field_ptr206 = getelementptr inbounds nuw %funcRef, ptr %82, i32 0, i32 0
  store ptr @"std::print", ptr %field_ptr206, align 8
  %field_ptr207 = getelementptr inbounds nuw %funcRef, ptr %82, i32 0, i32 1
  store ptr null, ptr %field_ptr207, align 8
  %load208 = load %funcRef, ptr %82, align 8
  %load209 = load ptr, ptr %78, align 8
  store %funcRef %load208, ptr %83, align 8
  %field_load210 = getelementptr inbounds nuw %funcRef, ptr %83, i32 0, i32 0
  %load211 = load ptr, ptr %field_load210, align 8
  store %funcRef %load208, ptr %84, align 8
  %field_load212 = getelementptr inbounds nuw %funcRef, ptr %84, i32 0, i32 1
  %load213 = load ptr, ptr %field_load212, align 8
  %name214 = call {} %load211(ptr %load209, ptr %load213)
  %field_ptr215 = getelementptr inbounds nuw %funcRef, ptr %85, i32 0, i32 0
  store ptr @"std::println", ptr %field_ptr215, align 8
  %field_ptr216 = getelementptr inbounds nuw %funcRef, ptr %85, i32 0, i32 1
  store ptr null, ptr %field_ptr216, align 8
  %load217 = load %funcRef, ptr %85, align 8
  store %funcRef %load217, ptr %86, align 8
  %field_load218 = getelementptr inbounds nuw %funcRef, ptr %86, i32 0, i32 0
  %load219 = load ptr, ptr %field_load218, align 8
  store %funcRef %load217, ptr %87, align 8
  %field_load220 = getelementptr inbounds nuw %funcRef, ptr %87, i32 0, i32 1
  %load221 = load ptr, ptr %field_load220, align 8
  %name222 = call {} %load219(ptr @str.24, ptr %load221)
  %field_ptr223 = getelementptr inbounds nuw %funcRef, ptr %88, i32 0, i32 0
  store ptr @"std::println", ptr %field_ptr223, align 8
  %field_ptr224 = getelementptr inbounds nuw %funcRef, ptr %88, i32 0, i32 1
  store ptr null, ptr %field_ptr224, align 8
  %load225 = load %funcRef, ptr %88, align 8
  %field_ptr226 = getelementptr inbounds nuw %funcRef, ptr %89, i32 0, i32 0
  store ptr @"str::nth", ptr %field_ptr226, align 8
  %field_ptr227 = getelementptr inbounds nuw %funcRef, ptr %89, i32 0, i32 1
  store ptr null, ptr %field_ptr227, align 8
  %load228 = load %funcRef, ptr %89, align 8
  store %funcRef %load228, ptr %90, align 8
  %field_load229 = getelementptr inbounds nuw %funcRef, ptr %90, i32 0, i32 0
  %load230 = load ptr, ptr %field_load229, align 8
  store %funcRef %load228, ptr %91, align 8
  %field_load231 = getelementptr inbounds nuw %funcRef, ptr %91, i32 0, i32 1
  %load232 = load ptr, ptr %field_load231, align 8
  %name233 = call ptr %load230(ptr @str.25, i64 2, ptr %load232)
  store %funcRef %load225, ptr %92, align 8
  %field_load234 = getelementptr inbounds nuw %funcRef, ptr %92, i32 0, i32 0
  %load235 = load ptr, ptr %field_load234, align 8
  store %funcRef %load225, ptr %93, align 8
  %field_load236 = getelementptr inbounds nuw %funcRef, ptr %93, i32 0, i32 1
  %load237 = load ptr, ptr %field_load236, align 8
  %name238 = call {} %load235(ptr %name233, ptr %load237)
  %field_ptr239 = getelementptr inbounds nuw %funcRef, ptr %94, i32 0, i32 0
  store ptr @"std::println", ptr %field_ptr239, align 8
  %field_ptr240 = getelementptr inbounds nuw %funcRef, ptr %94, i32 0, i32 1
  store ptr null, ptr %field_ptr240, align 8
  %load241 = load %funcRef, ptr %94, align 8
  %field_ptr242 = getelementptr inbounds nuw %funcRef, ptr %95, i32 0, i32 0
  store ptr @"str::slice", ptr %field_ptr242, align 8
  %field_ptr243 = getelementptr inbounds nuw %funcRef, ptr %95, i32 0, i32 1
  store ptr null, ptr %field_ptr243, align 8
  %load244 = load %funcRef, ptr %95, align 8
  %name245 = call ptr @margarineAlloc(i64 16)
  %field_ptr246 = getelementptr inbounds nuw %Range, ptr %name245, i32 0, i32 0
  store i64 2, ptr %field_ptr246, align 4
  %field_ptr247 = getelementptr inbounds nuw %Range, ptr %name245, i32 0, i32 1
  store i64 5, ptr %field_ptr247, align 4
  store %funcRef %load244, ptr %96, align 8
  %field_load248 = getelementptr inbounds nuw %funcRef, ptr %96, i32 0, i32 0
  %load249 = load ptr, ptr %field_load248, align 8
  store %funcRef %load244, ptr %97, align 8
  %field_load250 = getelementptr inbounds nuw %funcRef, ptr %97, i32 0, i32 1
  %load251 = load ptr, ptr %field_load250, align 8
  %name252 = call ptr %load249(ptr @str.26, ptr %name245, ptr %load251)
  store %funcRef %load241, ptr %98, align 8
  %field_load253 = getelementptr inbounds nuw %funcRef, ptr %98, i32 0, i32 0
  %load254 = load ptr, ptr %field_load253, align 8
  store %funcRef %load241, ptr %99, align 8
  %field_load255 = getelementptr inbounds nuw %funcRef, ptr %99, i32 0, i32 1
  %load256 = load ptr, ptr %field_load255, align 8
  %name257 = call {} %load254(ptr %name252, ptr %load256)
  %field_ptr258 = getelementptr inbounds nuw %funcRef, ptr %100, i32 0, i32 0
  store ptr @"str::parse", ptr %field_ptr258, align 8
  %field_ptr259 = getelementptr inbounds nuw %funcRef, ptr %100, i32 0, i32 1
  store ptr null, ptr %field_ptr259, align 8
  %load260 = load %funcRef, ptr %100, align 8
  store %funcRef %load260, ptr %101, align 8
  %field_load261 = getelementptr inbounds nuw %funcRef, ptr %101, i32 0, i32 0
  %load262 = load ptr, ptr %field_load261, align 8
  store %funcRef %load260, ptr %102, align 8
  %field_load263 = getelementptr inbounds nuw %funcRef, ptr %102, i32 0, i32 1
  %load264 = load ptr, ptr %field_load263, align 8
  %name265 = call %enumRef %load262(ptr @str.27, ptr %load264)
  store %enumRef %name265, ptr %103, align 8
  %field_load266 = getelementptr inbounds nuw %enumRef, ptr %103, i32 0, i32 1
  %load267 = load i32, ptr %field_load266, align 4
  %icmp268 = icmp eq i32 %load267, 0
  br i1 %icmp268, label %then269, label %else270

then269:                                          ; preds = %cont188
  br label %cont271

else270:                                          ; preds = %cont188
  call void @margarineAbort()
  br label %cont271

cont271:                                          ; preds = %else270, %then269
  store %enumRef %name265, ptr %104, align 8
  %field_load272 = getelementptr inbounds nuw %enumRef, ptr %104, i32 0, i32 0
  %load273 = load ptr, ptr %field_load272, align 8
  %load274 = load i64, ptr %load273, align 4
  store i64 %load274, ptr %105, align 4
  %field_ptr275 = getelementptr inbounds nuw %funcRef, ptr %106, i32 0, i32 0
  store ptr @"std::println", ptr %field_ptr275, align 8
  %field_ptr276 = getelementptr inbounds nuw %funcRef, ptr %106, i32 0, i32 1
  store ptr null, ptr %field_ptr276, align 8
  %load277 = load %funcRef, ptr %106, align 8
  store %funcRef %load277, ptr %107, align 8
  %field_load278 = getelementptr inbounds nuw %funcRef, ptr %107, i32 0, i32 0
  %load279 = load ptr, ptr %field_load278, align 8
  store %funcRef %load277, ptr %108, align 8
  %field_load280 = getelementptr inbounds nuw %funcRef, ptr %108, i32 0, i32 1
  %load281 = load ptr, ptr %field_load280, align 8
  %name282 = call {} %load279(ptr @str.30, ptr %load281)
  %field_ptr283 = getelementptr inbounds nuw %funcRef, ptr %109, i32 0, i32 0
  store ptr @"std::println.12", ptr %field_ptr283, align 8
  %field_ptr284 = getelementptr inbounds nuw %funcRef, ptr %109, i32 0, i32 1
  store ptr null, ptr %field_ptr284, align 8
  %load285 = load %funcRef, ptr %109, align 8
  %load286 = load i64, ptr %105, align 4
  store %funcRef %load285, ptr %110, align 8
  %field_load287 = getelementptr inbounds nuw %funcRef, ptr %110, i32 0, i32 0
  %load288 = load ptr, ptr %field_load287, align 8
  store %funcRef %load285, ptr %111, align 8
  %field_load289 = getelementptr inbounds nuw %funcRef, ptr %111, i32 0, i32 1
  %load290 = load ptr, ptr %field_load289, align 8
  %name291 = call {} %load288(i64 %load286, ptr %load290)
  %field_ptr292 = getelementptr inbounds nuw %funcRef, ptr %112, i32 0, i32 0
  store ptr @"std::println.31", ptr %field_ptr292, align 8
  %field_ptr293 = getelementptr inbounds nuw %funcRef, ptr %112, i32 0, i32 1
  store ptr null, ptr %field_ptr293, align 8
  %load294 = load %funcRef, ptr %112, align 8
  %field_ptr295 = getelementptr inbounds nuw %funcRef, ptr %113, i32 0, i32 0
  store ptr @"str::parse.36", ptr %field_ptr295, align 8
  %field_ptr296 = getelementptr inbounds nuw %funcRef, ptr %113, i32 0, i32 1
  store ptr null, ptr %field_ptr296, align 8
  %load297 = load %funcRef, ptr %113, align 8
  store %funcRef %load297, ptr %114, align 8
  %field_load298 = getelementptr inbounds nuw %funcRef, ptr %114, i32 0, i32 0
  %load299 = load ptr, ptr %field_load298, align 8
  store %funcRef %load297, ptr %115, align 8
  %field_load300 = getelementptr inbounds nuw %funcRef, ptr %115, i32 0, i32 1
  %load301 = load ptr, ptr %field_load300, align 8
  %name302 = call %enumRef %load299(ptr @str.35, ptr %load301)
  store %enumRef %name302, ptr %116, align 8
  %field_load303 = getelementptr inbounds nuw %enumRef, ptr %116, i32 0, i32 1
  %load304 = load i32, ptr %field_load303, align 4
  %icmp305 = icmp eq i32 %load304, 0
  br i1 %icmp305, label %then306, label %else307

then306:                                          ; preds = %cont271
  br label %cont308

else307:                                          ; preds = %cont271
  call void @margarineAbort()
  br label %cont308

cont308:                                          ; preds = %else307, %then306
  store %enumRef %name302, ptr %117, align 8
  %field_load309 = getelementptr inbounds nuw %enumRef, ptr %117, i32 0, i32 0
  %load310 = load ptr, ptr %field_load309, align 8
  %load311 = load double, ptr %load310, align 8
  store %funcRef %load294, ptr %118, align 8
  %field_load312 = getelementptr inbounds nuw %funcRef, ptr %118, i32 0, i32 0
  %load313 = load ptr, ptr %field_load312, align 8
  store %funcRef %load294, ptr %119, align 8
  %field_load314 = getelementptr inbounds nuw %funcRef, ptr %119, i32 0, i32 1
  %load315 = load ptr, ptr %field_load314, align 8
  %name316 = call {} %load313(double %load311, ptr %load315)
  %field_ptr317 = getelementptr inbounds nuw %funcRef, ptr %120, i32 0, i32 0
  store ptr @"std::println.12", ptr %field_ptr317, align 8
  %field_ptr318 = getelementptr inbounds nuw %funcRef, ptr %120, i32 0, i32 1
  store ptr null, ptr %field_ptr318, align 8
  %load319 = load %funcRef, ptr %120, align 8
  %field_ptr320 = getelementptr inbounds nuw %funcRef, ptr %121, i32 0, i32 0
  store ptr @"$sizeof", ptr %field_ptr320, align 8
  %field_ptr321 = getelementptr inbounds nuw %funcRef, ptr %121, i32 0, i32 1
  store ptr null, ptr %field_ptr321, align 8
  %load322 = load %funcRef, ptr %121, align 8
  store %funcRef %load322, ptr %122, align 8
  %field_load323 = getelementptr inbounds nuw %funcRef, ptr %122, i32 0, i32 0
  %load324 = load ptr, ptr %field_load323, align 8
  store %funcRef %load322, ptr %123, align 8
  %field_load325 = getelementptr inbounds nuw %funcRef, ptr %123, i32 0, i32 1
  %load326 = load ptr, ptr %field_load325, align 8
  %name327 = call i64 %load324(ptr %load326)
  store %funcRef %load319, ptr %124, align 8
  %field_load328 = getelementptr inbounds nuw %funcRef, ptr %124, i32 0, i32 0
  %load329 = load ptr, ptr %field_load328, align 8
  store %funcRef %load319, ptr %125, align 8
  %field_load330 = getelementptr inbounds nuw %funcRef, ptr %125, i32 0, i32 1
  %load331 = load ptr, ptr %field_load330, align 8
  %name332 = call {} %load329(i64 %name327, ptr %load331)
  %name333 = call ptr @margarineAlloc(i64 40)
  %gep = getelementptr ptr, ptr %name333, i32 0
  store i64 5, ptr %gep, align 4
  %gep334 = getelementptr ptr, ptr %name333, i32 1
  store i64 4, ptr %gep334, align 4
  %gep335 = getelementptr ptr, ptr %name333, i32 2
  store i64 3, ptr %gep335, align 4
  %gep336 = getelementptr ptr, ptr %name333, i32 3
  store i64 2, ptr %gep336, align 4
  %gep337 = getelementptr ptr, ptr %name333, i32 4
  store i64 1, ptr %gep337, align 4
  %field_ptr338 = getelementptr inbounds nuw %listType, ptr %126, i32 0, i32 0
  store i32 5, ptr %field_ptr338, align 4
  %field_ptr339 = getelementptr inbounds nuw %listType, ptr %126, i32 0, i32 1
  store i32 5, ptr %field_ptr339, align 4
  %field_ptr340 = getelementptr inbounds nuw %listType, ptr %126, i32 0, i32 2
  store ptr %name333, ptr %field_ptr340, align 8
  %load341 = load %listType, ptr %126, align 8
  %name342 = call ptr @margarineAlloc(i64 16)
  store %listType %load341, ptr %name342, align 8
  store ptr %name342, ptr %127, align 8
  %field_ptr343 = getelementptr inbounds nuw %funcRef, ptr %128, i32 0, i32 0
  store ptr @"std::println.12", ptr %field_ptr343, align 8
  %field_ptr344 = getelementptr inbounds nuw %funcRef, ptr %128, i32 0, i32 1
  store ptr null, ptr %field_ptr344, align 8
  %load345 = load %funcRef, ptr %128, align 8
  %load346 = load ptr, ptr %127, align 8
  %field_ptr347 = getelementptr inbounds nuw %funcRef, ptr %129, i32 0, i32 0
  store ptr @"List::pop", ptr %field_ptr347, align 8
  %field_ptr348 = getelementptr inbounds nuw %funcRef, ptr %129, i32 0, i32 1
  store ptr null, ptr %field_ptr348, align 8
  %load349 = load %funcRef, ptr %129, align 8
  store %funcRef %load349, ptr %130, align 8
  %field_load350 = getelementptr inbounds nuw %funcRef, ptr %130, i32 0, i32 0
  %load351 = load ptr, ptr %field_load350, align 8
  store %funcRef %load349, ptr %131, align 8
  %field_load352 = getelementptr inbounds nuw %funcRef, ptr %131, i32 0, i32 1
  %load353 = load ptr, ptr %field_load352, align 8
  %name354 = call %enumRef %load351(ptr %load346, ptr %load353)
  store %enumRef %name354, ptr %132, align 8
  %field_load355 = getelementptr inbounds nuw %enumRef, ptr %132, i32 0, i32 1
  %load356 = load i32, ptr %field_load355, align 4
  %icmp357 = icmp eq i32 %load356, 0
  br i1 %icmp357, label %then358, label %else359

then358:                                          ; preds = %cont308
  br label %cont360

else359:                                          ; preds = %cont308
  call void @margarineAbort()
  br label %cont360

cont360:                                          ; preds = %else359, %then358
  store %enumRef %name354, ptr %133, align 8
  %field_load361 = getelementptr inbounds nuw %enumRef, ptr %133, i32 0, i32 0
  %load362 = load ptr, ptr %field_load361, align 8
  %load363 = load i64, ptr %load362, align 4
  store %funcRef %load345, ptr %134, align 8
  %field_load364 = getelementptr inbounds nuw %funcRef, ptr %134, i32 0, i32 0
  %load365 = load ptr, ptr %field_load364, align 8
  store %funcRef %load345, ptr %135, align 8
  %field_load366 = getelementptr inbounds nuw %funcRef, ptr %135, i32 0, i32 1
  %load367 = load ptr, ptr %field_load366, align 8
  %name368 = call {} %load365(i64 %load363, ptr %load367)
  %field_ptr369 = getelementptr inbounds nuw %funcRef, ptr %136, i32 0, i32 0
  store ptr @"std::println.12", ptr %field_ptr369, align 8
  %field_ptr370 = getelementptr inbounds nuw %funcRef, ptr %136, i32 0, i32 1
  store ptr null, ptr %field_ptr370, align 8
  %load371 = load %funcRef, ptr %136, align 8
  %load372 = load ptr, ptr %127, align 8
  %field_ptr373 = getelementptr inbounds nuw %funcRef, ptr %137, i32 0, i32 0
  store ptr @"List::pop", ptr %field_ptr373, align 8
  %field_ptr374 = getelementptr inbounds nuw %funcRef, ptr %137, i32 0, i32 1
  store ptr null, ptr %field_ptr374, align 8
  %load375 = load %funcRef, ptr %137, align 8
  store %funcRef %load375, ptr %138, align 8
  %field_load376 = getelementptr inbounds nuw %funcRef, ptr %138, i32 0, i32 0
  %load377 = load ptr, ptr %field_load376, align 8
  store %funcRef %load375, ptr %139, align 8
  %field_load378 = getelementptr inbounds nuw %funcRef, ptr %139, i32 0, i32 1
  %load379 = load ptr, ptr %field_load378, align 8
  %name380 = call %enumRef %load377(ptr %load372, ptr %load379)
  store %enumRef %name380, ptr %140, align 8
  %field_load381 = getelementptr inbounds nuw %enumRef, ptr %140, i32 0, i32 1
  %load382 = load i32, ptr %field_load381, align 4
  %icmp383 = icmp eq i32 %load382, 0
  br i1 %icmp383, label %then384, label %else385

then384:                                          ; preds = %cont360
  br label %cont386

else385:                                          ; preds = %cont360
  call void @margarineAbort()
  br label %cont386

cont386:                                          ; preds = %else385, %then384
  store %enumRef %name380, ptr %141, align 8
  %field_load387 = getelementptr inbounds nuw %enumRef, ptr %141, i32 0, i32 0
  %load388 = load ptr, ptr %field_load387, align 8
  %load389 = load i64, ptr %load388, align 4
  store %funcRef %load371, ptr %142, align 8
  %field_load390 = getelementptr inbounds nuw %funcRef, ptr %142, i32 0, i32 0
  %load391 = load ptr, ptr %field_load390, align 8
  store %funcRef %load371, ptr %143, align 8
  %field_load392 = getelementptr inbounds nuw %funcRef, ptr %143, i32 0, i32 1
  %load393 = load ptr, ptr %field_load392, align 8
  %name394 = call {} %load391(i64 %load389, ptr %load393)
  %name395 = call ptr @margarineAlloc(i64 16)
  %field_ptr396 = getelementptr inbounds nuw %Range, ptr %name395, i32 0, i32 0
  store i64 0, ptr %field_ptr396, align 4
  %field_ptr397 = getelementptr inbounds nuw %Range, ptr %name395, i32 0, i32 1
  store i64 10, ptr %field_ptr397, align 4
  br label %loop_body398

loop_body398:                                     ; preds = %cont406, %cont386
  %name400 = call %enumRef @"Range::__next__"(ptr %name395, ptr null)
  store %enumRef %name400, ptr %144, align 8
  %field_load401 = getelementptr inbounds nuw %enumRef, ptr %144, i32 0, i32 1
  %load402 = load i32, ptr %field_load401, align 4
  %icmp403 = icmp eq i32 %load402, 1
  br i1 %icmp403, label %then404, label %else405

loop_cont399:                                     ; preds = %then404
  %field_ptr420 = getelementptr inbounds nuw %funcRef, ptr %150, i32 0, i32 0
  store ptr @"std::println.12", ptr %field_ptr420, align 8
  %field_ptr421 = getelementptr inbounds nuw %funcRef, ptr %150, i32 0, i32 1
  store ptr null, ptr %field_ptr421, align 8
  %load422 = load %funcRef, ptr %150, align 8
  %load423 = load ptr, ptr %127, align 8
  %field_ptr424 = getelementptr inbounds nuw %funcRef, ptr %151, i32 0, i32 0
  store ptr @"List::pop", ptr %field_ptr424, align 8
  %field_ptr425 = getelementptr inbounds nuw %funcRef, ptr %151, i32 0, i32 1
  store ptr null, ptr %field_ptr425, align 8
  %load426 = load %funcRef, ptr %151, align 8
  store %funcRef %load426, ptr %152, align 8
  %field_load427 = getelementptr inbounds nuw %funcRef, ptr %152, i32 0, i32 0
  %load428 = load ptr, ptr %field_load427, align 8
  store %funcRef %load426, ptr %153, align 8
  %field_load429 = getelementptr inbounds nuw %funcRef, ptr %153, i32 0, i32 1
  %load430 = load ptr, ptr %field_load429, align 8
  %name431 = call %enumRef %load428(ptr %load423, ptr %load430)
  store %enumRef %name431, ptr %154, align 8
  %field_load432 = getelementptr inbounds nuw %enumRef, ptr %154, i32 0, i32 1
  %load433 = load i32, ptr %field_load432, align 4
  %icmp434 = icmp eq i32 %load433, 0
  br i1 %icmp434, label %then435, label %else436

then404:                                          ; preds = %loop_body398
  br label %loop_cont399

else405:                                          ; preds = %loop_body398
  br label %cont406

cont406:                                          ; preds = %else405, %182
  store %enumRef %name400, ptr %145, align 8
  %field_load407 = getelementptr inbounds nuw %enumRef, ptr %145, i32 0, i32 0
  %load408 = load ptr, ptr %field_load407, align 8
  %load409 = load i64, ptr %load408, align 4
  store i64 %load409, ptr %146, align 4
  %load410 = load ptr, ptr %127, align 8
  %field_ptr411 = getelementptr inbounds nuw %funcRef, ptr %147, i32 0, i32 0
  store ptr @"List::push", ptr %field_ptr411, align 8
  %field_ptr412 = getelementptr inbounds nuw %funcRef, ptr %147, i32 0, i32 1
  store ptr null, ptr %field_ptr412, align 8
  %load413 = load %funcRef, ptr %147, align 8
  %load414 = load i64, ptr %146, align 4
  store %funcRef %load413, ptr %148, align 8
  %field_load415 = getelementptr inbounds nuw %funcRef, ptr %148, i32 0, i32 0
  %load416 = load ptr, ptr %field_load415, align 8
  store %funcRef %load413, ptr %149, align 8
  %field_load417 = getelementptr inbounds nuw %funcRef, ptr %149, i32 0, i32 1
  %load418 = load ptr, ptr %field_load417, align 8
  %name419 = call {} %load416(ptr %load410, i64 %load414, ptr %load418)
  br label %loop_body398

182:                                              ; No predecessors!
  br label %cont406

then435:                                          ; preds = %loop_cont399
  br label %cont437

else436:                                          ; preds = %loop_cont399
  call void @margarineAbort()
  br label %cont437

cont437:                                          ; preds = %else436, %then435
  store %enumRef %name431, ptr %155, align 8
  %field_load438 = getelementptr inbounds nuw %enumRef, ptr %155, i32 0, i32 0
  %load439 = load ptr, ptr %field_load438, align 8
  %load440 = load i64, ptr %load439, align 4
  store %funcRef %load422, ptr %156, align 8
  %field_load441 = getelementptr inbounds nuw %funcRef, ptr %156, i32 0, i32 0
  %load442 = load ptr, ptr %field_load441, align 8
  store %funcRef %load422, ptr %157, align 8
  %field_load443 = getelementptr inbounds nuw %funcRef, ptr %157, i32 0, i32 1
  %load444 = load ptr, ptr %field_load443, align 8
  %name445 = call {} %load442(i64 %load440, ptr %load444)
  %field_ptr446 = getelementptr inbounds nuw %funcRef, ptr %158, i32 0, i32 0
  store ptr @"std::println.12", ptr %field_ptr446, align 8
  %field_ptr447 = getelementptr inbounds nuw %funcRef, ptr %158, i32 0, i32 1
  store ptr null, ptr %field_ptr447, align 8
  %load448 = load %funcRef, ptr %158, align 8
  %load449 = load ptr, ptr %127, align 8
  %field_ptr450 = getelementptr inbounds nuw %funcRef, ptr %159, i32 0, i32 0
  store ptr @"List::pop", ptr %field_ptr450, align 8
  %field_ptr451 = getelementptr inbounds nuw %funcRef, ptr %159, i32 0, i32 1
  store ptr null, ptr %field_ptr451, align 8
  %load452 = load %funcRef, ptr %159, align 8
  store %funcRef %load452, ptr %160, align 8
  %field_load453 = getelementptr inbounds nuw %funcRef, ptr %160, i32 0, i32 0
  %load454 = load ptr, ptr %field_load453, align 8
  store %funcRef %load452, ptr %161, align 8
  %field_load455 = getelementptr inbounds nuw %funcRef, ptr %161, i32 0, i32 1
  %load456 = load ptr, ptr %field_load455, align 8
  %name457 = call %enumRef %load454(ptr %load449, ptr %load456)
  store %enumRef %name457, ptr %162, align 8
  %field_load458 = getelementptr inbounds nuw %enumRef, ptr %162, i32 0, i32 1
  %load459 = load i32, ptr %field_load458, align 4
  %icmp460 = icmp eq i32 %load459, 0
  br i1 %icmp460, label %then461, label %else462

then461:                                          ; preds = %cont437
  br label %cont463

else462:                                          ; preds = %cont437
  call void @margarineAbort()
  br label %cont463

cont463:                                          ; preds = %else462, %then461
  store %enumRef %name457, ptr %163, align 8
  %field_load464 = getelementptr inbounds nuw %enumRef, ptr %163, i32 0, i32 0
  %load465 = load ptr, ptr %field_load464, align 8
  %load466 = load i64, ptr %load465, align 4
  store %funcRef %load448, ptr %164, align 8
  %field_load467 = getelementptr inbounds nuw %funcRef, ptr %164, i32 0, i32 0
  %load468 = load ptr, ptr %field_load467, align 8
  store %funcRef %load448, ptr %165, align 8
  %field_load469 = getelementptr inbounds nuw %funcRef, ptr %165, i32 0, i32 1
  %load470 = load ptr, ptr %field_load469, align 8
  %name471 = call {} %load468(i64 %load466, ptr %load470)
  %field_ptr472 = getelementptr inbounds nuw %funcRef, ptr %166, i32 0, i32 0
  store ptr @"std::println", ptr %field_ptr472, align 8
  %field_ptr473 = getelementptr inbounds nuw %funcRef, ptr %166, i32 0, i32 1
  store ptr null, ptr %field_ptr473, align 8
  %load474 = load %funcRef, ptr %166, align 8
  %load475 = load ptr, ptr %5, align 8
  %field_ptr476 = getelementptr inbounds nuw %funcRef, ptr %167, i32 0, i32 0
  store ptr @"std::duration::Duration::elapsed", ptr %field_ptr476, align 8
  %field_ptr477 = getelementptr inbounds nuw %funcRef, ptr %167, i32 0, i32 1
  store ptr null, ptr %field_ptr477, align 8
  %load478 = load %funcRef, ptr %167, align 8
  store %funcRef %load478, ptr %168, align 8
  %field_load479 = getelementptr inbounds nuw %funcRef, ptr %168, i32 0, i32 0
  %load480 = load ptr, ptr %field_load479, align 8
  store %funcRef %load478, ptr %169, align 8
  %field_load481 = getelementptr inbounds nuw %funcRef, ptr %169, i32 0, i32 1
  %load482 = load ptr, ptr %field_load481, align 8
  %name483 = call ptr %load480(ptr %load475, ptr %load482)
  %field_ptr484 = getelementptr inbounds nuw %funcRef, ptr %170, i32 0, i32 0
  store ptr @"std::duration::Duration::as_secs_float", ptr %field_ptr484, align 8
  %field_ptr485 = getelementptr inbounds nuw %funcRef, ptr %170, i32 0, i32 1
  store ptr null, ptr %field_ptr485, align 8
  %load486 = load %funcRef, ptr %170, align 8
  store %funcRef %load486, ptr %171, align 8
  %field_load487 = getelementptr inbounds nuw %funcRef, ptr %171, i32 0, i32 0
  %load488 = load ptr, ptr %field_load487, align 8
  store %funcRef %load486, ptr %172, align 8
  %field_load489 = getelementptr inbounds nuw %funcRef, ptr %172, i32 0, i32 1
  %load490 = load ptr, ptr %field_load489, align 8
  %name491 = call double %load488(ptr %name483, ptr %load490)
  %field_ptr492 = getelementptr inbounds nuw %funcRef, ptr %173, i32 0, i32 0
  store ptr @"float::to_str", ptr %field_ptr492, align 8
  %field_ptr493 = getelementptr inbounds nuw %funcRef, ptr %173, i32 0, i32 1
  store ptr null, ptr %field_ptr493, align 8
  %load494 = load %funcRef, ptr %173, align 8
  store %funcRef %load494, ptr %174, align 8
  %field_load495 = getelementptr inbounds nuw %funcRef, ptr %174, i32 0, i32 0
  %load496 = load ptr, ptr %field_load495, align 8
  store %funcRef %load494, ptr %175, align 8
  %field_load497 = getelementptr inbounds nuw %funcRef, ptr %175, i32 0, i32 1
  %load498 = load ptr, ptr %field_load497, align 8
  %name499 = call ptr %load496(double %name491, ptr %load498)
  store %funcRef %load474, ptr %176, align 8
  %field_load500 = getelementptr inbounds nuw %funcRef, ptr %176, i32 0, i32 0
  %load501 = load ptr, ptr %field_load500, align 8
  store %funcRef %load474, ptr %177, align 8
  %field_load502 = getelementptr inbounds nuw %funcRef, ptr %177, i32 0, i32 1
  %load503 = load ptr, ptr %field_load502, align 8
  %name504 = call {} %load501(ptr %name499, ptr %load503)
  %field_ptr505 = getelementptr inbounds nuw %funcRef, ptr %178, i32 0, i32 0
  store ptr @"std::panic", ptr %field_ptr505, align 8
  %field_ptr506 = getelementptr inbounds nuw %funcRef, ptr %178, i32 0, i32 1
  store ptr null, ptr %field_ptr506, align 8
  %load507 = load %funcRef, ptr %178, align 8
  store %funcRef %load507, ptr %179, align 8
  %field_load508 = getelementptr inbounds nuw %funcRef, ptr %179, i32 0, i32 0
  %load509 = load ptr, ptr %field_load508, align 8
  store %funcRef %load507, ptr %180, align 8
  %field_load510 = getelementptr inbounds nuw %funcRef, ptr %180, i32 0, i32 1
  %load511 = load ptr, ptr %field_load510, align 8
  %name512 = call ptr %load509(ptr @str.43, ptr %load511)
  unreachable

183:                                              ; No predecessors!
  ret {} zeroinitializer
}

define {} @"std::println.12"(i64 %0, ptr %1) {
prelude:
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %2, align 4
  store ptr %1, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @"std::print.13", ptr %field_ptr, align 8
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
  %name = call {} %load3(i64 %load2, ptr %load5)
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
  %name13 = call {} %load10(ptr @str.15, ptr %load12)
  ret {} zeroinitializer
}

define {} @"std::print.13"(i64 %0, ptr %1) {
prelude:
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %2, align 4
  store ptr %1, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @print_raw, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %4, align 8
  %field_ptr2 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  store ptr @"$any.14", ptr %field_ptr2, align 8
  %field_ptr3 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 1
  store ptr null, ptr %field_ptr3, align 8
  %load4 = load %funcRef, ptr %5, align 8
  %load5 = load i64, ptr %2, align 4
  store %funcRef %load4, ptr %6, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  %load6 = load ptr, ptr %field_load, align 8
  store %funcRef %load4, ptr %7, align 8
  %field_load7 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  %load8 = load ptr, ptr %field_load7, align 8
  %name = call %anyType %load6(i64 %load5, ptr %load8)
  store %funcRef %load, ptr %8, align 8
  %field_load9 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 0
  %load10 = load ptr, ptr %field_load9, align 8
  store %funcRef %load, ptr %9, align 8
  %field_load11 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 1
  %load12 = load ptr, ptr %field_load11, align 8
  %name13 = call {} %load10(%anyType %name, ptr %load12)
  ret {} zeroinitializer
}

define %anyType @"$any.14"(i64 %0, ptr %1) {
prelude:
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %anyType, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %2, align 4
  store ptr %1, ptr %3, align 8
  %name = call ptr @margarineAlloc(i64 8)
  %load = load i64, ptr %2, align 4
  store i64 %load, ptr %name, align 4
  %field_ptr = getelementptr inbounds nuw %anyType, ptr %4, i32 0, i32 0
  store ptr %name, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %anyType, ptr %4, i32 0, i32 1
  store i32 1, ptr %field_ptr1, align 4
  %load2 = load %anyType, ptr %4, align 8
  ret %anyType %load2
}

define %enumRef @"$downcast_any"(%anyType %0, ptr %1) {
prelude:
  %2 = alloca %anyType, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %anyType, align 8
  %5 = alloca %anyType, align 8
  %6 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store %anyType %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load %anyType, ptr %2, align 8
  store %anyType %load, ptr %4, align 8
  %field_load = getelementptr inbounds nuw %anyType, ptr %4, i32 0, i32 1
  %load1 = load i32, ptr %field_load, align 4
  %icmp = icmp ne i32 16, %load1
  %icast = zext i1 %icmp to i32
  store %anyType %load, ptr %5, align 8
  %field_load2 = getelementptr inbounds nuw %anyType, ptr %5, i32 0, i32 0
  %load3 = load ptr, ptr %field_load2, align 8
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 0
  store ptr %load3, ptr %field_ptr, align 8
  %field_ptr4 = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 1
  store i32 %icast, ptr %field_ptr4, align 4
  %load5 = load %enumRef, ptr %6, align 8
  ret %enumRef %load5
}

declare %enumRef @io_read_file(ptr, ptr)

define %enumRef @"std::iter::Iter::__next__.18"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %"std::iter::Iter<str>", align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %load1 = load %"std::iter::Iter<str>", ptr %load, align 8
  store %"std::iter::Iter<str>" %load1, ptr %4, align 8
  %field_load = getelementptr inbounds nuw %"std::iter::Iter<str>", ptr %4, i32 0, i32 0
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

define %enumRef @"str::parse"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %funcRef, align 8
  %10 = alloca %funcRef, align 8
  %11 = alloca %captures.20, align 8
  %12 = alloca %funcRef, align 8
  %13 = alloca %funcRef, align 8
  %14 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @str_parse, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %4, align 8
  %load2 = load ptr, ptr %2, align 8
  %field_ptr3 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  store ptr @"$type_id", ptr %field_ptr3, align 8
  %field_ptr4 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 1
  store ptr null, ptr %field_ptr4, align 8
  %load5 = load %funcRef, ptr %5, align 8
  store %funcRef %load5, ptr %6, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  %load6 = load ptr, ptr %field_load, align 8
  store %funcRef %load5, ptr %7, align 8
  %field_load7 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  %load8 = load ptr, ptr %field_load7, align 8
  %name = call i64 %load6(ptr %load8)
  store %funcRef %load, ptr %8, align 8
  %field_load9 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 0
  %load10 = load ptr, ptr %field_load9, align 8
  store %funcRef %load, ptr %9, align 8
  %field_load11 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 1
  %load12 = load ptr, ptr %field_load11, align 8
  %name13 = call %enumRef %load10(ptr %load2, i64 %name, ptr %load12)
  %field_ptr14 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 0
  store ptr @"Option::map", ptr %field_ptr14, align 8
  %field_ptr15 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 1
  store ptr null, ptr %field_ptr15, align 8
  %load16 = load %funcRef, ptr %10, align 8
  %load17 = load %captures.20, ptr %11, align 1
  %name18 = call ptr @margarineAlloc(i64 0)
  store %captures.20 %load17, ptr %name18, align 1
  %field_ptr19 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 0
  store ptr @"<closure>.28", ptr %field_ptr19, align 8
  %field_ptr20 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 1
  store ptr %name18, ptr %field_ptr20, align 8
  %load21 = load %funcRef, ptr %12, align 8
  store %funcRef %load16, ptr %13, align 8
  %field_load22 = getelementptr inbounds nuw %funcRef, ptr %13, i32 0, i32 0
  %load23 = load ptr, ptr %field_load22, align 8
  store %funcRef %load16, ptr %14, align 8
  %field_load24 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 1
  %load25 = load ptr, ptr %field_load24, align 8
  %name26 = call %enumRef %load23(%enumRef %name13, %funcRef %load21, ptr %load25)
  ret %enumRef %name26
}

declare %enumRef @str_parse(ptr, i64, ptr)

define i64 @"$type_id"() {
prelude:
  br label %entry

entry:                                            ; preds = %prelude
  ret i64 1
}

define %enumRef @"Option::map"(%enumRef %0, %funcRef %1, ptr %2) {
prelude:
  %3 = alloca %enumRef, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %enumRef, align 8
  %7 = alloca %enumRef, align 8
  %8 = alloca %anyType, align 8
  %9 = alloca %enumRef, align 8
  %10 = alloca %funcRef, align 8
  %11 = alloca %funcRef, align 8
  %12 = alloca %funcRef, align 8
  %13 = alloca %funcRef, align 8
  %14 = alloca %funcRef, align 8
  %15 = alloca {}, align 8
  %16 = alloca %enumRef, align 8
  %17 = alloca %funcRef, align 8
  %18 = alloca %funcRef, align 8
  %19 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store %enumRef %0, ptr %3, align 8
  store %funcRef %1, ptr %4, align 8
  store ptr %2, ptr %5, align 8
  %load = load %enumRef, ptr %3, align 8
  store %enumRef %load, ptr %6, align 8
  %field_load = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 1
  %load1 = load i32, ptr %field_load, align 4
  switch i32 %load1, label %switch_end [
    i32 0, label %switch_br
    i32 1, label %switch_br18
  ]

switch_end:                                       ; preds = %switch_br18, %switch_br, %entry
  %load30 = load %enumRef, ptr %7, align 8
  ret %enumRef %load30

switch_br:                                        ; preds = %entry
  store %enumRef %load, ptr %9, align 8
  %field_load2 = getelementptr inbounds nuw %enumRef, ptr %9, i32 0, i32 0
  %load3 = load ptr, ptr %field_load2, align 8
  %load4 = load %anyType, ptr %load3, align 8
  store %anyType %load4, ptr %8, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 0
  store ptr @Option.9, ptr %field_ptr, align 8
  %field_ptr5 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 1
  store ptr null, ptr %field_ptr5, align 8
  %load6 = load %funcRef, ptr %10, align 8
  %load7 = load %funcRef, ptr %4, align 8
  %load8 = load %anyType, ptr %8, align 8
  store %funcRef %load7, ptr %11, align 8
  %field_load9 = getelementptr inbounds nuw %funcRef, ptr %11, i32 0, i32 0
  %load10 = load ptr, ptr %field_load9, align 8
  store %funcRef %load7, ptr %12, align 8
  %field_load11 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 1
  %load12 = load ptr, ptr %field_load11, align 8
  %name = call i64 %load10(%anyType %load8, ptr %load12)
  store %funcRef %load6, ptr %13, align 8
  %field_load13 = getelementptr inbounds nuw %funcRef, ptr %13, i32 0, i32 0
  %load14 = load ptr, ptr %field_load13, align 8
  store %funcRef %load6, ptr %14, align 8
  %field_load15 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 1
  %load16 = load ptr, ptr %field_load15, align 8
  %name17 = call %enumRef %load14(i64 %name, ptr %load16)
  store %enumRef %name17, ptr %7, align 8
  br label %switch_end

switch_br18:                                      ; preds = %entry
  store %enumRef %load, ptr %16, align 8
  %field_load19 = getelementptr inbounds nuw %enumRef, ptr %16, i32 0, i32 0
  %load20 = load ptr, ptr %field_load19, align 8
  %load21 = load {}, ptr %load20, align 1
  store {} %load21, ptr %15, align 1
  %field_ptr22 = getelementptr inbounds nuw %funcRef, ptr %17, i32 0, i32 0
  store ptr @Option.10, ptr %field_ptr22, align 8
  %field_ptr23 = getelementptr inbounds nuw %funcRef, ptr %17, i32 0, i32 1
  store ptr null, ptr %field_ptr23, align 8
  %load24 = load %funcRef, ptr %17, align 8
  store %funcRef %load24, ptr %18, align 8
  %field_load25 = getelementptr inbounds nuw %funcRef, ptr %18, i32 0, i32 0
  %load26 = load ptr, ptr %field_load25, align 8
  store %funcRef %load24, ptr %19, align 8
  %field_load27 = getelementptr inbounds nuw %funcRef, ptr %19, i32 0, i32 1
  %load28 = load ptr, ptr %field_load27, align 8
  %name29 = call %enumRef %load26(ptr %load28)
  store %enumRef %name29, ptr %7, align 8
  br label %switch_end
}

define i64 @"<closure>.28"(%anyType %0, ptr %1) {
prelude:
  %2 = alloca %anyType, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %enumRef, align 8
  %8 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store %anyType %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %3, align 8
  %load1 = load %captures.20, ptr %load, align 1
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @"$downcast_any.29", ptr %field_ptr, align 8
  %field_ptr2 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr2, align 8
  %load3 = load %funcRef, ptr %4, align 8
  %load4 = load %anyType, ptr %2, align 8
  store %funcRef %load3, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load5 = load ptr, ptr %field_load, align 8
  store %funcRef %load3, ptr %6, align 8
  %field_load6 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load7 = load ptr, ptr %field_load6, align 8
  %name = call %enumRef %load5(%anyType %load4, ptr %load7)
  store %enumRef %name, ptr %7, align 8
  %field_load8 = getelementptr inbounds nuw %enumRef, ptr %7, i32 0, i32 1
  %load9 = load i32, ptr %field_load8, align 4
  %icmp = icmp eq i32 %load9, 0
  br i1 %icmp, label %then, label %else

then:                                             ; preds = %entry
  br label %cont

else:                                             ; preds = %entry
  call void @margarineAbort()
  br label %cont

cont:                                             ; preds = %else, %then
  store %enumRef %name, ptr %8, align 8
  %field_load10 = getelementptr inbounds nuw %enumRef, ptr %8, i32 0, i32 0
  %load11 = load ptr, ptr %field_load10, align 8
  %load12 = load i64, ptr %load11, align 4
  ret i64 %load12
}

define %enumRef @"$downcast_any.29"(%anyType %0, ptr %1) {
prelude:
  %2 = alloca %anyType, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %anyType, align 8
  %5 = alloca %anyType, align 8
  %6 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store %anyType %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load %anyType, ptr %2, align 8
  store %anyType %load, ptr %4, align 8
  %field_load = getelementptr inbounds nuw %anyType, ptr %4, i32 0, i32 1
  %load1 = load i32, ptr %field_load, align 4
  %icmp = icmp ne i32 1, %load1
  %icast = zext i1 %icmp to i32
  store %anyType %load, ptr %5, align 8
  %field_load2 = getelementptr inbounds nuw %anyType, ptr %5, i32 0, i32 0
  %load3 = load ptr, ptr %field_load2, align 8
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 0
  store ptr %load3, ptr %field_ptr, align 8
  %field_ptr4 = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 1
  store i32 %icast, ptr %field_ptr4, align 4
  %load5 = load %enumRef, ptr %6, align 8
  ret %enumRef %load5
}

define {} @"std::println.31"(double %0, ptr %1) {
prelude:
  %2 = alloca double, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store double %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @"std::print.32", ptr %field_ptr, align 8
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
  %name = call {} %load3(double %load2, ptr %load5)
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
  %name13 = call {} %load10(ptr @str.34, ptr %load12)
  ret {} zeroinitializer
}

define {} @"std::print.32"(double %0, ptr %1) {
prelude:
  %2 = alloca double, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store double %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @print_raw, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %4, align 8
  %field_ptr2 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  store ptr @"$any.33", ptr %field_ptr2, align 8
  %field_ptr3 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 1
  store ptr null, ptr %field_ptr3, align 8
  %load4 = load %funcRef, ptr %5, align 8
  %load5 = load double, ptr %2, align 8
  store %funcRef %load4, ptr %6, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  %load6 = load ptr, ptr %field_load, align 8
  store %funcRef %load4, ptr %7, align 8
  %field_load7 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  %load8 = load ptr, ptr %field_load7, align 8
  %name = call %anyType %load6(double %load5, ptr %load8)
  store %funcRef %load, ptr %8, align 8
  %field_load9 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 0
  %load10 = load ptr, ptr %field_load9, align 8
  store %funcRef %load, ptr %9, align 8
  %field_load11 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 1
  %load12 = load ptr, ptr %field_load11, align 8
  %name13 = call {} %load10(%anyType %name, ptr %load12)
  ret {} zeroinitializer
}

define %anyType @"$any.33"(double %0, ptr %1) {
prelude:
  %2 = alloca double, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %anyType, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store double %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %name = call ptr @margarineAlloc(i64 8)
  %load = load double, ptr %2, align 8
  store double %load, ptr %name, align 8
  %field_ptr = getelementptr inbounds nuw %anyType, ptr %4, i32 0, i32 0
  store ptr %name, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %anyType, ptr %4, i32 0, i32 1
  store i32 2, ptr %field_ptr1, align 4
  %load2 = load %anyType, ptr %4, align 8
  ret %anyType %load2
}

define %enumRef @"str::parse.36"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %funcRef, align 8
  %10 = alloca %funcRef, align 8
  %11 = alloca %captures.24, align 8
  %12 = alloca %funcRef, align 8
  %13 = alloca %funcRef, align 8
  %14 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @str_parse, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %4, align 8
  %load2 = load ptr, ptr %2, align 8
  %field_ptr3 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  store ptr @"$type_id.37", ptr %field_ptr3, align 8
  %field_ptr4 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 1
  store ptr null, ptr %field_ptr4, align 8
  %load5 = load %funcRef, ptr %5, align 8
  store %funcRef %load5, ptr %6, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  %load6 = load ptr, ptr %field_load, align 8
  store %funcRef %load5, ptr %7, align 8
  %field_load7 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  %load8 = load ptr, ptr %field_load7, align 8
  %name = call i64 %load6(ptr %load8)
  store %funcRef %load, ptr %8, align 8
  %field_load9 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 0
  %load10 = load ptr, ptr %field_load9, align 8
  store %funcRef %load, ptr %9, align 8
  %field_load11 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 1
  %load12 = load ptr, ptr %field_load11, align 8
  %name13 = call %enumRef %load10(ptr %load2, i64 %name, ptr %load12)
  %field_ptr14 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 0
  store ptr @"Option::map.38", ptr %field_ptr14, align 8
  %field_ptr15 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 1
  store ptr null, ptr %field_ptr15, align 8
  %load16 = load %funcRef, ptr %10, align 8
  %load17 = load %captures.24, ptr %11, align 1
  %name18 = call ptr @margarineAlloc(i64 0)
  store %captures.24 %load17, ptr %name18, align 1
  %field_ptr19 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 0
  store ptr @"<closure>.41", ptr %field_ptr19, align 8
  %field_ptr20 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 1
  store ptr %name18, ptr %field_ptr20, align 8
  %load21 = load %funcRef, ptr %12, align 8
  store %funcRef %load16, ptr %13, align 8
  %field_load22 = getelementptr inbounds nuw %funcRef, ptr %13, i32 0, i32 0
  %load23 = load ptr, ptr %field_load22, align 8
  store %funcRef %load16, ptr %14, align 8
  %field_load24 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 1
  %load25 = load ptr, ptr %field_load24, align 8
  %name26 = call %enumRef %load23(%enumRef %name13, %funcRef %load21, ptr %load25)
  ret %enumRef %name26
}

define i64 @"$type_id.37"() {
prelude:
  br label %entry

entry:                                            ; preds = %prelude
  ret i64 2
}

define %enumRef @"Option::map.38"(%enumRef %0, %funcRef %1, ptr %2) {
prelude:
  %3 = alloca %enumRef, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %enumRef, align 8
  %7 = alloca %enumRef, align 8
  %8 = alloca %anyType, align 8
  %9 = alloca %enumRef, align 8
  %10 = alloca %funcRef, align 8
  %11 = alloca %funcRef, align 8
  %12 = alloca %funcRef, align 8
  %13 = alloca %funcRef, align 8
  %14 = alloca %funcRef, align 8
  %15 = alloca {}, align 8
  %16 = alloca %enumRef, align 8
  %17 = alloca %funcRef, align 8
  %18 = alloca %funcRef, align 8
  %19 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store %enumRef %0, ptr %3, align 8
  store %funcRef %1, ptr %4, align 8
  store ptr %2, ptr %5, align 8
  %load = load %enumRef, ptr %3, align 8
  store %enumRef %load, ptr %6, align 8
  %field_load = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 1
  %load1 = load i32, ptr %field_load, align 4
  switch i32 %load1, label %switch_end [
    i32 0, label %switch_br
    i32 1, label %switch_br18
  ]

switch_end:                                       ; preds = %switch_br18, %switch_br, %entry
  %load30 = load %enumRef, ptr %7, align 8
  ret %enumRef %load30

switch_br:                                        ; preds = %entry
  store %enumRef %load, ptr %9, align 8
  %field_load2 = getelementptr inbounds nuw %enumRef, ptr %9, i32 0, i32 0
  %load3 = load ptr, ptr %field_load2, align 8
  %load4 = load %anyType, ptr %load3, align 8
  store %anyType %load4, ptr %8, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 0
  store ptr @Option.39, ptr %field_ptr, align 8
  %field_ptr5 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 1
  store ptr null, ptr %field_ptr5, align 8
  %load6 = load %funcRef, ptr %10, align 8
  %load7 = load %funcRef, ptr %4, align 8
  %load8 = load %anyType, ptr %8, align 8
  store %funcRef %load7, ptr %11, align 8
  %field_load9 = getelementptr inbounds nuw %funcRef, ptr %11, i32 0, i32 0
  %load10 = load ptr, ptr %field_load9, align 8
  store %funcRef %load7, ptr %12, align 8
  %field_load11 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 1
  %load12 = load ptr, ptr %field_load11, align 8
  %name = call double %load10(%anyType %load8, ptr %load12)
  store %funcRef %load6, ptr %13, align 8
  %field_load13 = getelementptr inbounds nuw %funcRef, ptr %13, i32 0, i32 0
  %load14 = load ptr, ptr %field_load13, align 8
  store %funcRef %load6, ptr %14, align 8
  %field_load15 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 1
  %load16 = load ptr, ptr %field_load15, align 8
  %name17 = call %enumRef %load14(double %name, ptr %load16)
  store %enumRef %name17, ptr %7, align 8
  br label %switch_end

switch_br18:                                      ; preds = %entry
  store %enumRef %load, ptr %16, align 8
  %field_load19 = getelementptr inbounds nuw %enumRef, ptr %16, i32 0, i32 0
  %load20 = load ptr, ptr %field_load19, align 8
  %load21 = load {}, ptr %load20, align 1
  store {} %load21, ptr %15, align 1
  %field_ptr22 = getelementptr inbounds nuw %funcRef, ptr %17, i32 0, i32 0
  store ptr @Option.40, ptr %field_ptr22, align 8
  %field_ptr23 = getelementptr inbounds nuw %funcRef, ptr %17, i32 0, i32 1
  store ptr null, ptr %field_ptr23, align 8
  %load24 = load %funcRef, ptr %17, align 8
  store %funcRef %load24, ptr %18, align 8
  %field_load25 = getelementptr inbounds nuw %funcRef, ptr %18, i32 0, i32 0
  %load26 = load ptr, ptr %field_load25, align 8
  store %funcRef %load24, ptr %19, align 8
  %field_load27 = getelementptr inbounds nuw %funcRef, ptr %19, i32 0, i32 1
  %load28 = load ptr, ptr %field_load27, align 8
  %name29 = call %enumRef %load26(ptr %load28)
  store %enumRef %name29, ptr %7, align 8
  br label %switch_end
}

define %enumRef @Option.39(double %0, ptr %1) {
prelude:
  %2 = alloca double, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store double %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load double, ptr %2, align 8
  %name = call ptr @margarineAlloc(i64 8)
  store double %load, ptr %name, align 8
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %4, i32 0, i32 0
  store ptr %name, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %enumRef, ptr %4, i32 0, i32 1
  store i32 0, ptr %field_ptr1, align 4
  %load2 = load %enumRef, ptr %4, align 8
  ret %enumRef %load2
}

define %enumRef @Option.40(ptr %0) {
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

define double @"<closure>.41"(%anyType %0, ptr %1) {
prelude:
  %2 = alloca %anyType, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %enumRef, align 8
  %8 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store %anyType %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %3, align 8
  %load1 = load %captures.24, ptr %load, align 1
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @"$downcast_any.42", ptr %field_ptr, align 8
  %field_ptr2 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr2, align 8
  %load3 = load %funcRef, ptr %4, align 8
  %load4 = load %anyType, ptr %2, align 8
  store %funcRef %load3, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load5 = load ptr, ptr %field_load, align 8
  store %funcRef %load3, ptr %6, align 8
  %field_load6 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load7 = load ptr, ptr %field_load6, align 8
  %name = call %enumRef %load5(%anyType %load4, ptr %load7)
  store %enumRef %name, ptr %7, align 8
  %field_load8 = getelementptr inbounds nuw %enumRef, ptr %7, i32 0, i32 1
  %load9 = load i32, ptr %field_load8, align 4
  %icmp = icmp eq i32 %load9, 0
  br i1 %icmp, label %then, label %else

then:                                             ; preds = %entry
  br label %cont

else:                                             ; preds = %entry
  call void @margarineAbort()
  br label %cont

cont:                                             ; preds = %else, %then
  store %enumRef %name, ptr %8, align 8
  %field_load10 = getelementptr inbounds nuw %enumRef, ptr %8, i32 0, i32 0
  %load11 = load ptr, ptr %field_load10, align 8
  %load12 = load double, ptr %load11, align 8
  ret double %load12
}

define %enumRef @"$downcast_any.42"(%anyType %0, ptr %1) {
prelude:
  %2 = alloca %anyType, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %anyType, align 8
  %5 = alloca %anyType, align 8
  %6 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store %anyType %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load %anyType, ptr %2, align 8
  store %anyType %load, ptr %4, align 8
  %field_load = getelementptr inbounds nuw %anyType, ptr %4, i32 0, i32 1
  %load1 = load i32, ptr %field_load, align 4
  %icmp = icmp ne i32 2, %load1
  %icast = zext i1 %icmp to i32
  store %anyType %load, ptr %5, align 8
  %field_load2 = getelementptr inbounds nuw %anyType, ptr %5, i32 0, i32 0
  %load3 = load ptr, ptr %field_load2, align 8
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 0
  store ptr %load3, ptr %field_ptr, align 8
  %field_ptr4 = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 1
  store i32 %icast, ptr %field_ptr4, align 4
  %load5 = load %enumRef, ptr %6, align 8
  ret %enumRef %load5
}

define i64 @"$sizeof"() {
prelude:
  br label %entry

entry:                                            ; preds = %prelude
  ret i64 8
}

define %enumRef @"List::pop"(ptr %0, ptr %1) {
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
  store ptr @list_pop, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %4, align 8
  %load2 = load ptr, ptr %2, align 8
  %field_ptr3 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  store ptr @"$sizeof", ptr %field_ptr3, align 8
  %field_ptr4 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 1
  store ptr null, ptr %field_ptr4, align 8
  %load5 = load %funcRef, ptr %5, align 8
  store %funcRef %load5, ptr %6, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  %load6 = load ptr, ptr %field_load, align 8
  store %funcRef %load5, ptr %7, align 8
  %field_load7 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  %load8 = load ptr, ptr %field_load7, align 8
  %name = call i64 %load6(ptr %load8)
  store %funcRef %load, ptr %8, align 8
  %field_load9 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 0
  %load10 = load ptr, ptr %field_load9, align 8
  store %funcRef %load, ptr %9, align 8
  %field_load11 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 1
  %load12 = load ptr, ptr %field_load11, align 8
  %name13 = call %enumRef %load10(ptr %load2, i64 %name, ptr %load12)
  ret %enumRef %name13
}

declare %enumRef @list_pop(ptr, i64, ptr)

define {} @"List::push"(ptr %0, i64 %1, ptr %2) {
prelude:
  %3 = alloca ptr, align 8
  %4 = alloca i64, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %funcRef, align 8
  %10 = alloca %funcRef, align 8
  %11 = alloca %funcRef, align 8
  %12 = alloca %funcRef, align 8
  %13 = alloca %funcRef, align 8
  %14 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %3, align 8
  store i64 %1, ptr %4, align 4
  store ptr %2, ptr %5, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  store ptr @list_push, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %6, align 8
  %load2 = load ptr, ptr %3, align 8
  %field_ptr3 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  store ptr @"$any.14", ptr %field_ptr3, align 8
  %field_ptr4 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  store ptr null, ptr %field_ptr4, align 8
  %load5 = load %funcRef, ptr %7, align 8
  %load6 = load i64, ptr %4, align 4
  store %funcRef %load5, ptr %8, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 0
  %load7 = load ptr, ptr %field_load, align 8
  store %funcRef %load5, ptr %9, align 8
  %field_load8 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 1
  %load9 = load ptr, ptr %field_load8, align 8
  %name = call %anyType %load7(i64 %load6, ptr %load9)
  %field_ptr10 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 0
  store ptr @"$sizeof", ptr %field_ptr10, align 8
  %field_ptr11 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 1
  store ptr null, ptr %field_ptr11, align 8
  %load12 = load %funcRef, ptr %10, align 8
  store %funcRef %load12, ptr %11, align 8
  %field_load13 = getelementptr inbounds nuw %funcRef, ptr %11, i32 0, i32 0
  %load14 = load ptr, ptr %field_load13, align 8
  store %funcRef %load12, ptr %12, align 8
  %field_load15 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 1
  %load16 = load ptr, ptr %field_load15, align 8
  %name17 = call i64 %load14(ptr %load16)
  store %funcRef %load, ptr %13, align 8
  %field_load18 = getelementptr inbounds nuw %funcRef, ptr %13, i32 0, i32 0
  %load19 = load ptr, ptr %field_load18, align 8
  store %funcRef %load, ptr %14, align 8
  %field_load20 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 1
  %load21 = load ptr, ptr %field_load20, align 8
  %name22 = call {} %load19(ptr %load2, %anyType %name, i64 %name17, ptr %load21)
  ret {} %name22
}

declare {} @list_push(ptr, %anyType, i64, ptr)

attributes #0 = { noreturn }
