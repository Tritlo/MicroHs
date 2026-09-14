//! Temporary application spines used outside the hot stack loop.
use super::*;

pub(in crate::runtime) struct Spine {
    pub(in crate::runtime) head: NodeId,
    pub(in crate::runtime) storage: SpineStorage,
}

/// Temporary application spine used by the fallback reducer and serializer.
pub(in crate::runtime) struct EvalSpine {
    pub(in crate::runtime) inline_args: [MaybeUninit<NodeId>; INLINE_SPINE],
    pub(in crate::runtime) inline_apps: [MaybeUninit<NodeId>; INLINE_SPINE],
    pub(in crate::runtime) inline_len: usize,
    pub(in crate::runtime) heap_args: Vec<NodeId>,
    pub(in crate::runtime) heap_apps: Vec<NodeId>,
    pub(in crate::runtime) heap: bool,
}

pub(in crate::runtime) enum SpineStorage {
    Inline {
        args: [MaybeUninit<NodeId>; INLINE_SPINE],
        len: usize,
    },
    Heap {
        args: Vec<NodeId>,
    },
}

impl Default for EvalSpine {
    fn default() -> Self {
        Self {
            inline_args: [const { MaybeUninit::uninit() }; INLINE_SPINE],
            inline_apps: [const { MaybeUninit::uninit() }; INLINE_SPINE],
            inline_len: 0,
            heap_args: Vec::new(),
            heap_apps: Vec::new(),
            heap: false,
        }
    }
}

impl EvalSpine {
    pub(in crate::runtime) fn clear(&mut self) {
        self.inline_len = 0;
        self.heap = false;
        self.heap_args.clear();
        self.heap_apps.clear();
    }

    pub(in crate::runtime) fn len(&self) -> usize {
        if self.heap {
            self.heap_args.len()
        } else {
            self.inline_len
        }
    }

    pub(in crate::runtime) fn push_desc(&mut self, arg: NodeId, app: NodeId) {
        if self.heap {
            self.heap_args.push(arg);
            self.heap_apps.push(app);
        } else if self.inline_len < INLINE_SPINE {
            self.inline_args[self.inline_len].write(arg);
            self.inline_apps[self.inline_len].write(app);
            self.inline_len += 1;
        } else {
            self.heap = true;
            self.heap_args.reserve(INLINE_SPINE * 2);
            self.heap_apps.reserve(INLINE_SPINE * 2);
            for idx in 0..self.inline_len {
                // SAFETY: indices below inline_len were written before heap promotion.
                self.heap_args
                    .push(unsafe { self.inline_args[idx].assume_init() });
                // SAFETY: indices below inline_len were written before heap promotion.
                self.heap_apps
                    .push(unsafe { self.inline_apps[idx].assume_init() });
            }
            self.heap_args.push(arg);
            self.heap_apps.push(app);
        }
    }

    pub(in crate::runtime) fn desc_arg(&self, desc_idx: usize) -> NodeId {
        if self.heap {
            self.heap_args[desc_idx]
        } else {
            debug_assert!(desc_idx < self.inline_len);
            // SAFETY: desc_idx is below inline_len, so the slot was initialized.
            unsafe { self.inline_args[desc_idx].assume_init() }
        }
    }

    pub(in crate::runtime) fn desc_app(&self, desc_idx: usize) -> NodeId {
        if self.heap {
            self.heap_apps[desc_idx]
        } else {
            debug_assert!(desc_idx < self.inline_len);
            // SAFETY: desc_idx is below inline_len, so the slot was initialized.
            unsafe { self.inline_apps[desc_idx].assume_init() }
        }
    }

    pub(in crate::runtime) fn arg(&self, head_idx: usize) -> NodeId {
        let len = self.len();
        debug_assert!(head_idx < len);
        self.desc_arg(len - head_idx - 1)
    }

    pub(in crate::runtime) fn app(&self, head_idx: usize) -> NodeId {
        let len = self.len();
        debug_assert!(head_idx < len);
        self.desc_app(len - head_idx - 1)
    }

    pub(in crate::runtime) fn write_args_head_order(&self, args: &mut Vec<NodeId>) {
        args.clear();
        let len = self.len();
        args.reserve(len);
        for desc_idx in (0..len).rev() {
            args.push(self.desc_arg(desc_idx));
        }
    }

    pub(in crate::runtime) fn write_args_head_order_prefix(
        &self,
        args: &mut Vec<NodeId>,
        limit: usize,
    ) {
        args.clear();
        let len = self.len().min(limit);
        args.reserve(len);
        for head_idx in 0..len {
            args.push(self.arg(head_idx));
        }
    }
}

impl Spine {
    pub(in crate::runtime) fn args(&self) -> &[NodeId] {
        match &self.storage {
            SpineStorage::Inline { args, len } => initialized_node_slice(args, *len),
            SpineStorage::Heap { args } => args,
        }
    }
}

pub(in crate::runtime) fn initialized_node_slice(
    storage: &[MaybeUninit<NodeId>],
    len: usize,
) -> &[NodeId] {
    debug_assert!(len <= storage.len());
    // SAFETY: Spine::spine writes exactly the first `len` elements before storing
    // an Inline spine, and NodeId is Copy with no drop glue.
    unsafe { std::slice::from_raw_parts(storage.as_ptr().cast::<NodeId>(), len) }
}
