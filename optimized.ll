; ModuleID = 'out.ll'
source_filename = "margarine"

%str = type <{ i32, [0 x i8] }>
%lexer_err_ty = type { i32, ptr }
%parser_err_ty = type { i32, ptr }
%enumRef = type { i32, ptr }
%funcRef = type { ptr, ptr }

@str = global %str zeroinitializer
@fileCount = local_unnamed_addr global i32 8
@0 = internal global [0 x ptr] zeroinitializer
@1 = internal global [0 x ptr] zeroinitializer
@2 = internal global [0 x ptr] zeroinitializer
@3 = internal global [0 x ptr] zeroinitializer
@4 = internal global [0 x ptr] zeroinitializer
@5 = internal global [0 x ptr] zeroinitializer
@6 = internal global [0 x ptr] zeroinitializer
@7 = internal global [0 x ptr] zeroinitializer
@lexerErrors = local_unnamed_addr global [8 x %lexer_err_ty] [%lexer_err_ty { i32 0, ptr @0 }, %lexer_err_ty { i32 0, ptr @1 }, %lexer_err_ty { i32 0, ptr @2 }, %lexer_err_ty { i32 0, ptr @3 }, %lexer_err_ty { i32 0, ptr @4 }, %lexer_err_ty { i32 0, ptr @5 }, %lexer_err_ty { i32 0, ptr @6 }, %lexer_err_ty { i32 0, ptr @7 }]
@8 = internal global [0 x ptr] zeroinitializer
@9 = internal global [0 x ptr] zeroinitializer
@10 = internal global [0 x ptr] zeroinitializer
@11 = internal global [0 x ptr] zeroinitializer
@12 = internal global [0 x ptr] zeroinitializer
@13 = internal global [0 x ptr] zeroinitializer
@14 = internal global [0 x ptr] zeroinitializer
@15 = internal global [0 x ptr] zeroinitializer
@parserErrors = local_unnamed_addr global [8 x %parser_err_ty] [%parser_err_ty { i32 0, ptr @8 }, %parser_err_ty { i32 0, ptr @9 }, %parser_err_ty { i32 0, ptr @10 }, %parser_err_ty { i32 0, ptr @11 }, %parser_err_ty { i32 0, ptr @12 }, %parser_err_ty { i32 0, ptr @13 }, %parser_err_ty { i32 0, ptr @14 }, %parser_err_ty { i32 0, ptr @15 }]
@semaErrors = local_unnamed_addr global [0 x ptr] zeroinitializer

declare ptr @margarineAlloc(i32) local_unnamed_addr

; Function Attrs: noreturn
define void @__initStartupSystems__() local_unnamed_addr #0 {
prelude:
  %name.i = tail call i64 @now_secs(ptr null)
  %name15.i = tail call i64 @now_nanos(ptr null)
  %name15.i.frozen = freeze i64 %name15.i
  %divs.i.i = sdiv i64 %name15.i.frozen, 1000000000
  %addi.i.i = add i64 %divs.i.i, %name.i
  %0 = mul i64 %divs.i.i, 1000000000
  %rems.i.i.decomposed = sub i64 %name15.i.frozen, %0
  %name.i.i = tail call noundef ptr @margarineAlloc(i32 16)
  store i64 %addi.i.i, ptr %name.i.i, align 4
  %field_ptr5.i.i = getelementptr inbounds nuw i8, ptr %name.i.i, i64 8
  store i64 %rems.i.i.decomposed, ptr %field_ptr5.i.i, align 4
  %name.i1 = tail call noundef ptr @margarineAlloc(i32 16)
  tail call void @llvm.memset.p0.i64(ptr noundef nonnull align 4 dereferenceable(16) %name.i1, i8 0, i64 16, i1 false)
  %name.i.i2 = tail call i64 @now_secs(ptr null)
  %name15.i.i = tail call i64 @now_nanos(ptr null)
  %name.i.i.i = tail call noundef ptr @margarineAlloc(i32 16)
  unreachable
}

define noundef ptr @"std::duration::Duration::now"(ptr readnone captures(none) %0) local_unnamed_addr {
prelude:
  %name = tail call i64 @now_secs(ptr null)
  %name15 = tail call i64 @now_nanos(ptr null)
  %name15.frozen = freeze i64 %name15
  %divs.i = sdiv i64 %name15.frozen, 1000000000
  %addi.i = add i64 %divs.i, %name
  %1 = mul i64 %divs.i, 1000000000
  %rems.i.decomposed = sub i64 %name15.frozen, %1
  %name.i = tail call noundef ptr @margarineAlloc(i32 16)
  store i64 %addi.i, ptr %name.i, align 4
  %field_ptr5.i = getelementptr inbounds nuw i8, ptr %name.i, i64 8
  store i64 %rems.i.decomposed, ptr %field_ptr5.i, align 4
  ret ptr %name.i
}

define noundef ptr @"std::duration::Duration::new"(i64 %0, i64 %1, ptr readnone captures(none) %2) local_unnamed_addr {
prelude:
  %.frozen = freeze i64 %1
  %divs = sdiv i64 %.frozen, 1000000000
  %addi = add i64 %divs, %0
  %3 = mul i64 %divs, 1000000000
  %rems.decomposed = sub i64 %.frozen, %3
  %name = tail call ptr @margarineAlloc(i32 16)
  store i64 %addi, ptr %name, align 4
  %field_ptr5 = getelementptr inbounds nuw i8, ptr %name, i64 8
  store i64 %rems.decomposed, ptr %field_ptr5, align 4
  ret ptr %name
}

declare i64 @now_secs(ptr) local_unnamed_addr

declare i64 @now_nanos(ptr) local_unnamed_addr

define noundef ptr @"std::duration::Duration::elapsed"(ptr readonly captures(none) %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name.i = tail call i64 @now_secs(ptr null)
  %name15.i = tail call i64 @now_nanos(ptr null)
  %name15.i.frozen = freeze i64 %name15.i
  %divs.i.i = sdiv i64 %name15.i.frozen, 1000000000
  %addi.i.i = add i64 %divs.i.i, %name.i
  %2 = mul i64 %divs.i.i, 1000000000
  %rems.i.i.decomposed = sub i64 %name15.i.frozen, %2
  %name.i.i = tail call noundef ptr @margarineAlloc(i32 16)
  store i64 %addi.i.i, ptr %name.i.i, align 4
  %field_ptr5.i.i = getelementptr inbounds nuw i8, ptr %name.i.i, i64 8
  store i64 %rems.i.i.decomposed, ptr %field_ptr5.i.i, align 4
  %load4.unpack.i = load i64, ptr %0, align 4
  %load4.elt20.i = getelementptr inbounds nuw i8, ptr %0, i64 8
  %load4.unpack21.i = load i64, ptr %load4.elt20.i, align 4
  %subi.i = sub i64 %rems.i.i.decomposed, %load4.unpack21.i
  %subi15.i = sub i64 %addi.i.i, %load4.unpack.i
  %icmp.i = icmp slt i64 %subi.i, 0
  %addi.i = add nsw i64 %subi.i, 1000000000
  %subi.lobit.i = ashr i64 %subi.i, 63
  %.016.i = add i64 %subi15.i, %subi.lobit.i
  %.0.i = select i1 %icmp.i, i64 %addi.i, i64 %subi.i
  %name.i14 = tail call noundef ptr @margarineAlloc(i32 16)
  store i64 %.016.i, ptr %name.i14, align 4
  %field_ptr29.i = getelementptr inbounds nuw i8, ptr %name.i14, i64 8
  store i64 %.0.i, ptr %field_ptr29.i, align 4
  ret ptr %name.i14
}

define noundef ptr @"std::duration::Duration::sub"(ptr readonly captures(none) %0, ptr readonly captures(none) %1, ptr readnone captures(none) %2) local_unnamed_addr {
prelude:
  %load1.unpack = load i64, ptr %0, align 4
  %load1.elt17 = getelementptr inbounds nuw i8, ptr %0, i64 8
  %load1.unpack18 = load i64, ptr %load1.elt17, align 4
  %load4.unpack = load i64, ptr %1, align 4
  %load4.elt20 = getelementptr inbounds nuw i8, ptr %1, i64 8
  %load4.unpack21 = load i64, ptr %load4.elt20, align 4
  %subi = sub i64 %load1.unpack18, %load4.unpack21
  %subi15 = sub i64 %load1.unpack, %load4.unpack
  %icmp = icmp slt i64 %subi, 0
  %addi = add nsw i64 %subi, 1000000000
  %subi.lobit = ashr i64 %subi, 63
  %.016 = add i64 %subi15, %subi.lobit
  %.0 = select i1 %icmp, i64 %addi, i64 %subi
  %name = tail call ptr @margarineAlloc(i32 16)
  store i64 %.016, ptr %name, align 4
  %field_ptr29 = getelementptr inbounds nuw i8, ptr %name, i64 8
  store i64 %.0, ptr %field_ptr29, align 4
  ret ptr %name
}

define noundef ptr @"std::duration::Duration::from_secs"(i64 %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name = tail call ptr @margarineAlloc(i32 16)
  store i64 %0, ptr %name, align 4
  %field_ptr1 = getelementptr inbounds nuw i8, ptr %name, i64 8
  store i64 0, ptr %field_ptr1, align 4
  ret ptr %name
}

define noundef ptr @"std::duration::Duration::from_secs_float"(double %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %mulfp = fmul double %0, 1.000000e+09
  %fp_to_si = fptosi double %mulfp to i64
  %fp_to_si.frozen = freeze i64 %fp_to_si
  %divs.i = sdiv i64 %fp_to_si.frozen, 1000000000
  %2 = mul i64 %divs.i, 1000000000
  %rems.i.decomposed = sub i64 %fp_to_si.frozen, %2
  %name.i = tail call noundef ptr @margarineAlloc(i32 16)
  store i64 %divs.i, ptr %name.i, align 4
  %field_ptr4.i = getelementptr inbounds nuw i8, ptr %name.i, i64 8
  store i64 %rems.i.decomposed, ptr %field_ptr4.i, align 4
  ret ptr %name.i
}

define noundef ptr @"std::duration::Duration::from_nanos"(i64 %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %.frozen = freeze i64 %0
  %divs = sdiv i64 %.frozen, 1000000000
  %2 = mul i64 %divs, 1000000000
  %rems.decomposed = sub i64 %.frozen, %2
  %name = tail call ptr @margarineAlloc(i32 16)
  store i64 %divs, ptr %name, align 4
  %field_ptr4 = getelementptr inbounds nuw i8, ptr %name, i64 8
  store i64 %rems.decomposed, ptr %field_ptr4, align 4
  ret ptr %name
}

define noundef ptr @"std::duration::Duration::from_millis"(i64 %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %.frozen = freeze i64 %0
  %divs = sdiv i64 %.frozen, 1000
  %2 = mul i64 %divs, 1000
  %rems.decomposed = sub i64 %.frozen, %2
  %muli = mul nsw i64 %rems.decomposed, 1000000
  %name = tail call ptr @margarineAlloc(i32 16)
  store i64 %divs, ptr %name, align 4
  %field_ptr4 = getelementptr inbounds nuw i8, ptr %name, i64 8
  store i64 %muli, ptr %field_ptr4, align 4
  ret ptr %name
}

define noundef ptr @"std::duration::Duration::from_micros"(i64 %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %.frozen = freeze i64 %0
  %divs = sdiv i64 %.frozen, 1000000
  %2 = mul i64 %divs, 1000000
  %rems.decomposed = sub i64 %.frozen, %2
  %muli = mul nsw i64 %rems.decomposed, 1000
  %name = tail call ptr @margarineAlloc(i32 16)
  store i64 %divs, ptr %name, align 4
  %field_ptr4 = getelementptr inbounds nuw i8, ptr %name, i64 8
  store i64 %muli, ptr %field_ptr4, align 4
  ret ptr %name
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read)
define %enumRef @"std::duration::Duration::is_zero"(ptr readonly captures(none) %0, ptr readnone captures(none) %1) local_unnamed_addr #1 {
prelude:
  %load1.unpack = load i64, ptr %0, align 4
  %icmp = icmp eq i64 %load1.unpack, 0
  br i1 %icmp, label %then, label %cont

then:                                             ; preds = %prelude
  %load1.elt11 = getelementptr inbounds nuw i8, ptr %0, i64 8
  %load1.unpack12 = load i64, ptr %load1.elt11, align 4
  %icmp15 = icmp eq i64 %load1.unpack12, 0
  %icast18 = zext i1 %icmp15 to i32
  br label %cont

cont:                                             ; preds = %prelude, %then
  %.sroa.06.0 = phi i32 [ %icast18, %then ], [ 0, %prelude ]
  %load25.fca.0.insert = insertvalue %enumRef poison, i32 %.sroa.06.0, 0
  %load25.fca.1.insert = insertvalue %enumRef %load25.fca.0.insert, ptr null, 1
  ret %enumRef %load25.fca.1.insert
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read)
define i64 @"std::duration::Duration::as_secs"(ptr readonly captures(none) %0, ptr readnone captures(none) %1) local_unnamed_addr #1 {
prelude:
  %load1.unpack = load i64, ptr %0, align 4
  ret i64 %load1.unpack
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read)
define double @"std::duration::Duration::as_secs_float"(ptr readonly captures(none) %0, ptr readnone captures(none) %1) local_unnamed_addr #1 {
prelude:
  %load1.unpack = load i64, ptr %0, align 4
  %load1.elt2 = getelementptr inbounds nuw i8, ptr %0, i64 8
  %load1.unpack3 = load i64, ptr %load1.elt2, align 4
  %icast = sitofp i64 %load1.unpack to double
  %icast7 = sitofp i64 %load1.unpack3 to double
  %divfp = fdiv double %icast7, 1.000000e+09
  %addfp = fadd double %divfp, %icast
  ret double %addfp
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read)
define i64 @"std::duration::Duration::as_millis"(ptr readonly captures(none) %0, ptr readnone captures(none) %1) local_unnamed_addr #1 {
prelude:
  %load1.unpack = load i64, ptr %0, align 4
  %load1.elt2 = getelementptr inbounds nuw i8, ptr %0, i64 8
  %load1.unpack3 = load i64, ptr %load1.elt2, align 4
  %muli = mul i64 %load1.unpack, 1000
  %divs = sdiv i64 %load1.unpack3, 1000000
  %addi = add i64 %divs, %muli
  ret i64 %addi
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read)
define i64 @"std::duration::Duration::as_micros"(ptr readonly captures(none) %0, ptr readnone captures(none) %1) local_unnamed_addr #1 {
prelude:
  %load1.unpack = load i64, ptr %0, align 4
  %load1.elt2 = getelementptr inbounds nuw i8, ptr %0, i64 8
  %load1.unpack3 = load i64, ptr %load1.elt2, align 4
  %muli = mul i64 %load1.unpack, 1000000
  %divs = sdiv i64 %load1.unpack3, 1000
  %addi = add i64 %divs, %muli
  ret i64 %addi
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read)
define i64 @"std::duration::Duration::as_nanos"(ptr readonly captures(none) %0, ptr readnone captures(none) %1) local_unnamed_addr #1 {
prelude:
  %load1.unpack = load i64, ptr %0, align 4
  %load1.elt2 = getelementptr inbounds nuw i8, ptr %0, i64 8
  %load1.unpack3 = load i64, ptr %load1.elt2, align 4
  %muli = mul i64 %load1.unpack, 1000000000
  %addi = add i64 %muli, %load1.unpack3
  ret i64 %addi
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read)
define range(i64 -9223372036854, 9223372036855) i64 @"std::duration::Duration::subsec_millis"(ptr readonly captures(none) %0, ptr readnone captures(none) %1) local_unnamed_addr #1 {
prelude:
  %load1.elt1 = getelementptr inbounds nuw i8, ptr %0, i64 8
  %load1.unpack2 = load i64, ptr %load1.elt1, align 4
  %divs = sdiv i64 %load1.unpack2, 1000000
  ret i64 %divs
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read)
define range(i64 -9223372036854775, 9223372036854776) i64 @"std::duration::Duration::subsec_micros"(ptr readonly captures(none) %0, ptr readnone captures(none) %1) local_unnamed_addr #1 {
prelude:
  %load1.elt1 = getelementptr inbounds nuw i8, ptr %0, i64 8
  %load1.unpack2 = load i64, ptr %load1.elt1, align 4
  %divs = sdiv i64 %load1.unpack2, 1000
  ret i64 %divs
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read)
define i64 @"std::duration::Duration::subsec_nanos"(ptr readonly captures(none) %0, ptr readnone captures(none) %1) local_unnamed_addr #1 {
prelude:
  %load1.elt1 = getelementptr inbounds nuw i8, ptr %0, i64 8
  %load1.unpack2 = load i64, ptr %load1.elt1, align 4
  ret i64 %load1.unpack2
}

define noundef ptr @"std::duration::Duration::add"(ptr readonly captures(none) %0, ptr readonly captures(none) %1, ptr readnone captures(none) %2) local_unnamed_addr {
prelude:
  %load1.unpack = load i64, ptr %0, align 4
  %load1.elt17 = getelementptr inbounds nuw i8, ptr %0, i64 8
  %load1.unpack18 = load i64, ptr %load1.elt17, align 4
  %load4.unpack = load i64, ptr %1, align 4
  %load4.elt20 = getelementptr inbounds nuw i8, ptr %1, i64 8
  %load4.unpack21 = load i64, ptr %load4.elt20, align 4
  %addi = add i64 %load4.unpack21, %load1.unpack18
  %addi15 = add i64 %load4.unpack, %load1.unpack
  %icmp = icmp sgt i64 %addi, 999999999
  %subi = add nsw i64 %addi, -1000000000
  %addi23 = zext i1 %icmp to i64
  %.016 = add i64 %addi15, %addi23
  %.0 = select i1 %icmp, i64 %subi, i64 %addi
  %name = tail call ptr @margarineAlloc(i32 16)
  store i64 %.016, ptr %name, align 4
  %field_ptr29 = getelementptr inbounds nuw i8, ptr %name, i64 8
  store i64 %.0, ptr %field_ptr29, align 4
  ret ptr %name
}

define i64 @"std::iter::Iter::sum"(ptr %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name = tail call ptr @margarineAlloc(i32 8)
  store i64 0, ptr %name, align 4
  %name7 = tail call ptr @margarineAlloc(i32 8)
  store ptr %name, ptr %name7, align 8
  %name.i.i = tail call ptr @margarineAlloc(i32 24)
  store ptr @"<closure>.2", ptr %name.i.i, align 8
  %name.repack7.i.i = getelementptr inbounds nuw i8, ptr %name.i.i, i64 8
  store ptr %name7, ptr %name.repack7.i.i, align 8
  %name.repack5.i.i = getelementptr inbounds nuw i8, ptr %name.i.i, i64 16
  store ptr %0, ptr %name.repack5.i.i, align 8
  %name7.i.i = tail call noundef ptr @margarineAlloc(i32 16)
  store ptr @"<closure>", ptr %name7.i.i, align 8
  %name7.repack9.i.i = getelementptr inbounds nuw i8, ptr %name7.i.i, i64 8
  store ptr %name.i.i, ptr %name7.repack9.i.i, align 8
  br label %loop_body.i

loop_body.i:                                      ; preds = %loop_body.i, %prelude
  %load1.unpack.unpack.i.i = load ptr, ptr %name7.i.i, align 8
  %load1.unpack.unpack10.i.i = load ptr, ptr %name7.repack9.i.i, align 8
  %name.i14.i = tail call %enumRef %load1.unpack.unpack.i.i(ptr %load1.unpack.unpack10.i.i)
  %name7.fca.0.extract1.i = extractvalue %enumRef %name.i14.i, 0
  %icmp.i = icmp eq i32 %name7.fca.0.extract1.i, 1
  br i1 %icmp.i, label %"std::iter::Iter::for_each.exit", label %loop_body.i

"std::iter::Iter::for_each.exit":                 ; preds = %loop_body.i
  %load16.unpack = load i64, ptr %name, align 4
  ret i64 %load16.unpack
}

define {} @"std::iter::Iter::for_each"(ptr %0, %funcRef %1, ptr readnone captures(none) %2) local_unnamed_addr {
prelude:
  %name.i = tail call ptr @margarineAlloc(i32 24)
  %.elt.i = extractvalue %funcRef %1, 0
  store ptr %.elt.i, ptr %name.i, align 8
  %name.repack7.i = getelementptr inbounds nuw i8, ptr %name.i, i64 8
  %.elt8.i = extractvalue %funcRef %1, 1
  store ptr %.elt8.i, ptr %name.repack7.i, align 8
  %name.repack5.i = getelementptr inbounds nuw i8, ptr %name.i, i64 16
  store ptr %0, ptr %name.repack5.i, align 8
  %name7.i = tail call noundef ptr @margarineAlloc(i32 16)
  store ptr @"<closure>", ptr %name7.i, align 8
  %name7.repack9.i = getelementptr inbounds nuw i8, ptr %name7.i, i64 8
  store ptr %name.i, ptr %name7.repack9.i, align 8
  br label %loop_body

loop_body:                                        ; preds = %loop_body, %prelude
  %load1.unpack.unpack.i = load ptr, ptr %name7.i, align 8
  %load1.unpack.unpack10.i = load ptr, ptr %name7.repack9.i, align 8
  %name.i14 = tail call %enumRef %load1.unpack.unpack.i(ptr %load1.unpack.unpack10.i)
  %name7.fca.0.extract1 = extractvalue %enumRef %name.i14, 0
  %icmp = icmp eq i32 %name7.fca.0.extract1, 1
  br i1 %icmp, label %then, label %loop_body

then:                                             ; preds = %loop_body
  ret {} zeroinitializer
}

define noundef ptr @"std::iter::Iter::map"(ptr %0, %funcRef %1, ptr readnone captures(none) %2) local_unnamed_addr {
prelude:
  %name = tail call ptr @margarineAlloc(i32 24)
  %.elt = extractvalue %funcRef %1, 0
  store ptr %.elt, ptr %name, align 8
  %name.repack7 = getelementptr inbounds nuw i8, ptr %name, i64 8
  %.elt8 = extractvalue %funcRef %1, 1
  store ptr %.elt8, ptr %name.repack7, align 8
  %name.repack5 = getelementptr inbounds nuw i8, ptr %name, i64 16
  store ptr %0, ptr %name.repack5, align 8
  %name7 = tail call ptr @margarineAlloc(i32 16)
  store ptr @"<closure>", ptr %name7, align 8
  %name7.repack9 = getelementptr inbounds nuw i8, ptr %name7, i64 8
  store ptr %name, ptr %name7.repack9, align 8
  ret ptr %name7
}

define %enumRef @"<closure>"(ptr readonly captures(none) %0) {
prelude:
  %load1.unpack.unpack = load ptr, ptr %0, align 8
  %load1.unpack.elt40 = getelementptr inbounds nuw i8, ptr %0, i64 8
  %load1.unpack.unpack41 = load ptr, ptr %load1.unpack.elt40, align 8
  %load1.elt37 = getelementptr inbounds nuw i8, ptr %0, i64 16
  %load1.unpack38 = load ptr, ptr %load1.elt37, align 8
  %load1.unpack.unpack.i = load ptr, ptr %load1.unpack38, align 8
  %load1.unpack.elt9.i = getelementptr inbounds nuw i8, ptr %load1.unpack38, i64 8
  %load1.unpack.unpack10.i = load ptr, ptr %load1.unpack.elt9.i, align 8
  %name.i = tail call %enumRef %load1.unpack.unpack.i(ptr %load1.unpack.unpack10.i)
  %name.fca.0.extract14 = extractvalue %enumRef %name.i, 0
  %icmp = icmp eq i32 %name.fca.0.extract14, 0
  br i1 %icmp, label %cont, label %common.ret

common.ret:                                       ; preds = %prelude, %cont
  %common.ret.op = phi %enumRef [ %load.fca.1.insert.i, %cont ], [ %name.i, %prelude ]
  ret %enumRef %common.ret.op

cont:                                             ; preds = %prelude
  %name.fca.1.extract = extractvalue %enumRef %name.i, 1
  %load16 = load i64, ptr %name.fca.1.extract, align 4
  %name26 = tail call {} %load1.unpack.unpack(i64 %load16, ptr %load1.unpack.unpack41)
  %name.i43 = tail call ptr @margarineAlloc(i32 8)
  store ptr null, ptr %name.i43, align 8
  %load.fca.1.insert.i = insertvalue %enumRef { i32 0, ptr poison }, ptr %name.i43, 1
  br label %common.ret
}

define %enumRef @"std::iter::Iter::__next__"(ptr readonly captures(none) %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %load1.unpack.unpack = load ptr, ptr %0, align 8
  %load1.unpack.elt9 = getelementptr inbounds nuw i8, ptr %0, i64 8
  %load1.unpack.unpack10 = load ptr, ptr %load1.unpack.elt9, align 8
  %name = tail call %enumRef %load1.unpack.unpack(ptr %load1.unpack.unpack10)
  ret %enumRef %name
}

define %enumRef @Option({} %0) local_unnamed_addr {
prelude:
  %name = tail call ptr @margarineAlloc(i32 8)
  store ptr null, ptr %name, align 8
  %load.fca.1.insert = insertvalue %enumRef { i32 0, ptr poison }, ptr %name, 1
  ret %enumRef %load.fca.1.insert
}

define %enumRef @"std::iter::Iter::__next__.1"(ptr readonly captures(none) %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %load1.unpack.unpack = load ptr, ptr %0, align 8
  %load1.unpack.elt9 = getelementptr inbounds nuw i8, ptr %0, i64 8
  %load1.unpack.unpack10 = load ptr, ptr %load1.unpack.elt9, align 8
  %name = tail call %enumRef %load1.unpack.unpack(ptr %load1.unpack.unpack10)
  ret %enumRef %name
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(readwrite, inaccessiblemem: none)
define {} @"<closure>.2"(i64 %0, ptr readonly captures(none) %1) #2 {
prelude:
  %load1.unpack = load ptr, ptr %1, align 8
  %load4.unpack = load i64, ptr %load1.unpack, align 4
  %addi = add i64 %load4.unpack, %0
  store i64 %addi, ptr %load1.unpack, align 4
  ret {} zeroinitializer
}

define %enumRef @"std::io::read_line"(ptr readnone captures(none) %0) local_unnamed_addr {
prelude:
  %name = tail call %enumRef @io_read_line(ptr null)
  ret %enumRef %name
}

declare %enumRef @io_read_line(ptr) local_unnamed_addr

define noundef ptr @"str::lines"(ptr %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name = tail call ptr @str_lines_iter(ptr %0, ptr null)
  %name9 = tail call ptr @margarineAlloc(i32 8)
  store ptr %name, ptr %name9, align 8
  %name13 = tail call ptr @margarineAlloc(i32 16)
  store ptr @"<closure>.3", ptr %name13, align 8
  %name13.repack9 = getelementptr inbounds nuw i8, ptr %name13, i64 8
  store ptr %name9, ptr %name13.repack9, align 8
  ret ptr %name13
}

declare ptr @str_lines_iter(ptr, ptr) local_unnamed_addr

define %enumRef @"<closure>.3"(ptr readonly captures(none) %0) {
prelude:
  %load1.unpack = load ptr, ptr %0, align 8
  %name.i = tail call %enumRef @str_lines_iter_next(ptr %load1.unpack, ptr null)
  ret %enumRef %name.i
}

define %enumRef @"std::string::Lines::__next__"(ptr %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name = tail call %enumRef @str_lines_iter_next(ptr %0, ptr null)
  ret %enumRef %name
}

declare %enumRef @str_lines_iter_next(ptr, ptr) local_unnamed_addr

define ptr @"str::split_at"(ptr %0, i64 %1, ptr readnone captures(none) %2) local_unnamed_addr {
prelude:
  %name = tail call ptr @str_split_at(ptr %0, i64 %1, ptr null)
  ret ptr %name
}

declare ptr @str_split_at(ptr, i64, ptr) local_unnamed_addr

define %enumRef @"str::is_empty"(ptr %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name.i = tail call i64 @str_len(ptr %0, ptr null)
  %icmp = icmp eq i64 %name.i, 0
  %icast = zext i1 %icmp to i32
  %load10.fca.0.insert = insertvalue %enumRef poison, i32 %icast, 0
  %load10.fca.1.insert = insertvalue %enumRef %load10.fca.0.insert, ptr null, 1
  ret %enumRef %load10.fca.1.insert
}

define i64 @"str::len"(ptr %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name = tail call i64 @str_len(ptr %0, ptr null)
  ret i64 %name
}

declare i64 @str_len(ptr, ptr) local_unnamed_addr

define noundef ptr @"str::split"(ptr %0, ptr %1, ptr readnone captures(none) %2) local_unnamed_addr {
prelude:
  %name.i = tail call ptr @margarineAlloc(i32 8)
  store ptr %0, ptr %name.i, align 8
  %load2.fca.1.insert.i = insertvalue %enumRef { i32 0, ptr poison }, ptr %name.i, 1
  %name7 = tail call ptr @margarineAlloc(i32 24)
  store %enumRef %load2.fca.1.insert.i, ptr %name7, align 8
  %field_ptr9 = getelementptr inbounds nuw i8, ptr %name7, i64 16
  store ptr %1, ptr %field_ptr9, align 8
  %name15 = tail call ptr @margarineAlloc(i32 16)
  store ptr %1, ptr %name15, align 8
  %name15.repack10 = getelementptr inbounds nuw i8, ptr %name15, i64 8
  store ptr %name7, ptr %name15.repack10, align 8
  %name19 = tail call ptr @margarineAlloc(i32 16)
  store ptr @"<closure>.5", ptr %name19, align 8
  %name19.repack12 = getelementptr inbounds nuw i8, ptr %name19, i64 8
  store ptr %name15, ptr %name19.repack12, align 8
  ret ptr %name19
}

define %enumRef @Option.4(ptr %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name = tail call ptr @margarineAlloc(i32 8)
  store ptr %0, ptr %name, align 8
  %load2.fca.1.insert = insertvalue %enumRef { i32 0, ptr poison }, ptr %name, 1
  ret %enumRef %load2.fca.1.insert
}

define %enumRef @"<closure>.5"(ptr readonly captures(none) %0) {
prelude:
  %load1.elt99 = getelementptr inbounds nuw i8, ptr %0, i64 8
  %load1.unpack100 = load ptr, ptr %load1.elt99, align 8
  %load6.unpack = load %enumRef, ptr %load1.unpack100, align 8
  %load6.fca.0.0.extract = extractvalue %enumRef %load6.unpack, 0
  switch i32 %load6.fca.0.0.extract, label %switch_end [
    i32 0, label %switch_br
    i32 1, label %switch_end.sink.split
  ]

switch_end.sink.split:                            ; preds = %prelude, %cont50, %else42
  %load13.sink = phi ptr [ %load13, %else42 ], [ %load72.unpack, %cont50 ], [ null, %prelude ]
  %name.i112 = tail call ptr @margarineAlloc(i32 8)
  store ptr %load13.sink, ptr %name.i112, align 8
  br label %switch_end

switch_end:                                       ; preds = %switch_end.sink.split, %prelude
  %.sroa.385.0 = phi ptr [ undef, %prelude ], [ %name.i112, %switch_end.sink.split ]
  %load112.fca.1.insert = insertvalue %enumRef %load6.unpack, ptr %.sroa.385.0, 1
  ret %enumRef %load112.fca.1.insert

switch_br:                                        ; preds = %prelude
  %load1.unpack = load ptr, ptr %0, align 8
  %load6.fca.0.1.extract = extractvalue %enumRef %load6.unpack, 1
  %load13 = load ptr, ptr %load6.fca.0.1.extract, align 8
  %name.i = tail call %enumRef @str_split_once(ptr %load13, ptr %load1.unpack, ptr null)
  %.fca.0.extract.i = extractvalue %enumRef %name.i, 0
  %switch.selectcmp15.i = icmp eq i32 %.fca.0.extract.i, 0
  br i1 %switch.selectcmp15.i, label %cont50, label %else42

else42:                                           ; preds = %switch_br
  %name.i111 = tail call ptr @margarineAlloc(i32 8)
  store ptr null, ptr %name.i111, align 8
  %load.fca.1.insert.i = insertvalue %enumRef { i32 1, ptr poison }, ptr %name.i111, 1
  store %enumRef %load.fca.1.insert.i, ptr %load1.unpack100, align 8
  br label %switch_end.sink.split

cont50:                                           ; preds = %switch_br
  %name.fca.1.extract = extractvalue %enumRef %name.i, 1
  %load53 = load ptr, ptr %name.fca.1.extract, align 8
  %load58.elt105 = getelementptr inbounds nuw i8, ptr %load53, i64 8
  %load58.unpack106 = load ptr, ptr %load58.elt105, align 8
  %name.i113 = tail call ptr @margarineAlloc(i32 8)
  store ptr %load58.unpack106, ptr %name.i113, align 8
  %load2.fca.1.insert.i114 = insertvalue %enumRef { i32 0, ptr poison }, ptr %name.i113, 1
  store %enumRef %load2.fca.1.insert.i114, ptr %load1.unpack100, align 8
  %load72.unpack = load ptr, ptr %load53, align 8
  br label %switch_end.sink.split
}

define %enumRef @"str::split_once"(ptr %0, ptr %1, ptr readnone captures(none) %2) local_unnamed_addr {
prelude:
  %name = tail call %enumRef @str_split_once(ptr %0, ptr %1, ptr null)
  ret %enumRef %name
}

declare %enumRef @str_split_once(ptr, ptr, ptr) local_unnamed_addr

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define %enumRef @"Option::is_some"(%enumRef %0, ptr readnone captures(none) %1) local_unnamed_addr #3 {
prelude:
  %.fca.0.extract = extractvalue %enumRef %0, 0
  %switch.selectcmp15 = icmp eq i32 %.fca.0.extract, 0
  %switch.select16 = zext i1 %switch.selectcmp15 to i32
  %load14.fca.0.insert = insertvalue %enumRef poison, i32 %switch.select16, 0
  %load14.fca.1.insert = insertvalue %enumRef %load14.fca.0.insert, ptr null, 1
  ret %enumRef %load14.fca.1.insert
}

define %enumRef @Option.6(ptr readnone captures(none) %0) local_unnamed_addr {
prelude:
  %name = tail call ptr @margarineAlloc(i32 8)
  store ptr null, ptr %name, align 8
  %load.fca.1.insert = insertvalue %enumRef { i32 1, ptr poison }, ptr %name, 1
  ret %enumRef %load.fca.1.insert
}

define ptr @"str::slice"(ptr %0, ptr readonly captures(none) %1, ptr readnone captures(none) %2) local_unnamed_addr {
prelude:
  %load4.unpack = load i64, ptr %1, align 4
  %load4.elt10 = getelementptr inbounds nuw i8, ptr %1, i64 8
  %load4.unpack11 = load i64, ptr %load4.elt10, align 4
  %name = tail call ptr @str_slice(ptr %0, i64 %load4.unpack, i64 %load4.unpack11, ptr null)
  ret ptr %name
}

declare ptr @str_slice(ptr, i64, i64, ptr) local_unnamed_addr

define ptr @"str::nth"(ptr %0, i64 %1, ptr readnone captures(none) %2) local_unnamed_addr {
prelude:
  %name = tail call ptr @str_nth(ptr %0, i64 %1, ptr null)
  ret ptr %name
}

declare ptr @str_nth(ptr, i64, ptr) local_unnamed_addr

define noundef ptr @"str::chars"(ptr %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name = tail call ptr @margarineAlloc(i32 8)
  store ptr %0, ptr %name, align 8
  %name4 = tail call ptr @margarineAlloc(i32 8)
  store ptr %name, ptr %name4, align 8
  %name8 = tail call ptr @margarineAlloc(i32 16)
  store ptr @"<closure>.7", ptr %name8, align 8
  %name8.repack2 = getelementptr inbounds nuw i8, ptr %name8, i64 8
  store ptr %name4, ptr %name8.repack2, align 8
  ret ptr %name8
}

define %enumRef @"<closure>.7"(ptr readonly captures(none) %0) {
prelude:
  %load1.unpack = load ptr, ptr %0, align 8
  %load4.unpack = load ptr, ptr %load1.unpack, align 8
  %name.i.i = tail call i64 @str_len(ptr %load4.unpack, ptr null)
  %icmp.i = icmp eq i64 %name.i.i, 0
  br i1 %icmp.i, label %common.ret, label %cont

common.ret.sink.split:                            ; preds = %cont, %cont47
  %load76.unpack68.sink = phi ptr [ %load76.unpack68, %cont47 ], [ @str, %cont ]
  %load76.unpack.sink.ph = phi ptr [ %load76.unpack, %cont47 ], [ %load49.unpack, %cont ]
  store ptr %load76.unpack68.sink, ptr %load1.unpack, align 8
  br label %common.ret

common.ret:                                       ; preds = %common.ret.sink.split, %prelude
  %load76.unpack.sink = phi ptr [ null, %prelude ], [ %load76.unpack.sink.ph, %common.ret.sink.split ]
  %.pn = phi %enumRef [ { i32 1, ptr poison }, %prelude ], [ { i32 0, ptr poison }, %common.ret.sink.split ]
  %name.i74 = tail call ptr @margarineAlloc(i32 8)
  store ptr %load76.unpack.sink, ptr %name.i74, align 8
  %common.ret.op = insertvalue %enumRef %.pn, ptr %name.i74, 1
  ret %enumRef %common.ret.op

cont:                                             ; preds = %prelude
  %load25.unpack = load ptr, ptr %load1.unpack, align 8
  %name.i71 = tail call i64 @str_len(ptr %load25.unpack, ptr null)
  %icmp = icmp eq i64 %name.i71, 1
  %load49.unpack = load ptr, ptr %load1.unpack, align 8
  br i1 %icmp, label %common.ret.sink.split, label %cont47

cont47:                                           ; preds = %cont
  %name.i73 = tail call ptr @str_split_at(ptr %load49.unpack, i64 1, ptr null)
  %load76.unpack = load ptr, ptr %name.i73, align 8
  %load76.elt67 = getelementptr inbounds nuw i8, ptr %name.i73, i64 8
  %load76.unpack68 = load ptr, ptr %load76.elt67, align 8
  br label %common.ret.sink.split
}

define {} @"std::assert"(%enumRef %0, ptr %1, ptr readnone captures(none) %2) local_unnamed_addr {
prelude:
  %.sroa.08.sroa.0 = alloca i8, align 8
  %.fca.0.extract = extractvalue %enumRef %0, 0
  %icast = trunc i32 %.fca.0.extract to i1
  %bnot = xor i1 %icast, true
  store i1 %bnot, ptr %.sroa.08.sroa.0, align 8
  %.sroa.08.sroa.0.0..sroa.08.sroa.0.0..sroa.08.sroa.0.0..sroa.08.sroa.0.0..sroa.08.sroa.0.0..sroa.08.0.load2.fca.0.load = load i8, ptr %.sroa.08.sroa.0, align 8
  %icast5 = trunc i8 %.sroa.08.sroa.0.0..sroa.08.sroa.0.0..sroa.08.sroa.0.0..sroa.08.sroa.0.0..sroa.08.sroa.0.0..sroa.08.0.load2.fca.0.load to i1
  br i1 %icast5, label %then, label %cont

then:                                             ; preds = %prelude
  %name = tail call {} @panic(ptr %1, ptr null)
  br label %cont

cont:                                             ; preds = %prelude, %then
  ret {} zeroinitializer
}

declare {} @panic(ptr, ptr) local_unnamed_addr

define %enumRef @"Range::__next__"(ptr captures(none) %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %load1.unpack = load i64, ptr %0, align 4
  %load1.elt27 = getelementptr inbounds nuw i8, ptr %0, i64 8
  %load1.unpack28 = load i64, ptr %load1.elt27, align 4
  %icmp = icmp slt i64 %load1.unpack, %load1.unpack28
  br i1 %icmp, label %then, label %else

then:                                             ; preds = %prelude
  %addi = add nsw i64 %load1.unpack, 1
  store i64 %addi, ptr %0, align 4
  %name.i = tail call ptr @margarineAlloc(i32 8)
  store i64 %load1.unpack, ptr %name.i, align 4
  br label %cont

else:                                             ; preds = %prelude
  %name.i33 = tail call ptr @margarineAlloc(i32 8)
  store ptr null, ptr %name.i33, align 8
  br label %cont

cont:                                             ; preds = %else, %then
  %.sroa.020.0 = phi i32 [ 0, %then ], [ 1, %else ]
  %name.i.pn = phi ptr [ %name.i, %then ], [ %name.i33, %else ]
  %load37.fca.0.insert = insertvalue %enumRef poison, i32 %.sroa.020.0, 0
  %load37.fca.1.insert = insertvalue %enumRef %load37.fca.0.insert, ptr %name.i.pn, 1
  ret %enumRef %load37.fca.1.insert
}

define %enumRef @Option.8(i64 %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name = tail call ptr @margarineAlloc(i32 8)
  store i64 %0, ptr %name, align 4
  %load2.fca.1.insert = insertvalue %enumRef { i32 0, ptr poison }, ptr %name, 1
  ret %enumRef %load2.fca.1.insert
}

define %enumRef @Option.9(ptr readnone captures(none) %0) local_unnamed_addr {
prelude:
  %name = tail call ptr @margarineAlloc(i32 8)
  store ptr null, ptr %name, align 8
  %load.fca.1.insert = insertvalue %enumRef { i32 1, ptr poison }, ptr %name, 1
  ret %enumRef %load.fca.1.insert
}

define noundef ptr @"Range::iter"(ptr %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name = tail call ptr @margarineAlloc(i32 8)
  store ptr %0, ptr %name, align 8
  %name5 = tail call ptr @margarineAlloc(i32 16)
  store ptr @"<closure>.10", ptr %name5, align 8
  %name5.repack2 = getelementptr inbounds nuw i8, ptr %name5, i64 8
  store ptr %name, ptr %name5.repack2, align 8
  ret ptr %name5
}

define %enumRef @"<closure>.10"(ptr readonly captures(none) %0) {
prelude:
  %load1.unpack = load ptr, ptr %0, align 8
  %load1.unpack.i = load i64, ptr %load1.unpack, align 4
  %load1.elt27.i = getelementptr inbounds nuw i8, ptr %load1.unpack, i64 8
  %load1.unpack28.i = load i64, ptr %load1.elt27.i, align 4
  %icmp.i = icmp slt i64 %load1.unpack.i, %load1.unpack28.i
  br i1 %icmp.i, label %then.i, label %else.i

then.i:                                           ; preds = %prelude
  %addi.i = add nsw i64 %load1.unpack.i, 1
  store i64 %addi.i, ptr %load1.unpack, align 4
  %name.i.i = tail call ptr @margarineAlloc(i32 8)
  store i64 %load1.unpack.i, ptr %name.i.i, align 4
  br label %"Range::__next__.exit"

else.i:                                           ; preds = %prelude
  %name.i33.i = tail call ptr @margarineAlloc(i32 8)
  store ptr null, ptr %name.i33.i, align 8
  br label %"Range::__next__.exit"

"Range::__next__.exit":                           ; preds = %then.i, %else.i
  %.sroa.020.0.i = phi i32 [ 0, %then.i ], [ 1, %else.i ]
  %name.i.pn.i = phi ptr [ %name.i.i, %then.i ], [ %name.i33.i, %else.i ]
  %load37.fca.0.insert.i = insertvalue %enumRef poison, i32 %.sroa.020.0.i, 0
  %load37.fca.1.insert.i = insertvalue %enumRef %load37.fca.0.insert.i, ptr %name.i.pn.i, 1
  ret %enumRef %load37.fca.1.insert.i
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define range(i64 0, -9223372036854775807) i64 @"int::abs"(i64 %0, ptr readnone captures(none) %1) local_unnamed_addr #3 {
prelude:
  %subi = tail call i64 @llvm.abs.i64(i64 %0, i1 false)
  ret i64 %subi
}

define ptr @"int::to_str"(i64 %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name = tail call ptr @int_to_str(i64 %0, ptr null)
  ret ptr %name
}

declare ptr @int_to_str(i64, ptr) local_unnamed_addr

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define i64 @"int::max"(i64 %0, i64 %1, ptr readnone captures(none) %2) local_unnamed_addr #3 {
prelude:
  %. = tail call i64 @llvm.smax.i64(i64 %0, i64 %1)
  ret i64 %.
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define i64 @"int::min"(i64 %0, i64 %1, ptr readnone captures(none) %2) local_unnamed_addr #3 {
prelude:
  %. = tail call i64 @llvm.smin.i64(i64 %0, i64 %1)
  ret i64 %.
}

define i64 @"int::pow"(i64 %0, i64 %1, ptr readnone captures(none) %2) local_unnamed_addr {
prelude:
  %name = tail call ptr @margarineAlloc(i32 16)
  store i64 0, ptr %name, align 4
  %field_ptr1 = getelementptr inbounds nuw i8, ptr %name, i64 8
  store i64 %1, ptr %field_ptr1, align 4
  %icmp.i14 = icmp sgt i64 %1, 0
  br i1 %icmp.i14, label %cont, label %then

then:                                             ; preds = %cont, %prelude
  %.0.lcssa = phi i64 [ 1, %prelude ], [ %muli, %cont ]
  %name.i33.i = tail call ptr @margarineAlloc(i32 8)
  store ptr null, ptr %name.i33.i, align 8
  ret i64 %.0.lcssa

cont:                                             ; preds = %prelude, %cont
  %load1.unpack.i16 = phi i64 [ %load1.unpack.i, %cont ], [ 0, %prelude ]
  %.015 = phi i64 [ %muli, %cont ], [ 1, %prelude ]
  %addi.i = add nsw i64 %load1.unpack.i16, 1
  store i64 %addi.i, ptr %name, align 4
  %name.i.i = tail call ptr @margarineAlloc(i32 8)
  store i64 %load1.unpack.i16, ptr %name.i.i, align 4
  %muli = mul i64 %.015, %0
  %load1.unpack.i = load i64, ptr %name, align 4
  %load1.unpack28.i = load i64, ptr %field_ptr1, align 4
  %icmp.i = icmp slt i64 %load1.unpack.i, %load1.unpack28.i
  br i1 %icmp.i, label %cont, label %then
}

define ptr @"float::to_str"(double %0, ptr readnone captures(none) %1) local_unnamed_addr {
prelude:
  %name = tail call ptr @float_to_str(double %0, ptr null)
  ret ptr %name
}

declare ptr @float_to_str(double, ptr) local_unnamed_addr

; Function Attrs: nofree nosync nounwind memory(none)
define i64 @fib(i64 %0, ptr readnone captures(none) %1) local_unnamed_addr #4 {
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

; Function Attrs: nofree nosync nounwind memory(none)
define {} @main(ptr readnone captures(none) %0) local_unnamed_addr #4 {
prelude:
  %name = tail call i64 @fib(i64 40, ptr poison)
  ret {} zeroinitializer
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.abs.i64(i64, i1 immarg) #5

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.smax.i64(i64, i64) #5

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.smin.i64(i64, i64) #5

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: write)
declare void @llvm.memset.p0.i64(ptr writeonly captures(none), i8, i64, i1 immarg) #6

attributes #0 = { noreturn }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
attributes #2 = { mustprogress nofree norecurse nosync nounwind willreturn memory(readwrite, inaccessiblemem: none) }
attributes #3 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) }
attributes #4 = { nofree nosync nounwind memory(none) }
attributes #5 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #6 = { nocallback nofree nounwind willreturn memory(argmem: write) }
