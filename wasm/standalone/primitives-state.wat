;; Stable pointers, weak pointers, and cooperative thread operations.
;; Every operation is a leaf: it returns a graph, raises an exception, or parks
;; the running thread. The evaluator owns all evaluation and resumption.

;; Require a typed runtime handle and return its managed descriptor.
(func $state_handle (param $node i32) (param $tag i32) (result i32)
  (if (i32.ne (call $tag (local.get $node)) (local.get $tag))
    (then (call $fail (i32.const 92)) (unreachable)))
  (i32.load offset=4 (local.get $node)))

;; Encode Maybe with the same Scott constructors as the C and Rust runtimes.
(func $state_maybe (param $value i32) (result i32)
  (if (result i32) (local.get $value)
    (then (call $ap (call $ap (call $prim (global.get $T_Z))
      (call $prim (global.get $T_U))) (local.get $value)))
    (else (call $prim (global.get $T_K)))))

;; Stable pointer slot zero is invalid. Freed slots are reused in ascending
;; order. The managed table strongly roots each occupied slot during collection.
(func $stable_new (param $value i32) (result i32)
  (local $table i32) (local $capacity i32) (local $index i32) (local $new i32)
  (local.set $table (i32.load (i32.const 0x30b0)))
  (local.set $capacity (i32.load (i32.const 0x30b4)))
  (if (i32.eqz (local.get $table))
    (then
      (local.set $capacity (i32.const 256))
      (local.set $table (call $blob_alloc (i32.const 1024) (i32.const -1)))
      (i32.store (i32.const 0x30b0) (local.get $table))
      (i32.store (i32.const 0x30b4) (local.get $capacity))
      (i32.store (i32.const 0x30b8) (i32.const 1))))
  (local.set $index (i32.load (i32.const 0x30b8)))
  (block $found
    (loop $search
      (br_if $found (i32.ge_u (local.get $index) (local.get $capacity)))
      (br_if $found (i32.eqz (i32.load (i32.add (local.get $table)
        (i32.shl (local.get $index) (i32.const 2))))))
      (local.set $index (i32.add (local.get $index) (i32.const 1)))
      (br $search)))
  (if (i32.eq (local.get $index) (local.get $capacity))
    (then
      (if (i32.gt_u (local.get $capacity) (i32.const 134217728))
        (then (call $fail (i32.const 73)) (unreachable)))
      (local.set $new (call $blob_alloc (i32.shl (local.get $capacity) (i32.const 3)) (i32.const -1)))
      (memory.copy (local.get $new) (local.get $table)
        (i32.shl (local.get $capacity) (i32.const 2)))
      (call $blob_free (local.get $table))
      (local.set $table (local.get $new))
      (i32.store (i32.const 0x30b0) (local.get $table))
      (i32.store (i32.const 0x30b4) (i32.shl (local.get $capacity) (i32.const 1)))))
  (i32.store (i32.add (local.get $table) (i32.shl (local.get $index) (i32.const 2)))
    (local.get $value))
  (i32.store (i32.const 0x30b8) (i32.add (local.get $index) (i32.const 1)))
  (local.get $index))

;; Validate a stable handle before reading or releasing its slot.
(func $stable_slot (param $index i32) (result i32)
  (local $slot i32)
  (if (i32.or (i32.eqz (local.get $index))
        (i32.ge_u (local.get $index) (i32.load (i32.const 0x30b4))))
    (then (call $fail (i32.const 93)) (unreachable)))
  (local.set $slot (i32.add (i32.load (i32.const 0x30b0))
    (i32.shl (local.get $index) (i32.const 2))))
  (if (i32.eqz (i32.load (local.get $slot)))
    (then (call $fail (i32.const 93)) (unreachable)))
  (local.get $slot))

;; The weak registry contains metadata links, not strong graph references.
;; Its 32-byte records have no pointer mask. $weak_collect marks live fields.
(func $weak_new (param $key i32) (param $value i32) (param $finalizer i32) (result i32)
  (local $record i32) (local $node i32)
  (local.set $record (call $blob_alloc (i32.const 32) (i32.const 0)))
  (local.set $node (call $node (global.get $T_WEAK)))
  (i32.store offset=4 (local.get $node) (local.get $record))
  (i32.store (local.get $record) (i32.load (i32.const 0x30bc)))
  (i32.store offset=4 (local.get $record) (local.get $node))
  (i32.store offset=8 (local.get $record) (local.get $key))
  (i32.store offset=12 (local.get $record) (local.get $value))
  (if (local.get $finalizer)
    (then (i32.store offset=16 (local.get $record)
      (call $ap (local.get $finalizer) (i32.load (i32.const 0x3000))))))
  (i32.store (i32.const 0x30bc) (local.get $record))
  (local.get $node))

;; Weak-key liveness follows existing IND links without evaluation or marking.
;; The compression helper terminates on a pure IND cycle. Its representative
;; is live only when the completed strong/weak fixed-point marking reached it.
(func $weak_key_live (param $key i32) (result i32)
  (call $heap_is_marked (call $heap_gc_target (local.get $key))))

;; Compute weak-key reachability to a fixed point before clearing any dead
;; value. This permits chains whose registry order differs from dependency order.
;; Finalizer marking follows this phase and cannot revive a dead weak value.
(func $weak_collect
  (local $record i32) (local $next i32) (local $previous i32)
  (local $changed i32) (local $action i32) (local $tail i32)
  (local.set $record (i32.load (i32.const 0x30a8)))
  (block $queued_done
    (loop $queued
      (br_if $queued_done (i32.eqz (local.get $record)))
      (call $heap_mark (local.get $record))
      (call $heap_mark (i32.load offset=16 (local.get $record)))
      (call $heap_mark_drain)
      (local.set $record (i32.load offset=20 (local.get $record)))
      (br $queued)))
  (loop $fixed_point
    (local.set $changed (i32.const 0))
    (local.set $record (i32.load (i32.const 0x30bc)))
    (block $pass_done
      (loop $pass
        (br_if $pass_done (i32.eqz (local.get $record)))
        (if (i32.load offset=12 (local.get $record))
          (then
            (if (call $weak_key_live (i32.load offset=8 (local.get $record)))
              (then
                (call $heap_mark (local.get $record))
                (call $heap_mark (i32.load offset=4 (local.get $record)))
                (call $heap_mark (i32.load offset=8 (local.get $record)))
                (call $heap_mark (i32.load offset=12 (local.get $record)))
                (call $heap_mark (i32.load offset=16 (local.get $record)))
                (if (i32.load (i32.const 0x16c)) (then (local.set $changed (i32.const 1))))
                (call $heap_mark_drain)))))
        (local.set $record (i32.load (local.get $record)))
        (br $pass)))
    (br_if $fixed_point (local.get $changed)))
  (local.set $record (i32.load (i32.const 0x30bc)))
  (block $done
    (loop $sweep
      (br_if $done (i32.eqz (local.get $record)))
      (local.set $next (i32.load (local.get $record)))
      (if (i32.load offset=12 (local.get $record))
        (then
          (if (i32.eqz (call $weak_key_live (i32.load offset=8 (local.get $record))))
            (then
              (i32.store offset=8 (local.get $record) (i32.const 0))
              (i32.store offset=12 (local.get $record) (i32.const 0))
              (local.set $action (i32.load offset=16 (local.get $record)))
              (if (local.get $action)
                (then
                  (i32.store offset=28 (local.get $record) (i32.const 1))
                  (local.set $tail (i32.load (i32.const 0x30ac)))
                  (if (local.get $tail)
                    (then (i32.store offset=20 (local.get $tail) (local.get $record)))
                    (else (i32.store (i32.const 0x30a8) (local.get $record))))
                  (i32.store (i32.const 0x30ac) (local.get $record))))))))
      ;; Clear stale owner addresses before the node allocator can reuse them.
      (if (i32.eqz (call $heap_is_marked (i32.load offset=4 (local.get $record))))
        (then (i32.store offset=4 (local.get $record) (i32.const 0))))
      (if (i32.or (i32.load offset=12 (local.get $record))
            (i32.or (i32.load offset=4 (local.get $record))
              (i32.load offset=28 (local.get $record))))
        (then
          (call $heap_mark (local.get $record))
          (local.set $previous (local.get $record)))
        (else
          (if (local.get $previous)
            (then (i32.store (local.get $previous) (local.get $next)))
            (else (i32.store (i32.const 0x30bc) (local.get $next))))))
      (local.set $record (local.get $next))
      (br $sweep)))
  ;; Mark all newly queued actions only after all weak values were classified.
  (local.set $record (i32.load (i32.const 0x30a8)))
  (block $finalizers_done
    (loop $finalizers
      (br_if $finalizers_done (i32.eqz (local.get $record)))
      (call $heap_mark (i32.load offset=16 (local.get $record)))
      (call $heap_mark_drain)
      (local.set $record (i32.load offset=20 (local.get $record)))
      (br $finalizers)))
  (call $heap_mark_drain))

;; Park a primitive. The evaluator removes its frame and retries its head after
;; wakeup. Strict arguments remain memoized in the original application graph.
(func $state_park (param $object i32) (param $kind i32) (param $status i32)
  (local $thread i32)
  (local.set $thread (i32.load (i32.const 0x3084)))
  (i32.store offset=64 (local.get $thread) (local.get $object))
  (i32.store offset=68 (local.get $thread) (local.get $kind))
  (i32.store offset=4 (local.get $thread) (local.get $status))
  (i32.store (i32.const 0x309c) (i32.const 2)))

;; Wake the oldest waiter that can use the new occupancy. A runnable waiter
;; can lose its opportunity before it runs, so the queue can contain both kinds.
(func $mvar_wake_one (param $mvar i32)
  (local $thread i32) (local $kind i32)
  (local.set $kind (select (i32.const 1) (i32.const 2) (i32.load (local.get $mvar))))
  (local.set $thread (i32.load offset=4 (local.get $mvar)))
  (block $done
    (loop $search
      (br_if $done (i32.eqz (local.get $thread)))
      (if (i32.eq (i32.load offset=68 (local.get $thread)) (local.get $kind))
        (then
          (call $thread_queue_remove (i32.add (local.get $mvar) (i32.const 4))
            (i32.add (local.get $mvar) (i32.const 8)) (local.get $thread))
          (call $thread_runnable (local.get $thread))
          (return)))
      (local.set $thread (i32.load offset=12 (local.get $thread)))
      (br $search))))

;; Readers present at a successful put receive that exact value. A taker may
;; empty the MVar before a reader runs. The captured value remains a thread root.
(func $mvar_wake_readers (param $mvar i32) (param $value i32)
  (local $thread i32)
  (block $done
    (loop $readers
      (local.set $thread (call $thread_queue_pop (i32.add (local.get $mvar) (i32.const 12))
        (i32.add (local.get $mvar) (i32.const 16))))
      (br_if $done (i32.eqz (local.get $thread)))
      (i32.store offset=60 (local.get $thread) (local.get $value))
      (call $thread_runnable (local.get $thread))
      (br $readers))))

;; Dispatch stateful primitive groups. Unsupported tags return zero. A zero
;; with exception 0x3004 or switch request 2 must propagate to the evaluator.
(func $primitive_state (param $tag i32) (param $args i32) (result i32)
  (local $x i32) (local $y i32) (local $z i32) (local $world i32)
  (local $object i32) (local $thread i32) (local $value i32) (local $result i32)
  (local $index i32) (local $kind i32) (local $try i32) (local $error i32)
  (local.set $x (i32.load (local.get $args)))
  (local.set $y (i32.load offset=4 (local.get $args)))
  (local.set $z (i32.load offset=8 (local.get $args)))
  (local.set $thread (i32.load (i32.const 0x3084)))
  (if (i32.eq (local.get $tag) (global.get $T_SPNEW))
    (then (return (call $pair (call $int (call $stable_new (local.get $x))) (local.get $y)))))
  (if (i32.or (i32.eq (local.get $tag) (global.get $T_SPDEREF))
        (i32.eq (local.get $tag) (global.get $T_SPFREE)))
    (then
      (local.set $index (call $ival (local.get $x)))
      (local.set $object (call $stable_slot (local.get $index)))
      (local.set $value (i32.load (local.get $object)))
      (if (i32.eq (local.get $tag) (global.get $T_SPFREE))
        (then
          (i32.store (local.get $object) (i32.const 0))
          (if (i32.lt_u (local.get $index) (i32.load (i32.const 0x30b8)))
            (then (i32.store (i32.const 0x30b8) (local.get $index))))
          (local.set $value (call $prim (global.get $T_I)))))
      (return (call $pair (local.get $value) (local.get $y)))))
  (if (i32.or (i32.eq (local.get $tag) (global.get $T_WKNEW))
        (i32.eq (local.get $tag) (global.get $T_WKNEWFIN)))
    (then
      (local.set $world (local.get $z))
      (if (i32.eq (local.get $tag) (global.get $T_WKNEWFIN))
        (then (local.set $value (local.get $z))
          (local.set $world (i32.load offset=12 (local.get $args)))))
      (return (call $pair (call $weak_new (local.get $x) (local.get $y) (local.get $value))
        (local.get $world)))))
  (if (i32.or (i32.eq (local.get $tag) (global.get $T_WKDEREF))
        (i32.eq (local.get $tag) (global.get $T_WKFINAL)))
    (then
      (local.set $object (call $state_handle (local.get $x) (global.get $T_WEAK)))
      (if (i32.eq (local.get $tag) (global.get $T_WKDEREF))
        (then (return (call $pair (call $state_maybe (i32.load offset=12 (local.get $object)))
          (local.get $y)))))
      (local.set $value (i32.load offset=16 (local.get $object)))
      (i32.store offset=16 (local.get $object) (i32.const 0))
      (local.set $result (call $pair (call $prim (global.get $T_I)) (local.get $y)))
      (if (local.get $value)
        (then (local.set $result (call $ap (call $ap (call $prim (global.get $T_SEQ))
          (local.get $value)) (local.get $result)))))
      (return (local.get $result))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_FORK))
    (then
      (local.set $object (call $thread_new (call $ap (local.get $x) (local.get $y))
        (i32.const -1) (i32.load (i32.const 0x3040))))
      (call $thread_queue_push (i32.const 0x308c) (i32.const 0x3090) (local.get $object))
      (return (call $pair (i32.load offset=104 (local.get $object)) (local.get $y)))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_THID))
    (then (return (call $pair (i32.load offset=104 (local.get $thread)) (local.get $x)))))
  (if (i32.or (i32.eq (local.get $tag) (global.get $T_THNUM))
        (i32.eq (local.get $tag) (global.get $T_IO_THREADSTATUS)))
    (then
      (local.set $object (call $state_handle (local.get $x) (global.get $T_THID)))
      (if (i32.eq (local.get $tag) (global.get $T_THNUM))
        (then (return (call $int (i32.load (local.get $object))))))
      (local.set $value (i32.load offset=4 (local.get $object)))
      (local.set $kind (i32.load offset=68 (local.get $object)))
      (if (i32.or (i32.eq (local.get $kind) (i32.const 6))
            (i32.eq (local.get $kind) (i32.const 7)))
        (then (local.set $value (i32.const 5))))
      (return (call $pair (call $int (local.get $value)) (local.get $y)))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_YIELD))
    (then
      (i32.store (i32.const 0x309c) (i32.const 1))
      (return (call $pair (call $prim (global.get $T_I)) (local.get $x)))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_NEWMVAR))
    (then
      (local.set $object (call $blob_alloc (i32.const 32) (i32.const 31)))
      (local.set $value (call $node (global.get $T_MVAR)))
      (i32.store offset=4 (local.get $value) (local.get $object))
      (return (call $pair (local.get $value) (local.get $x)))))
  (if (i32.and (i32.ge_u (local.get $tag) (global.get $T_IO_TAKEMVAR))
        (i32.le_u (local.get $tag) (global.get $T_IO_TRYREADMVAR)))
    (then
      (local.set $try (i32.ge_u (local.get $tag) (global.get $T_IO_TRYTAKEMVAR)))
      (local.set $kind (i32.add (i32.rem_u (i32.sub (local.get $tag)
        (global.get $T_IO_TAKEMVAR)) (i32.const 3)) (i32.const 1)))
      (local.set $world (select (local.get $z) (local.get $y)
        (i32.eq (local.get $kind) (i32.const 2))))
      (if (i32.eqz (local.get $try))
        (then
          (local.set $value (call $thread_take_exception (i32.const 1)))
          (if (local.get $value) (then (return (call $raise (local.get $value)))))))
      (local.set $object (call $state_handle (local.get $x) (global.get $T_MVAR)))
      (local.set $value (i32.load (local.get $object)))
      (if (i32.eq (local.get $kind) (i32.const 3))
        (then
          (if (i32.load offset=60 (local.get $thread))
            (then
              (local.set $value (i32.load offset=60 (local.get $thread)))
              (i32.store offset=60 (local.get $thread) (i32.const 0))))))
      (if (i32.eq (local.get $kind) (i32.const 2))
        (then
          (if (i32.eqz (local.get $value))
            (then
              (i32.store (local.get $object) (local.get $y))
              (call $mvar_wake_readers (local.get $object) (local.get $y))
              (call $mvar_wake_one (local.get $object))
              (return (call $pair (select (call $boolean (i32.const 1))
                (call $prim (global.get $T_I)) (local.get $try)) (local.get $world)))))
          (if (local.get $try)
            (then (return (call $pair (call $boolean (i32.const 0)) (local.get $world))))))
        (else
          (if (local.get $value)
            (then
              (if (i32.eq (local.get $kind) (i32.const 1))
                (then (i32.store (local.get $object) (i32.const 0))
                  (call $mvar_wake_one (local.get $object))))
              (if (local.get $try) (then (local.set $value (call $state_maybe (local.get $value)))))
              (return (call $pair (local.get $value) (local.get $world)))))
          (if (local.get $try)
            (then (return (call $pair (call $state_maybe (i32.const 0)) (local.get $world)))))))
      (local.set $index (select (i32.const 12) (i32.const 4)
        (i32.eq (local.get $kind) (i32.const 3))))
      (call $thread_queue_push (i32.add (local.get $object) (local.get $index))
        (i32.add (local.get $object) (i32.add (local.get $index) (i32.const 4))) (local.get $thread))
      (call $state_park (local.get $object) (local.get $kind) (i32.const 1))
      (return (i32.const 0))))
  (if (i32.eq (local.get $tag) (global.get $T_IO_THROWTO))
    (then
      (local.set $value (call $thread_take_exception (i32.const 1)))
      (if (local.get $value) (then (return (call $raise (local.get $value)))))
      (local.set $object (call $state_handle (local.get $x) (global.get $T_THID)))
      (if (i32.lt_u (i32.load offset=4 (local.get $object)) (i32.const 3))
        (then
          (if (i32.eq (local.get $object) (local.get $thread))
            (then (return (call $raise (local.get $y)))))
          (if (i32.load offset=56 (local.get $object))
            (then
              (call $thread_queue_push (i32.add (local.get $object) (i32.const 88))
                (i32.add (local.get $object) (i32.const 92)) (local.get $thread))
              (call $state_park (local.get $object) (i32.const 5) (i32.const 2))
              (return (i32.const 0))))
          (i32.store offset=56 (local.get $object) (local.get $y))
          (i32.store (i32.const 0x30c8) (i32.add (i32.load (i32.const 0x30c8)) (i32.const 1)))
          (if (i32.ne (i32.load offset=44 (local.get $object)) (i32.const 2))
            (then (call $thread_unpark (local.get $object))))))
      (return (call $pair (call $prim (global.get $T_I)) (local.get $z)))))
  (if (i32.or (i32.eq (local.get $tag) (global.get $T_IO_THREADDELAY))
        (i32.or (i32.eq (local.get $tag) (global.get $T_IO_WAITRDFD))
          (i32.eq (local.get $tag) (global.get $T_IO_WAITWRFD))))
    (then
      (local.set $value (call $thread_take_exception (i32.const 1)))
      (if (local.get $value) (then (return (call $raise (local.get $value)))))
      (local.set $kind (select (i32.const 4)
        (select (i32.const 6) (i32.const 7)
          (i32.eq (local.get $tag) (global.get $T_IO_WAITRDFD)))
        (i32.eq (local.get $tag) (global.get $T_IO_THREADDELAY))))
      (if (i32.load offset=80 (local.get $thread))
        (then
          (i32.store offset=80 (local.get $thread) (i32.const 0))
          (local.set $error (i32.load offset=120 (local.get $thread)))
          (i32.store offset=120 (local.get $thread) (i32.const 0))
          (if (local.get $error) (then (i32.store (i32.const 0x240) (local.get $error))))
          (return (call $pair
            (if (result i32) (i32.eq (local.get $kind) (i32.const 4))
              (then (call $prim (global.get $T_I)))
              (else (call $int (select (i32.const -1) (i32.const 0) (local.get $error)))))
            (local.get $y)))))
      (local.set $index (call $ival (local.get $x)))
      (if (i32.eq (local.get $kind) (i32.const 4))
        (then
          (if (i32.le_s (local.get $index) (i32.const 0))
            (then
              (i32.store (i32.const 0x309c) (i32.const 1))
              (return (call $pair (call $prim (global.get $T_I)) (local.get $y)))))
          (i64.store offset=72 (local.get $thread) (i64.add (call $thread_now)
            (i64.mul (i64.extend_i32_u (local.get $index)) (i64.const 1000)))))
        (else (i32.store offset=116 (local.get $thread) (local.get $index))))
      (i32.store (i32.const 0x30c4) (i32.add (i32.load (i32.const 0x30c4)) (i32.const 1)))
      (call $state_park (i32.const 0) (local.get $kind) (i32.const 2))
      (return (i32.const 0))))
  (i32.const 0))
