; ModuleID = 'out.ll'
source_filename = "margarine"

%str = type <{ i32, [1 x i8] }>
%str.0 = type <{ i32, [11 x i8] }>
%str.1 = type <{ i32, [1 x i8] }>
%str.2 = type <{ i32, [7 x i8] }>
%lexer_err_ty = type { i32, ptr }
%parser_err_ty = type { i32, ptr }
%anyType = type { ptr, i32 }
%enumRef = type { ptr, i32 }

@str = global %str <{ i32 1, [1 x i8] c"\0A" }>
@str.3 = global %str.0 <{ i32 11, [11 x i8] c"hello world" }>
@str.5 = global %str.1 <{ i32 1, [1 x i8] c"\0A" }>
@str.6 = global %str.2 <{ i32 7, [7 x i8] c"paniced" }>
@fileCount = local_unnamed_addr global i32 1
@0 = internal global [0 x ptr] zeroinitializer
@lexerErrors = local_unnamed_addr global [1 x %lexer_err_ty] [%lexer_err_ty { i32 0, ptr @0 }]
@1 = internal global [0 x ptr] zeroinitializer
@parserErrors = local_unnamed_addr global [1 x %parser_err_ty] [%parser_err_ty { i32 0, ptr @1 }]
@semaErrors = local_unnamed_addr global [0 x ptr] zeroinitializer
@semaErrorsLen = local_unnamed_addr global i32 0

; Function Attrs: noreturn
declare void @margarineAbort() local_unnamed_addr #0

declare ptr @margarineAlloc(i64) local_unnamed_addr

; Function Attrs: nofree nosync nounwind memory(none)
define i64 @fib(i64 %0, ptr readnone captures(none) %1) local_unnamed_addr #1 {
prelude:
  %icmp24 = icmp slt i64 %0, 2
  br i1 %icmp24, label %cont, label %else

else:                                             ; preds = %prelude, %else
  %.tr26 = phi i64 [ %subi18, %else ], [ %0, %prelude ]
  %accumulator.tr25 = phi i64 [ %addi, %else ], [ 0, %prelude ]
  %subi = add nsw i64 %.tr26, -1
  %name = tail call i64 @fib(i64 %subi, ptr poison)
  %subi18 = add nsw i64 %.tr26, -2
  %addi = add i64 %name, %accumulator.tr25
  %icmp = icmp samesign ult i64 %.tr26, 4
  br i1 %icmp, label %cont, label %else

cont:                                             ; preds = %else, %prelude
  %accumulator.tr.lcssa = phi i64 [ 0, %prelude ], [ %addi, %else ]
  %.tr.lcssa = phi i64 [ %0, %prelude ], [ %subi18, %else ]
  %accumulator.ret.tr = add i64 %.tr.lcssa, %accumulator.tr.lcssa
  ret i64 %accumulator.ret.tr
}

; Function Attrs: noreturn
define noundef {} @main(ptr readnone captures(none) %0) local_unnamed_addr #0 {
cont:
  %name = tail call i64 @fib(i64 40, ptr poison)
  %name.i.i.i = tail call ptr @margarineAlloc(i64 8)
  store i64 %name, ptr %name.i.i.i, align 4
  %load2.fca.0.insert.i.i.i = insertvalue %anyType poison, ptr %name.i.i.i, 0
  %load2.fca.1.insert.i.i.i = insertvalue %anyType %load2.fca.0.insert.i.i.i, i32 1, 1
  %name13.i.i = tail call {} @print_raw(%anyType %load2.fca.1.insert.i.i.i, ptr null)
  %name.i.i14.i = tail call ptr @margarineAlloc(i64 8)
  store ptr @str, ptr %name.i.i14.i, align 8
  %load2.fca.0.insert.i.i15.i = insertvalue %anyType poison, ptr %name.i.i14.i, 0
  %load2.fca.1.insert.i.i16.i = insertvalue %anyType %load2.fca.0.insert.i.i15.i, i32 16, 1
  %name13.i17.i = tail call {} @print_raw(%anyType %load2.fca.1.insert.i.i16.i, ptr null)
  %name.i = tail call ptr @margarineAlloc(i64 8)
  store ptr @str.3, ptr %name.i, align 8
  %name42 = tail call {} @println.4(ptr nonnull @str.3, ptr poison)
  %name50 = tail call ptr @panic(ptr nonnull @str.6, ptr poison)
  unreachable
}

define {} @println(i64 %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name.i.i = tail call ptr @margarineAlloc(i64 8)
  store i64 %0, ptr %name.i.i, align 4
  %load2.fca.0.insert.i.i = insertvalue %anyType poison, ptr %name.i.i, 0
  %load2.fca.1.insert.i.i = insertvalue %anyType %load2.fca.0.insert.i.i, i32 1, 1
  %name13.i = tail call {} @print_raw(%anyType %load2.fca.1.insert.i.i, ptr null)
  %name.i.i14 = tail call ptr @margarineAlloc(i64 8)
  store ptr @str, ptr %name.i.i14, align 8
  %load2.fca.0.insert.i.i15 = insertvalue %anyType poison, ptr %name.i.i14, 0
  %load2.fca.1.insert.i.i16 = insertvalue %anyType %load2.fca.0.insert.i.i15, i32 16, 1
  %name13.i17 = tail call {} @print_raw(%anyType %load2.fca.1.insert.i.i16, ptr null)
  ret {} zeroinitializer
}

define {} @print(i64 %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name.i = tail call ptr @margarineAlloc(i64 8)
  store i64 %0, ptr %name.i, align 4
  %load2.fca.0.insert.i = insertvalue %anyType poison, ptr %name.i, 0
  %load2.fca.1.insert.i = insertvalue %anyType %load2.fca.0.insert.i, i32 1, 1
  %name13 = tail call {} @print_raw(%anyType %load2.fca.1.insert.i, ptr null)
  ret {} zeroinitializer
}

declare {} @print_raw(%anyType, ptr) local_unnamed_addr

define %anyType @"$any"(i64 %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name = tail call ptr @margarineAlloc(i64 8)
  store i64 %0, ptr %name, align 4
  %load2.fca.0.insert = insertvalue %anyType poison, ptr %name, 0
  %load2.fca.1.insert = insertvalue %anyType %load2.fca.0.insert, i32 1, 1
  ret %anyType %load2.fca.1.insert
}

define {} @print.1(ptr %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name.i = tail call ptr @margarineAlloc(i64 8)
  store ptr %0, ptr %name.i, align 8
  %load2.fca.0.insert.i = insertvalue %anyType poison, ptr %name.i, 0
  %load2.fca.1.insert.i = insertvalue %anyType %load2.fca.0.insert.i, i32 16, 1
  %name13 = tail call {} @print_raw(%anyType %load2.fca.1.insert.i, ptr null)
  ret {} zeroinitializer
}

define %anyType @"$any.2"(ptr %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name = tail call ptr @margarineAlloc(i64 8)
  store ptr %0, ptr %name, align 8
  %load2.fca.0.insert = insertvalue %anyType poison, ptr %name, 0
  %load2.fca.1.insert = insertvalue %anyType %load2.fca.0.insert, i32 16, 1
  ret %anyType %load2.fca.1.insert
}

define {} @println.4(ptr %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name.i.i = tail call ptr @margarineAlloc(i64 8)
  store ptr %0, ptr %name.i.i, align 8
  %load2.fca.0.insert.i.i = insertvalue %anyType poison, ptr %name.i.i, 0
  %load2.fca.1.insert.i.i = insertvalue %anyType %load2.fca.0.insert.i.i, i32 16, 1
  %name13.i = tail call {} @print_raw(%anyType %load2.fca.1.insert.i.i, ptr null)
  %name.i.i14 = tail call ptr @margarineAlloc(i64 8)
  store ptr @str.5, ptr %name.i.i14, align 8
  %load2.fca.0.insert.i.i15 = insertvalue %anyType poison, ptr %name.i.i14, 0
  %load2.fca.1.insert.i.i16 = insertvalue %anyType %load2.fca.0.insert.i.i15, i32 16, 1
  %name13.i17 = tail call {} @print_raw(%anyType %load2.fca.1.insert.i.i16, ptr null)
  ret {} zeroinitializer
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define %enumRef @"$downcast_any"(%anyType %0, ptr readnone captures(none) %1) local_unnamed_addr #2 {
prelude:
  %.fca.0.extract = extractvalue %anyType %0, 0
  %.fca.1.extract = extractvalue %anyType %0, 1
  %icmp = icmp ne i32 %.fca.1.extract, 16
  %icast = zext i1 %icmp to i32
  %load5.fca.0.insert = insertvalue %enumRef poison, ptr %.fca.0.extract, 0
  %load5.fca.1.insert = insertvalue %enumRef %load5.fca.0.insert, i32 %icast, 1
  ret %enumRef %load5.fca.1.insert
}

; Function Attrs: noreturn
define noundef ptr @panic(ptr %0, ptr readnone captures(none) %1) local_unnamed_addr #0 {
prelude:
  %name = tail call {} @println.4(ptr %0, ptr poison)
  %name13 = tail call ptr @margarineAbort(ptr null)
  unreachable
}

attributes #0 = { noreturn }
attributes #1 = { nofree nosync nounwind memory(none) }
attributes #2 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) }
