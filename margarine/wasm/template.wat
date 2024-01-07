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
