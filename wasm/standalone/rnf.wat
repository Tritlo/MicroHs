;; Reduce a graph to normal form through the main evaluator loop.
;; This traversal follows application function and argument links. It does not
;; enter array elements or other payloads. This is the serialized RNF contract.
;; See RNF.md for traversal order, noerr behavior, and frame layout.
;;
;; State 0x3054 is the current noerr flag. State 0x3058 is the active visited
;; bitmap address. A kind-3 frame owns its bitmap and pending-node vector.
;; Nested RNF calls save the previous state in that frame. All allocation is
;; outside collection, and no helper calls the evaluator recursively.

;; Report the primitive guards used by noerr traversal. This is not a general
;; exception handler. Numeric errors still raise their runtime exceptions.
(func $rnf_guarded (param $tag i32) (result i32)
  (i32.and (i32.ne (i32.load (i32.const 0x3054)) (i32.const 0))
    (i32.or (i32.eq (local.get $tag) (global.get $T_RAISE))
      (i32.or (i32.eq (local.get $tag) (global.get $T_IO_PERFORMIO))
        (i32.or (i32.eq (local.get $tag) (global.get $T_BSFROMUTF8))
          (i32.or (i32.eq (local.get $tag) (global.get $T_BSUNPACK))
            (i32.eq (local.get $tag) (global.get $T_RNF))))))))

;; Push a pending node onto a managed vector. Mask -1 keeps every pending node
;; live across collection. Unused words are zero. Growth copies only live words
;; and replaces the frame's pointer before it frees the old vector.
(func $rnf_push (param $frame i32) (param $n i32)
  (local $count i32) (local $capacity i32) (local $old i32) (local $new i32)
  (local.set $count (i32.load offset=48 (local.get $frame)))
  (local.set $capacity (i32.load offset=52 (local.get $frame)))
  (if (i32.eq (local.get $count) (local.get $capacity))
    (then
      (if (i32.ge_u (local.get $capacity) (i32.const 0x10000000))
        (then (call $fail (i32.const 79)) (unreachable)))
      (local.set $old (i32.load offset=44 (local.get $frame)))
      (local.set $capacity (i32.shl (local.get $capacity) (i32.const 1)))
      (local.set $new (call $blob_alloc
        (i32.shl (local.get $capacity) (i32.const 2)) (i32.const -1)))
      (memory.copy (local.get $new) (local.get $old)
        (i32.shl (local.get $count) (i32.const 2)))
      (i32.store offset=44 (local.get $frame) (local.get $new))
      (i32.store offset=52 (local.get $frame) (local.get $capacity))
      (call $blob_free (local.get $old))))
  (i32.store (i32.add (i32.load offset=44 (local.get $frame))
    (i32.shl (local.get $count) (i32.const 2))) (local.get $n))
  (i32.store offset=48 (local.get $frame) (i32.add (local.get $count) (i32.const 1))))

;; Return the next unvisited node, or zero when traversal is complete.
;; Mark the original node address before evaluation. This preserves sharing
;; and terminates cycles through constructor fields. A cyclic function spine
;; can still diverge when the evaluator demands it, as normal evaluation does.
(func $rnf_next (param $frame i32) (result i32)
  (local $count i32) (local $slot i32) (local $n i32) (local $index i32)
  (local $word_address i32) (local $word i32) (local $bit i32)
  (loop $next
    (local.set $count (i32.load offset=48 (local.get $frame)))
    (if (i32.eqz (local.get $count)) (then (return (i32.const 0))))
    (local.set $count (i32.sub (local.get $count) (i32.const 1)))
    (i32.store offset=48 (local.get $frame) (local.get $count))
    (local.set $slot (i32.add (i32.load offset=44 (local.get $frame))
      (i32.shl (local.get $count) (i32.const 2))))
    (local.set $n (i32.load (local.get $slot)))
    (i32.store (local.get $slot) (i32.const 0))
    (local.set $index (i32.sub (local.get $n) (i32.const 0x02000000)))
    (if (i32.or (i32.ge_u (local.get $index) (i32.const 600000000))
          (i32.rem_u (local.get $index) (i32.const 8)))
      (then (call $fail (i32.const 80)) (unreachable)))
    (local.set $index (i32.div_u (local.get $index) (i32.const 8)))
    (local.set $word_address (i32.add (i32.load offset=40 (local.get $frame))
      (i32.shl (i32.shr_u (local.get $index) (i32.const 5)) (i32.const 2))))
    (local.set $word (i32.load (local.get $word_address)))
    (local.set $bit (i32.shl (i32.const 1) (i32.and (local.get $index) (i32.const 31))))
    (br_if $next (i32.and (local.get $word) (local.get $bit)))
    (i32.store (local.get $word_address) (i32.or (local.get $word) (local.get $bit)))
    (return (local.get $n)))
  (unreachable))

;; Convert the existing RNF primitive frame into a traversal frame. Its noerr
;; argument has already reached WHNF. The graph argument remains lazy and rooted
;; at offset 36. The caller next evaluates the node returned by $rnf_next.
(func $rnf_begin (param $frame i32)
  (i32.store (local.get $frame) (i32.const 3))
  (i32.store offset=24 (local.get $frame) (i32.load (i32.const 0x3054)))
  (i32.store offset=56 (local.get $frame) (i32.load (i32.const 0x3058)))
  (i32.store offset=40 (local.get $frame)
    (call $blob_alloc (i32.const 9375000) (i32.const 0)))
  (i32.store offset=44 (local.get $frame)
    (call $blob_alloc (i32.const 1024) (i32.const -1)))
  (i32.store offset=48 (local.get $frame) (i32.const 0))
  (i32.store offset=52 (local.get $frame) (i32.const 256))
  (i32.store (i32.const 0x3054)
    (i32.ne (call $ival (i32.load offset=32 (local.get $frame))) (i32.const 0)))
  (i32.store (i32.const 0x3058) (i32.load offset=40 (local.get $frame)))
  (call $rnf_push (local.get $frame) (i32.load offset=36 (local.get $frame))))

;; Continue after one pending node reaches WHNF. Push argument before function
;; so the function is processed first. A value or an opaque payload has no graph
;; children for this traversal. No evaluation or collection occurs here.
(func $rnf_resume (param $frame i32) (param $n i32) (result i32)
  (local.set $n (call $resolve (local.get $n)))
  (if (i32.eqz (i32.and (i32.load (local.get $n)) (i32.const 3)))
    (then
      (call $rnf_push (local.get $frame) (i32.load offset=4 (local.get $n)))
      (call $rnf_push (local.get $frame) (i32.load (local.get $n)))))
  (call $rnf_next (local.get $frame)))

;; Release a traversal on normal completion or exception unwinding. Restore
;; the enclosing traversal before releasing this frame's temporary payloads.
;; The caller then removes the frame without a collection point in between.
(func $rnf_finish (param $frame i32)
  (i32.store (i32.const 0x3054) (i32.load offset=24 (local.get $frame)))
  (i32.store (i32.const 0x3058) (i32.load offset=56 (local.get $frame)))
  (call $blob_free (i32.load offset=40 (local.get $frame)))
  (call $blob_free (i32.load offset=44 (local.get $frame)))
  (i32.store offset=40 (local.get $frame) (i32.const 0))
  (i32.store offset=44 (local.get $frame) (i32.const 0)))

;; Clear visited bits for node addresses made free by collection. A later
;; allocation can reuse those addresses for new graph nodes. Retaining the old
;; visited bit would incorrectly skip the new node. Live visited nodes retain
;; their bits, so collection does not lose the traversal's cycle detection.
;; Apply this to every nested traversal, not only the active bitmap.
(func $rnf_after_gc
  (local $count i32) (local $frame i32) (local $bitmap i32) (local $i i32)
  (local.set $count (i32.load (i32.const 0x158)))
  (local.set $frame (i32.const 0x500000))
  (block $done
    (loop $frames
      (br_if $done (i32.eqz (local.get $count)))
      (if (i32.eq (i32.load (local.get $frame)) (i32.const 3))
        (then
          (local.set $bitmap (i32.load offset=40 (local.get $frame)))
          (local.set $i (i32.const 0))
          (loop $words
            (i32.store (i32.add (local.get $bitmap) (local.get $i))
              (i32.and (i32.load (i32.add (local.get $bitmap) (local.get $i)))
                (i32.xor (i32.load (i32.add (i32.const 0x26000000) (local.get $i)))
                  (i32.const -1))))
            (local.set $i (i32.add (local.get $i) (i32.const 4)))
            (br_if $words (i32.lt_u (local.get $i) (i32.const 9375000))))))
      (local.set $count (i32.sub (local.get $count) (i32.const 1)))
      (local.set $frame (i32.add (local.get $frame) (i32.const 64)))
      (br $frames))))
