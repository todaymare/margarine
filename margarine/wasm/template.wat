(func $push (param $amount i32)
    ;; Increment the stack pointer
    (global.set $stack_pointer (i32.add (global.get $stack_pointer) (local.get $amount)))
)

(func $pop (param $amount i32) 
    ;; Decrement the stack pointer
    (global.set $stack_pointer (i32.sub (global.get $stack_pointer) (local.get $amount)))
)


(func $write_i32_to_stack (param $data i32) (param $ptr i32)
    (i32.store (local.get $ptr) (local.get $data))
)


(func $write_i64_to_stack (param $data i64) (param $ptr i32)
    (i64.store (local.get $ptr) (local.get $data))
)


(func $write_f32_to_stack (param $data f32) (param $ptr i32)
    (f32.store (local.get $ptr) (local.get $data))
)


(func $write_f64_to_stack (param $data f64) (param $ptr i32)
    (f64.store (local.get $ptr) (local.get $data))
)

(func $copy_memory
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
