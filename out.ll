; ModuleID = 'margarine'
source_filename = "margarine"

%str = type <{ i32, [23 x i8] }>
%lexer_err_ty = type { i32, ptr }
%parser_err_ty = type { i32, ptr }
%funcRef = type { ptr, ptr }
%captures = type {}
%"(int)" = type { i64 }
%enumRef = type { i32, ptr }
%Range = type { i64, i64 }

@str = global %str <{ i32 23, [23 x i8] c"root module test passes" }>
@fileCount = global i32 3
@0 = global [0 x ptr] zeroinitializer
@1 = global [0 x ptr] zeroinitializer
@2 = global [0 x ptr] zeroinitializer
@lexerErrors = global [3 x %lexer_err_ty] [%lexer_err_ty { i32 0, ptr @0 }, %lexer_err_ty { i32 0, ptr @1 }, %lexer_err_ty { i32 0, ptr @2 }]
@3 = global [0 x ptr] zeroinitializer
@4 = global [0 x ptr] zeroinitializer
@5 = global [0 x ptr] zeroinitializer
@parserErrors = global [3 x %parser_err_ty] [%parser_err_ty { i32 0, ptr @3 }, %parser_err_ty { i32 0, ptr @4 }, %parser_err_ty { i32 0, ptr @5 }]
@6 = global [582 x i8] c"\1B[38;2;255;0;0;1merror\1B[0m: \1B[38;2;255;255;255;1munknown type\1B[0m\0A   \1B[38;2;255;165;0m\E2\94\8F\E2\94\80\E2\96\B6\1B[0m /Users/macbook/Desktop/projects/programming languages/untitled compiler/artifacts/<>Users<>macbook<>Desktop<>projects<>programming languages<>untitled compiler<>tests<>std/lib.mar:13:47\0A   \1B[38;2;255;165;0m\E2\94\83\1B[0m \0A\1B[38;2;255;165;0m13\1B[0m \1B[38;2;255;165;0m\E2\94\83\1B[0m         fn new_any<T>(value: T, type_id: int): any \0A   \1B[38;2;255;165;0m\E2\94\83\1B[0m                                                \1B[38;2;255;0;0m\E2\96\94\E2\96\94\E2\96\94\1B[0m there's no type named 'any'\0A  \1B[38;2;255;165;0m\E2\94\81\E2\94\BB\E2\94\81\1B[0m \0A"
@7 = global [576 x i8] c"\1B[38;2;255;0;0;1merror\1B[0m: \1B[38;2;255;255;255;1munknown type\1B[0m\0A   \1B[38;2;255;165;0m\E2\94\8F\E2\94\80\E2\96\B6\1B[0m /Users/macbook/Desktop/projects/programming languages/untitled compiler/artifacts/<>Users<>macbook<>Desktop<>projects<>programming languages<>untitled compiler<>tests<>std/lib.mar:14:32\0A   \1B[38;2;255;165;0m\E2\94\83\1B[0m \0A\1B[38;2;255;165;0m14\1B[0m \1B[38;2;255;165;0m\E2\94\83\1B[0m         fn downcast_any<T>(ptr: any, target: int): Option<T>\0A   \1B[38;2;255;165;0m\E2\94\83\1B[0m                                 \1B[38;2;255;0;0m\E2\96\94\E2\96\94\E2\96\94\1B[0m there's no type named 'any'\0A  \1B[38;2;255;165;0m\E2\94\81\E2\94\BB\E2\94\81\1B[0m \0A"
@8 = global [552 x i8] c"\1B[38;2;255;0;0;1merror\1B[0m: \1B[38;2;255;255;255;1munknown type\1B[0m\0A   \1B[38;2;255;165;0m\E2\94\8F\E2\94\80\E2\96\B6\1B[0m /Users/macbook/Desktop/projects/programming languages/untitled compiler/artifacts/<>Users<>macbook<>Desktop<>projects<>programming languages<>untitled compiler<>tests<>std/lib.mar:55:19\0A   \1B[38;2;255;165;0m\E2\94\83\1B[0m \0A\1B[38;2;255;165;0m55\1B[0m \1B[38;2;255;165;0m\E2\94\83\1B[0m     fn iter(self): Iter<int> {\0A   \1B[38;2;255;165;0m\E2\94\83\1B[0m                    \1B[38;2;255;0;0m\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\1B[0m there's no type named 'Iter'\0A  \1B[38;2;255;165;0m\E2\94\81\E2\94\BB\E2\94\81\1B[0m \0A"
@9 = global [563 x i8] c"\1B[38;2;255;0;0;1merror\1B[0m: \1B[38;2;255;255;255;1mfield doesn't exist\1B[0m\0A   \1B[38;2;255;165;0m\E2\94\8F\E2\94\80\E2\96\B6\1B[0m /Users/macbook/Desktop/projects/programming languages/untitled compiler/artifacts/<>Users<>macbook<>Desktop<>projects<>programming languages<>untitled compiler<>tests<>std/lib/iter.mar:8:13\0A   \1B[38;2;255;165;0m\E2\94\83\1B[0m \0A\1B[38;2;255;165;0m8\1B[0m  \1B[38;2;255;165;0m\E2\94\83\1B[0m         self.map(f);\0A   \1B[38;2;255;165;0m\E2\94\83\1B[0m              \1B[38;2;255;0;0m\E2\96\94\E2\96\94\E2\96\94\1B[0m the type 'std::iter::Iter<T>' doesn't have a field named 'map'\0A  \1B[38;2;255;165;0m\E2\94\81\E2\94\BB\E2\94\81\1B[0m \0A"
@10 = global [529 x i8] c"\1B[38;2;255;0;0;1merror\1B[0m: \1B[38;2;255;255;255;1mcall on non-function\1B[0m\0A   \1B[38;2;255;165;0m\E2\94\8F\E2\94\80\E2\96\B6\1B[0m /Users/macbook/Desktop/projects/programming languages/untitled compiler/artifacts/<>Users<>macbook<>Desktop<>projects<>programming languages<>untitled compiler<>tests<>std/lib/iter.mar:8:13\0A   \1B[38;2;255;165;0m\E2\94\83\1B[0m \0A\1B[38;2;255;165;0m8\1B[0m  \1B[38;2;255;165;0m\E2\94\83\1B[0m         self.map(f);\0A   \1B[38;2;255;165;0m\E2\94\83\1B[0m              \1B[38;2;255;0;0m\E2\96\94\E2\96\94\E2\96\94\1B[0m the symbol isn't a function\0A  \1B[38;2;255;165;0m\E2\94\81\E2\94\BB\E2\94\81\1B[0m \0A"
@11 = global [538 x i8] c"\1B[38;2;255;0;0;1merror\1B[0m: \1B[38;2;255;255;255;1mvariable not found\1B[0m\0A   \1B[38;2;255;165;0m\E2\94\8F\E2\94\80\E2\96\B6\1B[0m /Users/macbook/Desktop/projects/programming languages/untitled compiler/artifacts/<>Users<>macbook<>Desktop<>projects<>programming languages<>untitled compiler<>tests<>std/lib.mar:39:16\0A   \1B[38;2;255;165;0m\E2\94\83\1B[0m \0A\1B[38;2;255;165;0m39\1B[0m \1B[38;2;255;165;0m\E2\94\83\1B[0m     if !value { panic(msg) }\0A   \1B[38;2;255;165;0m\E2\94\83\1B[0m                 \1B[38;2;255;0;0m\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\1B[0m no variable named 'panic'\0A  \1B[38;2;255;165;0m\E2\94\81\E2\94\BB\E2\94\81\1B[0m \0A"
@12 = global [542 x i8] c"\1B[38;2;255;0;0;1merror\1B[0m: \1B[38;2;255;255;255;1mcall on non-function\1B[0m\0A   \1B[38;2;255;165;0m\E2\94\8F\E2\94\80\E2\96\B6\1B[0m /Users/macbook/Desktop/projects/programming languages/untitled compiler/artifacts/<>Users<>macbook<>Desktop<>projects<>programming languages<>untitled compiler<>tests<>std/lib.mar:39:16\0A   \1B[38;2;255;165;0m\E2\94\83\1B[0m \0A\1B[38;2;255;165;0m39\1B[0m \1B[38;2;255;165;0m\E2\94\83\1B[0m     if !value { panic(msg) }\0A   \1B[38;2;255;165;0m\E2\94\83\1B[0m                 \1B[38;2;255;0;0m\E2\96\94\E2\96\94\E2\96\94\E2\96\94\E2\96\94\1B[0m the symbol isn't a function\0A  \1B[38;2;255;165;0m\E2\94\81\E2\94\BB\E2\94\81\1B[0m \0A"
@13 = global [509 x i8] c"\1B[38;2;255;0;0;1merror\1B[0m: \1B[38;2;255;255;255;1munknown type\1B[0m\0A   \1B[38;2;255;165;0m\E2\94\8F\E2\94\80\E2\96\B6\1B[0m /Users/macbook/Desktop/projects/programming languages/untitled compiler/artifacts/<>Users<>macbook<>Desktop<>projects<>programming languages<>untitled compiler<>tests<>std/lib.mar:56:8\0A   \1B[38;2;255;165;0m\E2\94\83\1B[0m \0A\1B[38;2;255;165;0m56\1B[0m \1B[38;2;255;165;0m\E2\94\83\1B[0m         Iter {\0A   \1B[38;2;255;165;0m\E2\94\83\1B[0m         \1B[38;2;255;0;0m\E2\96\94\E2\96\94\E2\96\94\E2\96\94\1B[0m there's no type named 'Iter'\0A  \1B[38;2;255;165;0m\E2\94\81\E2\94\BB\E2\94\81\1B[0m \0A"
@semaErrors = global [8 x ptr] [ptr @6, ptr @7, ptr @8, ptr @9, ptr @10, ptr @11, ptr @12, ptr @13]

; Function Attrs: noreturn
declare void @margarineAbort() #0

; Function Attrs: noreturn
declare void @margarineError() #0

declare ptr @margarineAlloc(i32)

define void @__initStartupSystems__() {
prelude:
  br label %entry

entry:                                            ; preds = %prelude
  call void @"std::iter::Iter::sum"()
  call void @"std::assert"()
  call void @"Range::__next__"()
  call void @"Range::iter"()
  call void @test_root_module()
  call void @for_loops()
  call void @for_loop_break()
  ret void
}

define i64 @"std::iter::Iter::sum"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %5 = alloca %funcRef, align 8
  %6 = alloca %captures, align 8
  %7 = alloca %funcRef, align 8
  %8 = alloca %funcRef, align 8
  %9 = alloca %funcRef, align 8
  %10 = alloca %"(int)", align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  %name = call ptr @margarineAlloc(i32 8)
  %field_ptr = getelementptr inbounds nuw %"(int)", ptr %name, i32 0, i32 0
  store i64 0, ptr %field_ptr, align 4
  store ptr %name, ptr %4, align 8
  %load = load ptr, ptr %2, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 0
  store ptr @"std::iter::Iter::for_each", ptr %field_ptr1, align 8
  %field_ptr2 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 1
  store ptr null, ptr %field_ptr2, align 8
  %load3 = load %funcRef, ptr %5, align 8
  %load4 = load %captures, ptr %6, align 1
  %name5 = call ptr @margarineAlloc(i32 0)
  store %captures %load4, ptr %name5, align 1
  %field_ptr6 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 0
  store ptr @"<closure>", ptr %field_ptr6, align 8
  %field_ptr7 = getelementptr inbounds nuw %funcRef, ptr %7, i32 0, i32 1
  store ptr %name5, ptr %field_ptr7, align 8
  %load8 = load %funcRef, ptr %7, align 8
  store %funcRef %load3, ptr %8, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %8, i32 0, i32 0
  %load9 = load ptr, ptr %field_load, align 8
  store %funcRef %load3, ptr %9, align 8
  %field_load10 = getelementptr inbounds nuw %funcRef, ptr %9, i32 0, i32 1
  %load11 = load ptr, ptr %field_load10, align 8
  %name12 = call {} %load9(ptr %load, %funcRef %load8, ptr %load11)
  %load13 = load ptr, ptr %4, align 8
  %load14 = load %"(int)", ptr %load13, align 4
  store %"(int)" %load14, ptr %10, align 4
  %field_load15 = getelementptr inbounds nuw %"(int)", ptr %10, i32 0, i32 0
  %load16 = load i64, ptr %field_load15, align 4
  ret i64 %load16
}

define {} @"std::iter::Iter::for_each"(ptr %0, %funcRef %1, ptr %2) {
prelude:
  %3 = alloca ptr, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca ptr, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %3, align 8
  store %funcRef %1, ptr %4, align 8
  store ptr %2, ptr %5, align 8
  %load = load ptr, ptr %3, align 8
  call void @margarineError(i32 2, i32 0, i32 3)
  unreachable
}

define {} @"<closure>"(i64 %0, ptr %1) {
prelude:
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %2, align 4
  store ptr %1, ptr %3, align 8
  %load = load ptr, ptr %3, align 8
  %load1 = load %captures, ptr %load, align 1
  ret {} zeroinitializer
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
  br label %entry

entry:                                            ; preds = %prelude
  store %enumRef %0, ptr %3, align 8
  store ptr %1, ptr %4, align 8
  store ptr %2, ptr %5, align 8
  %load = load %enumRef, ptr %3, align 8
  store %enumRef %load, ptr %6, align 8
  %field_load = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 0
  %load1 = load i32, ptr %field_load, align 4
  %icast = trunc i32 %load1 to i1
  %bnot = xor i1 %icast, true
  store %enumRef %load, ptr %7, align 8
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %7, i32 0, i32 0
  store i1 %bnot, ptr %field_ptr, align 1
  %load2 = load %enumRef, ptr %field_ptr, align 8
  store %enumRef %load2, ptr %9, align 8
  %field_load3 = getelementptr inbounds nuw %enumRef, ptr %9, i32 0, i32 0
  %load4 = load i32, ptr %field_load3, align 4
  %icast5 = trunc i32 %load4 to i1
  br i1 %icast5, label %then, label %else

then:                                             ; preds = %entry
  br label %cont

else:                                             ; preds = %entry
  br label %cont

cont:                                             ; preds = %else, %then
  %load6 = load {}, ptr %8, align 1
  ret {} %load6
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
  store i32 %icast, ptr %field_ptr, align 4
  %field_ptr7 = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 1
  store ptr null, ptr %field_ptr7, align 8
  %load8 = load %enumRef, ptr %6, align 8
  store %enumRef %load8, ptr %8, align 8
  %field_load9 = getelementptr inbounds nuw %enumRef, ptr %8, i32 0, i32 0
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
  store ptr @Option, ptr %field_ptr18, align 8
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
  store ptr @Option.1, ptr %field_ptr29, align 8
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

define %enumRef @Option(i64 %0, ptr %1) {
prelude:
  %2 = alloca i64, align 8
  %3 = alloca ptr, align 8
  %4 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store i64 %0, ptr %2, align 4
  store ptr %1, ptr %3, align 8
  %load = load i64, ptr %2, align 4
  %name = call ptr @margarineAlloc(i32 8)
  store i64 %load, ptr %name, align 4
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %4, i32 0, i32 0
  store i32 0, ptr %field_ptr, align 4
  %field_ptr1 = getelementptr inbounds nuw %enumRef, ptr %4, i32 0, i32 1
  store ptr %name, ptr %field_ptr1, align 8
  %load2 = load %enumRef, ptr %4, align 8
  ret %enumRef %load2
}

define %enumRef @Option.1(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %name = call ptr @margarineAlloc(i32 8)
  store ptr null, ptr %name, align 8
  %field_ptr = getelementptr inbounds nuw %enumRef, ptr %2, i32 0, i32 0
  store i32 1, ptr %field_ptr, align 4
  %field_ptr1 = getelementptr inbounds nuw %enumRef, ptr %2, i32 0, i32 1
  store ptr %name, ptr %field_ptr1, align 8
  %load = load %enumRef, ptr %2, align 8
  ret %enumRef %load
}

define ptr @"Range::iter"(ptr %0, ptr %1) {
prelude:
  %2 = alloca ptr, align 8
  %3 = alloca ptr, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %2, align 8
  store ptr %1, ptr %3, align 8
  call void @margarineError(i32 2, i32 0, i32 2)
  unreachable

4:                                                ; No predecessors!
  call void @margarineError(i32 2, i32 0, i32 7)
  unreachable
}

define {} @test_root_module(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %funcRef, align 8
  %3 = alloca %enumRef, align 8
  %4 = alloca %funcRef, align 8
  %5 = alloca %funcRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %field_ptr = getelementptr inbounds nuw %funcRef, ptr %2, i32 0, i32 0
  store ptr @"std::assert", ptr %field_ptr, align 8
  %field_ptr1 = getelementptr inbounds nuw %funcRef, ptr %2, i32 0, i32 1
  store ptr null, ptr %field_ptr1, align 8
  %load = load %funcRef, ptr %2, align 8
  %field_ptr2 = getelementptr inbounds nuw %enumRef, ptr %3, i32 0, i32 0
  store i32 1, ptr %field_ptr2, align 4
  %field_ptr3 = getelementptr inbounds nuw %enumRef, ptr %3, i32 0, i32 1
  store ptr null, ptr %field_ptr3, align 8
  %load4 = load %enumRef, ptr %3, align 8
  store %funcRef %load, ptr %4, align 8
  %field_load = getelementptr inbounds nuw %funcRef, ptr %4, i32 0, i32 0
  %load5 = load ptr, ptr %field_load, align 8
  store %funcRef %load, ptr %5, align 8
  %field_load6 = getelementptr inbounds nuw %funcRef, ptr %5, i32 0, i32 1
  %load7 = load ptr, ptr %field_load6, align 8
  %name = call {} %load5(%enumRef %load4, ptr @str, ptr %load7)
  ret {} zeroinitializer
}

define {} @for_loops(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %enumRef, align 8
  %3 = alloca %enumRef, align 8
  %4 = alloca i64, align 8
  %5 = alloca i64, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %name = call ptr @margarineAlloc(i32 16)
  %field_ptr = getelementptr inbounds nuw %Range, ptr %name, i32 0, i32 0
  store i64 0, ptr %field_ptr, align 4
  %field_ptr1 = getelementptr inbounds nuw %Range, ptr %name, i32 0, i32 1
  store i64 10, ptr %field_ptr1, align 4
  br label %loop_body

loop_body:                                        ; preds = %cont, %entry
  %name2 = call %enumRef @"Range::__next__"(ptr %name, ptr null)
  store %enumRef %name2, ptr %2, align 8
  %field_load = getelementptr inbounds nuw %enumRef, ptr %2, i32 0, i32 0
  %load = load i32, ptr %field_load, align 4
  %icmp = icmp eq i32 %load, 1
  br i1 %icmp, label %then, label %else

loop_cont:                                        ; preds = %then
  store i64 0, ptr %5, align 4
  ret {} zeroinitializer

then:                                             ; preds = %loop_body
  br label %loop_cont

else:                                             ; preds = %loop_body
  br label %cont

cont:                                             ; preds = %else, %6
  store %enumRef %name2, ptr %3, align 8
  %field_load3 = getelementptr inbounds nuw %enumRef, ptr %3, i32 0, i32 1
  %load4 = load ptr, ptr %field_load3, align 8
  %load5 = load i64, ptr %load4, align 4
  store i64 %load5, ptr %4, align 4
  br label %loop_body

6:                                                ; No predecessors!
  br label %cont
}

define {} @for_loop_break(ptr %0) {
prelude:
  %1 = alloca ptr, align 8
  %2 = alloca %enumRef, align 8
  %3 = alloca %enumRef, align 8
  %4 = alloca i64, align 8
  %5 = alloca i1, align 1
  %6 = alloca %enumRef, align 8
  %7 = alloca {}, align 8
  %8 = alloca %enumRef, align 8
  br label %entry

entry:                                            ; preds = %prelude
  store ptr %0, ptr %1, align 8
  %name = call ptr @margarineAlloc(i32 16)
  %field_ptr = getelementptr inbounds nuw %Range, ptr %name, i32 0, i32 0
  store i64 0, ptr %field_ptr, align 4
  %field_ptr1 = getelementptr inbounds nuw %Range, ptr %name, i32 0, i32 1
  store i64 10, ptr %field_ptr1, align 4
  br label %loop_body

loop_body:                                        ; preds = %cont18, %entry
  %name2 = call %enumRef @"Range::__next__"(ptr %name, ptr null)
  store %enumRef %name2, ptr %2, align 8
  %field_load = getelementptr inbounds nuw %enumRef, ptr %2, i32 0, i32 0
  %load = load i32, ptr %field_load, align 4
  %icmp = icmp eq i32 %load, 1
  br i1 %icmp, label %then, label %else

loop_cont:                                        ; preds = %then16, %then
  ret {} zeroinitializer

then:                                             ; preds = %loop_body
  br label %loop_cont

else:                                             ; preds = %loop_body
  br label %cont

cont:                                             ; preds = %else, %9
  store %enumRef %name2, ptr %3, align 8
  %field_load3 = getelementptr inbounds nuw %enumRef, ptr %3, i32 0, i32 1
  %load4 = load ptr, ptr %field_load3, align 8
  %load5 = load i64, ptr %load4, align 4
  store i64 %load5, ptr %4, align 4
  %load6 = load i64, ptr %4, align 4
  store i1 true, ptr %5, align 1
  %load7 = load i1, ptr %5, align 1
  %icmp8 = icmp eq i64 %load6, 5
  %and = and i1 %load7, %icmp8
  store i1 %and, ptr %5, align 1
  %load9 = load i1, ptr %5, align 1
  %icast = zext i1 %load9 to i32
  %field_ptr10 = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 0
  store i32 %icast, ptr %field_ptr10, align 4
  %field_ptr11 = getelementptr inbounds nuw %enumRef, ptr %6, i32 0, i32 1
  store ptr null, ptr %field_ptr11, align 8
  %load12 = load %enumRef, ptr %6, align 8
  store %enumRef %load12, ptr %8, align 8
  %field_load13 = getelementptr inbounds nuw %enumRef, ptr %8, i32 0, i32 0
  %load14 = load i32, ptr %field_load13, align 4
  %icast15 = trunc i32 %load14 to i1
  br i1 %icast15, label %then16, label %else17

9:                                                ; No predecessors!
  br label %cont

then16:                                           ; preds = %cont
  br label %loop_cont

else17:                                           ; preds = %cont
  br label %cont18

cont18:                                           ; preds = %else17, %10
  %load19 = load {}, ptr %7, align 1
  br label %loop_body

10:                                               ; No predecessors!
  store {} zeroinitializer, ptr %7, align 1
  br label %cont18
}

attributes #0 = { noreturn }
