; ModuleID = 'margarine'
source_filename = "margarine"

%str = type <{ i32, [0 x i8] }>
%str.6 = type <{ i32, [1 x i8] }>
%str.8 = type <{ i32, [27 x i8] }>
%str.9 = type <{ i32, [1 x i8] }>
%str.14 = type <{ i32, [1 x i8] }>
%str.18 = type <{ i32, [1 x i8] }>
%str.20 = type <{ i32, [1 x i8] }>
%lexer_err_ty = type { i32, ptr }
%parser_err_ty = type { i32, ptr }
%funcRef = type { ptr, ptr }
%captures.0 = type { ptr }
%"(int)" = type { i64 }
%enumRef = type { ptr, i32 }
%captures = type { %funcRef, ptr }
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
%captures.11 = type {}
%captures.13 = type {}
%captures.10 = type { %funcRef, ptr }
%captures.12 = type { %funcRef, ptr }
%captures.17 = type {}
%captures.15 = type {}
%captures.16 = type { %funcRef, ptr }
%captures.19 = type {}
%captures.21 = type {}

@str = global %str zeroinitializer
@str.8 = global %str.6 <{ i32 1, [1 x i8] c"\0A" }>
@str.12 = global %str.8 <{ i32 27, [27 x i8] c"./examples/aoc2025/day1.txt" }>
@str.13 = global %str.9 <{ i32 1, [1 x i8] c"," }>
@str.20 = global %str.14 <{ i32 1, [1 x i8] c"-" }>
@str.25 = global %str.18 <{ i32 1, [1 x i8] c"," }>
@str.27 = global %str.20 <{ i32 1, [1 x i8] c"-" }>
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
@14 = global [488 x i8] c"\1B[38;2;255;0;0;1merror\1B[0m: \1B[38;2;255;255;255;1minvalid type\1B[0m\0A    \1B[38;2;255;165;0m\E2\94\8F\E2\94\80\E2\96\B6\1B[0m /Users/macbook/Desktop/projects/programming languages/untitled compiler/examples/aoc2025/day1.mar:8:18\0A    \1B[38;2;255;165;0m\E2\94\83\1B[0m \0A\1B[38;2;255;165;0m8\1B[0m   \1B[38;2;255;165;0m\E2\94\83\1B[0m     println(part1(input));\0A    \1B[38;2;255;165;0m\E2\94\83\1B[0m                   \1B[38;2;255;0;0m\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\1B[0m expected a value of type 'str' but found 'Result<str, str>'\0A   \1B[38;2;255;165;0m\E2\94\81\E2\94\BB\E2\94\81\1B[0m \0A"
@15 = global [488 x i8] c"\1B[38;2;255;0;0;1merror\1B[0m: \1B[38;2;255;255;255;1minvalid type\1B[0m\0A    \1B[38;2;255;165;0m\E2\94\8F\E2\94\80\E2\96\B6\1B[0m /Users/macbook/Desktop/projects/programming languages/untitled compiler/examples/aoc2025/day1.mar:9:18\0A    \1B[38;2;255;165;0m\E2\94\83\1B[0m \0A\1B[38;2;255;165;0m9\1B[0m   \1B[38;2;255;165;0m\E2\94\83\1B[0m     println(part2(input));\0A    \1B[38;2;255;165;0m\E2\94\83\1B[0m                   \1B[38;2;255;0;0m\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\1B[0m expected a value of type 'str' but found 'Result<str, str>'\0A   \1B[38;2;255;165;0m\E2\94\81\E2\94\BB\E2\94\81\1B[0m \0A"
@16 = global [506 x i8] c"\1B[38;2;255;0;0;1merror\1B[0m: \1B[38;2;255;255;255;1minvalid binary operation\1B[0m\0A    \1B[38;2;255;165;0m\E2\94\8F\E2\94\80\E2\96\B6\1B[0m /Users/macbook/Desktop/projects/programming languages/untitled compiler/examples/aoc2025/day1.mar:38:4\0A    \1B[38;2;255;165;0m\E2\94\83\1B[0m \0A\1B[38;2;255;165;0m38\1B[0m  \1B[38;2;255;165;0m\E2\94\83\1B[0m     left == right\0A    \1B[38;2;255;165;0m\E2\94\83\1B[0m     \1B[38;2;255;0;0m\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\1B[0m can't apply the binary op '==' between the types 'str' and 'str'\0A   \1B[38;2;255;165;0m\E2\94\81\E2\94\BB\E2\94\81\1B[0m \0A"
@17 = global [601 x i8] c"\1B[38;2;255;0;0;1merror\1B[0m: \1B[38;2;255;255;255;1minvalid binary operation\1B[0m\0A    \1B[38;2;255;165;0m\E2\94\8F\E2\94\80\E2\96\B6\1B[0m /Users/macbook/Desktop/projects/programming languages/untitled compiler/examples/aoc2025/day1.mar:54:22\0A    \1B[38;2;255;165;0m\E2\94\83\1B[0m \0A\1B[38;2;255;165;0m54\1B[0m  \1B[38;2;255;165;0m\E2\94\83\1B[0m             if s.slice(i..(i+block_len)) != block {\0A    \1B[38;2;255;165;0m\E2\94\83\1B[0m                       \1B[38;2;255;0;0m\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\1B[0m can't apply the binary op '!=' between the types 'str' and 'str'\0A   \1B[38;2;255;165;0m\E2\94\81\E2\94\BB\E2\94\81\1B[0m \0A"
@semaErrors = global [4 x ptr] [ptr @14, ptr @15, ptr @16, ptr @17]
@semaErrorsLen = global i32 4

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
  %load10 = load ptr, ptr %4, align 8
  %load11 = load ptr, ptr %9, align 8
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
  %load5 = load ptr, ptr %5, align 8
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
  %load14 = load ptr, ptr %5, align 8
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
  %load25 = load ptr, ptr %3, align 8
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
  %load88 = load ptr, ptr %5, align 8
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
  %load66 = load ptr, ptr %5, align 8
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

define {} @main(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %funcRef, align 8
  %3 = alloca %funcRef, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %enumRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %2, i32 0, i32 0
  store ptr @io_read_file, ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %2, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %2, align 8
  store %funcRef %load, ptr %3, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %3, i32 0, i32 0
  %load2 = load ptr, ptr %field_load, align 8
  store %funcRef %load, ptr %4, align 8
  %field_load3 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  %load4 = load ptr, ptr %field_load3, align 8
  %name = call %enumRef %load2(ptr @str.12, ptr %load4)
  store %enumRef %name, ptr %5, align 8
  %field_ptr5 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  store ptr @"std::println", ptr %field_ptr5, align 8
  %field_ptr6 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  store ptr null, ptr %field_ptr6, align 8
  %load7 = load %funcRef, ptr %6, align 8
  %field_ptr8 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  store ptr @part1, ptr %field_ptr8, align 8
  %field_ptr9 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  store ptr null, ptr %field_ptr9, align 8
  %load10 = load %funcRef, ptr %7, align 8
  call void @margarineError(i32 2, i32 0, i32 0)
  unreachable
}

declare %enumRef @io_read_file(ptr, ptr)

define ptr @part1(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %captures.11, align 8
  %9 = alloca %funcRef, align 8
  %10 = alloca %funcRef, align 8
  %11 = alloca %funcRef, align 8
  %12 = alloca %funcRef, align 8
  %13 = alloca %captures.13, align 8
  %14 = alloca %funcRef, align 8
  %15 = alloca %funcRef, align 8
  %16 = alloca %funcRef, align 8
  %17 = alloca %funcRef, align 8
  %18 = alloca %funcRef, align 8
  %19 = alloca %funcRef, align 8
  %20 = alloca %funcRef, align 8
  %21 = alloca %funcRef, align 8
  %22 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @"str::split", ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load2 = load %funcRef, ptr %4, align 8
  store %funcRef %load2, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load3 = load ptr, ptr %field_load, align 8
  store %funcRef %load2, ptr %6, align 8
  %field_load4 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load5 = load ptr, ptr %field_load4, align 8
  %name = call ptr %load3(ptr %load, ptr @str.13, ptr %load5)
  %field_ptr6 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  store ptr @"std::iter::Iter::filter", ptr %field_ptr6, align 8
  %field_ptr7 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  store ptr null, ptr %field_ptr7, align 8
  %load8 = load %funcRef, ptr %7, align 8
  %load9 = load %captures.11, ptr %8, align 1
  %name10 = call ptr @margarineAlloc(i64 0)
  store %captures.11 %load9, ptr %name10, align 1
  %field_ptr11 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 0
  store ptr @"<closure>.16", ptr %field_ptr11, align 8
  %field_ptr12 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 1
  store ptr %name10, ptr %field_ptr12, align 8
  %load13 = load %funcRef, ptr %9, align 8
  store %funcRef %load8, ptr %10, align 8
  %field_load14 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 0
  %load15 = load ptr, ptr %field_load14, align 8
  store %funcRef %load8, ptr %11, align 8
  %field_load16 = getelementptr inbounds nuw %funcRef, ptr %11, i32 0, i32 1
  %load17 = load ptr, ptr %field_load16, align 8
  %name18 = call ptr %load15(ptr %name, %funcRef %load13, ptr %load17)
  %field_ptr19 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 0
  store ptr @"std::iter::Iter::map.17", ptr %field_ptr19, align 8
  %field_ptr20 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 1
  store ptr null, ptr %field_ptr20, align 8
  %load21 = load %funcRef, ptr %12, align 8
  %load22 = load %captures.13, ptr %13, align 1
  %name23 = call ptr @margarineAlloc(i64 0)
  store %captures.13 %load22, ptr %name23, align 1
  %field_ptr24 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 0
  store ptr @"<closure>.19", ptr %field_ptr24, align 8
  %field_ptr25 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 1
  store ptr %name23, ptr %field_ptr25, align 8
  %load26 = load %funcRef, ptr %14, align 8
  store %funcRef %load21, ptr %15, align 8
  %field_load27 = getelementptr inbounds nuw %funcRef, ptr %15, i32 0, i32 0
  %load28 = load ptr, ptr %field_load27, align 8
  store %funcRef %load21, ptr %16, align 8
  %field_load29 = getelementptr inbounds nuw %funcRef, ptr %16, i32 0, i32 1
  %load30 = load ptr, ptr %field_load29, align 8
  %name31 = call ptr %load28(ptr %name18, %funcRef %load26, ptr %load30)
  %field_ptr32 = getelementptr inbounds nuw %funcRef, ptr %17, i32 0, i32 0
  store ptr @"std::iter::Iter::sum", ptr %field_ptr32, align 8
  %field_ptr33 = getelementptr inbounds nuw %funcRef, ptr %17, i32 0, i32 1
  store ptr null, ptr %field_ptr33, align 8
  %load34 = load %funcRef, ptr %17, align 8
  store %funcRef %load34, ptr %18, align 8
  %field_load35 = getelementptr inbounds nuw %funcRef, ptr %18, i32 0, i32 0
  %load36 = load ptr, ptr %field_load35, align 8
  store %funcRef %load34, ptr %19, align 8
  %field_load37 = getelementptr inbounds nuw %funcRef, ptr %19, i32 0, i32 1
  %load38 = load ptr, ptr %field_load37, align 8
  %name39 = call i64 %load36(ptr %name31, ptr %load38)
  %field_ptr40 = getelementptr inbounds nuw %funcRef, ptr %20, i32 0, i32 0
  store ptr @"int::to_str", ptr %field_ptr40, align 8
  %field_ptr41 = getelementptr inbounds nuw %funcRef, ptr %20, i32 0, i32 1
  store ptr null, ptr %field_ptr41, align 8
  %load42 = load %funcRef, ptr %20, align 8
  store %funcRef %load42, ptr %21, align 8
  %field_load43 = getelementptr inbounds nuw %funcRef, ptr %21, i32 0, i32 0
  %load44 = load ptr, ptr %field_load43, align 8
  store %funcRef %load42, ptr %22, align 8
  %field_load45 = getelementptr inbounds nuw %funcRef, ptr %22, i32 0, i32 1
  %load46 = load ptr, ptr %field_load45, align 8
  %name47 = call ptr %load44(i64 %name39, ptr %load46)
  ret ptr %name47
}

define ptr @"std::iter::Iter::filter"(ptr %0, %funcRef %1, ptr %2) {
prelude:
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %captures.10, align 8
  %7 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %3, align 8
  store %funcRef %1, ptr %4, align 8
  store ptr %2, ptr %5, align 8
  %load = load %funcRef, ptr %4, align 8
  %load1 = load ptr, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %captures.10, ptr %6, i32 0, i32 0
  store %funcRef %load, ptr %field_ptr, align 8
  %field_ptr2 = getelementptr inbounds nuw %captures.10, ptr %6, i32 0, i32 1
  store ptr %load1, ptr %field_ptr2, align 8
  %load3 = load %captures.10, ptr %6, align 8
  %name = call ptr @margarineAlloc(i64 24)
  store %captures.10 %load3, ptr %name, align 8
  %field_ptr4 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  store ptr @"<closure>.14", ptr %field_ptr4, align 8
  %field_ptr5 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  store ptr %name, ptr %field_ptr5, align 8
  %load6 = load %funcRef, ptr %7, align 8
  %name7 = call ptr @margarineAlloc(i64 16)
  %field_ptr8 = getelementptr inbounds nuw %"std::iter::Iter<str>", ptr %name7, i32 0, i32 0
  store %funcRef %load6, ptr %field_ptr8, align 8
  ret ptr %name7
}

define %enumRef @"<closure>.14"(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %captures.10, align 8
  %3 = alloca %funcRef, align 8
  %4 = alloca %captures.10, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %enumRef, align 8
  %10 = alloca %enumRef, align 8
  %11 = alloca ptr, align 8
  %12 = alloca %funcRef, align 8
  %13 = alloca %funcRef, align 8
  %14 = alloca {}, align 8
  %15 = alloca %enumRef, align 8
  %16 = alloca %funcRef, align 8
  %17 = alloca %funcRef, align 8
  %18 = alloca %funcRef, align 8
  %19 = alloca %funcRef, align 8
  %20 = alloca %funcRef, align 8
  %21 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %load = load ptr, ptr %1, align 8
  %load1 = load %captures.10, ptr %load, align 8
  store %captures.10 %load1, ptr %2, align 8
  %field_load = getelementptr inbounds nuw %captures.10, ptr %2, i32 0, i32 0
  %load2 = load %funcRef, ptr %field_load, align 8
  store %funcRef %load2, ptr %3, align 8
  store %captures.10 %load1, ptr %4, align 8
  %field_load3 = getelementptr inbounds nuw %captures.10, ptr %4, i32 0, i32 1
  %load4 = load ptr, ptr %field_load3, align 8
  store ptr %load4, ptr %5, align 8
  br label %loop_body

loop_body:                                        ; preds = %cont28, %entry
  %load5 = load ptr, ptr %5, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  store ptr @"std::iter::Iter::__next__.15", ptr %field_ptr, align 8
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

loop_cont:                                        ; No predecessors!
  %field_ptr39 = getelementptr inbounds nuw %funcRef, ptr %19, i32 0, i32 0
  store ptr @Option.6, ptr %field_ptr39, align 8
  %field_ptr40 = getelementptr inbounds nuw %funcRef, ptr %19, i32 0, i32 1
  store ptr null, ptr %field_ptr40, align 8
  %load41 = load %funcRef, ptr %19, align 8
  store %funcRef %load41, ptr %20, align 8
  %field_load42 = getelementptr inbounds nuw %funcRef, ptr %20, i32 0, i32 0
  %load43 = load ptr, ptr %field_load42, align 8
  store %funcRef %load41, ptr %21, align 8
  %field_load44 = getelementptr inbounds nuw %funcRef, ptr %21, i32 0, i32 1
  %load45 = load ptr, ptr %field_load44, align 8
  %name46 = call %enumRef %load43(ptr %load45)
  ret %enumRef %name46

then:                                             ; preds = %loop_body
  br label %cont

else:                                             ; preds = %loop_body
  ret %enumRef %name

cont:                                             ; preds = %22, %then
  %load16 = load ptr, ptr %load15, align 8
  store ptr %load16, ptr %11, align 8
  %load17 = load %funcRef, ptr %3, align 8
  %load18 = load ptr, ptr %11, align 8
  store %funcRef %load17, ptr %12, align 8
  %field_load19 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 0
  %load20 = load ptr, ptr %field_load19, align 8
  store %funcRef %load17, ptr %13, align 8
  %field_load21 = getelementptr inbounds nuw %funcRef, ptr %13, i32 0, i32 1
  %load22 = load ptr, ptr %field_load21, align 8
  %name23 = call %enumRef %load20(ptr %load18, ptr %load22)
  store %enumRef %name23, ptr %15, align 8
  %field_load24 = getelementptr inbounds nuw %enumRef, ptr %15, i32 0, i32 1
  %load25 = load i32, ptr %field_load24, align 4
  %icast = trunc i32 %load25 to i1
  br i1 %icast, label %then26, label %else27

22:                                               ; No predecessors!
  br label %cont

then26:                                           ; preds = %cont
  %field_ptr29 = getelementptr inbounds nuw %funcRef, ptr %16, i32 0, i32 0
  store ptr @Option.4, ptr %field_ptr29, align 8
  %field_ptr30 = getelementptr inbounds nuw %funcRef, ptr %16, i32 0, i32 1
  store ptr null, ptr %field_ptr30, align 8
  %load31 = load %funcRef, ptr %16, align 8
  %load32 = load ptr, ptr %11, align 8
  store %funcRef %load31, ptr %17, align 8
  %field_load33 = getelementptr inbounds nuw %funcRef, ptr %17, i32 0, i32 0
  %load34 = load ptr, ptr %field_load33, align 8
  store %funcRef %load31, ptr %18, align 8
  %field_load35 = getelementptr inbounds nuw %funcRef, ptr %18, i32 0, i32 1
  %load36 = load ptr, ptr %field_load35, align 8
  %name37 = call %enumRef %load34(ptr %load32, ptr %load36)
  ret %enumRef %name37

else27:                                           ; preds = %cont
  br label %cont28

cont28:                                           ; preds = %else27, %24
  %load38 = load {}, ptr %14, align 1
  br label %loop_body

23:                                               ; No predecessors!
  unreachable

24:                                               ; No predecessors!
  store {} zeroinitializer, ptr %14, align 1
  br label %cont28
}

define %enumRef @"std::iter::Iter::__next__.15"(ptr %0, ptr %1) {
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

define %enumRef @"<closure>.16"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %enumRef, align 8
  %8 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %3, align 8
  %load1 = load %captures.11, ptr %load, align 1
  %load2 = load ptr, ptr %2, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @"str::is_empty", ptr %field_ptr, align 8
  %field_ptr3 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr3, align 8
  %load4 = load %funcRef, ptr %4, align 8
  store %funcRef %load4, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load5 = load ptr, ptr %field_load, align 8
  store %funcRef %load4, ptr %6, align 8
  %field_load6 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load7 = load ptr, ptr %field_load6, align 8
  %name = call %enumRef %load5(ptr %load2, ptr %load7)
  store %enumRef %name, ptr %7, align 8
  %field_load8 = getelementptr inbounds nuw %enumRef, ptr %7, i32 0, i32 1
  %load9 = load i32, ptr %field_load8, align 4
  %icast = trunc i32 %load9 to i1
  %bnot = xor i1 %icast, true
  store %enumRef %name, ptr %8, align 8
  %field_ptr10 = getelementptr inbounds nuw %enumRef, ptr %8, i32 0, i32 1
  store i1 %bnot, ptr %field_ptr10, align 1
  %load11 = load %enumRef, ptr %field_ptr10, align 8
  ret %enumRef %load11
}

define ptr @"std::iter::Iter::map.17"(ptr %0, %funcRef %1, ptr %2) {
prelude:
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %captures.12, align 8
  %7 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %3, align 8
  store %funcRef %1, ptr %4, align 8
  store ptr %2, ptr %5, align 8
  %load = load %funcRef, ptr %4, align 8
  %load1 = load ptr, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %captures.12, ptr %6, i32 0, i32 0
  store %funcRef %load, ptr %field_ptr, align 8
  %field_ptr2 = getelementptr inbounds nuw %captures.12, ptr %6, i32 0, i32 1
  store ptr %load1, ptr %field_ptr2, align 8
  %load3 = load %captures.12, ptr %6, align 8
  %name = call ptr @margarineAlloc(i64 24)
  store %captures.12 %load3, ptr %name, align 8
  %field_ptr4 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  store ptr @"<closure>.18", ptr %field_ptr4, align 8
  %field_ptr5 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  store ptr %name, ptr %field_ptr5, align 8
  %load6 = load %funcRef, ptr %7, align 8
  %name7 = call ptr @margarineAlloc(i64 16)
  %field_ptr8 = getelementptr inbounds nuw %"std::iter::Iter<int>", ptr %name7, i32 0, i32 0
  store %funcRef %load6, ptr %field_ptr8, align 8
  ret ptr %name7
}

define %enumRef @"<closure>.18"(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %captures.12, align 8
  %3 = alloca %funcRef, align 8
  %4 = alloca %captures.12, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %enumRef, align 8
  %10 = alloca %enumRef, align 8
  %11 = alloca ptr, align 8
  %12 = alloca %funcRef, align 8
  %13 = alloca %funcRef, align 8
  %14 = alloca %funcRef, align 8
  %15 = alloca %funcRef, align 8
  %16 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %load = load ptr, ptr %1, align 8
  %load1 = load %captures.12, ptr %load, align 8
  store %captures.12 %load1, ptr %2, align 8
  %field_load = getelementptr inbounds nuw %captures.12, ptr %2, i32 0, i32 0
  %load2 = load %funcRef, ptr %field_load, align 8
  store %funcRef %load2, ptr %3, align 8
  store %captures.12 %load1, ptr %4, align 8
  %field_load3 = getelementptr inbounds nuw %captures.12, ptr %4, i32 0, i32 1
  %load4 = load ptr, ptr %field_load3, align 8
  store ptr %load4, ptr %5, align 8
  %load5 = load ptr, ptr %5, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 0
  store ptr @"std::iter::Iter::__next__.15", ptr %field_ptr, align 8
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
  %load16 = load ptr, ptr %load15, align 8
  store ptr %load16, ptr %11, align 8
  %field_ptr17 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 0
  store ptr @Option.9, ptr %field_ptr17, align 8
  %field_ptr18 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 1
  store ptr null, ptr %field_ptr18, align 8
  %load19 = load %funcRef, ptr %12, align 8
  %load20 = load %funcRef, ptr %3, align 8
  %load21 = load ptr, ptr %11, align 8
  store %funcRef %load20, ptr %13, align 8
  %field_load22 = getelementptr inbounds nuw %funcRef, ptr %13, i32 0, i32 0
  %load23 = load ptr, ptr %field_load22, align 8
  store %funcRef %load20, ptr %14, align 8
  %field_load24 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 1
  %load25 = load ptr, ptr %field_load24, align 8
  %name26 = call i64 %load23(ptr %load21, ptr %load25)
  store %funcRef %load19, ptr %15, align 8
  %field_load27 = getelementptr inbounds nuw %funcRef, ptr %15, i32 0, i32 0
  %load28 = load ptr, ptr %field_load27, align 8
  store %funcRef %load19, ptr %16, align 8
  %field_load29 = getelementptr inbounds nuw %funcRef, ptr %16, i32 0, i32 1
  %load30 = load ptr, ptr %field_load29, align 8
  %name31 = call %enumRef %load28(i64 %name26, ptr %load30)
  ret %enumRef %name31

17:                                               ; No predecessors!
  br label %cont
}

define i64 @"<closure>.19"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %enumRef, align 8
  %8 = alloca %enumRef, align 8
  %9 = alloca %"(str, str).4", align 8
  %10 = alloca ptr, align 8
  %11 = alloca %"(str, str).4", align 8
  %12 = alloca ptr, align 8
  %13 = alloca %funcRef, align 8
  %14 = alloca %funcRef, align 8
  %15 = alloca %funcRef, align 8
  %16 = alloca %enumRef, align 8
  %17 = alloca %enumRef, align 8
  %18 = alloca i64, align 8
  %19 = alloca %funcRef, align 8
  %20 = alloca %funcRef, align 8
  %21 = alloca %funcRef, align 8
  %22 = alloca %enumRef, align 8
  %23 = alloca %enumRef, align 8
  %24 = alloca i64, align 8
  %25 = alloca %funcRef, align 8
  %26 = alloca %funcRef, align 8
  %27 = alloca %funcRef, align 8
  %28 = alloca %funcRef, align 8
  %29 = alloca %captures.17, align 8
  %30 = alloca %funcRef, align 8
  %31 = alloca %funcRef, align 8
  %32 = alloca %funcRef, align 8
  %33 = alloca %funcRef, align 8
  %34 = alloca %funcRef, align 8
  %35 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %3, align 8
  %load1 = load %captures.13, ptr %load, align 1
  %load2 = load ptr, ptr %2, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @"str::split_once", ptr %field_ptr, align 8
  %field_ptr3 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr3, align 8
  %load4 = load %funcRef, ptr %4, align 8
  store %funcRef %load4, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load5 = load ptr, ptr %field_load, align 8
  store %funcRef %load4, ptr %6, align 8
  %field_load6 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load7 = load ptr, ptr %field_load6, align 8
  %name = call %enumRef %load5(ptr %load2, ptr @str.20, ptr %load7)
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
  %load12 = load ptr, ptr %load11, align 8
  %load13 = load %"(str, str).4", ptr %load12, align 8
  store %"(str, str).4" %load13, ptr %9, align 8
  %field_load14 = getelementptr inbounds nuw %"(str, str).4", ptr %9, i32 0, i32 0
  %load15 = load ptr, ptr %field_load14, align 8
  store ptr %load15, ptr %10, align 8
  store %"(str, str).4" %load13, ptr %11, align 8
  %field_load16 = getelementptr inbounds nuw %"(str, str).4", ptr %11, i32 0, i32 1
  %load17 = load ptr, ptr %field_load16, align 8
  store ptr %load17, ptr %12, align 8
  %load18 = load ptr, ptr %10, align 8
  %field_ptr19 = getelementptr inbounds nuw %funcRef, ptr %13, i32 0, i32 0
  store ptr @"str::parse", ptr %field_ptr19, align 8
  %field_ptr20 = getelementptr inbounds nuw %funcRef, ptr %13, i32 0, i32 1
  store ptr null, ptr %field_ptr20, align 8
  %load21 = load %funcRef, ptr %13, align 8
  store %funcRef %load21, ptr %14, align 8
  %field_load22 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 0
  %load23 = load ptr, ptr %field_load22, align 8
  store %funcRef %load21, ptr %15, align 8
  %field_load24 = getelementptr inbounds nuw %funcRef, ptr %15, i32 0, i32 1
  %load25 = load ptr, ptr %field_load24, align 8
  %name26 = call %enumRef %load23(ptr %load18, ptr %load25)
  store %enumRef %name26, ptr %16, align 8
  %field_load27 = getelementptr inbounds nuw %enumRef, ptr %16, i32 0, i32 1
  %load28 = load i32, ptr %field_load27, align 4
  %icmp29 = icmp eq i32 %load28, 0
  br i1 %icmp29, label %then30, label %else31

then30:                                           ; preds = %cont
  br label %cont32

else31:                                           ; preds = %cont
  call void @margarineAbort()
  br label %cont32

cont32:                                           ; preds = %else31, %then30
  store %enumRef %name26, ptr %17, align 8
  %field_load33 = getelementptr inbounds nuw %enumRef, ptr %17, i32 0, i32 0
  %load34 = load ptr, ptr %field_load33, align 8
  %load35 = load i64, ptr %load34, align 4
  store i64 %load35, ptr %18, align 4
  %load36 = load ptr, ptr %12, align 8
  %field_ptr37 = getelementptr inbounds nuw %funcRef, ptr %19, i32 0, i32 0
  store ptr @"str::parse", ptr %field_ptr37, align 8
  %field_ptr38 = getelementptr inbounds nuw %funcRef, ptr %19, i32 0, i32 1
  store ptr null, ptr %field_ptr38, align 8
  %load39 = load %funcRef, ptr %19, align 8
  store %funcRef %load39, ptr %20, align 8
  %field_load40 = getelementptr inbounds nuw %funcRef, ptr %20, i32 0, i32 0
  %load41 = load ptr, ptr %field_load40, align 8
  store %funcRef %load39, ptr %21, align 8
  %field_load42 = getelementptr inbounds nuw %funcRef, ptr %21, i32 0, i32 1
  %load43 = load ptr, ptr %field_load42, align 8
  %name44 = call %enumRef %load41(ptr %load36, ptr %load43)
  store %enumRef %name44, ptr %22, align 8
  %field_load45 = getelementptr inbounds nuw %enumRef, ptr %22, i32 0, i32 1
  %load46 = load i32, ptr %field_load45, align 4
  %icmp47 = icmp eq i32 %load46, 0
  br i1 %icmp47, label %then48, label %else49

then48:                                           ; preds = %cont32
  br label %cont50

else49:                                           ; preds = %cont32
  call void @margarineAbort()
  br label %cont50

cont50:                                           ; preds = %else49, %then48
  store %enumRef %name44, ptr %23, align 8
  %field_load51 = getelementptr inbounds nuw %enumRef, ptr %23, i32 0, i32 0
  %load52 = load ptr, ptr %field_load51, align 8
  %load53 = load i64, ptr %load52, align 4
  store i64 %load53, ptr %24, align 4
  %load54 = load i64, ptr %18, align 4
  %load55 = load i64, ptr %24, align 4
  %addi = add i64 %load55, 1
  %name56 = call ptr @margarineAlloc(i64 16)
  %field_ptr57 = getelementptr inbounds nuw %Range, ptr %name56, i32 0, i32 0
  store i64 %load54, ptr %field_ptr57, align 4
  %field_ptr58 = getelementptr inbounds nuw %Range, ptr %name56, i32 0, i32 1
  store i64 %addi, ptr %field_ptr58, align 4
  %field_ptr59 = getelementptr inbounds nuw %funcRef, ptr %25, i32 0, i32 0
  store ptr @"Range::iter", ptr %field_ptr59, align 8
  %field_ptr60 = getelementptr inbounds nuw %funcRef, ptr %25, i32 0, i32 1
  store ptr null, ptr %field_ptr60, align 8
  %load61 = load %funcRef, ptr %25, align 8
  store %funcRef %load61, ptr %26, align 8
  %field_load62 = getelementptr inbounds nuw %funcRef, ptr %26, i32 0, i32 0
  %load63 = load ptr, ptr %field_load62, align 8
  store %funcRef %load61, ptr %27, align 8
  %field_load64 = getelementptr inbounds nuw %funcRef, ptr %27, i32 0, i32 1
  %load65 = load ptr, ptr %field_load64, align 8
  %name66 = call ptr %load63(ptr %name56, ptr %load65)
  %field_ptr67 = getelementptr inbounds nuw %funcRef, ptr %28, i32 0, i32 0
  store ptr @"std::iter::Iter::filter.22", ptr %field_ptr67, align 8
  %field_ptr68 = getelementptr inbounds nuw %funcRef, ptr %28, i32 0, i32 1
  store ptr null, ptr %field_ptr68, align 8
  %load69 = load %funcRef, ptr %28, align 8
  %load70 = load %captures.17, ptr %29, align 1
  %name71 = call ptr @margarineAlloc(i64 0)
  store %captures.17 %load70, ptr %name71, align 1
  %field_ptr72 = getelementptr inbounds nuw %funcRef, ptr %30, i32 0, i32 0
  store ptr @"<closure>.24", ptr %field_ptr72, align 8
  %field_ptr73 = getelementptr inbounds nuw %funcRef, ptr %30, i32 0, i32 1
  store ptr %name71, ptr %field_ptr73, align 8
  %load74 = load %funcRef, ptr %30, align 8
  store %funcRef %load69, ptr %31, align 8
  %field_load75 = getelementptr inbounds nuw %funcRef, ptr %31, i32 0, i32 0
  %load76 = load ptr, ptr %field_load75, align 8
  store %funcRef %load69, ptr %32, align 8
  %field_load77 = getelementptr inbounds nuw %funcRef, ptr %32, i32 0, i32 1
  %load78 = load ptr, ptr %field_load77, align 8
  %name79 = call ptr %load76(ptr %name66, %funcRef %load74, ptr %load78)
  %field_ptr80 = getelementptr inbounds nuw %funcRef, ptr %33, i32 0, i32 0
  store ptr @"std::iter::Iter::sum", ptr %field_ptr80, align 8
  %field_ptr81 = getelementptr inbounds nuw %funcRef, ptr %33, i32 0, i32 1
  store ptr null, ptr %field_ptr81, align 8
  %load82 = load %funcRef, ptr %33, align 8
  store %funcRef %load82, ptr %34, align 8
  %field_load83 = getelementptr inbounds nuw %funcRef, ptr %34, i32 0, i32 0
  %load84 = load ptr, ptr %field_load83, align 8
  store %funcRef %load82, ptr %35, align 8
  %field_load85 = getelementptr inbounds nuw %funcRef, ptr %35, i32 0, i32 1
  %load86 = load ptr, ptr %field_load85, align 8
  %name87 = call i64 %load84(ptr %name79, ptr %load86)
  ret i64 %name87
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
  %11 = alloca %captures.15, align 8
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
  %load17 = load %captures.15, ptr %11, align 1
  %name18 = call ptr @margarineAlloc(i64 0)
  store %captures.15 %load17, ptr %name18, align 1
  %field_ptr19 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 0
  store ptr @"<closure>.21", ptr %field_ptr19, align 8
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

define i64 @"<closure>.21"(%anyType %0, ptr %1) {
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
  %load1 = load %captures.15, ptr %load, align 1
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @"$downcast_any", ptr %field_ptr, align 8
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

define ptr @"std::iter::Iter::filter.22"(ptr %0, %funcRef %1, ptr %2) {
prelude:
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %captures.16, align 8
  %7 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %3, align 8
  store %funcRef %1, ptr %4, align 8
  store ptr %2, ptr %5, align 8
  %load = load %funcRef, ptr %4, align 8
  %load1 = load ptr, ptr %3, align 8
  %field_ptr = getelementptr inbounds nuw %captures.16, ptr %6, i32 0, i32 0
  store %funcRef %load, ptr %field_ptr, align 8
  %field_ptr2 = getelementptr inbounds nuw %captures.16, ptr %6, i32 0, i32 1
  store ptr %load1, ptr %field_ptr2, align 8
  %load3 = load %captures.16, ptr %6, align 8
  %name = call ptr @margarineAlloc(i64 24)
  store %captures.16 %load3, ptr %name, align 8
  %field_ptr4 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  store ptr @"<closure>.23", ptr %field_ptr4, align 8
  %field_ptr5 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  store ptr %name, ptr %field_ptr5, align 8
  %load6 = load %funcRef, ptr %7, align 8
  %name7 = call ptr @margarineAlloc(i64 16)
  %field_ptr8 = getelementptr inbounds nuw %"std::iter::Iter<int>", ptr %name7, i32 0, i32 0
  store %funcRef %load6, ptr %field_ptr8, align 8
  ret ptr %name7
}

define %enumRef @"<closure>.23"(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %captures.16, align 8
  %3 = alloca %funcRef, align 8
  %4 = alloca %captures.16, align 8
  %5 = alloca ptr, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %enumRef, align 8
  %10 = alloca %enumRef, align 8
  %11 = alloca i64, align 8
  %12 = alloca %funcRef, align 8
  %13 = alloca %funcRef, align 8
  %14 = alloca {}, align 8
  %15 = alloca %enumRef, align 8
  %16 = alloca %funcRef, align 8
  %17 = alloca %funcRef, align 8
  %18 = alloca %funcRef, align 8
  %19 = alloca %funcRef, align 8
  %20 = alloca %funcRef, align 8
  %21 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %load = load ptr, ptr %1, align 8
  %load1 = load %captures.16, ptr %load, align 8
  store %captures.16 %load1, ptr %2, align 8
  %field_load = getelementptr inbounds nuw %captures.16, ptr %2, i32 0, i32 0
  %load2 = load %funcRef, ptr %field_load, align 8
  store %funcRef %load2, ptr %3, align 8
  store %captures.16 %load1, ptr %4, align 8
  %field_load3 = getelementptr inbounds nuw %captures.16, ptr %4, i32 0, i32 1
  %load4 = load ptr, ptr %field_load3, align 8
  store ptr %load4, ptr %5, align 8
  br label %loop_body

loop_body:                                        ; preds = %cont28, %entry
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

loop_cont:                                        ; No predecessors!
  %field_ptr39 = getelementptr inbounds nuw %funcRef, ptr %19, i32 0, i32 0
  store ptr @Option.10, ptr %field_ptr39, align 8
  %field_ptr40 = getelementptr inbounds nuw %funcRef, ptr %19, i32 0, i32 1
  store ptr null, ptr %field_ptr40, align 8
  %load41 = load %funcRef, ptr %19, align 8
  store %funcRef %load41, ptr %20, align 8
  %field_load42 = getelementptr inbounds nuw %funcRef, ptr %20, i32 0, i32 0
  %load43 = load ptr, ptr %field_load42, align 8
  store %funcRef %load41, ptr %21, align 8
  %field_load44 = getelementptr inbounds nuw %funcRef, ptr %21, i32 0, i32 1
  %load45 = load ptr, ptr %field_load44, align 8
  %name46 = call %enumRef %load43(ptr %load45)
  ret %enumRef %name46

then:                                             ; preds = %loop_body
  br label %cont

else:                                             ; preds = %loop_body
  ret %enumRef %name

cont:                                             ; preds = %22, %then
  %load16 = load i64, ptr %load15, align 4
  store i64 %load16, ptr %11, align 4
  %load17 = load %funcRef, ptr %3, align 8
  %load18 = load i64, ptr %11, align 4
  store %funcRef %load17, ptr %12, align 8
  %field_load19 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 0
  %load20 = load ptr, ptr %field_load19, align 8
  store %funcRef %load17, ptr %13, align 8
  %field_load21 = getelementptr inbounds nuw %funcRef, ptr %13, i32 0, i32 1
  %load22 = load ptr, ptr %field_load21, align 8
  %name23 = call %enumRef %load20(i64 %load18, ptr %load22)
  store %enumRef %name23, ptr %15, align 8
  %field_load24 = getelementptr inbounds nuw %enumRef, ptr %15, i32 0, i32 1
  %load25 = load i32, ptr %field_load24, align 4
  %icast = trunc i32 %load25 to i1
  br i1 %icast, label %then26, label %else27

22:                                               ; No predecessors!
  br label %cont

then26:                                           ; preds = %cont
  %field_ptr29 = getelementptr inbounds nuw %funcRef, ptr %16, i32 0, i32 0
  store ptr @Option.9, ptr %field_ptr29, align 8
  %field_ptr30 = getelementptr inbounds nuw %funcRef, ptr %16, i32 0, i32 1
  store ptr null, ptr %field_ptr30, align 8
  %load31 = load %funcRef, ptr %16, align 8
  %load32 = load i64, ptr %11, align 4
  store %funcRef %load31, ptr %17, align 8
  %field_load33 = getelementptr inbounds nuw %funcRef, ptr %17, i32 0, i32 0
  %load34 = load ptr, ptr %field_load33, align 8
  store %funcRef %load31, ptr %18, align 8
  %field_load35 = getelementptr inbounds nuw %funcRef, ptr %18, i32 0, i32 1
  %load36 = load ptr, ptr %field_load35, align 8
  %name37 = call %enumRef %load34(i64 %load32, ptr %load36)
  ret %enumRef %name37

else27:                                           ; preds = %cont
  br label %cont28

cont28:                                           ; preds = %else27, %24
  %load38 = load {}, ptr %14, align 1
  br label %loop_body

23:                                               ; No predecessors!
  unreachable

24:                                               ; No predecessors!
  store {} zeroinitializer, ptr %14, align 1
  br label %cont28
}

define %enumRef @"<closure>.24"(i64 %0, ptr %1) {
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
  %load = load ptr, ptr %3, align 8
  %load1 = load %captures.17, ptr %load, align 1
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @is_repeated, ptr %field_ptr, align 8
  %field_ptr2 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr2, align 8
  %load3 = load %funcRef, ptr %4, align 8
  %load4 = load i64, ptr %2, align 4
  store %funcRef %load3, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load5 = load ptr, ptr %field_load, align 8
  store %funcRef %load3, ptr %6, align 8
  %field_load6 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load7 = load ptr, ptr %field_load6, align 8
  %name = call %enumRef %load5(i64 %load4, ptr %load7)
  ret %enumRef %name
}

define %enumRef @is_repeated(i64 %0, ptr %1) {
prelude:
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca ptr, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %funcRef, align 8
  %10 = alloca %funcRef, align 8
  %11 = alloca i1, align 1
  %12 = alloca %enumRef, align 8
  %13 = alloca {}, align 8
  %14 = alloca %enumRef, align 8
  %15 = alloca %enumRef, align 8
  %16 = alloca %funcRef, align 8
  %17 = alloca %funcRef, align 8
  %18 = alloca %funcRef, align 8
  %19 = alloca i64, align 8
  %20 = alloca %funcRef, align 8
  %21 = alloca %funcRef, align 8
  %22 = alloca %funcRef, align 8
  %23 = alloca %"(str, str)", align 8
  %24 = alloca ptr, align 8
  %25 = alloca %"(str, str)", align 8
  %26 = alloca ptr, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %2, align 4
  store ptr %1, ptr %3, align 8
  %load = load i64, ptr %2, align 4
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @"int::to_str", ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load2 = load %funcRef, ptr %4, align 8
  store %funcRef %load2, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load3 = load ptr, ptr %field_load, align 8
  store %funcRef %load2, ptr %6, align 8
  %field_load4 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load5 = load ptr, ptr %field_load4, align 8
  %name = call ptr %load3(i64 %load, ptr %load5)
  store ptr %name, ptr %7, align 8
  %load6 = load ptr, ptr %7, align 8
  %field_ptr7 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 0
  store ptr @"str::len", ptr %field_ptr7, align 8
  %field_ptr8 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 1
  store ptr null, ptr %field_ptr8, align 8
  %load9 = load %funcRef, ptr %8, align 8
  store %funcRef %load9, ptr %9, align 8
  %field_load10 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 0
  %load11 = load ptr, ptr %field_load10, align 8
  store %funcRef %load9, ptr %10, align 8
  %field_load12 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 1
  %load13 = load ptr, ptr %field_load12, align 8
  %name14 = call i64 %load11(ptr %load6, ptr %load13)
  %rems = srem i64 %name14, 2
  store i1 true, ptr %11, align 1
  %load15 = load i1, ptr %11, align 1
  %icmp = icmp eq i64 %rems, 0
  %and = and i1 %load15, %icmp
  store i1 %and, ptr %11, align 1
  %load16 = load i1, ptr %11, align 1
  %bnot = xor i1 %load16, true
  %icast = zext i1 %bnot to i32
  %field_ptr17 = getelementptr inbounds nuw %enumRef, ptr %12, i32 0, i32 0
  store ptr null, ptr %field_ptr17, align 8
  %field_ptr18 = getelementptr inbounds nuw %enumRef, ptr %12, i32 0, i32 1
  store i32 %icast, ptr %field_ptr18, align 4
  %load19 = load %enumRef, ptr %12, align 8
  store %enumRef %load19, ptr %14, align 8
  %field_load20 = getelementptr inbounds nuw %enumRef, ptr %14, i32 0, i32 1
  %load21 = load i32, ptr %field_load20, align 4
  %icast22 = trunc i32 %load21 to i1
  br i1 %icast22, label %then, label %else

then:                                             ; preds = %entry
  %field_ptr23 = getelementptr inbounds nuw %enumRef, ptr %15, i32 0, i32 0
  store ptr null, ptr %field_ptr23, align 8
  %field_ptr24 = getelementptr inbounds nuw %enumRef, ptr %15, i32 0, i32 1
  store i32 0, ptr %field_ptr24, align 4
  %load25 = load %enumRef, ptr %15, align 8
  ret %enumRef %load25

else:                                             ; preds = %entry
  br label %cont

cont:                                             ; preds = %else, %28
  %load26 = load {}, ptr %13, align 1
  %load27 = load ptr, ptr %7, align 8
  %field_ptr28 = getelementptr inbounds nuw %funcRef, ptr %16, i32 0, i32 0
  store ptr @"str::len", ptr %field_ptr28, align 8
  %field_ptr29 = getelementptr inbounds nuw %funcRef, ptr %16, i32 0, i32 1
  store ptr null, ptr %field_ptr29, align 8
  %load30 = load %funcRef, ptr %16, align 8
  store %funcRef %load30, ptr %17, align 8
  %field_load31 = getelementptr inbounds nuw %funcRef, ptr %17, i32 0, i32 0
  %load32 = load ptr, ptr %field_load31, align 8
  store %funcRef %load30, ptr %18, align 8
  %field_load33 = getelementptr inbounds nuw %funcRef, ptr %18, i32 0, i32 1
  %load34 = load ptr, ptr %field_load33, align 8
  %name35 = call i64 %load32(ptr %load27, ptr %load34)
  %divs = sdiv i64 %name35, 2
  store i64 %divs, ptr %19, align 4
  %load36 = load ptr, ptr %7, align 8
  %field_ptr37 = getelementptr inbounds nuw %funcRef, ptr %20, i32 0, i32 0
  store ptr @"str::split_at", ptr %field_ptr37, align 8
  %field_ptr38 = getelementptr inbounds nuw %funcRef, ptr %20, i32 0, i32 1
  store ptr null, ptr %field_ptr38, align 8
  %load39 = load %funcRef, ptr %20, align 8
  %load40 = load i64, ptr %19, align 4
  store %funcRef %load39, ptr %21, align 8
  %field_load41 = getelementptr inbounds nuw %funcRef, ptr %21, i32 0, i32 0
  %load42 = load ptr, ptr %field_load41, align 8
  store %funcRef %load39, ptr %22, align 8
  %field_load43 = getelementptr inbounds nuw %funcRef, ptr %22, i32 0, i32 1
  %load44 = load ptr, ptr %field_load43, align 8
  %name45 = call ptr %load42(ptr %load36, i64 %load40, ptr %load44)
  %load46 = load %"(str, str)", ptr %name45, align 8
  store %"(str, str)" %load46, ptr %23, align 8
  %field_load47 = getelementptr inbounds nuw %"(str, str)", ptr %23, i32 0, i32 0
  %load48 = load ptr, ptr %field_load47, align 8
  store ptr %load48, ptr %24, align 8
  store %"(str, str)" %load46, ptr %25, align 8
  %field_load49 = getelementptr inbounds nuw %"(str, str)", ptr %25, i32 0, i32 1
  %load50 = load ptr, ptr %field_load49, align 8
  store ptr %load50, ptr %26, align 8
  %load51 = load ptr, ptr %24, align 8
  %load52 = load ptr, ptr %26, align 8
  call void @margarineError(i32 2, i32 0, i32 2)
  unreachable

27:                                               ; No predecessors!
  unreachable

28:                                               ; No predecessors!
  store {} zeroinitializer, ptr %13, align 1
  br label %cont
}

define %enumRef @is_repeated_at_least_twice(i64 %0, ptr %1) {
prelude:
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca ptr, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %funcRef, align 8
  %10 = alloca %funcRef, align 8
  %11 = alloca i64, align 8
  %12 = alloca %enumRef, align 8
  %13 = alloca %enumRef, align 8
  %14 = alloca i64, align 8
  %15 = alloca i1, align 1
  %16 = alloca %enumRef, align 8
  %17 = alloca {}, align 8
  %18 = alloca %enumRef, align 8
  %19 = alloca %funcRef, align 8
  %20 = alloca %funcRef, align 8
  %21 = alloca %funcRef, align 8
  %22 = alloca %"(str, str)", align 8
  %23 = alloca ptr, align 8
  %24 = alloca %"(str, str)", align 8
  %25 = alloca ptr, align 8
  %26 = alloca %enumRef, align 8
  %27 = alloca %enumRef, align 8
  %28 = alloca i64, align 8
  %29 = alloca %enumRef, align 8
  %30 = alloca {}, align 8
  %31 = alloca %enumRef, align 8
  %32 = alloca %funcRef, align 8
  %33 = alloca %funcRef, align 8
  %34 = alloca %funcRef, align 8
  %35 = alloca {}, align 8
  %36 = alloca %enumRef, align 8
  %37 = alloca %enumRef, align 8
  %38 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %2, align 4
  store ptr %1, ptr %3, align 8
  %load = load i64, ptr %2, align 4
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @"int::to_str", ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load2 = load %funcRef, ptr %4, align 8
  store %funcRef %load2, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load3 = load ptr, ptr %field_load, align 8
  store %funcRef %load2, ptr %6, align 8
  %field_load4 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load5 = load ptr, ptr %field_load4, align 8
  %name = call ptr %load3(i64 %load, ptr %load5)
  store ptr %name, ptr %7, align 8
  %load6 = load ptr, ptr %7, align 8
  %field_ptr7 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 0
  store ptr @"str::len", ptr %field_ptr7, align 8
  %field_ptr8 = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 1
  store ptr null, ptr %field_ptr8, align 8
  %load9 = load %funcRef, ptr %8, align 8
  store %funcRef %load9, ptr %9, align 8
  %field_load10 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 0
  %load11 = load ptr, ptr %field_load10, align 8
  store %funcRef %load9, ptr %10, align 8
  %field_load12 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 1
  %load13 = load ptr, ptr %field_load12, align 8
  %name14 = call i64 %load11(ptr %load6, ptr %load13)
  store i64 %name14, ptr %11, align 4
  %load15 = load i64, ptr %11, align 4
  %divs = sdiv i64 %load15, 2
  %addi = add i64 %divs, 1
  %name16 = call ptr @margarineAlloc(i64 16)
  %field_ptr17 = getelementptr inbounds nuw %Range, ptr %name16, i32 0, i32 0
  store i64 1, ptr %field_ptr17, align 4
  %field_ptr18 = getelementptr inbounds nuw %Range, ptr %name16, i32 0, i32 1
  store i64 %addi, ptr %field_ptr18, align 4
  br label %loop_body

loop_body:                                        ; preds = %cont98, %then36, %entry
  %name19 = call %enumRef @"Range::__next__"(ptr %name16, ptr null)
  store %enumRef %name19, ptr %12, align 8
  %field_load20 = getelementptr inbounds nuw %enumRef, ptr %12, i32 0, i32 1
  %load21 = load i32, ptr %field_load20, align 4
  %icmp = icmp eq i32 %load21, 1
  br i1 %icmp, label %then, label %else

loop_cont:                                        ; preds = %then
  %field_ptr103 = getelementptr inbounds nuw %enumRef, ptr %38, i32 0, i32 0
  store ptr null, ptr %field_ptr103, align 8
  %field_ptr104 = getelementptr inbounds nuw %enumRef, ptr %38, i32 0, i32 1
  store i32 0, ptr %field_ptr104, align 4
  %load105 = load %enumRef, ptr %38, align 8
  ret %enumRef %load105

then:                                             ; preds = %loop_body
  br label %loop_cont

else:                                             ; preds = %loop_body
  br label %cont

cont:                                             ; preds = %else, %39
  store %enumRef %name19, ptr %13, align 8
  %field_load22 = getelementptr inbounds nuw %enumRef, ptr %13, i32 0, i32 0
  %load23 = load ptr, ptr %field_load22, align 8
  %load24 = load i64, ptr %load23, align 4
  store i64 %load24, ptr %14, align 4
  %load25 = load i64, ptr %11, align 4
  %load26 = load i64, ptr %14, align 4
  %rems = srem i64 %load25, %load26
  store i1 true, ptr %15, align 1
  %load27 = load i1, ptr %15, align 1
  %icmp28 = icmp eq i64 %rems, 0
  %and = and i1 %load27, %icmp28
  store i1 %and, ptr %15, align 1
  %load29 = load i1, ptr %15, align 1
  %bnot = xor i1 %load29, true
  %icast = zext i1 %bnot to i32
  %field_ptr30 = getelementptr inbounds nuw %enumRef, ptr %16, i32 0, i32 0
  store ptr null, ptr %field_ptr30, align 8
  %field_ptr31 = getelementptr inbounds nuw %enumRef, ptr %16, i32 0, i32 1
  store i32 %icast, ptr %field_ptr31, align 4
  %load32 = load %enumRef, ptr %16, align 8
  store %enumRef %load32, ptr %18, align 8
  %field_load33 = getelementptr inbounds nuw %enumRef, ptr %18, i32 0, i32 1
  %load34 = load i32, ptr %field_load33, align 4
  %icast35 = trunc i32 %load34 to i1
  br i1 %icast35, label %then36, label %else37

39:                                               ; No predecessors!
  br label %cont

then36:                                           ; preds = %cont
  br label %loop_body

else37:                                           ; preds = %cont
  br label %cont38

cont38:                                           ; preds = %else37, %41
  %load39 = load {}, ptr %17, align 1
  %load40 = load ptr, ptr %7, align 8
  %field_ptr41 = getelementptr inbounds nuw %funcRef, ptr %19, i32 0, i32 0
  store ptr @"str::split_at", ptr %field_ptr41, align 8
  %field_ptr42 = getelementptr inbounds nuw %funcRef, ptr %19, i32 0, i32 1
  store ptr null, ptr %field_ptr42, align 8
  %load43 = load %funcRef, ptr %19, align 8
  %load44 = load i64, ptr %14, align 4
  store %funcRef %load43, ptr %20, align 8
  %field_load45 = getelementptr inbounds nuw %funcRef, ptr %20, i32 0, i32 0
  %load46 = load ptr, ptr %field_load45, align 8
  store %funcRef %load43, ptr %21, align 8
  %field_load47 = getelementptr inbounds nuw %funcRef, ptr %21, i32 0, i32 1
  %load48 = load ptr, ptr %field_load47, align 8
  %name49 = call ptr %load46(ptr %load40, i64 %load44, ptr %load48)
  %load50 = load %"(str, str)", ptr %name49, align 8
  store %"(str, str)" %load50, ptr %22, align 8
  %field_load51 = getelementptr inbounds nuw %"(str, str)", ptr %22, i32 0, i32 0
  %load52 = load ptr, ptr %field_load51, align 8
  store ptr %load52, ptr %23, align 8
  store %"(str, str)" %load50, ptr %24, align 8
  %field_load53 = getelementptr inbounds nuw %"(str, str)", ptr %24, i32 0, i32 1
  %load54 = load ptr, ptr %field_load53, align 8
  store ptr %load54, ptr %25, align 8
  %field_ptr55 = getelementptr inbounds nuw %enumRef, ptr %26, i32 0, i32 0
  store ptr null, ptr %field_ptr55, align 8
  %field_ptr56 = getelementptr inbounds nuw %enumRef, ptr %26, i32 0, i32 1
  store i32 1, ptr %field_ptr56, align 4
  %load57 = load %enumRef, ptr %26, align 8
  store %enumRef %load57, ptr %27, align 8
  %load58 = load i64, ptr %14, align 4
  store i64 %load58, ptr %28, align 4
  br label %loop_body59

40:                                               ; No predecessors!
  unreachable

41:                                               ; No predecessors!
  store {} zeroinitializer, ptr %17, align 1
  br label %cont38

loop_body59:                                      ; preds = %cont73, %cont38
  %load61 = load i64, ptr %28, align 4
  %load62 = load i64, ptr %11, align 4
  %icmp63 = icmp slt i64 %load61, %load62
  %icast64 = zext i1 %icmp63 to i32
  %field_ptr65 = getelementptr inbounds nuw %enumRef, ptr %29, i32 0, i32 0
  store ptr null, ptr %field_ptr65, align 8
  %field_ptr66 = getelementptr inbounds nuw %enumRef, ptr %29, i32 0, i32 1
  store i32 %icast64, ptr %field_ptr66, align 4
  %load67 = load %enumRef, ptr %29, align 8
  store %enumRef %load67, ptr %31, align 8
  %field_load68 = getelementptr inbounds nuw %enumRef, ptr %31, i32 0, i32 1
  %load69 = load i32, ptr %field_load68, align 4
  %icast70 = trunc i32 %load69 to i1
  br i1 %icast70, label %then71, label %else72

loop_cont60:                                      ; preds = %else72
  %load92 = load %enumRef, ptr %27, align 8
  store %enumRef %load92, ptr %36, align 8
  %field_load93 = getelementptr inbounds nuw %enumRef, ptr %36, i32 0, i32 1
  %load94 = load i32, ptr %field_load93, align 4
  %icast95 = trunc i32 %load94 to i1
  br i1 %icast95, label %then96, label %else97

then71:                                           ; preds = %loop_body59
  %load74 = load ptr, ptr %7, align 8
  %field_ptr75 = getelementptr inbounds nuw %funcRef, ptr %32, i32 0, i32 0
  store ptr @"str::slice", ptr %field_ptr75, align 8
  %field_ptr76 = getelementptr inbounds nuw %funcRef, ptr %32, i32 0, i32 1
  store ptr null, ptr %field_ptr76, align 8
  %load77 = load %funcRef, ptr %32, align 8
  %load78 = load i64, ptr %28, align 4
  %load79 = load i64, ptr %28, align 4
  %load80 = load i64, ptr %14, align 4
  %addi81 = add i64 %load79, %load80
  %name82 = call ptr @margarineAlloc(i64 16)
  %field_ptr83 = getelementptr inbounds nuw %Range, ptr %name82, i32 0, i32 0
  store i64 %load78, ptr %field_ptr83, align 4
  %field_ptr84 = getelementptr inbounds nuw %Range, ptr %name82, i32 0, i32 1
  store i64 %addi81, ptr %field_ptr84, align 4
  store %funcRef %load77, ptr %33, align 8
  %field_load85 = getelementptr inbounds nuw %funcRef, ptr %33, i32 0, i32 0
  %load86 = load ptr, ptr %field_load85, align 8
  store %funcRef %load77, ptr %34, align 8
  %field_load87 = getelementptr inbounds nuw %funcRef, ptr %34, i32 0, i32 1
  %load88 = load ptr, ptr %field_load87, align 8
  %name89 = call ptr %load86(ptr %load74, ptr %name82, ptr %load88)
  %load90 = load ptr, ptr %23, align 8
  br label %cont73

else72:                                           ; preds = %loop_body59
  br label %loop_cont60

cont73:                                           ; preds = %42, %then71
  %load91 = load {}, ptr %30, align 1
  br label %loop_body59

42:                                               ; No predecessors!
  store {} zeroinitializer, ptr %30, align 1
  br label %cont73

then96:                                           ; preds = %loop_cont60
  %field_ptr99 = getelementptr inbounds nuw %enumRef, ptr %37, i32 0, i32 0
  store ptr null, ptr %field_ptr99, align 8
  %field_ptr100 = getelementptr inbounds nuw %enumRef, ptr %37, i32 0, i32 1
  store i32 1, ptr %field_ptr100, align 4
  %load101 = load %enumRef, ptr %37, align 8
  ret %enumRef %load101

else97:                                           ; preds = %loop_cont60
  br label %cont98

cont98:                                           ; preds = %else97, %44
  %load102 = load {}, ptr %35, align 1
  br label %loop_body

43:                                               ; No predecessors!
  unreachable

44:                                               ; No predecessors!
  store {} zeroinitializer, ptr %35, align 1
  br label %cont98
}

define ptr @part2(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %captures.19, align 8
  %9 = alloca %funcRef, align 8
  %10 = alloca %funcRef, align 8
  %11 = alloca %funcRef, align 8
  %12 = alloca %funcRef, align 8
  %13 = alloca %funcRef, align 8
  %14 = alloca %funcRef, align 8
  %15 = alloca %funcRef, align 8
  %16 = alloca %funcRef, align 8
  %17 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %2, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @"str::split", ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load2 = load %funcRef, ptr %4, align 8
  store %funcRef %load2, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load3 = load ptr, ptr %field_load, align 8
  store %funcRef %load2, ptr %6, align 8
  %field_load4 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load5 = load ptr, ptr %field_load4, align 8
  %name = call ptr %load3(ptr %load, ptr @str.25, ptr %load5)
  %field_ptr6 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  store ptr @"std::iter::Iter::map.17", ptr %field_ptr6, align 8
  %field_ptr7 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  store ptr null, ptr %field_ptr7, align 8
  %load8 = load %funcRef, ptr %7, align 8
  %load9 = load %captures.19, ptr %8, align 1
  %name10 = call ptr @margarineAlloc(i64 0)
  store %captures.19 %load9, ptr %name10, align 1
  %field_ptr11 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 0
  store ptr @"<closure>.26", ptr %field_ptr11, align 8
  %field_ptr12 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 1
  store ptr %name10, ptr %field_ptr12, align 8
  %load13 = load %funcRef, ptr %9, align 8
  store %funcRef %load8, ptr %10, align 8
  %field_load14 = getelementptr inbounds nuw %funcRef, ptr %10, i32 0, i32 0
  %load15 = load ptr, ptr %field_load14, align 8
  store %funcRef %load8, ptr %11, align 8
  %field_load16 = getelementptr inbounds nuw %funcRef, ptr %11, i32 0, i32 1
  %load17 = load ptr, ptr %field_load16, align 8
  %name18 = call ptr %load15(ptr %name, %funcRef %load13, ptr %load17)
  %field_ptr19 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 0
  store ptr @"std::iter::Iter::sum", ptr %field_ptr19, align 8
  %field_ptr20 = getelementptr inbounds nuw %funcRef, ptr %12, i32 0, i32 1
  store ptr null, ptr %field_ptr20, align 8
  %load21 = load %funcRef, ptr %12, align 8
  store %funcRef %load21, ptr %13, align 8
  %field_load22 = getelementptr inbounds nuw %funcRef, ptr %13, i32 0, i32 0
  %load23 = load ptr, ptr %field_load22, align 8
  store %funcRef %load21, ptr %14, align 8
  %field_load24 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 1
  %load25 = load ptr, ptr %field_load24, align 8
  %name26 = call i64 %load23(ptr %name18, ptr %load25)
  %field_ptr27 = getelementptr inbounds nuw %funcRef, ptr %15, i32 0, i32 0
  store ptr @"int::to_str", ptr %field_ptr27, align 8
  %field_ptr28 = getelementptr inbounds nuw %funcRef, ptr %15, i32 0, i32 1
  store ptr null, ptr %field_ptr28, align 8
  %load29 = load %funcRef, ptr %15, align 8
  store %funcRef %load29, ptr %16, align 8
  %field_load30 = getelementptr inbounds nuw %funcRef, ptr %16, i32 0, i32 0
  %load31 = load ptr, ptr %field_load30, align 8
  store %funcRef %load29, ptr %17, align 8
  %field_load32 = getelementptr inbounds nuw %funcRef, ptr %17, i32 0, i32 1
  %load33 = load ptr, ptr %field_load32, align 8
  %name34 = call ptr %load31(i64 %name26, ptr %load33)
  ret ptr %name34
}

define i64 @"<closure>.26"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %funcRef, align 8
  %7 = alloca %enumRef, align 8
  %8 = alloca %enumRef, align 8
  %9 = alloca %"(str, str).4", align 8
  %10 = alloca ptr, align 8
  %11 = alloca %"(str, str).4", align 8
  %12 = alloca ptr, align 8
  %13 = alloca %funcRef, align 8
  %14 = alloca %funcRef, align 8
  %15 = alloca %funcRef, align 8
  %16 = alloca %enumRef, align 8
  %17 = alloca %enumRef, align 8
  %18 = alloca i64, align 8
  %19 = alloca %funcRef, align 8
  %20 = alloca %funcRef, align 8
  %21 = alloca %funcRef, align 8
  %22 = alloca %enumRef, align 8
  %23 = alloca %enumRef, align 8
  %24 = alloca i64, align 8
  %25 = alloca %funcRef, align 8
  %26 = alloca %funcRef, align 8
  %27 = alloca %funcRef, align 8
  %28 = alloca %funcRef, align 8
  %29 = alloca %captures.21, align 8
  %30 = alloca %funcRef, align 8
  %31 = alloca %funcRef, align 8
  %32 = alloca %funcRef, align 8
  %33 = alloca %funcRef, align 8
  %34 = alloca %funcRef, align 8
  %35 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %3, align 8
  %load1 = load %captures.19, ptr %load, align 1
  %load2 = load ptr, ptr %2, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @"str::split_once", ptr %field_ptr, align 8
  %field_ptr3 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr3, align 8
  %load4 = load %funcRef, ptr %4, align 8
  store %funcRef %load4, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load5 = load ptr, ptr %field_load, align 8
  store %funcRef %load4, ptr %6, align 8
  %field_load6 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load7 = load ptr, ptr %field_load6, align 8
  %name = call %enumRef %load5(ptr %load2, ptr @str.27, ptr %load7)
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
  %load12 = load ptr, ptr %load11, align 8
  %load13 = load %"(str, str).4", ptr %load12, align 8
  store %"(str, str).4" %load13, ptr %9, align 8
  %field_load14 = getelementptr inbounds nuw %"(str, str).4", ptr %9, i32 0, i32 0
  %load15 = load ptr, ptr %field_load14, align 8
  store ptr %load15, ptr %10, align 8
  store %"(str, str).4" %load13, ptr %11, align 8
  %field_load16 = getelementptr inbounds nuw %"(str, str).4", ptr %11, i32 0, i32 1
  %load17 = load ptr, ptr %field_load16, align 8
  store ptr %load17, ptr %12, align 8
  %load18 = load ptr, ptr %10, align 8
  %field_ptr19 = getelementptr inbounds nuw %funcRef, ptr %13, i32 0, i32 0
  store ptr @"str::parse", ptr %field_ptr19, align 8
  %field_ptr20 = getelementptr inbounds nuw %funcRef, ptr %13, i32 0, i32 1
  store ptr null, ptr %field_ptr20, align 8
  %load21 = load %funcRef, ptr %13, align 8
  store %funcRef %load21, ptr %14, align 8
  %field_load22 = getelementptr inbounds nuw %funcRef, ptr %14, i32 0, i32 0
  %load23 = load ptr, ptr %field_load22, align 8
  store %funcRef %load21, ptr %15, align 8
  %field_load24 = getelementptr inbounds nuw %funcRef, ptr %15, i32 0, i32 1
  %load25 = load ptr, ptr %field_load24, align 8
  %name26 = call %enumRef %load23(ptr %load18, ptr %load25)
  store %enumRef %name26, ptr %16, align 8
  %field_load27 = getelementptr inbounds nuw %enumRef, ptr %16, i32 0, i32 1
  %load28 = load i32, ptr %field_load27, align 4
  %icmp29 = icmp eq i32 %load28, 0
  br i1 %icmp29, label %then30, label %else31

then30:                                           ; preds = %cont
  br label %cont32

else31:                                           ; preds = %cont
  call void @margarineAbort()
  br label %cont32

cont32:                                           ; preds = %else31, %then30
  store %enumRef %name26, ptr %17, align 8
  %field_load33 = getelementptr inbounds nuw %enumRef, ptr %17, i32 0, i32 0
  %load34 = load ptr, ptr %field_load33, align 8
  %load35 = load i64, ptr %load34, align 4
  store i64 %load35, ptr %18, align 4
  %load36 = load ptr, ptr %12, align 8
  %field_ptr37 = getelementptr inbounds nuw %funcRef, ptr %19, i32 0, i32 0
  store ptr @"str::parse", ptr %field_ptr37, align 8
  %field_ptr38 = getelementptr inbounds nuw %funcRef, ptr %19, i32 0, i32 1
  store ptr null, ptr %field_ptr38, align 8
  %load39 = load %funcRef, ptr %19, align 8
  store %funcRef %load39, ptr %20, align 8
  %field_load40 = getelementptr inbounds nuw %funcRef, ptr %20, i32 0, i32 0
  %load41 = load ptr, ptr %field_load40, align 8
  store %funcRef %load39, ptr %21, align 8
  %field_load42 = getelementptr inbounds nuw %funcRef, ptr %21, i32 0, i32 1
  %load43 = load ptr, ptr %field_load42, align 8
  %name44 = call %enumRef %load41(ptr %load36, ptr %load43)
  store %enumRef %name44, ptr %22, align 8
  %field_load45 = getelementptr inbounds nuw %enumRef, ptr %22, i32 0, i32 1
  %load46 = load i32, ptr %field_load45, align 4
  %icmp47 = icmp eq i32 %load46, 0
  br i1 %icmp47, label %then48, label %else49

then48:                                           ; preds = %cont32
  br label %cont50

else49:                                           ; preds = %cont32
  call void @margarineAbort()
  br label %cont50

cont50:                                           ; preds = %else49, %then48
  store %enumRef %name44, ptr %23, align 8
  %field_load51 = getelementptr inbounds nuw %enumRef, ptr %23, i32 0, i32 0
  %load52 = load ptr, ptr %field_load51, align 8
  %load53 = load i64, ptr %load52, align 4
  store i64 %load53, ptr %24, align 4
  %load54 = load i64, ptr %18, align 4
  %load55 = load i64, ptr %24, align 4
  %addi = add i64 %load55, 1
  %name56 = call ptr @margarineAlloc(i64 16)
  %field_ptr57 = getelementptr inbounds nuw %Range, ptr %name56, i32 0, i32 0
  store i64 %load54, ptr %field_ptr57, align 4
  %field_ptr58 = getelementptr inbounds nuw %Range, ptr %name56, i32 0, i32 1
  store i64 %addi, ptr %field_ptr58, align 4
  %field_ptr59 = getelementptr inbounds nuw %funcRef, ptr %25, i32 0, i32 0
  store ptr @"Range::iter", ptr %field_ptr59, align 8
  %field_ptr60 = getelementptr inbounds nuw %funcRef, ptr %25, i32 0, i32 1
  store ptr null, ptr %field_ptr60, align 8
  %load61 = load %funcRef, ptr %25, align 8
  store %funcRef %load61, ptr %26, align 8
  %field_load62 = getelementptr inbounds nuw %funcRef, ptr %26, i32 0, i32 0
  %load63 = load ptr, ptr %field_load62, align 8
  store %funcRef %load61, ptr %27, align 8
  %field_load64 = getelementptr inbounds nuw %funcRef, ptr %27, i32 0, i32 1
  %load65 = load ptr, ptr %field_load64, align 8
  %name66 = call ptr %load63(ptr %name56, ptr %load65)
  %field_ptr67 = getelementptr inbounds nuw %funcRef, ptr %28, i32 0, i32 0
  store ptr @"std::iter::Iter::filter.22", ptr %field_ptr67, align 8
  %field_ptr68 = getelementptr inbounds nuw %funcRef, ptr %28, i32 0, i32 1
  store ptr null, ptr %field_ptr68, align 8
  %load69 = load %funcRef, ptr %28, align 8
  %load70 = load %captures.21, ptr %29, align 1
  %name71 = call ptr @margarineAlloc(i64 0)
  store %captures.21 %load70, ptr %name71, align 1
  %field_ptr72 = getelementptr inbounds nuw %funcRef, ptr %30, i32 0, i32 0
  store ptr @"<closure>.28", ptr %field_ptr72, align 8
  %field_ptr73 = getelementptr inbounds nuw %funcRef, ptr %30, i32 0, i32 1
  store ptr %name71, ptr %field_ptr73, align 8
  %load74 = load %funcRef, ptr %30, align 8
  store %funcRef %load69, ptr %31, align 8
  %field_load75 = getelementptr inbounds nuw %funcRef, ptr %31, i32 0, i32 0
  %load76 = load ptr, ptr %field_load75, align 8
  store %funcRef %load69, ptr %32, align 8
  %field_load77 = getelementptr inbounds nuw %funcRef, ptr %32, i32 0, i32 1
  %load78 = load ptr, ptr %field_load77, align 8
  %name79 = call ptr %load76(ptr %name66, %funcRef %load74, ptr %load78)
  %field_ptr80 = getelementptr inbounds nuw %funcRef, ptr %33, i32 0, i32 0
  store ptr @"std::iter::Iter::sum", ptr %field_ptr80, align 8
  %field_ptr81 = getelementptr inbounds nuw %funcRef, ptr %33, i32 0, i32 1
  store ptr null, ptr %field_ptr81, align 8
  %load82 = load %funcRef, ptr %33, align 8
  store %funcRef %load82, ptr %34, align 8
  %field_load83 = getelementptr inbounds nuw %funcRef, ptr %34, i32 0, i32 0
  %load84 = load ptr, ptr %field_load83, align 8
  store %funcRef %load82, ptr %35, align 8
  %field_load85 = getelementptr inbounds nuw %funcRef, ptr %35, i32 0, i32 1
  %load86 = load ptr, ptr %field_load85, align 8
  %name87 = call i64 %load84(ptr %name79, ptr %load86)
  ret i64 %name87
}

define %enumRef @"<closure>.28"(i64 %0, ptr %1) {
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
  %load = load ptr, ptr %3, align 8
  %load1 = load %captures.21, ptr %load, align 1
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  store ptr @is_repeated_at_least_twice, ptr %field_ptr, align 8
  %field_ptr2 = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 1
  store ptr null, ptr %field_ptr2, align 8
  %load3 = load %funcRef, ptr %4, align 8
  %load4 = load i64, ptr %2, align 4
  store %funcRef %load3, ptr %5, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  %load5 = load ptr, ptr %field_load, align 8
  store %funcRef %load3, ptr %6, align 8
  %field_load6 = getelementptr inbounds nuw %funcRef, ptr %6, i32 0, i32 1
  %load7 = load ptr, ptr %field_load6, align 8
  %name = call %enumRef %load5(i64 %load4, ptr %load7)
  ret %enumRef %name
}

attributes #0 = { noreturn }
