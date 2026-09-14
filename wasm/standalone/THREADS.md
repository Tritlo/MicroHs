# Threads and state

`threads.wat` schedules graph evaluation. `primitives-state.wat` implements
thread operations, MVars, stable pointers, and weak pointers. These fragments
do not call the evaluator. They return graphs or request a thread switch.
`evaluate.wat` owns every evaluation and resumption.

## Scheduling

Each exported evaluation starts a main thread. A fork creates another thread
with the caller's World and masking state. Thread IDs increase across exported
evaluations. An ID node keeps its thread record alive after completion.
The main thread's completion ends that exported evaluation. Other live threads
receive status Died. Their snapshots and pending operations are released.
Stable pointers and weak-pointer records persist across exported evaluations.

The application stack and continuation frames occupy fixed memory while a
thread runs. A switch copies them into managed payloads owned by the thread.
Restoration copies them back and releases the snapshots. Thus a thread resumes
its exact evaluator state, including strict arguments and exception handlers.
It does not repeat evaluation from its original root. A single runnable thread
does not allocate snapshots when its slice expires.

The scheduling slice has 100,000 steps. Reducer steps, runtime primitive
entries, and RNF visits consume the slice. Charging RNF visits permits another
thread to run when a large graph contains only partial applications.
Explicit yield also requests a switch. Atomic evaluation delays automatic and
explicit yields until its result reaches WHNF. A blocking operation still
permits a switch. Nested atomic evaluation uses frame kind 4 and a saved depth.
Normal completion and exception unwinding restore that depth.

A blocked primitive returns zero with switch request 2. The evaluator restores
its saved application stack and removes that primitive's frame. After wakeup,
the thread retries the primitive head. Strict argument graphs retain their
updates. Captured reader values and completed poll results prevent a retry
from losing the event that caused its wakeup.

## MVars and exceptions

An MVar descriptor has 32 bytes and pointer mask 31. Offset 0 holds its value,
or zero when empty. Offsets 4 and 8 hold a FIFO take/put queue. Offsets 12 and 16
hold a FIFO reader queue. A blocked thread strongly retains its MVar descriptor.
A change in occupancy wakes the oldest eligible take/put waiter. Eligibility
is checked because another thread can use the MVar before a woken waiter runs.

A successful put captures its value in every blocked reader's thread record.
Each reader consumes that captured value on retry, even if a taker has already
emptied the MVar. Exception unwinding discards captured values. Try operations
return immediately and use the existing Bool and Maybe constructors.

Each thread has one pending asynchronous-exception slot. A throw to a full
slot blocks its sender in a FIFO queue. Consuming the slot wakes one sender.
Thread completion wakes all remaining senders. A throw to an ended thread
returns unit. A thread that throws to itself raises the exception immediately.

Mask 0 permits delivery at evaluator boundaries. Mask 1 permits delivery at
interruptible operations. Mask 2 prevents asynchronous delivery. A throw wakes
an interruptibly blocked target when its mask permits it. Its saved wake flag
allows delivery before the target retries the operation. Catch uses explicit
handler frames and restores the previous mask after the handler's IO result.

## Time and file descriptors

Positive thread delays convert microseconds to an absolute monotonic deadline
in nanoseconds. Nonpositive delays request a yield. Clock reads use WASI
`clock_time_get`. Clock and file-descriptor waits use canonical Preview 1
`poll_oneoff` subscriptions. Each subscription has 48 bytes. Each event has
32 bytes. Userdata contains the waiting thread's descriptor address.

A nonblocking poll adds an immediate clock subscription. When no runnable
thread exists, a blocking poll waits for an external event. A descriptor event
error wakes its operation with result -1 and stores the WASI error in errno.
A successful descriptor wait returns zero. The Haskell thread-status operation
reports code 5 for descriptor waits, as required by `Control.Concurrent`.

If every live thread waits on an MVar or a pending-exception slot, no external
event can make progress. The scheduler reports failure 89 instead of polling
without an event source. A host that implements this WASI interface must
provide clock and descriptor polling. The runtime does not import a host thread
library or use a C scheduler.

## Stable and weak pointers

A stable pointer is a positive integer slot in a managed table. Slot zero is
invalid. The table starts with 256 slots and doubles when necessary. Its pointer
mask is -1, so occupied slots strongly retain graph values. Freeing a slot
clears its value. Allocation reuses the lowest available slot. Invalid and
already freed handles report failure 93.

A weak record has 32 bytes and pointer mask zero. The registry links metadata
without retaining keys or values as strong references. The fields are:

| Offset | Meaning |
| --- | --- |
| 0 | Next registry record |
| 4 | Owner weak node, or zero after its node becomes unreachable |
| 8 | Key |
| 12 | Value; zero means the weak pointer is dead |
| 16 | Finalizer action applied to World, or zero |
| 20 | Next queued finalizer record |
| 24 | Reserved |
| 28 | Finalizer queue membership |

After strong marking, collection marks actions already queued by a previous
collection. It then computes weak reachability to a fixed point. A live key
retains its record, key, value, finalizer, and owner node. Indirection keys use
the liveness of their target without evaluating it. Newly retained values can
make other keys live. No dead value is cleared until this phase is complete.
This order supports weak chains independently of registry order.

A pure IND cycle is a possible representation of a divergent expression.
Weak lookup uses the cycle-safe link resolver and tests its representative's
mark bit. A strongly reachable divergent key stays live. An unreachable cycle
does not become live merely because the lookup visits it. This also prevents
accepted cyclic bytecode from making weak collection loop indefinitely.

The next phase clears dead keys and values and queues their finalizers.
Only after all weak values are classified does the collector mark the newly
queued actions. Those actions cannot revive a weak value in that collection.
Unreachable dead records leave the registry before payload reclamation.
The collector does not allocate a thread or execute an action.

At the next evaluator boundary, queued actions become ordinary threads.
The queue keeps each action rooted until its thread owns it. The runtime clears
the finalizer field before execution, so it runs at most once. Manual
finalization returns a sequencing graph and clears the same field. It does not
invalidate a live weak value. Raw foreign finalizers use the separate fixed
service path described in `HEAP.md`.

## Thread record and roots

A thread record has 128 bytes and pointer mask `0x04c1e33c`.
The live-thread registry retains every active record. A ThreadId node retains
its record after the thread leaves that registry. Ending a thread clears its
graph references and releases its snapshots. The ID and completion status stay
available for later queries.

| Offset | Meaning |
| --- | --- |
| 0 | Monotonic positive thread ID |
| 4 | Internal state: 0 runnable, 1 MVar wait, 2 other wait, 3 finished, 4 died |
| 8 | Next live-thread record |
| 12 | Next queue member |
| 16 | Reserved root |
| 20 | Current evaluator node |
| 24 | Evaluator base index |
| 28 | Application-stack index |
| 32 | Application-stack snapshot |
| 36 | Continuation-frame snapshot |
| 40 | Continuation-frame count |
| 44 | Masking state |
| 48 | RNF noerr flag |
| 52 | Active RNF bitmap |
| 56 | Pending asynchronous exception |
| 60 | Captured reader value |
| 64 | Wait object: MVar or target thread |
| 68 | Wait kind: 1 take, 2 put, 3 read, 4 delay, 5 throwTo, 6 read FD, 7 write FD |
| 72 | Absolute monotonic deadline, i64 nanoseconds |
| 80 | Completed poll result flag |
| 84 | Queue membership flag |
| 88 | Blocked throwTo sender queue head |
| 92 | Blocked throwTo sender queue tail |
| 96 | Atomic depth |
| 100 | Entry stack index |
| 104 | Cached ThreadId node |
| 108 | Reserved |
| 112 | Interrupted-wait delivery flag |
| 116 | File descriptor |
| 120 | Poll event error |
| 124 | Reserved |

The running thread publishes its current node and mask before collection.
Its fixed stack and frames remain normal collector roots. Suspended snapshots
use pointer mask -1. Their lengths bound copying and restoration. After
collection, both active and suspended RNF bitmaps clear bits for reclaimed
nodes. A later allocation can therefore reuse an address during RNF safely.

The fixed scheduler state occupies these words in the extra root region:

| Address | Meaning |
| --- | --- |
| `0x3080` | Live-thread registry head |
| `0x3084` | Running thread |
| `0x3088` | Main thread for the exported evaluation |
| `0x308c`, `0x3090` | Runnable queue head and tail |
| `0x3094` | Next thread ID |
| `0x3098` | Evaluation active flag |
| `0x309c` | Switch request: 0 none, 1 yield, 2 blocked |
| `0x30a0` | Restored evaluator base |
| `0x30a4` | Running atomic depth |
| `0x30a8`, `0x30ac` | Pending weak-finalizer queue head and tail |
| `0x30b0` | Stable-pointer table |
| `0x30b4` | Stable-pointer capacity |
| `0x30b8` | First potentially free stable slot |
| `0x30bc` | Weak-record registry head |
| `0x30c0` | Live-thread count |
| `0x30c4` | Clock and descriptor wait count |
| `0x30c8` | Pending asynchronous-exception count |
| `0x30d0` | Main evaluation entry stack index |

Failure 88 reports a WASI clock or poll call error. Failure 90 reports an
inconsistent scheduler queue or active-evaluation state. Failure 91 reports
thread-ID exhaustion. Failure 92 reports a runtime-handle type mismatch.
