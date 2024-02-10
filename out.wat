(module (memory (export "memory") 65536)(global $string_pointer i32 (i32.const 32768))(global $host_memory_offset (export "host_memory_offset") (mut i64) (i64.const 0))(global $stack_pointer (export "stack_pointer") (mut i32) (i32.const 32768))(global $bstack_pointer (export "bstack_pointer") i32 (i32.const 32768))
;;
;; TEMPLATE START
;;
;;
;; Increments the stack pointer by `amount`
;;
(func $push (export "push") (param $amount i32) (local $ptr i32)
    (i32.sub (global.get $stack_pointer) (local.get $amount))
    local.tee $ptr
    i32.const 0
    i32.le_s
    (if (then 
        unreachable
    ))

    (global.set $stack_pointer (local.get $ptr))
)

;;
;; Decrements the stack pointer by `amount`
;;
(func $pop (export "pop") (param $amount i32) (local $ptr i32)
    (i32.add (global.get $stack_pointer) (local.get $amount))
    local.tee $ptr
    global.get $bstack_pointer
    i32.gt_s
    (if (then 
        unreachable
    ))

    (global.set $stack_pointer (local.get $ptr))
)


(func $write_i32_to_stack (export "write_i32_to_stack") (param $data i32) (param $ptr i32)
    (i32.store (local.get $ptr) (local.get $data))
)


(func $write_i64_to_stack (export "write_i64_to_stack") (param $data i64) (param $ptr i32)
    (i64.store (local.get $ptr) (local.get $data))
)


(func $write_f32_to_stack (export "write_f32_to_stack") (param $data f32) (param $ptr i32)
    (f32.store (local.get $ptr) (local.get $data))
)


(func $write_f64_to_stack (export "write_f64_to_stack") (param $data f64) (param $ptr i32)
    (f64.store (local.get $ptr) (local.get $data))
)

;;
;; Copies `length` bytes from `source_offset` to `dest_offset`
;;
(func $memcpy
    (export "memcpy")
    (param $source_offset i32)
    (param $dest_offset i32)
    (param $length i32)

    ;; Copy memory from source to destination
    (memory.copy
        (local.get $dest_offset)   ;; destination offset
        (local.get $source_offset) ;; source offset
        (local.get $length)        ;; length
    )
)


;;
;; Checks if the following `length` bytes of `v1` are
;; equal to the following `length` bytes of `v2`. Returning
;; 1 if they are and 0 if they are not.
;;
(func $bcmp
    (export "bcmp")
    (param $v1 i32)
    (param $v2 i32)
    (param $length i32)
    (result i32)

    local.get $v1
    local.get $v2
    i32.eq
    (if (then 
        i32.const 1
        return
    ))

    (loop $loop
        (i32.load8_u (local.get $v1))
        (i32.load8_u (local.get $v2))
        i32.ne

        if
            (return (i32.const 0))
        end
        
        (local.set $v1 (i32.add (local.get $v1) (i32.const 1)))
        (local.set $v2 (i32.add (local.get $v2) (i32.const 1)))

        (local.tee $length (i32.sub (local.get $length) (i32.const 1)))
        br_if $loop
    )

    i32.const 1
)

;;
;; TEMPLATE OVER
;;
(global $s_0 i32 (i32.const 0))(global $s_1 i32 (i32.const 0))(global $s_2 i32 (i32.const 4))(global $s_3 i32 (i32.const 4))(global $s_4 i32 (i32.const 4))(global $s_6 i32 (i32.const 4))(func $_0 (export "0")(result i32)(local $_ret i32)(block $_ret i32.const 0 local.set $_ret br $_ret)local.get $_ret return)(func $_1 (export "1")(result i32)(local $_ret i32)(block $_ret i32.const 1 local.set $_ret br $_ret)local.get $_ret return)(func $_2 (export "_init") (block $_ret i64.const 0 drop i64.const 0 drop i64.const 69 (i32.add (global.get $stack_pointer) (i32.const 4)) global.get $s_2(call $push) call $_6 global.get $s_2(call $pop) (i32.add (global.get $stack_pointer) (i32.const 4)) drop br $_ret)return)(func $_3 (export "3")(param i32) (param i32) (local $_ret i32)(global.get $stack_pointer)(block $_ret i32.const 0 (i32.add (global.get $stack_pointer) (i32.const 4)) (call $write_i32_to_stack) local.get 0 (i32.add (global.get $stack_pointer) (i32.const 8)) i32.const 0 (call $memcpy) (i32.add (global.get $stack_pointer) (i32.const 4)) local.set $_ret br $_ret)local.get $_ret local.get 1 i32.const 4 call $memcpy return)(func $_4 (export "4")(param i32) (local $_ret i32)(global.get $stack_pointer)(block $_ret i32.const 1 (i32.add (global.get $stack_pointer) (i32.const 4)) (call $write_i32_to_stack) (i32.add (global.get $stack_pointer) (i32.const 4)) local.set $_ret br $_ret)local.get $_ret local.get 0 i32.const 4 call $memcpy return)(func $_6 (export "6")(param i64) (local $_ret i32)(global.get $stack_pointer)(block $_ret i64.const 0 local.set $_ret br $_ret)local.get $_ret local.get 0 i32.const 4 call $memcpy return))