# Standalone heap

`heap.wat` implements node construction, payload allocation, and collection.
It is a handwritten module fragment. The enclosing module supplies memory,
the checked-in tag constants, the non-returning `$fail(code)` function,
`$finalize(handle, data)` for fixed finalizer operations, and `$weak_collect`
for weak reachability and finalizer queuing.
No C or Rust compilation step is part of this implementation.

## Memory and state

The memory starts with 10,240 WebAssembly pages. Each page has 65,536 bytes.
The payload allocator can grow memory to 32,768 pages, or two GiB.
The node arena contains 75,000,000 nodes at `0x02000000`.
Each node has eight bytes. The exclusive arena end is `0x25c34600`.
The node bitmap starts at `0x26000000` and occupies 9,375,000 bytes.
A set bit denotes a free node. There are exactly 2,343,750 bitmap words.
The bitmap follows the node arena. It cannot overlap the generated foreign
names at `0x01800000` through `0x01ffffff`.

The following exports contain constant addresses. They do not contain the
current state values. This preserves the linked reducer's import contract.

| Export | Address | Value at that address |
| --- | --- | --- |
| `stack` | `0x100` | Application-stack base, `0x00010000` |
| `stack_ptr` | `0x104` | Top stack index; `-1` means empty |
| `stack_size` | `0x108` | Stack capacity, 1,000,000 entries |
| `glob_slice` | `0x10c` | Remaining reduction steps |
| `cells` | `0x110` | Node-arena base |
| `free_map` | `0x114` | Node-bitmap base |
| `next_scan_index` | `0x118` | Last allocated node index |
| `num_free` | `0x11c` | Number of set bitmap bits |
| `num_alloc` | `0x120` | Allocation count since initialization, modulo 2^32 |
| `combB` | `0x124` | Permanent B node |
| `combC` | `0x128` | Permanent C node |
| `combK` | `0x12c` | Permanent K node |
| `combK2` | `0x130` | Permanent K2 node |
| `combK3` | `0x134` | Permanent K3 node |
| `red_bb` | `0x138` | Reducer B' rewrite count |
| `red_z` | `0x13c` | Reducer Z rewrite count |
| `red_r` | `0x140` | Reducer R rewrite count |
| `red_k2` | `0x144` | Reducer K2 rewrite count |
| `red_k3` | `0x148` | Reducer K3 rewrite count |
| `red_k4` | `0x14c` | Reducer K4 rewrite count |
| `red_ccb` | `0x150` | Reducer C'B rewrite count |
| `intTable` | `0x1000` | First entry of the permanent integer array |

`intTable` is an array export. Its address points directly to entry zero,
which holds the node for `-10`. Entry 265 holds the node for `255`.
All other exports in the table point to one state word.

The support module uses these additional state words:

| Address | Meaning |
| --- | --- |
| `0x154` | Current evaluator node; publish it before collection |
| `0x158` | Number of live continuation frames |
| `0x15c` | Argument-array node root |
| `0x160` | Permanent-node count, rounded up to a multiple of 32 |
| `0x164` | Exclusive end of the payload region used so far |
| `0x168` | Number of entries in the ordered payload-block index |
| `0x16c` | Collector work-stack count; zero outside marking |
| `0x170` | Live-node count after initialization or the last collection |
| `0x174` | Payload allocations since the last collection |
| `0x178` | Payload capacity bytes plus headers allocated since collection |
| `0x180` through `0x1e8` | 27 payload free-list heads |
| `0x3000` through `0x7fff` | Additional runtime and thread roots |
| `0x3048` | Pending collection request; consumed by the next safe point |

Each additional root is a four-byte node or payload address. An unused root
must contain zero. Scalar values are accepted in this region, but a scalar
that happens to match a valid address can retain an object unnecessarily.

## Node construction

An application stores its function at offset zero and argument at offset four.
An indirection stores `target | 2` at offset zero.
A value stores `(tag << 2) | 1` at offset zero.
Int, Float, and pointer payloads occupy the four bytes at offset four.
Int64 and Double instead store a managed payload pointer there. Their eight
scalar bytes occupy a separate raw payload allocation. The collector marks
that allocation without interpreting the scalar bits as references.

`$heap_init` allocates a node for every tag, in tag order.
Thus `$prim(tag)` returns `0x02000000 + 8 * tag`.
The integer cache follows these nodes. Alignment cells complete the permanent
region. Compound roots, such as World and PairUnit, belong to the entry point.
The entry point stores them in the additional root region.

The allocator clears one free bit before it returns a node.
It clears all eight node bytes before a constructor writes the tag or links.
The linked reducer uses the same bitmap and counters.
Neither allocator moves its scan position backward between collections.
Collection resets the scan position to zero.

`$node`, `$ap`, `$int`, `$int64`, `$double`, `$float`, and `$ptr` do not collect.
They do not evaluate their arguments. `$int` uses the permanent cache when
possible. `$resolve` follows indirections without evaluation or allocation.

## Payload allocation

Each payload has a 16-byte header immediately before its returned address.
The header and payload both have 16-byte alignment.

| Header offset | Meaning |
| --- | --- |
| 0 | Payload capacity in bytes |
| 4 | Pointer-field mask |
| 8 | Flags: bit 0 allocated, bit 1 marked, bit 2 finalizer record |
| 12 | Next free block in the same size class |

Size classes contain 16, 32, 64, and subsequent powers of two through 2^30
bytes. A zero-byte request still receives 16 bytes. The allocator first checks
the matching free list. Otherwise, it appends a block after the previous block
and grows linear memory if necessary. It does not split or coalesce blocks.
All payload capacity is cleared on allocation, including reused padding.

The ordered index starts at `0x00d00000` and has 1,048,576 four-byte entries.
Each entry contains a header address. New addresses append in increasing order.
Freeing or reusing a block does not change its index entry. The index therefore
supports binary search for an allocation that contains an interior pointer.
Memory does not shrink. Free space remains available through its size class.

Int64 and Double each use one raw payload block. The index limit therefore
also bounds the number of these values that can remain live at once. This is
the space tradeoff for compact application nodes. Collection reuses blocks,
but it does not raise the limit for simultaneously live allocations.

Payload pressure can request collection before the node arena is nearly full.
An allocation sets the request after 131,072 payload allocations or 32 MiB of
allocated capacity and headers since the last collection. New blocks also
request collection once their index reaches 900,000. Free-list reuse does not
repeat that index-based request. The counters include reused allocations.
The allocation completes before the evaluator acts on the request.
`$gc` consumes the request and resets both pressure counters.

Mask bit N identifies pointer word N in the payload. A mask of `-1` scans all
capacity words. Other masks select only the first 32 words. A mask of zero
describes raw bytes. Arrays can use `-1`; scanning their scalar length field
can retain a node conservatively if its value happens to be a valid address.

`$blob_free` accepts zero or the exact address returned by `$blob_alloc`.
It rejects interior pointers and double frees. The caller must relinquish all
usable aliases before an explicit free. A live alias cannot prevent reuse
after the caller explicitly releases the allocation.

## Byte strings and lazy strings

A byte-string node has tag `T_FORPTR`, descriptor address at offset four,
and kind 1 in bit 0 of that address. A foreign pointer has kind 0.
Descriptors have 16-byte alignment, so this bit is available. `$bval` clears
the bit before access. The descriptor contains these four words:

| Descriptor offset | Meaning |
| --- | --- |
| 0 | Byte length |
| 4 | Data address |
| 8 | Mutable capacity; zero for an immutable view |
| 12 | Shared finalizer-record address |

The descriptor uses pointer mask 10. This marks its data address and finalizer
record. The data address can be an interior address used by a substring.
`$bytes_view` creates a descriptor over existing memory. It does not copy data.
It creates an empty finalizer record before any descriptor aliases exist.
`$bytes` copies data into a managed allocation and then creates the descriptor.
`$string` returns an application of `T_BSFROMUTF8` to a byte-string view.
The evaluator performs UTF-8 conversion when it demands the string.

Static memory does not need a payload allocation to survive collection.
A view over managed memory retains the containing allocation. A view over
temporary unmanaged memory does not extend that memory's lifetime.
The caller must keep such memory valid until the string is no longer needed.
`$finalizer_new(handle, data)` creates a managed finalizer record. Its first word
contains a service handle. Its second word contains the original data address.
Its pointer mask is 2. Its header sets the finalizer-record flag.
The finalizer registration primitive can replace a zero handle later.
Descriptor aliases share this record. They do not copy the service handle.
Registration through any alias updates the same record, including aliases
created before the finalizer was registered.

A live record retains its original data allocation. When the record becomes
unreachable, the collector clears its handle and calls `$finalize(handle, data)`.
A zero handle needs no call. The record is then reclaimed with other payloads.
Clearing the handle before the call prevents duplicate invocation.
The operation can release payloads explicitly. It must not evaluate a graph,
run collection, register another finalizer, or publish new graph roots.
The finalizer must have a fixed host implementation. There is no C callback.

## Collection and roots

Collection runs only at an outer evaluator safe point. No primitive may leave
an unfinished graph update when it returns to that point. Every live reference
must be in one of the following locations before `$gc` starts:

1. The permanent node region.
2. The current-node state word at `0x154`.
3. The argument-array root at `0x15c`.
4. The application stack, from index zero through `stack_ptr`, inclusive.
5. A live continuation frame at `0x00500000`.
6. An additional root word from `0x3000` through `0x7fff`.

Each continuation frame has 64 bytes. The collector scans the update node at
offset 16 and the eight reference words at offsets 32 through 60.
It does not scan the other frame fields. The frame region holds 65,536 frames.
Live thread records must be reachable through the additional root region.
Their managed descriptors must use pointer masks for their graph references.

The collector first sets every node bitmap bit to free. It then marks roots
and their children by clearing bits. It marks a node before it schedules its
children. This handles sharing and cycles without duplicate work.
Applications retain both links. Indirections retain their target.
Before marking those links, the collector follows and shortens IND chains.
It changes a link only when the target address changes. It does not replace
root slots, saved update addresses, or conservative payload words. Other roots
can therefore keep an original IND node alive after its graph links shorten.
Floyd cycle detection uses constant scratch space and has no length limit.
A pure IND cycle uses a self-IND representative. It still denotes divergence.
Value tags 7 through 14 retain their descriptor or pointer at offset four.
Int64 and Double also retain their separate scalar payloads at that offset.
`T_TICK` retains its byte-string label node at offset four.
Scalar node payloads are not scanned.

Managed payloads have a separate mark bit in their header. A pointer into a
payload retains the whole allocation. The binary search rejects addresses in
headers, free blocks, and the unused region above the allocation limit.
It also rejects an address exactly one byte after the capacity.
An empty view at the end of its source needs no source bytes to remain valid.

The collector work stack starts at `0x00900000` and holds 1,048,576 entries.
A node work item is its address. A payload work item is its header address
with bit zero set. The collector uses no recursive Wasm function calls.
Each completed root traversal drains the work stack before the next root.

The drain loads four eight-byte node or payload headers before scanning them.
The scanner consumes these values; the loads are not discarded prefetches.
This permits the engine to overlap independent memory accesses. A partial batch
uses the same scanner one object at a time. Popped work remains in local values
while children can overwrite its former stack slots.

Application headers stay valid during the batch because compression changes
only IND nodes, and each marked application enters the queue once. Payload
sizes and pointer masks also stay fixed. An IND header can change when another
object compresses a shared chain. The scanner therefore reloads its current
word before following that target. Weak processing and finalizers run after
the pending batch has completed.

After strong marking, `$weak_collect` computes weak reachability to a fixed
point. It then queues dead weak pointers' Haskell finalizers and marks their
action graphs. It does not execute them. See `THREADS.md` for the phase order.

The collector then invokes all unmarked foreign finalizer records.
This pass completes before automatic payload reclamation starts, so original
data remains allocated when the finalizer receives its address. Each explicit
free from a finalizer takes effect immediately.

The next pass adds each remaining unmarked allocation to its free list.
It clears only the mark bit on survivors. The finalizer-record flag remains.
The collector then counts the set node bitmap bits and updates `num_free`.
It shortens IND links without evaluating combinators, moving objects, or
shrinking memory. It does not allocate while it invokes a finalizer.

## Resource failures

All constructors can fail without collection. The evaluator must collect with
enough reserve for the next primitive's complete graph update.
The allocator checks resource arithmetic before it forms memory addresses.
The failure codes used in this fragment are:

| Code | Cause |
| --- | --- |
| 71 | No free node, or an exhausted bitmap scan |
| 72 | Oversized payload request or invalid byte interval |
| 73 | Payload exceeds the two-GiB limit, or memory growth fails |
| 74 | Ordered payload-block index is full |
| 75 | Invalid explicit free |
| 76 | Collector work stack is full |
| 77 | Application-stack or continuation-frame root count is invalid |
| 78 | Primitive tag is outside the checked-in table |

The size-class allocator can fail from payload fragmentation even if other
classes contain free space. The ordered index limits the number of distinct
blocks ever created. These are explicit resource limits of this implementation.
