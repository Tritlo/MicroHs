;; One evaluator loop controls graph reduction and strict continuations.
;; No function in this file calls the evaluator recursively. The graph stack
;; and continuation frames contain all suspended graph references.
;;
;; Frame words: kind, tag, saved sp, saved base, update node, arity,
;; pending strict mask, argument being evaluated, then eight node references.
;; Kinds 0 and 1 are forward and reverse strict argument evaluation.
;; Kind 2 is an exception handler. Kind 3 is an RNF traversal; see rnf.wat.
;; Kind 4 protects atomic evaluation and restores the saved atomic depth.
;; Argument slot 7 retains the primitive head for service IDs and tick names.

(import "reducer" "reduce" (func $reduce (param i32 i32) (result i32)))
(global $reduction_count (mut i64) (i64.const 0))

;; Read one application-stack entry by absolute index.
(func $stack_at (param $index i32) (result i32)
  (i32.load (i32.add (i32.const 0x10000)
    (i32.shl (local.get $index) (i32.const 2)))))

;; Return the node for argument INDEX without forcing it.
(func $stack_arg (param $sp i32) (param $index i32) (result i32)
  (i32.load offset=4
    (call $stack_at (i32.sub (local.get $sp) (local.get $index)))))

;; Select the next strict argument. A nonzero mask is required.
(func $next_argument (param $mask i32) (param $reverse i32) (result i32)
  (if (result i32) (local.get $reverse)
    (then (i32.sub (i32.const 31) (i32.clz (local.get $mask))))
    (else (i32.ctz (local.get $mask)))))

;; Create a rooted primitive frame. The application stack is still intact.
(func $push_frame (param $tag i32) (param $head i32)
  (param $base i32) (param $plan i32) (result i32)
  (local $count i32) (local $frame i32) (local $arity i32)
  (local $sp i32) (local $i i32)
  (local.set $count (i32.load (i32.const 0x158)))
  (if (i32.ge_u (local.get $count) (i32.const 65536))
    (then (call $fail (i32.const 28))))
  (local.set $frame (i32.add (i32.const 0x500000)
    (i32.shl (local.get $count) (i32.const 6))))
  (memory.fill (local.get $frame) (i32.const 0) (i32.const 64))
  (local.set $sp (i32.load (i32.const 0x104)))
  (local.set $arity (i32.and (local.get $plan) (i32.const 15)))
  (i32.store (local.get $frame)
    (i32.shr_u (i32.and (local.get $plan) (i32.const 4096)) (i32.const 12)))
  (i32.store offset=4 (local.get $frame) (local.get $tag))
  (i32.store offset=8 (local.get $frame) (local.get $sp))
  (i32.store offset=12 (local.get $frame) (local.get $base))
  (i32.store offset=16 (local.get $frame)
    (call $stack_at (i32.add (i32.sub (local.get $sp) (local.get $arity)) (i32.const 1))))
  (i32.store offset=20 (local.get $frame) (local.get $arity))
  (i32.store offset=24 (local.get $frame)
    (i32.and (i32.shr_u (local.get $plan) (i32.const 4)) (i32.const 255)))
  (i32.store offset=60 (local.get $frame) (local.get $head))
  (loop $arguments
    (i32.store
      (i32.add (local.get $frame)
        (i32.add (i32.const 32) (i32.shl (local.get $i) (i32.const 2))))
      (call $stack_arg (local.get $sp) (local.get $i)))
    (local.set $i (i32.add (local.get $i) (i32.const 1)))
    (br_if $arguments (i32.lt_u (local.get $i) (local.get $arity))))
  (i32.store (i32.const 0x158) (i32.add (local.get $count) (i32.const 1)))
  (local.get $frame))

;; Convert a failed computation into the nearest exception handler's graph.
;; The caller has stored the exception at 0x3004. On success, the saved stack
;; and parent base belong to the handler. Zero means an uncaught exception.
(func $unwind (result i32)
  (local $count i32) (local $frame i32) (local $handler i32)
  (local $world i32) (local $oldmask i32) (local $action i32) (local $restore i32)
  (local.set $count (i32.load (i32.const 0x158)))
  (i32.store offset=60 (i32.load (i32.const 0x3084)) (i32.const 0))
  (i32.store offset=80 (i32.load (i32.const 0x3084)) (i32.const 0))
  (i32.store offset=112 (i32.load (i32.const 0x3084)) (i32.const 0))
  (i32.store offset=120 (i32.load (i32.const 0x3084)) (i32.const 0))
  (block $found
    (loop $search
      (if (i32.eqz (local.get $count)) (then (return (i32.const 0))))
      (local.set $count (i32.sub (local.get $count) (i32.const 1)))
      (local.set $frame (i32.add (i32.const 0x500000)
        (i32.shl (local.get $count) (i32.const 6))))
      (if (i32.eq (i32.load (local.get $frame)) (i32.const 3))
        (then (call $rnf_finish (local.get $frame))))
      (if (i32.eq (i32.load (local.get $frame)) (i32.const 4))
        (then (i32.store (i32.const 0x30a4) (i32.load offset=24 (local.get $frame)))))
      (br_if $found (i32.eq (i32.load (local.get $frame)) (i32.const 2)))
      (br $search)))
  (local.set $handler (i32.load offset=36 (local.get $frame)))
  (local.set $world (i32.load offset=40 (local.get $frame)))
  (local.set $oldmask (i32.load offset=24 (local.get $frame)))
  ;; Run the handler with interruptible masking. Restore the prior mask after
  ;; its IO result, using ordinary graph operations visible to the collector.
  (i32.store (i32.const 0x3040) (i32.const 1))
  (local.set $action (call $ap (local.get $handler) (i32.load (i32.const 0x3004))))
  (local.set $restore
    (call $ap
      (call $ap
        (call $ap (call $prim (global.get $T_BB))
          (call $prim (global.get $T_IO_THEN)))
        (call $ap (call $prim (global.get $T_IO_SETMASKINGSTATE))
          (call $int (local.get $oldmask))))
      (call $prim (global.get $T_IO_RETURN))))
  (local.set $action
    (call $ap
      (call $ap (call $ap (call $prim (global.get $T_IO_BIND))
        (local.get $action)) (local.get $restore))
      (local.get $world)))
  (i32.store (i32.load offset=16 (local.get $frame))
    (i32.or (local.get $action) (i32.const 2)))
  (i32.store (i32.const 0x104)
    (i32.sub (i32.load offset=8 (local.get $frame))
      (i32.load offset=20 (local.get $frame))))
  (i32.store (i32.const 0x3044) (i32.load offset=12 (local.get $frame)))
  (i32.store (i32.const 0x158) (local.get $count))
  (i32.store (i32.const 0x3004) (i32.const 0))
  (local.get $action))

;; Evaluate ROOT to WHNF. A nonzero result is a graph node. Zero reports an
;; uncaught Haskell exception, whose value remains in the exception root.
;; Both outcomes restore the entry stack depth and release all continuations.
(func $execute (export "evaluate") (param $root i32) (result i32)
  (local $n i32) (local $base i32) (local $sp i32) (local $tag i32)
  (local $frame i32) (local $count i32) (local $plan i32)
  (local $mask i32) (local $index i32) (local $result i32)
  (local $arity i32) (local $update i32)
  (local $before i32) (local $entry_sp i32)
  (local.set $n (local.get $root))
  (local.set $base (i32.load (i32.const 0x104)))
  (local.set $entry_sp (local.get $base))
  (if (i32.load (i32.const 0x158))
    (then (call $fail (i32.const 29))))
  (i32.store (i32.const 0x3004) (i32.const 0))
  (call $threads_begin (local.get $root) (local.get $entry_sp))
  (loop $top
    (block $commit
    (block $resume
    (i32.store (i32.const 0x154) (local.get $n))
    (call $threads_publish (local.get $n) (local.get $base))
    ;; All live local nodes also occur in the current root or a frame here.
    ;; Reserve enough nodes for one primitive and for exception unwinding.
    (if (i32.or (i32.load (i32.const 0x3048))
          (i32.lt_u (i32.load (i32.const 0x11c)) (i32.const 32)))
      (then
        (i32.store (i32.const 0x3048) (i32.const 0))
        (call $gc)
        (call $rnf_after_gc)
        (call $threads_after_gc)
        (if (i32.lt_u (i32.load (i32.const 0x11c)) (i32.const 32))
          (then (call $fail (i32.const 71))))))
    (local.set $n (call $threads_boundary (local.get $n) (local.get $base)))
    (local.set $base (i32.load (i32.const 0x30a0)))
    (i32.store (i32.const 0x154) (local.get $n))
    (br_if $top (i32.load (i32.const 0x3048)))
    (br_if $top (i32.lt_u (i32.load (i32.const 0x11c)) (i32.const 32)))
    (local.set $result (call $thread_take_exception (i32.const 0)))
    (if (local.get $result)
      (then
        (drop (call $raise (local.get $result)))
        (local.set $n (call $unwind))
        (if (i32.eqz (local.get $n))
          (then
            (local.set $n (call $threads_complete (i32.load (i32.const 0x3004)) (i32.const 4)))
            (if (i32.eqz (i32.load (i32.const 0x3098))) (then (return (i32.const 0))))
            (local.set $base (i32.load (i32.const 0x30a0))))
          (else (local.set $base (i32.load (i32.const 0x3044)))))
        (br $top)))
    (local.set $before (i32.load (i32.const 0x10c)))
    (local.set $n (call $reduce (local.get $n) (local.get $base)))
    (global.set $reduction_count (i64.add (global.get $reduction_count)
      (i64.extend_i32_u (i32.sub (local.get $before) (i32.load (i32.const 0x10c))))))
    (i32.store (i32.const 0x154) (local.get $n))
    ;; A reducer resource boundary resumes after collection or a new slice.
    (br_if $top (i32.lt_u (i32.load (i32.const 0x11c)) (i32.const 32)))
    (br_if $top (i32.le_s (i32.load (i32.const 0x10c)) (i32.const 1)))
    (if (i32.ge_s (i32.load (i32.const 0x104)) (i32.const 999998))
      (then (call $fail (i32.const 30))))
    (local.set $tag (call $tag (local.get $n)))
    (local.set $sp (i32.load (i32.const 0x104)))

    (block $whnf
      ;; noerr RNF leaves guarded applications unreduced. The traversal still
      ;; visits their function and argument links after this WHNF return.
      (br_if $whnf (call $rnf_guarded (local.get $tag)))
      ;; All typed values except BADDYN are WHNF. An unsupported core
      ;; combinator is a partial application because resources were checked.
      (br_if $whnf
        (i32.and (i32.ge_u (local.get $tag) (global.get $T_INT))
          (i32.and (i32.le_u (local.get $tag) (global.get $T_TAG32))
            (i32.ne (local.get $tag) (global.get $T_BADDYN)))))
      (local.set $plan (call $primitive_plan (local.get $tag) (local.get $n)))
      (if (i32.eqz (local.get $plan))
        (then
          (i32.store (i32.const 0x2f0) (local.get $tag))
          (call $fail (i32.const 31))))
      (local.set $arity (i32.and (local.get $plan) (i32.const 15)))
      (br_if $whnf (i32.lt_s (i32.sub (local.get $sp) (local.get $base)) (local.get $arity)))
      (local.set $frame (call $push_frame
        (local.get $tag) (local.get $n) (local.get $base) (local.get $plan)))
      (global.set $reduction_count (i64.add (global.get $reduction_count) (i64.const 1)))
      (i32.store (i32.const 0x10c) (i32.sub (i32.load (i32.const 0x10c)) (i32.const 1)))
      (br $resume)
    )

    ;; Return the complete partial application when the spine is nonempty.
    (if (i32.ne (local.get $sp) (local.get $base))
      (then (local.set $n (call $stack_at (i32.add (local.get $base) (i32.const 1))))))
    (i32.store (i32.const 0x104) (local.get $base))
    (local.set $count (i32.load (i32.const 0x158)))
    (if (i32.eqz (local.get $count))
      (then
        (local.set $n (call $threads_complete (local.get $n) (i32.const 3)))
        (if (i32.eqz (i32.load (i32.const 0x3098))) (then (return (local.get $n))))
        (local.set $base (i32.load (i32.const 0x30a0)))
        (br $top)))
    (local.set $frame (i32.add (i32.const 0x500000)
      (i32.shl (i32.sub (local.get $count) (i32.const 1)) (i32.const 6))))
    (if (i32.eq (i32.load (local.get $frame)) (i32.const 3))
      (then
        ;; RNF can visit many partial applications without a graph reduction.
        ;; Charge each visit to the scheduling slice to permit preemption.
        (i32.store (i32.const 0x10c) (i32.sub (i32.load (i32.const 0x10c)) (i32.const 1)))
        (local.set $n (call $rnf_resume (local.get $frame) (local.get $n)))
        (if (local.get $n)
          (then
            (local.set $base (i32.load offset=8 (local.get $frame)))
            (i32.store (i32.const 0x104) (local.get $base))
            (br $top)))
        (call $rnf_finish (local.get $frame))
        (local.set $result (call $prim (global.get $T_I)))
        (br $commit)))
    (if (i32.eq (i32.load (local.get $frame)) (i32.const 2))
      (then
        ;; The protected action completed. Its pair is the catch result.
        (local.set $result (local.get $n))
        (br $commit)))
    (if (i32.eq (i32.load (local.get $frame)) (i32.const 4))
      (then
        (i32.store (i32.const 0x30a4) (i32.load offset=24 (local.get $frame)))
        (local.set $result (call $pair (local.get $n) (i32.load offset=36 (local.get $frame))))
        (br $commit)))
    (i32.store
      (i32.add (local.get $frame)
        (i32.add (i32.const 32)
          (i32.shl (i32.load offset=28 (local.get $frame)) (i32.const 2))))
      (local.get $n))

    )
    ;; Either a new primitive frame or a completed strict argument arrives
    ;; here. Each pending mask bit is removed before evaluation starts.
    (local.set $mask (i32.load offset=24 (local.get $frame)))
    (if (local.get $mask)
      (then
        (local.set $index (call $next_argument (local.get $mask)
          (i32.load (local.get $frame))))
        (i32.store offset=24 (local.get $frame)
          (i32.xor (local.get $mask) (i32.shl (i32.const 1) (local.get $index))))
        (i32.store offset=28 (local.get $frame) (local.get $index))
        (local.set $base (i32.load offset=8 (local.get $frame)))
        (i32.store (i32.const 0x104) (local.get $base))
        (local.set $n (call $value_arg
          (i32.add (local.get $frame) (i32.const 32)) (local.get $index)))
        (br $top)))

    (local.set $tag (i32.load offset=4 (local.get $frame)))
    (if (i32.eq (local.get $tag) (global.get $T_IO_ATOMIC))
      (then
        (i32.store (local.get $frame) (i32.const 4))
        (i32.store offset=24 (local.get $frame) (i32.load (i32.const 0x30a4)))
        (i32.store (i32.const 0x30a4) (i32.add (i32.load (i32.const 0x30a4)) (i32.const 1)))
        (local.set $n (call $ap (call $ap (i32.load offset=32 (local.get $frame))
          (i32.load offset=36 (local.get $frame))) (call $prim (global.get $T_K))))
        (i32.store offset=40 (local.get $frame) (local.get $n))
        (local.set $base (i32.load offset=8 (local.get $frame)))
        (i32.store (i32.const 0x104) (local.get $base))
        (br $top)))
    (if (i32.eq (local.get $tag) (global.get $T_RNF))
      (then
        (call $rnf_begin (local.get $frame))
        (local.set $base (i32.load offset=8 (local.get $frame)))
        (i32.store (i32.const 0x104) (local.get $base))
        (local.set $n (call $rnf_next (local.get $frame)))
        (br $top)))
    (if (i32.eq (local.get $tag) (global.get $T_CATCHR))
      (then
        (i32.store (local.get $frame) (i32.const 2))
        (i32.store offset=24 (local.get $frame) (i32.load (i32.const 0x3040)))
        (local.set $base (i32.load offset=8 (local.get $frame)))
        (i32.store (i32.const 0x104) (local.get $base))
        (local.set $n (i32.load offset=32 (local.get $frame)))
        (br $top)))

    (local.set $result (call $primitive (local.get $tag)
      (i32.add (local.get $frame) (i32.const 32))
      (i32.load offset=60 (local.get $frame))))
    (if (i32.eqz (local.get $result))
      (then
        (if (i32.eq (i32.load (i32.const 0x309c)) (i32.const 2))
          (then
            ;; Retry this primitive after wakeup. Its application stack still
            ;; contains the original arguments, whose strict thunks are updated.
            (local.set $n (i32.load offset=60 (local.get $frame)))
            (local.set $base (i32.load offset=12 (local.get $frame)))
            (i32.store (i32.const 0x104) (i32.load offset=8 (local.get $frame)))
            (i32.store (i32.const 0x158) (i32.sub (i32.load (i32.const 0x158)) (i32.const 1)))
            (br $top)))
        (if (i32.eqz (i32.load (i32.const 0x3004)))
          (then
            (i32.store (i32.const 0x2f0) (local.get $tag))
            (call $fail (i32.const 32))))
        (local.set $n (call $unwind))
        (if (i32.eqz (local.get $n))
          (then
            (local.set $n (call $threads_complete (i32.load (i32.const 0x3004)) (i32.const 4)))
            (if (i32.eqz (i32.load (i32.const 0x3098))) (then (return (i32.const 0))))
            (local.set $base (i32.load (i32.const 0x30a0)))
            (br $top)))
        (local.set $base (i32.load (i32.const 0x3044)))
        (br $top)))
    )

    ;; Update only after all strict evaluation and allocation have completed.
    ;; The result becomes the current root before the next collection point.
    (local.set $update (i32.load offset=16 (local.get $frame)))
    (i32.store (local.get $update) (i32.or (local.get $result) (i32.const 2)))
    (i32.store (i32.const 0x104)
      (i32.sub (i32.load offset=8 (local.get $frame))
        (i32.load offset=20 (local.get $frame))))
    (local.set $base (i32.load offset=12 (local.get $frame)))
    (i32.store (i32.const 0x158)
      (i32.sub (i32.load (i32.const 0x158)) (i32.const 1)))
    (local.set $n (local.get $result))
    (br $top)
  )
  (unreachable))
