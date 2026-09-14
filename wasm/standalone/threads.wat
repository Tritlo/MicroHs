;; Cooperative scheduling over explicit evaluator state.
;; A thread owns a current node, a spine snapshot, continuation frames, masking
;; state, and any delivered MVar value or asynchronous exception. All snapshots
;; are managed payloads. No scheduler function evaluates a graph recursively.
;; See THREADS.md for the 128-byte record and fixed state-word layouts.

;; Append a thread to a FIFO queue. A thread can belong to only one queue.
;; The same next field serves the run queue and the blocked-operation queues.
(func $thread_queue_push (param $head i32) (param $tail i32) (param $thread i32)
  (local $last i32)
  (if (i32.load offset=84 (local.get $thread))
    (then (call $fail (i32.const 90)) (unreachable)))
  (i32.store offset=12 (local.get $thread) (i32.const 0))
  (i32.store offset=84 (local.get $thread) (i32.const 1))
  (local.set $last (i32.load (local.get $tail)))
  (if (local.get $last)
    (then (i32.store offset=12 (local.get $last) (local.get $thread)))
    (else (i32.store (local.get $head) (local.get $thread))))
  (i32.store (local.get $tail) (local.get $thread)))

;; Remove the oldest queued thread. Zero means the queue is empty.
(func $thread_queue_pop (param $head i32) (param $tail i32) (result i32)
  (local $thread i32) (local $next i32)
  (local.set $thread (i32.load (local.get $head)))
  (if (i32.eqz (local.get $thread)) (then (return (i32.const 0))))
  (local.set $next (i32.load offset=12 (local.get $thread)))
  (i32.store (local.get $head) (local.get $next))
  (if (i32.eqz (local.get $next)) (then (i32.store (local.get $tail) (i32.const 0))))
  (i32.store offset=12 (local.get $thread) (i32.const 0))
  (i32.store offset=84 (local.get $thread) (i32.const 0))
  (local.get $thread))

;; Unlink a specified thread when an asynchronous exception cancels its wait.
;; A queue entry cannot remain after the thread moves to the run queue.
(func $thread_queue_remove (param $head i32) (param $tail i32) (param $thread i32)
  (local $p i32) (local $previous i32) (local $next i32)
  (local.set $p (i32.load (local.get $head)))
  (block $found
    (loop $search
      (if (i32.eqz (local.get $p))
        (then (call $fail (i32.const 90)) (unreachable)))
      (br_if $found (i32.eq (local.get $p) (local.get $thread)))
      (local.set $previous (local.get $p))
      (local.set $p (i32.load offset=12 (local.get $p)))
      (br $search)))
  (local.set $next (i32.load offset=12 (local.get $thread)))
  (if (local.get $previous)
    (then (i32.store offset=12 (local.get $previous) (local.get $next)))
    (else (i32.store (local.get $head) (local.get $next))))
  (if (i32.eq (i32.load (local.get $tail)) (local.get $thread))
    (then (i32.store (local.get $tail) (local.get $previous))))
  (i32.store offset=12 (local.get $thread) (i32.const 0))
  (i32.store offset=84 (local.get $thread) (i32.const 0)))

;; Test whether a blocked operation participates in WASI polling.
(func $thread_event_wait (param $kind i32) (result i32)
  (i32.or (i32.eq (local.get $kind) (i32.const 4))
    (i32.or (i32.eq (local.get $kind) (i32.const 6))
      (i32.eq (local.get $kind) (i32.const 7)))))

;; Make a dequeued or externally ready thread runnable. A delivered reader value
;; is retained. The running thread is not also inserted into the run queue.
(func $thread_runnable (param $thread i32)
  (if (i32.ge_u (i32.load offset=4 (local.get $thread)) (i32.const 3)) (then (return)))
  (if (call $thread_event_wait (i32.load offset=68 (local.get $thread)))
    (then (i32.store (i32.const 0x30c4)
      (i32.sub (i32.load (i32.const 0x30c4)) (i32.const 1)))))
  (i32.store offset=4 (local.get $thread) (i32.const 0))
  (i32.store offset=64 (local.get $thread) (i32.const 0))
  (i32.store offset=68 (local.get $thread) (i32.const 0))
  (if (i32.ne (local.get $thread) (i32.load (i32.const 0x3084)))
    (then (call $thread_queue_push (i32.const 0x308c) (i32.const 0x3090)
      (local.get $thread)))))

;; Cancel a blocked wait before exception delivery or thread shutdown.
;; readMVar values captured by a previous wake must not reach another read.
(func $thread_unpark (param $thread i32)
  (local $kind i32) (local $object i32)
  (if (i32.or (i32.eqz (i32.load offset=4 (local.get $thread)))
        (i32.ge_u (i32.load offset=4 (local.get $thread)) (i32.const 3)))
    (then (return)))
  (local.set $kind (i32.load offset=68 (local.get $thread)))
  (local.set $object (i32.load offset=64 (local.get $thread)))
  (if (i32.or (i32.eq (local.get $kind) (i32.const 1))
        (i32.eq (local.get $kind) (i32.const 2)))
    (then (call $thread_queue_remove (i32.add (local.get $object) (i32.const 4))
      (i32.add (local.get $object) (i32.const 8)) (local.get $thread))))
  (if (i32.eq (local.get $kind) (i32.const 3))
    (then (call $thread_queue_remove (i32.add (local.get $object) (i32.const 12))
      (i32.add (local.get $object) (i32.const 16)) (local.get $thread))))
  (if (i32.eq (local.get $kind) (i32.const 5))
    (then (call $thread_queue_remove (i32.add (local.get $object) (i32.const 88))
      (i32.add (local.get $object) (i32.const 92)) (local.get $thread))))
  (i32.store offset=60 (local.get $thread) (i32.const 0))
  (i32.store offset=112 (local.get $thread) (i32.const 1))
  (call $thread_runnable (local.get $thread)))

;; Allocate a runnable thread record and its stable ThreadId node. The caller
;; chooses whether to enqueue it. Ended records survive through ThreadId nodes.
(func $thread_new (param $root i32) (param $entry_sp i32) (param $mask i32) (result i32)
  (local $thread i32) (local $node i32) (local $id i32)
  (local.set $id (i32.load (i32.const 0x3094)))
  (if (i32.eqz (local.get $id)) (then (local.set $id (i32.const 1))))
  (if (i32.gt_u (local.get $id) (i32.const 2147483647))
    (then (call $fail (i32.const 91)) (unreachable)))
  (i32.store (i32.const 0x3094) (i32.add (local.get $id) (i32.const 1)))
  (local.set $thread (call $blob_alloc (i32.const 128) (i32.const 0x04c1e33c)))
  (i32.store (local.get $thread) (local.get $id))
  (i32.store offset=8 (local.get $thread) (i32.load (i32.const 0x3080)))
  (i32.store (i32.const 0x3080) (local.get $thread))
  (i32.store offset=20 (local.get $thread) (local.get $root))
  (i32.store offset=24 (local.get $thread) (local.get $entry_sp))
  (i32.store offset=28 (local.get $thread) (local.get $entry_sp))
  (i32.store offset=44 (local.get $thread) (local.get $mask))
  (i32.store offset=100 (local.get $thread) (local.get $entry_sp))
  (local.set $node (call $node (global.get $T_THID)))
  (i32.store offset=4 (local.get $node) (local.get $thread))
  (i32.store offset=104 (local.get $thread) (local.get $node))
  (i32.store (i32.const 0x30c0) (i32.add (i32.load (i32.const 0x30c0)) (i32.const 1)))
  (local.get $thread))

;; Begin one exported evaluation. Its initial thread uses the caller's stack
;; prefix in place. The stable and weak-pointer registries persist across calls.
(func $threads_begin (param $root i32) (param $entry_sp i32)
  (local $thread i32)
  (if (i32.load (i32.const 0x3098))
    (then (call $fail (i32.const 90)) (unreachable)))
  (i32.store (i32.const 0x3098) (i32.const 1))
  (i32.store (i32.const 0x30d0) (local.get $entry_sp))
  (local.set $thread (call $thread_new (local.get $root) (local.get $entry_sp)
    (i32.load (i32.const 0x3040))))
  (i32.store (i32.const 0x3084) (local.get $thread))
  (i32.store (i32.const 0x3088) (local.get $thread))
  (i32.store (i32.const 0x309c) (i32.const 0))
  (i32.store (i32.const 0x30a0) (local.get $entry_sp))
  (i32.store (i32.const 0x30a4) (i32.const 0))
  (i32.store (i32.const 0x10c) (i32.const 100000)))

;; Publish the running node before a possible collection. Snapshot fields are
;; empty while a thread runs; its live spine and frames remain in fixed memory.
(func $threads_publish (param $n i32) (param $base i32)
  (local $thread i32)
  (local.set $thread (i32.load (i32.const 0x3084)))
  (i32.store offset=20 (local.get $thread) (local.get $n))
  (i32.store offset=24 (local.get $thread) (local.get $base))
  (i32.store offset=44 (local.get $thread) (i32.load (i32.const 0x3040))))

;; Copy the running machine state before another thread uses fixed memory.
;; These functions cannot collect. The record owns each snapshot immediately.
(func $thread_save (param $n i32) (param $base i32)
  (local $thread i32) (local $count i32) (local $p i32)
  (local.set $thread (i32.load (i32.const 0x3084)))
  (call $threads_publish (local.get $n) (local.get $base))
  (i32.store offset=28 (local.get $thread) (i32.load (i32.const 0x104)))
  (local.set $count (i32.add (i32.load (i32.const 0x104)) (i32.const 1)))
  (if (local.get $count)
    (then
      (local.set $p (call $blob_alloc (i32.shl (local.get $count) (i32.const 2)) (i32.const -1)))
      (i32.store offset=32 (local.get $thread) (local.get $p))
      (memory.copy (local.get $p) (i32.const 0x10000)
        (i32.shl (local.get $count) (i32.const 2)))))
  (local.set $count (i32.load (i32.const 0x158)))
  (i32.store offset=40 (local.get $thread) (local.get $count))
  (if (local.get $count)
    (then
      (local.set $p (call $blob_alloc (i32.shl (local.get $count) (i32.const 6)) (i32.const -1)))
      (i32.store offset=36 (local.get $thread) (local.get $p))
      (memory.copy (local.get $p) (i32.const 0x500000)
        (i32.shl (local.get $count) (i32.const 6)))))
  (i32.store offset=48 (local.get $thread) (i32.load (i32.const 0x3054)))
  (i32.store offset=52 (local.get $thread) (i32.load (i32.const 0x3058)))
  (i32.store offset=96 (local.get $thread) (i32.load (i32.const 0x30a4))))

;; Restore a dequeued thread and release its snapshot storage. All graph
;; references are in fixed roots before the next evaluator collection point.
(func $thread_restore (param $thread i32) (result i32)
  (local $p i32) (local $count i32)
  (i32.store (i32.const 0x3084) (local.get $thread))
  (i32.store (i32.const 0x104) (i32.load offset=28 (local.get $thread)))
  (local.set $p (i32.load offset=32 (local.get $thread)))
  (if (local.get $p)
    (then
      (memory.copy (i32.const 0x10000) (local.get $p)
        (i32.shl (i32.add (i32.load offset=28 (local.get $thread)) (i32.const 1)) (i32.const 2)))
      (call $blob_free (local.get $p))
      (i32.store offset=32 (local.get $thread) (i32.const 0))))
  (local.set $count (i32.load offset=40 (local.get $thread)))
  (i32.store (i32.const 0x158) (local.get $count))
  (local.set $p (i32.load offset=36 (local.get $thread)))
  (if (local.get $p)
    (then
      (memory.copy (i32.const 0x500000) (local.get $p)
        (i32.shl (local.get $count) (i32.const 6)))
      (call $blob_free (local.get $p))
      (i32.store offset=36 (local.get $thread) (i32.const 0))))
  (i32.store (i32.const 0x3040) (i32.load offset=44 (local.get $thread)))
  (i32.store (i32.const 0x3054) (i32.load offset=48 (local.get $thread)))
  (i32.store (i32.const 0x3058) (i32.load offset=52 (local.get $thread)))
  (i32.store (i32.const 0x30a4) (i32.load offset=96 (local.get $thread)))
  (i32.store (i32.const 0x30a0) (i32.load offset=24 (local.get $thread)))
  (i32.store (i32.const 0x3004) (i32.const 0))
  (i32.store (i32.const 0x10c) (i32.const 100000))
  (i32.load offset=20 (local.get $thread)))

;; Read monotonic nanoseconds for deadlines. This uses synchronous WASI scratch.
(func $thread_now (result i64)
  (local $error i32)
  (local.set $error (call $wasi_clock_time_get (i32.const 1) (i64.const 1000) (i32.const 0x220)))
  (if (local.get $error)
    (then (i32.store (i32.const 0x240) (local.get $error))
      (call $fail (i32.const 88)) (unreachable)))
  (i64.load (i32.const 0x220)))

;; Poll clock and file-descriptor waits. Each canonical WASI subscription has
;; 48 bytes; each event has 32 bytes. Nonblocking polls add an immediate clock.
;; An event error wakes the operation with an error result instead of spinning.
(func $threads_poll (param $wait i32)
  (local $count i32) (local $thread i32) (local $kind i32) (local $index i32)
  (local $subscriptions i32) (local $events i32) (local $p i32) (local $error i32)
  (local.set $count (i32.load (i32.const 0x30c4)))
  (if (i32.eqz (local.get $count)) (then (return)))
  (if (i32.eqz (local.get $wait))
    (then (local.set $count (i32.add (local.get $count) (i32.const 1)))))
  (if (i32.gt_u (local.get $count) (i32.const 22369621))
    (then (call $fail (i32.const 73)) (unreachable)))
  (local.set $subscriptions (call $blob_alloc (i32.mul (local.get $count) (i32.const 48)) (i32.const 0)))
  (local.set $events (call $blob_alloc (i32.shl (local.get $count) (i32.const 5)) (i32.const 0)))
  (local.set $thread (i32.load (i32.const 0x3080)))
  (block $filled
    (loop $threads
      (br_if $filled (i32.eqz (local.get $thread)))
      (local.set $kind (i32.load offset=68 (local.get $thread)))
      (if (i32.and (i32.eq (i32.load offset=4 (local.get $thread)) (i32.const 2))
            (call $thread_event_wait (local.get $kind)))
        (then
          (local.set $p (i32.add (local.get $subscriptions)
            (i32.mul (local.get $index) (i32.const 48))))
          (i64.store (local.get $p) (i64.extend_i32_u (local.get $thread)))
          (if (i32.eq (local.get $kind) (i32.const 4))
            (then
              (i32.store offset=16 (local.get $p) (i32.const 1))
              (i64.store offset=24 (local.get $p) (i64.load offset=72 (local.get $thread)))
              (i64.store offset=32 (local.get $p) (i64.const 1000))
              (i32.store16 offset=40 (local.get $p) (i32.const 1)))
            (else
              (i32.store8 offset=8 (local.get $p) (i32.sub (local.get $kind) (i32.const 5)))
              (i32.store offset=16 (local.get $p) (i32.load offset=116 (local.get $thread)))))
          (local.set $index (i32.add (local.get $index) (i32.const 1)))))
      (local.set $thread (i32.load offset=8 (local.get $thread)))
      (br $threads)))
  (if (i32.eqz (local.get $wait))
    (then
      (local.set $p (i32.add (local.get $subscriptions)
        (i32.mul (local.get $index) (i32.const 48))))
      (i32.store offset=16 (local.get $p) (i32.const 1))
      (local.set $index (i32.add (local.get $index) (i32.const 1)))))
  (if (i32.ne (local.get $index) (local.get $count))
    (then (call $fail (i32.const 90)) (unreachable)))
  (local.set $error (call $wasi_poll_oneoff (local.get $subscriptions)
    (local.get $events) (local.get $count) (i32.const 0x220)))
  (if (local.get $error)
    (then (i32.store (i32.const 0x240) (local.get $error))
      (call $fail (i32.const 88)) (unreachable)))
  (local.set $count (i32.load (i32.const 0x220)))
  (local.set $index (i32.const 0))
  (block $done
    (loop $ready
      (br_if $done (i32.ge_u (local.get $index) (local.get $count)))
      (local.set $p (i32.add (local.get $events) (i32.shl (local.get $index) (i32.const 5))))
      (local.set $thread (i32.load (local.get $p)))
      (if (local.get $thread)
        (then
          (i32.store offset=80 (local.get $thread) (i32.const 1))
          (i32.store offset=120 (local.get $thread) (i32.load16_u offset=8 (local.get $p)))
          (call $thread_runnable (local.get $thread))))
      (local.set $index (i32.add (local.get $index) (i32.const 1)))
      (br $ready)))
  (call $blob_free (local.get $subscriptions))
  (call $blob_free (local.get $events)))

;; Wait until a runnable thread exists. A graph with only blocked MVars or
;; throwTo senders has no external event that can make progress: fail explicitly.
(func $thread_choose (result i32)
  (local $thread i32)
  (loop $choose
    (local.set $thread (call $thread_queue_pop (i32.const 0x308c) (i32.const 0x3090)))
    (if (local.get $thread) (then (return (local.get $thread))))
    (if (i32.eqz (i32.load (i32.const 0x30c4)))
      (then (call $fail (i32.const 89)) (unreachable)))
    (call $threads_poll (i32.const 1))
    (br $choose))
  (unreachable))

;; Start queued Haskell weak finalizers as ordinary threads after collection.
;; Their action graphs were marked before heap sweep. Raw foreign finalizers
;; have a separate fixed-service path and never enter this queue.
(func $threads_start_finalizers
  (local $record i32) (local $action i32) (local $thread i32)
  (block $done
    (loop $next
      (local.set $record (i32.load (i32.const 0x30a8)))
      (br_if $done (i32.eqz (local.get $record)))
      (br_if $done (i32.lt_u (i32.load (i32.const 0x11c)) (i32.const 32)))
      (i32.store (i32.const 0x30a8) (i32.load offset=20 (local.get $record)))
      (if (i32.eqz (i32.load (i32.const 0x30a8)))
        (then (i32.store (i32.const 0x30ac) (i32.const 0))))
      (i32.store offset=20 (local.get $record) (i32.const 0))
      (i32.store offset=28 (local.get $record) (i32.const 0))
      (local.set $action (i32.load offset=16 (local.get $record)))
      (i32.store offset=16 (local.get $record) (i32.const 0))
      (if (local.get $action)
        (then
          (local.set $thread (call $thread_new (local.get $action) (i32.const -1) (i32.const 0)))
          (call $thread_queue_push (i32.const 0x308c) (i32.const 0x3090) (local.get $thread))
          (i32.store (i32.const 0x309c) (i32.const 1))))
      (br $next))))

;; Process a safe scheduling boundary. Automatic and explicit yields wait until
;; atomic evaluation finishes. A blocked operation always permits another thread.
;; A single runnable thread keeps its fixed stack and avoids snapshot allocation.
(func $threads_boundary (param $n i32) (param $base i32) (result i32)
  (local $thread i32)
  (call $threads_publish (local.get $n) (local.get $base))
  (i32.store (i32.const 0x30a0) (local.get $base))
  (call $threads_start_finalizers)
  (local.set $thread (i32.load (i32.const 0x3084)))
  (if (i32.and (i32.eqz (i32.load offset=4 (local.get $thread)))
        (i32.and (i32.eqz (i32.load (i32.const 0x309c)))
          (i32.gt_s (i32.load (i32.const 0x10c)) (i32.const 1))))
    (then (return (local.get $n))))
  (if (i32.and (i32.eqz (i32.load offset=4 (local.get $thread)))
        (i32.ne (i32.load (i32.const 0x30a4)) (i32.const 0)))
    (then
      (i32.store (i32.const 0x10c) (i32.const 100000))
      (return (local.get $n))))
  (call $threads_poll (i32.const 0))
  (i32.store (i32.const 0x309c) (i32.const 0))
  (if (i32.and (i32.eqz (i32.load offset=4 (local.get $thread)))
        (i32.eqz (i32.load (i32.const 0x308c))))
    (then
      (i32.store (i32.const 0x10c) (i32.const 100000))
      (return (local.get $n))))
  (call $thread_save (local.get $n) (local.get $base))
  (if (i32.eqz (i32.load offset=4 (local.get $thread)))
    (then (call $thread_queue_push (i32.const 0x308c) (i32.const 0x3090) (local.get $thread))))
  ;; The old current thread must no longer be excluded by wake operations.
  (i32.store (i32.const 0x3084) (i32.const 0))
  (call $thread_restore (call $thread_choose)))

;; Take a pending exception when the mask permits delivery. A forced wake from
;; an interruptible wait also qualifies. Wake one blocked throwTo sender when
;; the pending slot becomes empty. Discard any abandoned readMVar delivery.
(func $thread_take_exception (param $interruptible i32) (result i32)
  (local $thread i32) (local $exception i32) (local $sender i32)
  (if (i32.eqz (i32.load (i32.const 0x30c8))) (then (return (i32.const 0))))
  (local.set $thread (i32.load (i32.const 0x3084)))
  (if (i32.or (i32.eq (i32.load (i32.const 0x3040)) (i32.const 2))
        (i32.and (i32.eq (i32.load (i32.const 0x3040)) (i32.const 1))
          (i32.eqz (i32.or (local.get $interruptible)
            (i32.load offset=112 (local.get $thread))))))
    (then (return (i32.const 0))))
  (local.set $exception (i32.load offset=56 (local.get $thread)))
  (if (i32.eqz (local.get $exception)) (then (return (i32.const 0))))
  (i32.store offset=56 (local.get $thread) (i32.const 0))
  (i32.store offset=60 (local.get $thread) (i32.const 0))
  (i32.store offset=112 (local.get $thread) (i32.const 0))
  (i32.store (i32.const 0x30c8) (i32.sub (i32.load (i32.const 0x30c8)) (i32.const 1)))
  (local.set $sender (call $thread_queue_pop (i32.add (local.get $thread) (i32.const 88))
    (i32.add (local.get $thread) (i32.const 92))))
  (if (local.get $sender) (then (call $thread_runnable (local.get $sender))))
  (local.get $exception))

;; Remove a completed thread from the live registry. The record retains only
;; its identity and status after heavy state is released. ThreadId keeps it live.
(func $thread_end (param $thread i32) (param $status i32)
  (local $p i32) (local $previous i32) (local $sender i32)
  (local.set $p (i32.load (i32.const 0x3080)))
  (block $found
    (loop $find
      (if (i32.eqz (local.get $p)) (then (call $fail (i32.const 90)) (unreachable)))
      (br_if $found (i32.eq (local.get $p) (local.get $thread)))
      (local.set $previous (local.get $p))
      (local.set $p (i32.load offset=8 (local.get $p)))
      (br $find)))
  (if (local.get $previous)
    (then (i32.store offset=8 (local.get $previous) (i32.load offset=8 (local.get $thread))))
    (else (i32.store (i32.const 0x3080) (i32.load offset=8 (local.get $thread)))))
  (if (i32.load offset=56 (local.get $thread))
    (then (i32.store (i32.const 0x30c8)
      (i32.sub (i32.load (i32.const 0x30c8)) (i32.const 1)))))
  (i32.store offset=4 (local.get $thread) (local.get $status))
  (i32.store offset=8 (local.get $thread) (i32.const 0))
  (i32.store offset=16 (local.get $thread) (i32.const 0))
  (i32.store offset=20 (local.get $thread) (i32.const 0))
  (i32.store offset=52 (local.get $thread) (i32.const 0))
  (i32.store offset=56 (local.get $thread) (i32.const 0))
  (i32.store offset=60 (local.get $thread) (i32.const 0))
  (i32.store offset=64 (local.get $thread) (i32.const 0))
  (i32.store offset=68 (local.get $thread) (i32.const 0))
  (i32.store (i32.const 0x30c0) (i32.sub (i32.load (i32.const 0x30c0)) (i32.const 1)))
  (block $done
    (loop $senders
      (local.set $sender (call $thread_queue_pop (i32.add (local.get $thread) (i32.const 88))
        (i32.add (local.get $thread) (i32.const 92))))
      (br_if $done (i32.eqz (local.get $sender)))
      (call $thread_runnable (local.get $sender))
      (br $senders))))

;; Release suspended RNF scratch and the two machine snapshots at shutdown.
;; The abandoned graph remains ordinary GC-managed memory.
(func $thread_drop_snapshots (param $thread i32)
  (local $p i32) (local $count i32) (local $frame i32)
  (local.set $p (i32.load offset=36 (local.get $thread)))
  (local.set $frame (local.get $p))
  (local.set $count (i32.load offset=40 (local.get $thread)))
  (block $done
    (loop $frames
      (br_if $done (i32.eqz (local.get $count)))
      (if (i32.eq (i32.load (local.get $frame)) (i32.const 3))
        (then
          (call $blob_free (i32.load offset=40 (local.get $frame)))
          (call $blob_free (i32.load offset=44 (local.get $frame)))))
      (local.set $frame (i32.add (local.get $frame) (i32.const 64)))
      (local.set $count (i32.sub (local.get $count) (i32.const 1)))
      (br $frames)))
  (call $blob_free (local.get $p))
  (call $blob_free (i32.load offset=32 (local.get $thread)))
  (i32.store offset=32 (local.get $thread) (i32.const 0))
  (i32.store offset=36 (local.get $thread) (i32.const 0))
  (i32.store offset=40 (local.get $thread) (i32.const 0)))

;; Complete the running thread. Main completion ends the evaluation immediately.
;; Other live threads are cancelled and retain status Died through their IDs.
;; Child completion restores the next runnable thread instead of returning.
(func $threads_complete (param $result i32) (param $status i32) (result i32)
  (local $thread i32) (local $is_main i32) (local $next i32)
  (local.set $thread (i32.load (i32.const 0x3084)))
  (local.set $is_main (i32.eq (local.get $thread) (i32.load (i32.const 0x3088))))
  (i32.store offset=40 (local.get $thread) (i32.const 0))
  (call $thread_end (local.get $thread) (local.get $status))
  (if (local.get $is_main)
    (then
      (block $done
        (loop $cancel
          (local.set $thread (i32.load (i32.const 0x3080)))
          (br_if $done (i32.eqz (local.get $thread)))
          ;; Remove every outstanding queue membership before ending a child.
          (if (i32.eqz (i32.load offset=4 (local.get $thread)))
            (then
              (if (i32.load offset=84 (local.get $thread))
                (then (call $thread_queue_remove (i32.const 0x308c) (i32.const 0x3090)
                  (local.get $thread)))))
            (else
              (call $thread_unpark (local.get $thread))
              (if (i32.load offset=84 (local.get $thread))
                (then (call $thread_queue_remove (i32.const 0x308c) (i32.const 0x3090)
                  (local.get $thread))))))
          (call $thread_drop_snapshots (local.get $thread))
          (call $thread_end (local.get $thread) (i32.const 4))
          (br $cancel)))
      (i32.store (i32.const 0x3084) (i32.const 0))
      (i32.store (i32.const 0x3088) (i32.const 0))
      (i32.store (i32.const 0x308c) (i32.const 0))
      (i32.store (i32.const 0x3090) (i32.const 0))
      (i32.store (i32.const 0x3098) (i32.const 0))
      (i32.store (i32.const 0x309c) (i32.const 0))
      (i32.store (i32.const 0x30a4) (i32.const 0))
      (i32.store (i32.const 0x3054) (i32.const 0))
      (i32.store (i32.const 0x3058) (i32.const 0))
      (i32.store (i32.const 0x104) (i32.load (i32.const 0x30d0)))
      (i32.store (i32.const 0x158) (i32.const 0))
      (i32.store (i32.const 0x154) (local.get $result))
      (return (local.get $result))))
  (i32.store (i32.const 0x3084) (i32.const 0))
  (i32.store (i32.const 0x158) (i32.const 0))
  (call $threads_start_finalizers)
  (call $threads_poll (i32.const 0))
  (local.set $next (call $thread_choose))
  (i32.store (i32.const 0x309c) (i32.const 0))
  (call $thread_restore (local.get $next)))

;; Sanitize visited bitmaps in suspended RNF frames after collection. Current
;; frames are handled by $rnf_after_gc. This permits correct RNF preemption.
(func $threads_after_gc
  (local $thread i32) (local $frame i32) (local $count i32) (local $bitmap i32) (local $i i32)
  (local.set $thread (i32.load (i32.const 0x3080)))
  (block $done
    (loop $threads
      (br_if $done (i32.eqz (local.get $thread)))
      (local.set $frame (i32.load offset=36 (local.get $thread)))
      (if (local.get $frame)
        (then
          (local.set $count (i32.load offset=40 (local.get $thread)))
          (block $frames_done
            (loop $frames
              (br_if $frames_done (i32.eqz (local.get $count)))
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
              (local.set $frame (i32.add (local.get $frame) (i32.const 64)))
              (local.set $count (i32.sub (local.get $count) (i32.const 1)))
              (br $frames)))))
      (local.set $thread (i32.load offset=8 (local.get $thread)))
      (br $threads))))
