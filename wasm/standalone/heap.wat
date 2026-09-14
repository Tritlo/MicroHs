;; Graph and payload allocation for the standalone runtime.
;; This file is a WAT module fragment. It requires the memory, $T_* constants,
;; the non-returning $fail function, and $finalize from the support module.
;; See HEAP.md for the state addresses and collector invariants.
;;
;; All constructors return without collection or evaluation. The outer
;; evaluator publishes its roots and calls $gc at a safe point. A constructor
;; must fail if it cannot satisfy an allocation at the current safe point.

;; Return the next indirection address, or zero for another node shape.
;; Bounds checks protect conservative roots that resemble graph addresses.
;; This helper does not mark, allocate, evaluate, or rewrite a reference.
(func $heap_gc_next (param $node i32) (result i32)
  (local $word i32)
  (if (i32.or
        (i32.ge_u (i32.sub (local.get $node) (i32.const 0x02000000))
          (i32.const 600000000))
        (i32.and (local.get $node) (i32.const 7)))
    (then (return (i32.const 0))))
  (local.set $word (i32.load (local.get $node)))
  (if (i32.ne (i32.and (local.get $word) (i32.const 3)) (i32.const 2))
    (then (return (i32.const 0))))
  (i32.and (local.get $word) (i32.const -4)))

;; Canonicalize a known graph reference without changing its root slot.
;; Floyd detection has no chain-length limit and uses constant scratch space.
;; Acyclic paths end at a non-IND node. A pure IND cycle ends at its cycle entry,
;; which becomes a self-indirection. Both shapes preserve graph evaluation.
;; Every IND on the prefix is rewritten after the target is established.
;; Other roots still mark their original node addresses independently.
(func $heap_gc_target (param $start i32) (result i32)
  (local $slow i32) (local $fast i32) (local $next i32)
  (local $target i32) (local $current i32) (local $cycle i32)
  (if (i32.or
        (i32.ge_u (i32.sub (local.get $start) (i32.const 0x02000000))
          (i32.const 600000000))
        (i32.and (local.get $start) (i32.const 7)))
    (then (return (local.get $start))))
  (local.set $next (i32.load (local.get $start)))
  (if (i32.or (i32.ne (i32.and (local.get $next) (i32.const 3)) (i32.const 2))
        (i32.eqz (i32.and (local.get $next) (i32.const -4))))
    (then (return (local.get $start))))
  (local.set $slow (local.get $start))
  (local.set $fast (local.get $start))
  (block $found
    (loop $detect
      (local.set $next (call $heap_gc_next (local.get $fast)))
      (if (i32.eqz (local.get $next))
        (then (local.set $target (local.get $fast)) (br $found)))
      (local.set $fast (local.get $next))
      (local.set $next (call $heap_gc_next (local.get $fast)))
      (if (i32.eqz (local.get $next))
        (then (local.set $target (local.get $fast)) (br $found)))
      (local.set $fast (local.get $next))
      (local.set $slow (call $heap_gc_next (local.get $slow)))
      (if (i32.eq (local.get $slow) (local.get $fast))
        (then
          (local.set $cycle (i32.const 1))
          (local.set $slow (local.get $start))
          (block $entry
            (loop $locate
              (br_if $entry (i32.eq (local.get $slow) (local.get $fast)))
              (local.set $slow (call $heap_gc_next (local.get $slow)))
              (local.set $fast (call $heap_gc_next (local.get $fast)))
              (br $locate)))
          (local.set $target (local.get $slow))
          (br $found)))
      (br $detect)))
  ;; Leave malformed links unchanged. Ordinary marking ignores invalid targets.
  (if (i32.or
        (i32.ge_u (i32.sub (local.get $target) (i32.const 0x02000000))
          (i32.const 600000000))
        (i32.and (local.get $target) (i32.const 7)))
    (then (return (local.get $start))))
  (local.set $current (local.get $start))
  (block $compressed
    (loop $compress
      (br_if $compressed (i32.eq (local.get $current) (local.get $target)))
      (local.set $next (call $heap_gc_next (local.get $current)))
      (i32.store (local.get $current) (i32.or (local.get $target) (i32.const 2)))
      (local.set $current (local.get $next))
      (br $compress)))
  (if (local.get $cycle)
    (then (i32.store (local.get $target) (i32.or (local.get $target) (i32.const 2)))))
  (local.get $target))

;; The reducer imports addresses of state words, not the state values.
(global $stack_addr (export "stack") i32 (i32.const 0x100))
(global $sp_addr (export "stack_ptr") i32 (i32.const 0x104))
(global $stack_size_addr (export "stack_size") i32 (i32.const 0x108))
(global $slice_addr (export "glob_slice") i32 (i32.const 0x10c))
(global $cells_addr (export "cells") i32 (i32.const 0x110))
(global $map_addr (export "free_map") i32 (i32.const 0x114))
(global $scan_addr (export "next_scan_index") i32 (i32.const 0x118))
(global $free_addr (export "num_free") i32 (i32.const 0x11c))
(global $alloc_addr (export "num_alloc") i32 (i32.const 0x120))
(global $combB_addr (export "combB") i32 (i32.const 0x124))
(global $combC_addr (export "combC") i32 (i32.const 0x128))
(global $combK_addr (export "combK") i32 (i32.const 0x12c))
(global $combK2_addr (export "combK2") i32 (i32.const 0x130))
(global $combK3_addr (export "combK3") i32 (i32.const 0x134))
(global $red_bb_addr (export "red_bb") i32 (i32.const 0x138))
(global $red_z_addr (export "red_z") i32 (i32.const 0x13c))
(global $red_r_addr (export "red_r") i32 (i32.const 0x140))
(global $red_k2_addr (export "red_k2") i32 (i32.const 0x144))
(global $red_k3_addr (export "red_k3") i32 (i32.const 0x148))
(global $red_k4_addr (export "red_k4") i32 (i32.const 0x14c))
(global $red_ccb_addr (export "red_ccb") i32 (i32.const 0x150))
;; intTable is an array symbol. Its export is the array address itself.
(global $ints_addr (export "intTable") i32 (i32.const 0x1000))

;; Allocate one node from the free bitmap. A set bit denotes a free node.
;; next_scan_index never decreases between collections. Both this allocator
;; and the linked reducer clear the selected bit before they return a node.
(func $heap_alloc_node (result i32)
  (local $word_index i32) (local $word i32) (local $index i32) (local $n i32)
  (if (i32.eqz (i32.load (global.get $free_addr)))
    (then (call $fail (i32.const 71)) (unreachable)))
  (local.set $word_index
    (i32.shr_u (i32.load (global.get $scan_addr)) (i32.const 5)))
  (block $found
    (loop $scan
      (if (i32.ge_u (local.get $word_index) (i32.const 2343750))
        (then (call $fail (i32.const 71)) (unreachable)))
      (local.set $word
        (i32.load (i32.add (i32.const 0x26000000)
          (i32.shl (local.get $word_index) (i32.const 2)))))
      (br_if $found (local.get $word))
      (local.set $word_index (i32.add (local.get $word_index) (i32.const 1)))
      (br $scan)))
  (local.set $index
    (i32.add (i32.shl (local.get $word_index) (i32.const 5))
      (i32.ctz (local.get $word))))
  (i32.store (i32.add (i32.const 0x26000000)
      (i32.shl (local.get $word_index) (i32.const 2)))
    (i32.and (local.get $word) (i32.sub (local.get $word) (i32.const 1))))
  (i32.store (global.get $scan_addr) (local.get $index))
  (i32.store (global.get $free_addr)
    (i32.sub (i32.load (global.get $free_addr)) (i32.const 1)))
  (i32.store (global.get $alloc_addr)
    (i32.add (i32.load (global.get $alloc_addr)) (i32.const 1)))
  (local.set $n (i32.add (i32.const 0x02000000)
    (i32.mul (local.get $index) (i32.const 8))))
  (i64.store (local.get $n) (i64.const 0))
  (local.get $n))

;; Return a zeroed tagged node. Tags occupy the high 30 bits of word zero.
(func $node (param $tag i32) (result i32)
  (local $n i32)
  (local.set $n (call $heap_alloc_node))
  (i32.store (local.get $n)
    (i32.or (i32.shl (local.get $tag) (i32.const 2)) (i32.const 1)))
  (local.get $n))

;; Return an application. Its function pointer replaces the tagged word.
(func $ap (param $f i32) (param $a i32) (result i32)
  (local $n i32)
  (local.set $n (call $heap_alloc_node))
  (i32.store (local.get $n) (local.get $f))
  (i32.store offset=4 (local.get $n) (local.get $a))
  (local.get $n))

;; Return the permanent primitive at its tag-derived node address.
;; A tag outside the checked-in primitive table is a runtime error.
(func $prim (param $tag i32) (result i32)
  (if (i32.gt_u (local.get $tag) (global.get $T_LAST_TAG))
    (then (call $fail (i32.const 78)) (unreachable)))
  (i32.add (i32.const 0x02000000) (i32.mul (local.get $tag) (i32.const 8))))

;; Box a machine integer. The permanent table covers -10 through 255.
(func $int (param $value i32) (result i32)
  (local $n i32)
  (if (i32.lt_u (i32.add (local.get $value) (i32.const 10)) (i32.const 266))
    (then (return (i32.load (i32.add (i32.const 0x1000)
      (i32.shl (i32.add (local.get $value) (i32.const 10)) (i32.const 2)))))))
  (local.set $n (call $node (global.get $T_INT)))
  (i32.store offset=4 (local.get $n) (local.get $value))
  (local.get $n))

;; Box an Int64 in a separate raw payload. The node keeps its payload pointer.
;; Collection marks the allocation but does not scan the integer's bits.
(func $int64 (param $value i64) (result i32)
  (local $n i32) (local $payload i32)
  (local.set $payload (call $blob_alloc (i32.const 8) (i32.const 0)))
  (local.set $n (call $node (global.get $T_INT64)))
  (i64.store (local.get $payload) (local.get $value))
  (i32.store offset=4 (local.get $n) (local.get $payload))
  (local.get $n))

;; Box a Double in a separate raw payload without changing its IEEE 754 bits.
(func $double (param $value f64) (result i32)
  (local $n i32) (local $payload i32)
  (local.set $payload (call $blob_alloc (i32.const 8) (i32.const 0)))
  (local.set $n (call $node (global.get $T_DBL)))
  (f64.store (local.get $payload) (local.get $value))
  (i32.store offset=4 (local.get $n) (local.get $payload))
  (local.get $n))

;; Box a Float in the node's second word.
(func $float (param $value f32) (result i32)
  (local $n i32)
  (local.set $n (call $node (global.get $T_FLT32)))
  (f32.store offset=4 (local.get $n) (local.get $value))
  (local.get $n))

;; Box a linear-memory address. A managed payload address is a strong root.
(func $ptr (param $address i32) (result i32)
  (local $n i32)
  (local.set $n (call $node (global.get $T_PTR)))
  (i32.store offset=4 (local.get $n) (local.get $address))
  (local.get $n))

;; Follow indirections without evaluating application nodes. A pure IND cycle
;; represents a divergent computation. This helper does not return for a cycle.
;; Collection uses $heap_gc_target to follow links without this divergence.
(func $resolve (param $n i32) (result i32)
  (local $word i32)
  (block $done
    (loop $follow
      (local.set $word (i32.load (local.get $n)))
      (br_if $done (i32.ne (i32.and (local.get $word) (i32.const 3)) (i32.const 2)))
      (local.set $n (i32.and (local.get $word) (i32.const -4)))
      (br $follow)))
  (local.get $n))

;; Check an unsigned linear-memory interval before a copy or descriptor write.
;; The i64 sum detects wasm32 addition overflow before memory instructions run.
(func $heap_check_range (param $address i32) (param $size i32)
  (if (i64.gt_u
        (i64.add (i64.extend_i32_u (local.get $address))
          (i64.extend_i32_u (local.get $size)))
        (i64.shl (i64.extend_i32_u (memory.size)) (i64.const 16)))
    (then (call $fail (i32.const 72)) (unreachable))))

;; Find the size class for a request. Classes are 16, 32, ... 2^30 bytes.
;; The result is the class index, rather than its byte capacity.
(func $heap_blob_class (param $size i32) (result i32)
  (if (i32.gt_u (local.get $size) (i32.const 0x40000000))
    (then (call $fail (i32.const 72)) (unreachable)))
  (if (i32.le_u (local.get $size) (i32.const 16))
    (then (return (i32.const 0))))
  (i32.sub
    (i32.sub (i32.const 32) (i32.clz (i32.sub (local.get $size) (i32.const 1))))
    (i32.const 4)))

;; Allocate a zeroed payload. The 16-byte header contains capacity, pointer
;; mask, flags, and free-list next. Flag 1 means allocated; flag 2 means marked.
;; Flag 4 identifies a finalizer record. Normal allocation clears that flag.
;; Reuse retains the block address and its position in the ordered index.
;; New blocks append to that index. Headers and payloads have 16-byte alignment.
;; Memory grows only here. Neither growth nor free-list reuse invokes GC.
;; Payload pressure requests collection at the next evaluator safe point.
(func $blob_alloc (param $size i32) (param $mask i32) (result i32)
  (local $class i32) (local $capacity i32) (local $head i32) (local $block i32)
  (local $index_count i32) (local $pages i32) (local $end i64)
  (local.set $class (call $heap_blob_class (local.get $size)))
  (local.set $capacity (i32.shl (i32.const 16) (local.get $class)))
  (local.set $head (i32.add (i32.const 0x180)
    (i32.shl (local.get $class) (i32.const 2))))
  (local.set $block (i32.load (local.get $head)))
  (if (local.get $block)
    (then
      (i32.store (local.get $head) (i32.load offset=12 (local.get $block))))
    (else
      (local.set $index_count (i32.load (i32.const 0x168)))
      (if (i32.ge_u (local.get $index_count) (i32.const 1048576))
        (then (call $fail (i32.const 74)) (unreachable)))
      (local.set $block (i32.load (i32.const 0x164)))
      (local.set $end (i64.add (i64.extend_i32_u (local.get $block))
        (i64.add (i64.extend_i32_u (local.get $capacity)) (i64.const 16))))
      ;; The support module has a two-GiB limit. Check before truncation.
      (if (i64.gt_u (local.get $end) (i64.const 0x80000000))
        (then (call $fail (i32.const 73)) (unreachable)))
      (local.set $pages (i32.wrap_i64
        (i64.shr_u (i64.add (local.get $end) (i64.const 65535)) (i64.const 16))))
      (if (i32.gt_u (local.get $pages) (memory.size))
        (then
          (if (i32.eq
                (memory.grow (i32.sub (local.get $pages) (memory.size))) (i32.const -1))
            (then (call $fail (i32.const 73)) (unreachable)))))
      (i32.store (i32.const 0x164) (i32.wrap_i64 (local.get $end)))
      (i32.store (i32.add (i32.const 0x00d00000)
          (i32.shl (local.get $index_count) (i32.const 2))) (local.get $block))
      (i32.store (i32.const 0x168) (i32.add (local.get $index_count) (i32.const 1)))
      (if (i32.ge_u (local.get $index_count) (i32.const 900000))
        (then (i32.store (i32.const 0x3048) (i32.const 1))))
      (i32.store (local.get $block) (local.get $capacity))))
  (i32.store offset=4 (local.get $block) (local.get $mask))
  (i32.store offset=8 (local.get $block) (i32.const 1))
  (i32.store offset=12 (local.get $block) (i32.const 0))
  (memory.fill (i32.add (local.get $block) (i32.const 16))
    (i32.const 0) (local.get $capacity))
  (i32.store (i32.const 0x174)
    (i32.add (i32.load (i32.const 0x174)) (i32.const 1)))
  (i32.store (i32.const 0x178)
    (i32.add (i32.load (i32.const 0x178))
      (i32.add (local.get $capacity) (i32.const 16))))
  (if (i32.or (i32.ge_u (i32.load (i32.const 0x174)) (i32.const 131072))
        (i32.ge_u (i32.load (i32.const 0x178)) (i32.const 33554432)))
    (then (i32.store (i32.const 0x3048) (i32.const 1))))
  (i32.add (local.get $block) (i32.const 16)))

;; Find an allocated payload block that contains an address. The ordered index
;; stores every block, including free blocks. Binary search first selects the
;; last header at or before the address. Range and allocation checks then reject
;; header addresses, free blocks, and pointers past the end. Zero means absent.
(func $heap_blob_find (param $address i32) (result i32)
  (local $lo i32) (local $hi i32) (local $mid i32) (local $block i32)
  (if (i32.or (i32.lt_u (local.get $address) (i32.const 0x28000000))
        (i32.ge_u (local.get $address) (i32.load (i32.const 0x164))))
    (then (return (i32.const 0))))
  (local.set $hi (i32.load (i32.const 0x168)))
  (block $found
    (loop $search
      (br_if $found (i32.ge_u (local.get $lo) (local.get $hi)))
      (local.set $mid (i32.add (local.get $lo)
        (i32.shr_u (i32.sub (local.get $hi) (local.get $lo)) (i32.const 1))))
      (local.set $block (i32.load (i32.add (i32.const 0x00d00000)
        (i32.shl (local.get $mid) (i32.const 2)))))
      (if (i32.le_u (local.get $block) (local.get $address))
        (then (local.set $lo (i32.add (local.get $mid) (i32.const 1))))
        (else (local.set $hi (local.get $mid))))
      (br $search)))
  (if (i32.eqz (local.get $lo)) (then (return (i32.const 0))))
  (local.set $block (i32.load (i32.add (i32.const 0x00d00000)
    (i32.shl (i32.sub (local.get $lo) (i32.const 1)) (i32.const 2)))))
  (if (i32.or
        (i32.eqz (i32.and (i32.load offset=8 (local.get $block)) (i32.const 1)))
        (i32.or
          (i32.lt_u (local.get $address) (i32.add (local.get $block) (i32.const 16)))
          (i32.ge_u (i32.sub (local.get $address)
              (i32.add (local.get $block) (i32.const 16)))
            (i32.load (local.get $block)))))
    (then (return (i32.const 0))))
  (local.get $block))

;; Put a known allocated block on its size-class free list. The sweep calls this
;; helper directly after checking its flags. No payload is moved or inspected.
(func $heap_blob_release (param $block i32)
  (local $head i32)
  (local.set $head (i32.add (i32.const 0x180)
    (i32.shl (call $heap_blob_class (i32.load (local.get $block))) (i32.const 2))))
  (i32.store offset=8 (local.get $block) (i32.const 0))
  (i32.store offset=12 (local.get $block) (i32.load (local.get $head)))
  (i32.store (local.get $head) (local.get $block)))

;; Release an explicitly owned allocation. Null is accepted. Interior pointers
;; and double frees fail, because they do not transfer allocation ownership.
;; Callers must not retain a usable alias after an explicit free.
(func $blob_free (param $address i32)
  (local $block i32)
  (if (i32.eqz (local.get $address)) (then (return)))
  (local.set $block (call $heap_blob_find (local.get $address)))
  (if (i32.or (i32.eqz (local.get $block))
        (i32.ne (local.get $address) (i32.add (local.get $block) (i32.const 16))))
    (then (call $fail (i32.const 75)) (unreachable)))
  (call $heap_blob_release (local.get $block)))

;; Create a shared finalizer record. Its words are the service handle and the
;; original data address. Mask bit 1 retains the data while the record is live.
;; A zero handle can be replaced later by the finalizer registration primitive.
;; Descriptor aliases share this record, so collection invokes it only once.
(func $finalizer_new (param $handle i32) (param $data i32) (result i32)
  (local $record i32)
  (local.set $record (call $blob_alloc (i32.const 8) (i32.const 2)))
  (i32.store (local.get $record) (local.get $handle))
  (i32.store offset=4 (local.get $record) (local.get $data))
  (i32.store (i32.sub (local.get $record) (i32.const 8)) (i32.const 5))
  (local.get $record))

;; Make a 16-byte byte-string descriptor over an existing interval. Its words
;; are length, data, capacity, and finalizer-record pointer. Capacity is zero.
;; A shared record exists before any descriptor aliases are made. Later
;; finalizer registration through one alias is therefore visible to all aliases.
;; Pointer-mask bits 1 and 3 mark the pointer words.
;; The data pointer retains the containing allocation even when
;; the data address points into a substring. Static memory needs no GC mark.
(func $bytes_view (param $address i32) (param $length i32) (result i32)
  (local $descriptor i32) (local $n i32)
  (call $heap_check_range (local.get $address) (local.get $length))
  (local.set $descriptor (call $blob_alloc (i32.const 16) (i32.const 10)))
  (i32.store (local.get $descriptor) (local.get $length))
  (i32.store offset=4 (local.get $descriptor) (local.get $address))
  (i32.store offset=12 (local.get $descriptor)
    (call $finalizer_new (i32.const 0) (local.get $address)))
  (local.set $n (call $node (global.get $T_FORPTR)))
  (i32.store offset=4 (local.get $n) (i32.or (local.get $descriptor) (i32.const 1)))
  (local.get $n))

;; Copy a byte interval into managed payload memory and return its descriptor.
;; The copy completes before another allocation can change linear-memory size.
(func $bytes (param $address i32) (param $length i32) (result i32)
  (local $copy i32)
  (call $heap_check_range (local.get $address) (local.get $length))
  (local.set $copy (call $blob_alloc (local.get $length) (i32.const 0)))
  (memory.copy (local.get $copy) (local.get $address) (local.get $length))
  (call $bytes_view (local.get $copy) (local.get $length)))

;; Build the lazy UTF-8 conversion application. This preserves the source
;; payload through the byte-string descriptor. It does not decode or evaluate.
(func $string (param $address i32) (param $length i32) (result i32)
  (call $ap (call $prim (global.get $T_BSFROMUTF8))
    (call $bytes_view (local.get $address) (local.get $length))))

;; Add one already-marked object to the collector work stack. A node uses its
;; address unchanged. A payload header uses bit zero as a work-item tag.
(func $heap_mark_push (param $object i32)
  (local $count i32)
  (local.set $count (i32.load (i32.const 0x16c)))
  (if (i32.ge_u (local.get $count) (i32.const 1048576))
    (then (call $fail (i32.const 76)) (unreachable)))
  (i32.store (i32.add (i32.const 0x00900000)
    (i32.shl (local.get $count) (i32.const 2))) (local.get $object))
  (i32.store (i32.const 0x16c) (i32.add (local.get $count) (i32.const 1))))

;; Mark a possible node or payload reference. Exact node alignment is required.
;; Payload references may be interior pointers. Other values are ignored, so
;; pointer masks can safely include array length words and nullable references.
;; Mark before pushing to prevent cycles or shared edges from duplicating work.
(func $heap_mark (param $address i32)
  (local $index i32) (local $bitmap i32) (local $bit i32) (local $word i32)
  (local $block i32)
  (if (i32.and (i32.ge_u (local.get $address) (i32.const 0x02000000))
        (i32.lt_u (local.get $address) (i32.const 0x25c34600)))
    (then
      (local.set $index (i32.sub (local.get $address) (i32.const 0x02000000)))
      (if (i32.rem_u (local.get $index) (i32.const 8)) (then (return)))
      (local.set $index (i32.div_u (local.get $index) (i32.const 8)))
      (local.set $bitmap (i32.add (i32.const 0x26000000)
        (i32.shl (i32.shr_u (local.get $index) (i32.const 5)) (i32.const 2))))
      (local.set $bit (i32.shl (i32.const 1) (i32.and (local.get $index) (i32.const 31))))
      (local.set $word (i32.load (local.get $bitmap)))
      (if (i32.eqz (i32.and (local.get $word) (local.get $bit))) (then (return)))
      (i32.store (local.get $bitmap)
        (i32.and (local.get $word) (i32.xor (local.get $bit) (i32.const -1))))
      (call $heap_mark_push (local.get $address))
      (return)))
  (local.set $block (call $heap_blob_find (local.get $address)))
  (if (i32.eqz (local.get $block)) (then (return)))
  (if (i32.and (i32.load offset=8 (local.get $block)) (i32.const 2)) (then (return)))
  (i32.store offset=8 (local.get $block)
    (i32.or (i32.load offset=8 (local.get $block)) (i32.const 2)))
  ;; A marked raw buffer has no references to scan.
  (if (i32.load offset=4 (local.get $block))
    (then (call $heap_mark_push (i32.or (local.get $block) (i32.const 1))))))

;; Test an exact node address against the current collection's live bitmap.
;; Weak keys use this after strong marking. Other addresses are not live nodes.
(func $heap_is_marked (param $address i32) (result i32)
  (local $index i32)
  (if (i32.or (i32.lt_u (local.get $address) (i32.const 0x02000000))
        (i32.ge_u (local.get $address) (i32.const 0x25c34600)))
    (then (return (i32.const 0))))
  (local.set $index (i32.sub (local.get $address) (i32.const 0x02000000)))
  (if (i32.rem_u (local.get $index) (i32.const 8)) (then (return (i32.const 0))))
  (local.set $index (i32.div_u (local.get $index) (i32.const 8)))
  (i32.eqz (i32.and (i32.load (i32.add (i32.const 0x26000000)
    (i32.shl (i32.shr_u (local.get $index) (i32.const 5)) (i32.const 2))))
    (i32.shl (i32.const 1) (i32.and (local.get $index) (i32.const 31))))))

;; Scan one object from the two words loaded by the drain loop.
;; Each AP is queued once. Other scans can rewrite only IND nodes.
;; Reload IND words. Payload sizes and pointer masks do not change.
;; Weak processing and finalizers run only after the pending work is drained.
(func $heap_mark_object (param $object i32) (param $header i64)
  (local $word i32) (local $tag i32) (local $mask i32)
  (local $i i32) (local $words i32) (local $data i32)
  (if (i32.and (local.get $object) (i32.const 1))
    (then
      (local.set $object (i32.and (local.get $object) (i32.const -2)))
      (local.set $mask (i32.wrap_i64 (i64.shr_u (local.get $header) (i64.const 32))))
      (local.set $words (i32.shr_u (i32.wrap_i64 (local.get $header)) (i32.const 2)))
      (if (i32.and (i32.ne (local.get $mask) (i32.const -1))
            (i32.gt_u (local.get $words) (i32.const 32)))
        (then (local.set $words (i32.const 32))))
      (local.set $data (i32.add (local.get $object) (i32.const 16)))
      (local.set $i (i32.const 0))
      (block $blob_done
        (loop $blob_word
          (br_if $blob_done (i32.ge_u (local.get $i) (local.get $words)))
          (if (i32.or (i32.eq (local.get $mask) (i32.const -1))
                (i32.and (local.get $mask) (i32.shl (i32.const 1) (local.get $i))))
            (then (call $heap_mark (i32.load (i32.add (local.get $data)
              (i32.shl (local.get $i) (i32.const 2)))))))
          (local.set $i (i32.add (local.get $i) (i32.const 1)))
          (br $blob_word))))
    (else
      (local.set $word (i32.wrap_i64 (local.get $header)))
      (local.set $tag (i32.and (local.get $word) (i32.const 3)))
      (if (i32.eqz (local.get $tag))
        (then
          (local.set $data (local.get $word))
          (local.set $word (call $heap_gc_target (local.get $data)))
          (if (i32.ne (local.get $word) (local.get $data))
            (then (i32.store (local.get $object) (local.get $word))))
          (call $heap_mark (local.get $word))
          (local.set $data (i32.wrap_i64 (i64.shr_u (local.get $header) (i64.const 32))))
          (local.set $word (call $heap_gc_target (local.get $data)))
          (if (i32.ne (local.get $word) (local.get $data))
            (then (i32.store offset=4 (local.get $object) (local.get $word))))
          (call $heap_mark (local.get $word)))
        (else
          (if (i32.eq (local.get $tag) (i32.const 2))
            (then
              ;; Earlier batch entries can compress this IND after its load.
              (local.set $data (i32.load (local.get $object)))
              (local.set $word (call $heap_gc_target
                (i32.and (local.get $data) (i32.const -4))))
              (if (i32.ne (i32.or (local.get $word) (i32.const 2)) (local.get $data))
                (then (i32.store (local.get $object) (i32.or (local.get $word) (i32.const 2)))))
              (call $heap_mark (local.get $word)))
            (else
              (local.set $tag (i32.shr_u (local.get $word) (i32.const 2)))
              (if (i32.or (i32.or (i32.eq (local.get $tag) (global.get $T_TICK))
                    (i32.or (i32.eq (local.get $tag) (global.get $T_INT64))
                      (i32.eq (local.get $tag) (global.get $T_DBL))))
                    (i32.and (i32.ge_u (local.get $tag) (global.get $T_PTR))
                      (i32.le_u (local.get $tag) (global.get $T_WEAK))))
                (then (call $heap_mark (i32.wrap_i64 (i64.shr_u (local.get $header) (i64.const 32))))))))))))
)

;; Load 4 pending object headers before scanning any of them.
;; The loads are used by the scanner. They are not discarded prefetches.
;; The fixed work stack retains its existing bound. Popped entries remain
;; in at most 4 pairs of locals until this batch completes.
(func $heap_mark_drain
  (local $count i32)
  (local $object0 i32) (local $header0 i64)
  (local $object1 i32) (local $header1 i64)
  (local $object2 i32) (local $header2 i64)
  (local $object3 i32) (local $header3 i64)
  (block $done
    (loop $next
      (local.set $count (i32.load (i32.const 0x16c)))
      (br_if $done (i32.eqz (local.get $count)))
      (if (i32.ge_u (local.get $count) (i32.const 4))
        (then
          (local.set $count (i32.sub (local.get $count) (i32.const 4)))
          (i32.store (i32.const 0x16c) (local.get $count))
          (local.set $object0 (i32.load (i32.add (i32.const 9437196)
            (i32.shl (local.get $count) (i32.const 2)))))
          (local.set $object1 (i32.load (i32.add (i32.const 9437192)
            (i32.shl (local.get $count) (i32.const 2)))))
          (local.set $object2 (i32.load (i32.add (i32.const 9437188)
            (i32.shl (local.get $count) (i32.const 2)))))
          (local.set $object3 (i32.load (i32.add (i32.const 9437184)
            (i32.shl (local.get $count) (i32.const 2)))))
          (local.set $header0 (i64.load (i32.and (local.get $object0) (i32.const -2))))
          (local.set $header1 (i64.load (i32.and (local.get $object1) (i32.const -2))))
          (local.set $header2 (i64.load (i32.and (local.get $object2) (i32.const -2))))
          (local.set $header3 (i64.load (i32.and (local.get $object3) (i32.const -2))))
          (call $heap_mark_object (local.get $object0) (local.get $header0))
          (call $heap_mark_object (local.get $object1) (local.get $header1))
          (call $heap_mark_object (local.get $object2) (local.get $header2))
          (call $heap_mark_object (local.get $object3) (local.get $header3)))
        (else
          (local.set $count (i32.sub (local.get $count) (i32.const 1)))
          (i32.store (i32.const 0x16c) (local.get $count))
          (local.set $object0 (i32.load (i32.add (i32.const 0x00900000)
            (i32.shl (local.get $count) (i32.const 2)))))
          (call $heap_mark_object (local.get $object0)
            (i64.load (i32.and (local.get $object0) (i32.const -2))))))
      (br $next))))

;; Mark a bounded sequence of root words. Draining after each root prevents
;; the number of independent roots from consuming the collector work stack.
(func $heap_mark_roots (param $address i32) (param $count i32)
  (local $i i32)
  (block $done
    (loop $next
      (br_if $done (i32.ge_u (local.get $i) (local.get $count)))
      (call $heap_mark (i32.load (i32.add (local.get $address)
        (i32.shl (local.get $i) (i32.const 2)))))
      (call $heap_mark_drain)
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br $next))))

;; Collect at an outer evaluator safe point. All live graph references must be
;; in the published roots described in HEAP.md. The collector does not inspect
;; Wasm locals, run graph reductions, or move objects. Before payload reclamation
;; it invokes unreachable finalizer records through the fixed host operation.
;; That operation must not evaluate or collect. It can explicitly free payloads.
(func $gc
  (local $i i32) (local $count i32) (local $frame i32) (local $block i32)
  (local $flags i32) (local $free i32)
  (i32.store (i32.const 0x3048) (i32.const 0))
  (memory.fill (i32.const 0x26000000) (i32.const 255) (i32.const 9375000))
  (i32.store (i32.const 0x16c) (i32.const 0))
  ;; Permanent nodes include every tag, the integer table, and alignment cells.
  (local.set $count (i32.load (i32.const 0x160)))
  (block $permanent_done
    (loop $permanent
      (br_if $permanent_done (i32.ge_u (local.get $i) (local.get $count)))
      (call $heap_mark (i32.add (i32.const 0x02000000)
        (i32.mul (local.get $i) (i32.const 8))))
      (call $heap_mark_drain)
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br $permanent)))
  (call $heap_mark (i32.load (i32.const 0x154)))
  (call $heap_mark (i32.load (i32.const 0x15c)))
  (call $heap_mark_drain)
  (local.set $count (i32.add (i32.load (global.get $sp_addr)) (i32.const 1)))
  (if (i32.gt_u (local.get $count) (i32.const 1000000))
    (then (call $fail (i32.const 77)) (unreachable)))
  (call $heap_mark_roots (i32.const 0x00010000) (local.get $count))
  (local.set $count (i32.load (i32.const 0x158)))
  (if (i32.gt_u (local.get $count) (i32.const 65536))
    (then (call $fail (i32.const 77)) (unreachable)))
  (local.set $i (i32.const 0))
  (block $frames_done
    (loop $frames
      (br_if $frames_done (i32.ge_u (local.get $i) (local.get $count)))
      (local.set $frame (i32.add (i32.const 0x00500000)
        (i32.shl (local.get $i) (i32.const 6))))
      (call $heap_mark (i32.load offset=16 (local.get $frame)))
      (call $heap_mark_drain)
      (call $heap_mark_roots (i32.add (local.get $frame) (i32.const 32)) (i32.const 8))
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br $frames)))
  ;; Runtime globals and live thread records use this additional root region.
  (call $heap_mark_roots (i32.const 0x3000) (i32.const 5120))
  (call $weak_collect)
  (local.set $count (i32.load (i32.const 0x168)))
  (local.set $i (i32.const 0))
  ;; Keep all payload bytes allocated until every unreachable finalizer has
  ;; received its original data address. Clear the handle before invoking it.
  ;; Explicit frees by a finalizer are visible to the later reclamation pass.
  (block $finalizers_done
    (loop $finalizers
      (br_if $finalizers_done (i32.ge_u (local.get $i) (local.get $count)))
      (local.set $block (i32.load (i32.add (i32.const 0x00d00000)
        (i32.shl (local.get $i) (i32.const 2)))))
      (if (i32.eq (i32.and (i32.load offset=8 (local.get $block)) (i32.const 7))
            (i32.const 5))
        (then
          (local.set $flags (i32.load offset=16 (local.get $block)))
          (i32.store offset=16 (local.get $block) (i32.const 0))
          (if (local.get $flags)
            (then (call $finalize (local.get $flags)
              (i32.load offset=20 (local.get $block)))))))
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br $finalizers)))
  (local.set $i (i32.const 0))
  (block $payload_done
    (loop $payload
      (br_if $payload_done (i32.ge_u (local.get $i) (local.get $count)))
      (local.set $block (i32.load (i32.add (i32.const 0x00d00000)
        (i32.shl (local.get $i) (i32.const 2)))))
      (local.set $flags (i32.load offset=8 (local.get $block)))
      (if (i32.and (local.get $flags) (i32.const 1))
        (then
          (if (i32.and (local.get $flags) (i32.const 2))
            (then (i32.store offset=8 (local.get $block)
              (i32.and (local.get $flags) (i32.const -3))))
            (else (call $heap_blob_release (local.get $block))))))
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br $payload)))
  (local.set $i (i32.const 0))
  (loop $count_free
    (local.set $free (i32.add (local.get $free)
      (i32.popcnt (i32.load (i32.add (i32.const 0x26000000)
        (i32.shl (local.get $i) (i32.const 2)))))))
    (local.set $i (i32.add (local.get $i) (i32.const 1)))
    (br_if $count_free (i32.lt_u (local.get $i) (i32.const 2343750))))
  (i32.store (global.get $scan_addr) (i32.const 0))
  (i32.store (global.get $free_addr) (local.get $free))
  (i32.store (i32.const 0x174) (i32.const 0))
  (i32.store (i32.const 0x178) (i32.const 0))
  (i32.store (i32.const 0x170) (i32.sub (i32.const 75000000) (local.get $free))))

;; Initialize a fresh support-module memory exactly once before parsing.
;; Permanent primitive nodes are allocated in tag order. Small integers follow.
;; Rounding the permanent count to 32 preserves the reducer bitmap convention.
;; World, PairUnit, and other compound roots are installed by the entry point.
(func $heap_init
  (local $tag i32) (local $n i32) (local $i i32) (local $count i32)
  (memory.fill (i32.const 0x100) (i32.const 0) (i32.const 256))
  (memory.fill (i32.const 0x1000) (i32.const 0) (i32.const 28672))
  (memory.fill (i32.const 0x26000000) (i32.const 255) (i32.const 9375000))
  (i32.store (global.get $stack_addr) (i32.const 0x00010000))
  (i32.store (global.get $sp_addr) (i32.const -1))
  (i32.store (global.get $stack_size_addr) (i32.const 1000000))
  (i32.store (global.get $slice_addr) (i32.const 1000000000))
  (i32.store (global.get $cells_addr) (i32.const 0x02000000))
  (i32.store (global.get $map_addr) (i32.const 0x26000000))
  (i32.store (global.get $free_addr) (i32.const 75000000))
  (i32.store (i32.const 0x164) (i32.const 0x28000000))
  (loop $tags
    (drop (call $node (local.get $tag)))
    (local.set $tag (i32.add (local.get $tag) (i32.const 1)))
    (br_if $tags (i32.le_u (local.get $tag) (global.get $T_LAST_TAG))))
  (i32.store (global.get $combB_addr) (call $prim (global.get $T_B)))
  (i32.store (global.get $combC_addr) (call $prim (global.get $T_C)))
  (i32.store (global.get $combK_addr) (call $prim (global.get $T_K)))
  (i32.store (global.get $combK2_addr) (call $prim (global.get $T_K2)))
  (i32.store (global.get $combK3_addr) (call $prim (global.get $T_K3)))
  (loop $ints
    (local.set $n (call $node (global.get $T_INT)))
    (i32.store offset=4 (local.get $n) (i32.sub (local.get $i) (i32.const 10)))
    (i32.store (i32.add (i32.const 0x1000)
      (i32.shl (local.get $i) (i32.const 2))) (local.get $n))
    (local.set $i (i32.add (local.get $i) (i32.const 1)))
    (br_if $ints (i32.lt_u (local.get $i) (i32.const 266))))
  (local.set $count (i32.add (i32.load (global.get $scan_addr)) (i32.const 1)))
  (block $aligned
    (loop $pad
      (br_if $aligned (i32.eqz (i32.and (local.get $count) (i32.const 31))))
      (drop (call $node (i32.const 0)))
      (local.set $count (i32.add (local.get $count) (i32.const 1)))
      (br $pad)))
  (i32.store (i32.const 0x160) (local.get $count))
  (i32.store (i32.const 0x170) (local.get $count))
  (i32.store (global.get $alloc_addr) (i32.const 0)))
