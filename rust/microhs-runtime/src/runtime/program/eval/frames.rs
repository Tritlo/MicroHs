//! WHNF driver and strict-frame forcing machinery.
use super::*;

impl Program {
    #[inline]
    pub(in crate::runtime) fn ready_conversion_value(
        &self,
        kind: ConversionFrameKind,
        current: NodeId,
    ) -> Option<ConversionValue> {
        match kind {
            ConversionFrameKind::IntToInt64
            | ConversionFrameKind::IntToFloat64 { .. }
            | ConversionFrameKind::IntToFloat32 { .. }
            | ConversionFrameKind::IntBitsToFloat32 => {
                self.cell_int_value(current).map(ConversionValue::Int)
            }
            ConversionFrameKind::Int64ToInt
            | ConversionFrameKind::Int64ToFloat64
            | ConversionFrameKind::Int64ToFloat32
            | ConversionFrameKind::Int64BitsToFloat64 => {
                self.cell_int64_value(current).map(ConversionValue::Int64)
            }
            ConversionFrameKind::Float64ToInt
            | ConversionFrameKind::Float64ToFloat32
            | ConversionFrameKind::Float64BitsToInt64 => self
                .cell_float64_value(current)
                .map(ConversionValue::Float64),
            ConversionFrameKind::Float32ToInt
            | ConversionFrameKind::Float32ToFloat64
            | ConversionFrameKind::Float32BitsToInt => self
                .cell_float32_value(current)
                .map(ConversionValue::Float32),
        }
    }

    pub(in crate::runtime) fn finish_int_frame(
        &mut self,
        frame: IntFrame,
        value: i64,
        stack: &mut EvalFrameStack,
    ) -> Result<(NodeId, usize), EvalError> {
        let result = match frame.kind {
            IntFrameKind::BinSecond { op, x } => {
                let next = x;
                let next_frame = IntFrame {
                    redex: frame.redex,
                    profile_head: frame.profile_head,
                    kind: IntFrameKind::BinFirst { op, y: value },
                };
                if self.profile.is_some() {
                    self.profile_eval_frame_push("Int");
                }
                stack.push(EvalFrame::Int(next_frame));
                return Ok((next, 0));
            }
            IntFrameKind::BinFirst { op, y } => op
                .apply(value, y)
                .map_err(|err| self.arithmetic_eval_error(err))?,
            IntFrameKind::Un { op } => {
                let n = op
                    .apply(value)
                    .map_err(|err| self.arithmetic_eval_error(err))?;
                IntResult::Int(n)
            }
        };

        if frame.profile_head.is_some() {
            self.profile_reduction(frame.profile_head, 1);
        }
        let result_node = self.int_result_node(result);
        let node = self.apply_strict_redex(frame.redex, result_node)?;
        Ok((node, 1))
    }

    pub(in crate::runtime) fn int64_result_node(&mut self, result: Int64Result) -> NodeId {
        match result {
            Int64Result::Int64(n) => self.push_node(Node::Int64(n)),
            Int64Result::Bool(b) => self.prim(if b { "A" } else { "K" }),
            Int64Result::Ordering(ord) => self.ordering(ord),
        }
    }

    pub(in crate::runtime) fn int64_result_value_node(result: Int64Result) -> Node {
        match result {
            Int64Result::Int64(n) => Node::Int64(n),
            Int64Result::Bool(b) => Self::bool_value_node(b),
            Int64Result::Ordering(ord) => Self::ordering_value_node(ord),
        }
    }

    pub(in crate::runtime) fn int64_un_result_node(&mut self, result: Int64UnResult) -> NodeId {
        match result {
            Int64UnResult::Int64(n) => self.push_node(Node::Int64(n)),
            Int64UnResult::Int(n) => self.int(n),
        }
    }

    pub(in crate::runtime) fn finish_int64_frame(
        &mut self,
        frame: Int64Frame,
        value: i64,
        stack: &mut EvalFrameStack,
    ) -> Result<(NodeId, usize), EvalError> {
        let node = match frame.kind {
            Int64FrameKind::BinSecond { op, x } => {
                let next = x;
                let next_frame = Int64Frame {
                    redex: frame.redex,
                    profile_head: frame.profile_head,
                    kind: Int64FrameKind::BinFirst { op, y: value },
                };
                if self.profile.is_some() {
                    self.profile_eval_frame_push("Int64");
                }
                stack.push(EvalFrame::Int64(next_frame));
                return Ok((next, 0));
            }
            Int64FrameKind::BinFirst { op, y } => {
                let result = op
                    .apply(value, y)
                    .map_err(|err| self.arithmetic_eval_error(err))?;
                self.int64_result_node(result)
            }
            Int64FrameKind::ShiftFirst { op, y } => {
                let result = op
                    .apply(value, y)
                    .map_err(|err| self.arithmetic_eval_error(err))?;
                self.int64_result_node(result)
            }
            Int64FrameKind::Un { op } => {
                let result = op
                    .apply(value)
                    .map_err(|err| self.arithmetic_eval_error(err))?;
                self.int64_un_result_node(result)
            }
        };

        if frame.profile_head.is_some() {
            self.profile_reduction(frame.profile_head, 1);
        }
        let node = self.apply_strict_redex(frame.redex, node)?;
        Ok((node, 1))
    }

    pub(in crate::runtime) fn apply_strict_redex(
        &mut self,
        redex: StrictRedex,
        mut node: NodeId,
    ) -> Result<NodeId, EvalError> {
        match redex {
            StrictRedex::Root(root) => {
                if node != root {
                    self.set_cell_at(root.index(), Cell::indir(Some(node)));
                }
            }
            StrictRedex::Spine { root, used, apps } => {
                let in_place = self.apply_reduction_spine(&mut node, used, &apps)?;
                if !in_place && node != root {
                    self.set_cell_at(root.index(), Cell::indir(Some(node)));
                }
            }
        }
        Ok(node)
    }

    pub(in crate::runtime) fn stack_entry_app(
        &self,
        stack: &EvalStack,
        index: usize,
    ) -> Result<NodeId, EvalError> {
        stack
            .apps
            .get(index)
            .copied()
            .ok_or_else(|| EvalError::DanglingIndirection(NodeId::from_index(index)))
    }

    pub(in crate::runtime) fn apply_stack_rewrite(
        &mut self,
        stack: &mut EvalStack,
        used: usize,
        node: NodeId,
    ) -> NodeId {
        if used == 0 {
            return node;
        }
        let app_end = stack.apps.len();
        self.apply_stack_frame_rewrite(stack, app_end, used, node)
    }

    pub(in crate::runtime) fn apply_stack_frame_rewrite(
        &mut self,
        stack: &mut EvalStack,
        app_end: usize,
        used: usize,
        node: NodeId,
    ) -> NodeId {
        #[cfg(feature = "eval-phase-profile")]
        let started = self.profile.is_some().then(Instant::now);
        debug_assert!(used > 0);
        debug_assert!(used <= app_end);
        let redex_index = app_end - used;
        let redex = stack.app_unchecked(redex_index);
        let wrote_indirection = node != redex;
        if wrote_indirection {
            self.set_app_cell_at(redex.index(), Cell::indir(Some(node)));
        }
        if self.profile.is_some() {
            self.profile_stack_rewrite(used, wrote_indirection);
        }
        stack.apps.truncate(redex_index);
        #[cfg(feature = "eval-phase-profile")]
        if let Some(started) = started {
            self.profile_stack_apply_rewrite_time(started.elapsed().as_nanos());
        }
        node
    }

    pub(in crate::runtime) fn apply_stack_frame_value(
        &mut self,
        stack: &mut EvalStack,
        app_end: usize,
        used: usize,
        value: Node,
    ) -> NodeId {
        #[cfg(feature = "eval-phase-profile")]
        let started = self.profile.is_some().then(Instant::now);
        debug_assert!(used > 0);
        debug_assert!(used <= app_end);
        let redex_index = app_end - used;
        let redex = stack.app_unchecked(redex_index);
        self.set_app_node_at(redex.index(), value);
        if self.profile.is_some() {
            self.profile_stack_rewrite(used, false);
        }
        stack.apps.truncate(redex_index);
        #[cfg(feature = "eval-phase-profile")]
        if let Some(started) = started {
            self.profile_stack_apply_rewrite_time(started.elapsed().as_nanos());
        }
        redex
    }

    pub(in crate::runtime) fn apply_stack_redex_value(
        &mut self,
        redex: NodeId,
        used: usize,
        value: Node,
    ) -> NodeId {
        #[cfg(feature = "eval-phase-profile")]
        let started = self.profile.is_some().then(Instant::now);
        self.set_app_node_at(redex.index(), value);
        if self.profile.is_some() {
            self.profile_stack_rewrite(used, false);
        }
        #[cfg(feature = "eval-phase-profile")]
        if let Some(started) = started {
            self.profile_stack_apply_rewrite_time(started.elapsed().as_nanos());
        }
        redex
    }

    pub(in crate::runtime) fn apply_stack_app(
        &mut self,
        stack: &mut EvalStack,
        used: usize,
        fun: NodeId,
        arg: NodeId,
    ) -> NodeId {
        #[cfg(feature = "eval-phase-profile")]
        let started = self.profile.is_some().then(Instant::now);
        if self.profile.is_some() {
            self.profile_stack_app_update(used);
        }
        let node = if used == 0 {
            self.app(fun, arg)
        } else {
            let app_end = stack.apps.len();
            debug_assert!(used <= app_end);
            let redex_index = app_end - used;
            let redex = stack.app_unchecked(redex_index);
            self.set_app_cell_at(redex.index(), Cell::app(fun, arg));
            stack.apps.truncate(redex_index);
            redex
        };
        #[cfg(feature = "eval-phase-profile")]
        if let Some(started) = started {
            self.profile_stack_apply_app_time(started.elapsed().as_nanos());
        }
        node
    }

    pub(in crate::runtime) fn rethread_stack_app_segment(
        &mut self,
        stack: &mut EvalStack,
        mut node: NodeId,
    ) -> Result<NodeId, EvalError> {
        let base = stack.app_base();
        if self.profile.is_some() {
            self.profile_stack_rethread(stack.apps.len().saturating_sub(base));
        }
        for index in (base..stack.apps.len()).rev() {
            let app = self.stack_entry_app(stack, index)?;
            let Some((_, arg)) = self.cell(app).app_fields() else {
                return Err(EvalError::DanglingIndirection(app));
            };
            self.set_app_cell_at(app.index(), Cell::app(node, arg));
            node = app;
        }
        stack.apps.truncate(base);
        Ok(node)
    }

    pub(in crate::runtime) fn descend_stack_from(
        &mut self,
        mut current: NodeId,
        stack: &mut EvalStack,
        profile_resolve: bool,
        profiling: bool,
    ) -> Result<NodeId, EvalError> {
        current = self.resolve_for_whnf(current, profile_resolve)?;
        while let Some(fun) = self.app_fun_trusted(current) {
            if profiling {
                self.profile_stack_descent_push();
            }
            stack.push_app(current);
            current = self.resolve_for_whnf(fun, profile_resolve)?;
        }
        Ok(current)
    }

    pub(in crate::runtime) fn finish_stack_whnf_frame(
        &mut self,
        frame: StackWhnfFrame,
        _stack: &mut EvalStack,
        value: NodeId,
    ) -> Result<(NodeId, usize), EvalError> {
        let node = match frame.kind {
            WhnfFrameKind::Seq { result } => {
                if frame.profile_head.is_some() {
                    self.profile_reduction(frame.profile_head, 1);
                }
                #[cfg(feature = "eval-phase-profile")]
                let started = self.profile.is_some().then(Instant::now);
                let wrote_indirection = result != frame.redex;
                if wrote_indirection {
                    self.set_app_cell_at(frame.redex.index(), Cell::indir(Some(result)));
                }
                if self.profile.is_some() {
                    self.profile_stack_rewrite(frame.used, wrote_indirection);
                }
                #[cfg(feature = "eval-phase-profile")]
                if let Some(started) = started {
                    self.profile_stack_apply_rewrite_time(started.elapsed().as_nanos());
                }
                return Ok((result, 1));
            }
            WhnfFrameKind::IoStrict { action, value } => {
                if frame.profile_head.is_some() {
                    self.profile_reduction(frame.profile_head, 1);
                }
                #[cfg(feature = "eval-phase-profile")]
                let started = self.profile.is_some().then(Instant::now);
                if self.profile.is_some() {
                    self.profile_stack_app_update(frame.used);
                }
                self.set_app_cell_at(frame.redex.index(), Cell::app(action, value));
                #[cfg(feature = "eval-phase-profile")]
                if let Some(started) = started {
                    self.profile_stack_apply_app_time(started.elapsed().as_nanos());
                }
                return Ok((frame.redex, 1));
            }
            WhnfFrameKind::IsInt => {
                let value = self.resolve(value)?;
                let n = self.cell_int_value(value).unwrap_or(-1);
                self.apply_stack_redex_value(frame.redex, frame.used, Node::Int(n))
            }
        };

        if frame.profile_head.is_some() {
            self.profile_reduction(frame.profile_head, 1);
        }
        Ok((node, 1))
    }

    pub(in crate::runtime) fn finish_ready_stack_frame(
        &mut self,
        stack: &mut EvalStack,
        current: NodeId,
    ) -> Result<Option<(NodeId, usize)>, EvalError> {
        if !stack.top_is_frame() {
            return Ok(None);
        }

        enum ReadyFrame {
            Int(i64),
            Int64Shift(i64),
            Int64(i64),
            Float64(f64),
            Float32(f32),
            Bytes,
            Conversion(ConversionValue),
        }

        let ready = match stack.peek_frame() {
            Some(StackFrame::Int(_)) => self.cell_int_value(current).map(ReadyFrame::Int),
            Some(StackFrame::Int64Shift(_)) => {
                self.cell_int_value(current).map(ReadyFrame::Int64Shift)
            }
            Some(StackFrame::Int64(_)) => self.cell_int64_value(current).map(ReadyFrame::Int64),
            Some(StackFrame::Float64(_)) => {
                self.cell_float64_value(current).map(ReadyFrame::Float64)
            }
            Some(StackFrame::Float32(_)) => {
                self.cell_float32_value(current).map(ReadyFrame::Float32)
            }
            Some(StackFrame::Bytes(_))
                if matches!(
                    self.cold_node(current),
                    Some(Node::Bytes(_) | Node::MutableBytes(_))
                ) =>
            {
                Some(ReadyFrame::Bytes)
            }
            Some(StackFrame::Conversion(frame)) => self
                .ready_conversion_value(frame.kind, current)
                .map(ReadyFrame::Conversion),
            _ => None,
        };
        let Some(ready) = ready else {
            return Ok(None);
        };

        let frame = stack.pop_frame().expect("ready stack frame must exist");
        let result = match (frame, ready) {
            (StackFrame::Int(frame), ReadyFrame::Int(value)) => {
                let used = match &frame.kind {
                    IntFrameKind::Un { .. } => 1,
                    IntFrameKind::BinSecond { .. } | IntFrameKind::BinFirst { .. } => 2,
                };
                let result = match frame.kind {
                    IntFrameKind::BinSecond { op, x } => {
                        stack.push_int_frame(
                            frame.redex,
                            frame.profile_head,
                            IntFrameKind::BinFirst { op, y: value },
                        );
                        if self.profile.is_some() {
                            self.profile_eval_frame_push("Int");
                        }
                        return Ok(Some((x, 0)));
                    }
                    IntFrameKind::BinFirst { op, y } => op
                        .apply(value, y)
                        .map_err(|err| self.arithmetic_eval_error(err))?,
                    IntFrameKind::Un { op } => {
                        let n = op
                            .apply(value)
                            .map_err(|err| self.arithmetic_eval_error(err))?;
                        IntResult::Int(n)
                    }
                };
                if frame.profile_head.is_some() {
                    self.profile_reduction(frame.profile_head, 1);
                }
                let node = self.apply_stack_redex_value(
                    frame.redex,
                    used,
                    Self::int_result_value_node(result),
                );
                (node, 1)
            }
            (StackFrame::Int64(frame), ReadyFrame::Int64(value)) => {
                let result = match frame.kind {
                    Int64FrameKind::BinSecond { op, x } => {
                        stack.push_int64_frame(
                            frame.app_end,
                            frame.used,
                            frame.profile_head,
                            Int64FrameKind::BinFirst { op, y: value },
                        );
                        if self.profile.is_some() {
                            self.profile_eval_frame_push("Int64");
                        }
                        return Ok(Some((x, 0)));
                    }
                    Int64FrameKind::BinFirst { op, y } => op
                        .apply(value, y)
                        .map_err(|err| self.arithmetic_eval_error(err))?,
                    Int64FrameKind::ShiftFirst { op, y } => op
                        .apply(value, y)
                        .map_err(|err| self.arithmetic_eval_error(err))?,
                    Int64FrameKind::Un { op } => {
                        let result = op
                            .apply(value)
                            .map_err(|err| self.arithmetic_eval_error(err))?;
                        match result {
                            Int64UnResult::Int64(n) => Int64Result::Int64(n),
                            Int64UnResult::Int(n) => {
                                if frame.profile_head.is_some() {
                                    self.profile_reduction(frame.profile_head, 1);
                                }
                                let node = self.apply_stack_frame_value(
                                    stack,
                                    frame.app_end,
                                    frame.used,
                                    Node::Int(n),
                                );
                                return Ok(Some((node, 1)));
                            }
                        }
                    }
                };
                if frame.profile_head.is_some() {
                    self.profile_reduction(frame.profile_head, 1);
                }
                let node = self.apply_stack_frame_value(
                    stack,
                    frame.app_end,
                    frame.used,
                    Self::int64_result_value_node(result),
                );
                (node, 1)
            }
            (StackFrame::Int64Shift(frame), ReadyFrame::Int64Shift(value)) => {
                stack.push_int64_frame(
                    frame.app_end,
                    frame.used,
                    frame.profile_head,
                    Int64FrameKind::ShiftFirst {
                        op: frame.op,
                        y: value,
                    },
                );
                if self.profile.is_some() {
                    self.profile_eval_frame_push("Int64");
                }
                (frame.x, 0)
            }
            (StackFrame::Float64(frame), ReadyFrame::Float64(value)) => {
                let result = match frame.kind {
                    Float64FrameKind::BinSecond { op, x } => {
                        stack.push_float64_frame(
                            frame.app_end,
                            frame.used,
                            frame.profile_head,
                            Float64FrameKind::BinFirst { op, y: value },
                        );
                        if self.profile.is_some() {
                            self.profile_eval_frame_push("Float64");
                        }
                        return Ok(Some((x, 0)));
                    }
                    Float64FrameKind::BinFirst { op, y } => op.apply(value, y),
                    Float64FrameKind::Un { op } => Float64Result::Float(op.apply(value)),
                };
                if frame.profile_head.is_some() {
                    self.profile_reduction(frame.profile_head, 1);
                }
                let value = match result {
                    Float64Result::Float(n) => Node::Float64(n),
                    Float64Result::Bool(b) => Self::bool_value_node(b),
                };
                let node = self.apply_stack_frame_value(stack, frame.app_end, frame.used, value);
                (node, 1)
            }
            (StackFrame::Float32(frame), ReadyFrame::Float32(value)) => {
                let result = match frame.kind {
                    Float32FrameKind::BinSecond { op, x } => {
                        stack.push_float32_frame(
                            frame.app_end,
                            frame.used,
                            frame.profile_head,
                            Float32FrameKind::BinFirst { op, y: value },
                        );
                        if self.profile.is_some() {
                            self.profile_eval_frame_push("Float32");
                        }
                        return Ok(Some((x, 0)));
                    }
                    Float32FrameKind::BinFirst { op, y } => op.apply(value, y),
                    Float32FrameKind::Un { op } => Float32Result::Float(op.apply(value)),
                };
                if frame.profile_head.is_some() {
                    self.profile_reduction(frame.profile_head, 1);
                }
                let value = match result {
                    Float32Result::Float(n) => Node::Float32(n),
                    Float32Result::Bool(b) => Self::bool_value_node(b),
                };
                let node = self.apply_stack_frame_value(stack, frame.app_end, frame.used, value);
                (node, 1)
            }
            (StackFrame::Bytes(frame), ReadyFrame::Bytes) => {
                let node = match frame.kind {
                    BytesFrameKind::BinSecond { op, x } => {
                        stack.push_bytes_frame(
                            frame.app_end,
                            frame.used,
                            frame.profile_head,
                            BytesFrameKind::BinFirst { op, y: current },
                        );
                        if self.profile.is_some() {
                            self.profile_eval_frame_push("Bytes");
                        }
                        return Ok(Some((x, 0)));
                    }
                    BytesFrameKind::BinFirst { op, y } => {
                        self.bytes_bin_result_node(op, current, y)?
                    }
                };
                if frame.profile_head.is_some() {
                    self.profile_reduction(frame.profile_head, 1);
                }
                let node = self.apply_stack_frame_rewrite(stack, frame.app_end, frame.used, node);
                (node, 1)
            }
            (StackFrame::Conversion(frame), ReadyFrame::Conversion(value)) => {
                if frame.profile_head.is_some() {
                    self.profile_reduction(frame.profile_head, 1);
                }
                let node = self.apply_stack_frame_value(
                    stack,
                    frame.app_end,
                    frame.used,
                    Self::conversion_result_value_node(frame.kind, value),
                );
                (node, 1)
            }
            _ => unreachable!("ready stack frame kind changed before pop"),
        };
        Ok(Some(result))
    }

    pub(in crate::runtime) fn finish_whnf_stack_frame(
        &mut self,
        stack: &mut EvalStack,
        current: NodeId,
    ) -> Result<Option<(NodeId, usize)>, EvalError> {
        let Some(frame) = stack.pop_frame() else {
            return Ok(None);
        };
        let result = match frame {
            StackFrame::Whnf(frame) => self.finish_stack_whnf_frame(frame, stack, current)?,
            StackFrame::Int(_) => return Err(EvalError::ExpectedInt(current)),
            StackFrame::Int64Shift(_) => return Err(EvalError::ExpectedInt(current)),
            StackFrame::Int64(_) => return Err(EvalError::ExpectedInt64(current)),
            StackFrame::Float64(_) => return Err(EvalError::ExpectedFloat64(current)),
            StackFrame::Float32(_) => return Err(EvalError::ExpectedFloat32(current)),
            StackFrame::Bytes(_) => return Err(self.expected_bytes_error(current)),
            StackFrame::Conversion(frame) => return Err(frame.kind.expected_error(current)),
        };
        Ok(Some(result))
    }

    pub(in crate::runtime) fn begin_whnf_force_frame(
        &mut self,
        root: NodeId,
    ) -> Result<Option<(WhnfFrame, NodeId)>, EvalError> {
        let spine = self.spine(root)?;
        let head = spine.head;
        let args = spine.args();
        let Some(Prim::Known(known)) = self.cell(head).prim() else {
            return Ok(None);
        };
        use KnownPrim::*;
        let force = match known {
            Seq if args.len() >= 2 => Some((2, WhnfFrameKind::Seq { result: args[1] }, args[0])),
            IoStrict if args.len() >= 2 => Some((
                2,
                WhnfFrameKind::IoStrict {
                    action: args[0],
                    value: args[1],
                },
                args[1],
            )),
            IsInt if !args.is_empty() => Some((1, WhnfFrameKind::IsInt, args[0])),
            _ => None,
        };
        let Some((used, kind, next)) = force else {
            return Ok(None);
        };

        let profile_head = if self.profile.is_some() {
            self.profile_step(head, args.len())
        } else {
            None
        };
        let redex = if args.len() == used {
            StrictRedex::Root(root)
        } else {
            StrictRedex::Spine {
                root,
                used,
                apps: spine.apps().to_vec(),
            }
        };
        Ok(Some((
            WhnfFrame {
                redex,
                profile_head,
                kind,
            },
            next,
        )))
    }

    pub(in crate::runtime) fn finish_whnf_frame(
        &mut self,
        frame: WhnfFrame,
        value: NodeId,
    ) -> Result<(NodeId, usize), EvalError> {
        let node = match frame.kind {
            WhnfFrameKind::Seq { result } => result,
            WhnfFrameKind::IoStrict { action, value } => self.app(action, value),
            WhnfFrameKind::IsInt => {
                let value = self.resolve(value)?;
                let n = self.cell_int_value(value).unwrap_or(-1);
                self.int(n)
            }
        };

        if frame.profile_head.is_some() {
            self.profile_reduction(frame.profile_head, 1);
        }
        let node = self.apply_strict_redex(frame.redex, node)?;
        Ok((node, 1))
    }

    pub(in crate::runtime) fn conversion_result_node(
        &mut self,
        kind: ConversionFrameKind,
        value: ConversionValue,
    ) -> NodeId {
        match (kind, value) {
            (ConversionFrameKind::IntToInt64, ConversionValue::Int(n)) => {
                self.push_node(Node::Int64(n))
            }
            (ConversionFrameKind::Int64ToInt, ConversionValue::Int64(n)) => self.int(n),
            (ConversionFrameKind::IntToFloat64 { unsigned: false }, ConversionValue::Int(n)) => {
                self.push_node(Node::Float64(n as f64))
            }
            (ConversionFrameKind::IntToFloat64 { unsigned: true }, ConversionValue::Int(n)) => {
                self.push_node(Node::Float64((n as u64) as f64))
            }
            (ConversionFrameKind::Int64ToFloat64, ConversionValue::Int64(n)) => {
                self.push_node(Node::Float64(n as f64))
            }
            (ConversionFrameKind::Float64ToInt, ConversionValue::Float64(n)) => self.int(n as i64),
            (ConversionFrameKind::IntToFloat32 { unsigned: false }, ConversionValue::Int(n)) => {
                self.push_node(Node::Float32(n as f32))
            }
            (ConversionFrameKind::IntToFloat32 { unsigned: true }, ConversionValue::Int(n)) => {
                self.push_node(Node::Float32((n as u64) as f32))
            }
            (ConversionFrameKind::Int64ToFloat32, ConversionValue::Int64(n)) => {
                self.push_node(Node::Float32(n as f32))
            }
            (ConversionFrameKind::Float32ToInt, ConversionValue::Float32(n)) => self.int(n as i64),
            (ConversionFrameKind::Float64ToFloat32, ConversionValue::Float64(n)) => {
                self.push_node(Node::Float32(n as f32))
            }
            (ConversionFrameKind::Float32ToFloat64, ConversionValue::Float32(n)) => {
                self.push_node(Node::Float64(n as f64))
            }
            (ConversionFrameKind::Int64BitsToFloat64, ConversionValue::Int64(n)) => {
                self.push_node(Node::Float64(f64::from_bits(n as u64)))
            }
            (ConversionFrameKind::Float64BitsToInt64, ConversionValue::Float64(n)) => {
                self.push_node(Node::Int64(n.to_bits() as i64))
            }
            (ConversionFrameKind::IntBitsToFloat32, ConversionValue::Int(n)) => {
                self.push_node(Node::Float32(f32::from_bits(n as u32)))
            }
            (ConversionFrameKind::Float32BitsToInt, ConversionValue::Float32(n)) => {
                self.int((n.to_bits() as i32) as i64)
            }
            _ => unreachable!("conversion frame kind and value mismatch"),
        }
    }

    pub(in crate::runtime) fn conversion_result_value_node(
        kind: ConversionFrameKind,
        value: ConversionValue,
    ) -> Node {
        match (kind, value) {
            (ConversionFrameKind::IntToInt64, ConversionValue::Int(n)) => Node::Int64(n),
            (ConversionFrameKind::Int64ToInt, ConversionValue::Int64(n)) => Node::Int(n),
            (ConversionFrameKind::IntToFloat64 { unsigned: false }, ConversionValue::Int(n)) => {
                Node::Float64(n as f64)
            }
            (ConversionFrameKind::IntToFloat64 { unsigned: true }, ConversionValue::Int(n)) => {
                Node::Float64((n as u64) as f64)
            }
            (ConversionFrameKind::Int64ToFloat64, ConversionValue::Int64(n)) => {
                Node::Float64(n as f64)
            }
            (ConversionFrameKind::Float64ToInt, ConversionValue::Float64(n)) => Node::Int(n as i64),
            (ConversionFrameKind::IntToFloat32 { unsigned: false }, ConversionValue::Int(n)) => {
                Node::Float32(n as f32)
            }
            (ConversionFrameKind::IntToFloat32 { unsigned: true }, ConversionValue::Int(n)) => {
                Node::Float32((n as u64) as f32)
            }
            (ConversionFrameKind::Int64ToFloat32, ConversionValue::Int64(n)) => {
                Node::Float32(n as f32)
            }
            (ConversionFrameKind::Float32ToInt, ConversionValue::Float32(n)) => Node::Int(n as i64),
            (ConversionFrameKind::Float64ToFloat32, ConversionValue::Float64(n)) => {
                Node::Float32(n as f32)
            }
            (ConversionFrameKind::Float32ToFloat64, ConversionValue::Float32(n)) => {
                Node::Float64(n as f64)
            }
            (ConversionFrameKind::Int64BitsToFloat64, ConversionValue::Int64(n)) => {
                Node::Float64(f64::from_bits(n as u64))
            }
            (ConversionFrameKind::Float64BitsToInt64, ConversionValue::Float64(n)) => {
                Node::Int64(n.to_bits() as i64)
            }
            (ConversionFrameKind::IntBitsToFloat32, ConversionValue::Int(n)) => {
                Node::Float32(f32::from_bits(n as u32))
            }
            (ConversionFrameKind::Float32BitsToInt, ConversionValue::Float32(n)) => {
                Node::Int((n.to_bits() as i32) as i64)
            }
            _ => unreachable!("conversion frame kind and value mismatch"),
        }
    }

    pub(in crate::runtime) fn finish_conversion_frame(
        &mut self,
        frame: ConversionFrame,
        value: ConversionValue,
    ) -> Result<(NodeId, usize), EvalError> {
        let node = self.conversion_result_node(frame.kind, value);

        if frame.profile_head.is_some() {
            self.profile_reduction(frame.profile_head, 1);
        }
        let node = self.apply_strict_redex(frame.redex, node)?;
        Ok((node, 1))
    }

    pub(in crate::runtime) fn finish_ready_eval_frame(
        &mut self,
        stack: &mut EvalFrameStack,
        current: NodeId,
    ) -> Result<Option<(NodeId, usize)>, EvalError> {
        enum ReadyFrame {
            Int(i64),
            Int64Shift(i64),
            Int64(i64),
            Float64(f64),
            Float32(f32),
            Bytes,
            Conversion(ConversionValue),
        }

        let ready = match stack.peek() {
            Some(EvalFrame::Int(_)) => self.cell_int_value(current).map(ReadyFrame::Int),
            Some(EvalFrame::Int64Shift(_)) => {
                self.cell_int_value(current).map(ReadyFrame::Int64Shift)
            }
            Some(EvalFrame::Int64(_)) => self.cell_int64_value(current).map(ReadyFrame::Int64),
            Some(EvalFrame::Float64(_)) => {
                self.cell_float64_value(current).map(ReadyFrame::Float64)
            }
            Some(EvalFrame::Float32(_)) => {
                self.cell_float32_value(current).map(ReadyFrame::Float32)
            }
            Some(EvalFrame::Bytes(_))
                if matches!(
                    self.cold_node(current),
                    Some(Node::Bytes(_) | Node::MutableBytes(_))
                ) =>
            {
                Some(ReadyFrame::Bytes)
            }
            Some(EvalFrame::Conversion(frame)) => self
                .ready_conversion_value(frame.kind, current)
                .map(ReadyFrame::Conversion),
            _ => None,
        };
        let Some(ready) = ready else {
            return Ok(None);
        };

        let frame = stack.pop().expect("ready eval frame must have a frame");
        let result = match (frame, ready) {
            (EvalFrame::Int(frame), ReadyFrame::Int(value)) => {
                self.finish_int_frame(frame, value, stack)?
            }
            (EvalFrame::Int64(frame), ReadyFrame::Int64(value)) => {
                self.finish_int64_frame(frame, value, stack)?
            }
            (EvalFrame::Int64Shift(frame), ReadyFrame::Int64Shift(value)) => {
                let next = frame.x;
                stack.push(EvalFrame::Int64(Int64Frame {
                    redex: frame.redex,
                    profile_head: frame.profile_head,
                    kind: Int64FrameKind::ShiftFirst {
                        op: frame.op,
                        y: value,
                    },
                }));
                if self.profile.is_some() {
                    self.profile_eval_frame_push("Int64");
                }
                (next, 0)
            }
            (EvalFrame::Float64(frame), ReadyFrame::Float64(value)) => {
                self.finish_float64_frame(frame, value, stack)?
            }
            (EvalFrame::Float32(frame), ReadyFrame::Float32(value)) => {
                self.finish_float32_frame(frame, value, stack)?
            }
            (EvalFrame::Bytes(frame), ReadyFrame::Bytes) => {
                self.finish_bytes_frame(frame, current, stack)?
            }
            (EvalFrame::Conversion(frame), ReadyFrame::Conversion(value)) => {
                self.finish_conversion_frame(frame, value)?
            }
            _ => unreachable!("ready eval frame kind changed before pop"),
        };
        Ok(Some(result))
    }

    pub(in crate::runtime) fn finish_whnf_eval_frame(
        &mut self,
        stack: &mut EvalFrameStack,
        current: NodeId,
    ) -> Result<Option<(NodeId, usize)>, EvalError> {
        let Some(frame) = stack.pop() else {
            return Ok(None);
        };
        let result = match frame {
            EvalFrame::Whnf(frame) => self.finish_whnf_frame(frame, current)?,
            EvalFrame::Int(_) => return Err(EvalError::ExpectedInt(current)),
            EvalFrame::Int64Shift(_) => return Err(EvalError::ExpectedInt(current)),
            EvalFrame::Int64(_) => return Err(EvalError::ExpectedInt64(current)),
            EvalFrame::Float64(_) => return Err(EvalError::ExpectedFloat64(current)),
            EvalFrame::Float32(_) => return Err(EvalError::ExpectedFloat32(current)),
            EvalFrame::Bytes(_) => return Err(self.expected_bytes_error(current)),
            EvalFrame::Conversion(frame) => return Err(frame.kind.expected_error(current)),
        };
        Ok(Some(result))
    }

    pub(in crate::runtime) fn resolve_for_whnf(
        &mut self,
        root: NodeId,
        profile_resolve: bool,
    ) -> Result<NodeId, EvalError> {
        if profile_resolve {
            self.resolve_profiled(root)
        } else {
            Ok(self.resolve_whnf_trusted(root))
        }
    }

    pub(in crate::runtime) fn reduce_whnf_from(
        &mut self,
        root: NodeId,
        limit: usize,
        profile_resolve: bool,
        whnf_frames: bool,
    ) -> Result<(NodeId, usize), EvalError> {
        self.reduce_depth += 1;
        let result = self.reduce_whnf_from_inner(root, limit, profile_resolve, whnf_frames);
        self.reduce_depth -= 1;
        result
    }

    pub(in crate::runtime) fn reduce_whnf_from_inner(
        &mut self,
        mut root: NodeId,
        limit: usize,
        profile_resolve: bool,
        whnf_frames: bool,
    ) -> Result<(NodeId, usize), EvalError> {
        if !whnf_frames {
            return self.reduce_whnf_from_stack(root, limit, profile_resolve);
        }

        let mut steps = 0;
        let mut frame_stack = EvalFrameStack::default();
        let mut eval_spine = EvalSpine::default();
        let mut persistent_spine = PersistentSpine::default();
        let mut persistent_active = false;
        let mut scratch_args = Vec::new();
        let mut scratch_apps = Vec::new();
        while steps < limit {
            self.maybe_collect_garbage_between_steps(
                root,
                &frame_stack,
                &eval_spine,
                &persistent_spine,
                &scratch_args,
                &scratch_apps,
                None,
            )?;
            let mut current = self.resolve_for_whnf(root, profile_resolve)?;
            if let Some((next, reductions)) =
                self.finish_ready_eval_frame(&mut frame_stack, current)?
            {
                steps += reductions;
                self.reductions += reductions;
                if steps >= limit {
                    return Err(EvalError::StepLimit { limit });
                }
                root = next;
                persistent_active = false;
                persistent_spine.clear();
                continue;
            }

            if !whnf_frames {
                if !persistent_active {
                    persistent_spine.clear();
                }
                let head =
                    self.fill_persistent_spine(current, &mut persistent_spine, profile_resolve)?;
                match self.persistent_eval_step(
                    head,
                    &mut persistent_spine,
                    &mut frame_stack,
                    &mut scratch_args,
                    limit - steps,
                )? {
                    PersistentStep::Reduced { node, reductions } => {
                        steps += reductions;
                        self.reductions += reductions;
                        root = node;
                        persistent_active = true;
                        continue;
                    }
                    PersistentStep::Force { node } => {
                        root = node;
                        persistent_active = false;
                        persistent_spine.clear();
                        continue;
                    }
                    PersistentStep::Whnf { node } => {
                        let Some((next, reductions)) =
                            self.finish_whnf_eval_frame(&mut frame_stack, node)?
                        else {
                            return Ok((node, steps));
                        };
                        steps += reductions;
                        self.reductions += reductions;
                        if steps >= limit {
                            return Err(EvalError::StepLimit { limit });
                        }
                        root = next;
                        persistent_active = false;
                        persistent_spine.clear();
                        continue;
                    }
                    PersistentStep::Fallback { root: next_root } => {
                        if self.profile.is_some() {
                            self.profile_persistent_fallback();
                        }
                        root = next_root;
                        persistent_active = false;
                        persistent_spine.clear();
                        current = self.resolve_for_whnf(root, profile_resolve)?;
                    }
                }
            }

            if whnf_frames {
                if let Some((frame, next)) = self.begin_whnf_force_frame(current)? {
                    if self.profile.is_some() {
                        self.profile_eval_frame_push("Whnf");
                    }
                    frame_stack.push(EvalFrame::Whnf(frame));
                    root = next;
                    continue;
                }
            }

            let Some(step) = self.eval_loop_step(
                current,
                limit - steps,
                &mut eval_spine,
                &mut scratch_args,
                &mut scratch_apps,
                &mut frame_stack,
                true,
            )?
            else {
                let Some((next, reductions)) =
                    self.finish_whnf_eval_frame(&mut frame_stack, current)?
                else {
                    return Ok((current, steps));
                };
                steps += reductions;
                self.reductions += reductions;
                if steps >= limit {
                    return Err(EvalError::StepLimit { limit });
                }
                root = next;
                continue;
            };
            steps += step.reductions;
            self.reductions += step.reductions;
            root = step.node;
        }
        Err(EvalError::StepLimit { limit })
    }

    pub(in crate::runtime) fn reduce_whnf_from_stack(
        &mut self,
        mut current: NodeId,
        limit: usize,
        profile_resolve: bool,
    ) -> Result<(NodeId, usize), EvalError> {
        let mut steps = 0;
        let mut stack = EvalStack::default();
        let mut fallback_frame_stack = EvalFrameStack::default();
        let mut eval_spine = EvalSpine::default();
        let persistent_spine = PersistentSpine::default();
        let mut scratch_args = Vec::new();
        let mut scratch_apps = Vec::new();
        let profiling = self.profile.is_some();

        #[cfg(feature = "eval-phase-profile")]
        macro_rules! stack_phase_start {
            () => {
                profiling.then(Instant::now)
            };
        }
        #[cfg(not(feature = "eval-phase-profile"))]
        macro_rules! stack_phase_start {
            () => {
                ()
            };
        }
        #[cfg(feature = "eval-phase-profile")]
        macro_rules! record_stack_time {
            ($field:ident, $started:expr) => {{
                if let Some(started) = $started {
                    if let Some(profile) = self.profile.as_mut() {
                        profile.$field =
                            profile.$field.saturating_add(started.elapsed().as_nanos());
                    }
                }
            }};
        }
        #[cfg(not(feature = "eval-phase-profile"))]
        macro_rules! record_stack_time {
            ($field:ident, $started:expr) => {{
                let _ = &$started;
            }};
        }
        #[cfg(feature = "eval-phase-profile")]
        macro_rules! profile_stack_counter {
            ($field:ident) => {{
                if let Some(profile) = self.profile.as_mut() {
                    profile.$field = profile.$field.saturating_add(1);
                }
            }};
        }
        #[cfg(not(feature = "eval-phase-profile"))]
        macro_rules! profile_stack_counter {
            ($field:ident) => {};
        }
        macro_rules! profile_stack_step_result {
            ($step:expr) => {{
                #[cfg(feature = "eval-phase-profile")]
                {
                    profile_stack_counter!(stack_eval_step_calls);
                    match &$step {
                        StackStep::Reduced { .. } => profile_stack_counter!(stack_step_reduced),
                        StackStep::Whnf { .. } => profile_stack_counter!(stack_step_whnf),
                        StackStep::Fallback { .. } => profile_stack_counter!(stack_step_fallback),
                    }
                }
            }};
        }

        while steps < limit {
            profile_stack_counter!(stack_loop_iterations);
            let gc_started = stack_phase_start!();
            self.maybe_collect_garbage_between_steps(
                current,
                &fallback_frame_stack,
                &eval_spine,
                &persistent_spine,
                &scratch_args,
                &scratch_apps,
                Some(&stack),
            )?;
            record_stack_time!(stack_gc_check_nanos, gc_started);

            let resolve_started = stack_phase_start!();
            current = self.resolve_for_whnf(current, profile_resolve)?;
            record_stack_time!(stack_resolve_nanos, resolve_started);

            if stack.app_len() == 0 {
                profile_stack_counter!(stack_ready_checks);
                let ready_started = stack_phase_start!();
                if let Some((next, reductions)) =
                    self.finish_ready_stack_frame(&mut stack, current)?
                {
                    record_stack_time!(stack_ready_frame_nanos, ready_started);
                    profile_stack_counter!(stack_ready_successes);
                    steps += reductions;
                    self.reductions += reductions;
                    if steps >= limit {
                        return Err(EvalError::StepLimit { limit });
                    }
                    current = next;
                    continue;
                }
                record_stack_time!(stack_ready_frame_nanos, ready_started);
            }

            let descent_started = stack_phase_start!();
            while let Some(fun) = self.app_fun_trusted(current) {
                if profiling {
                    self.profile_stack_descent_push();
                }
                stack.push_app(current);
                current = self.resolve_for_whnf(fun, profile_resolve)?;
            }
            record_stack_time!(stack_descent_nanos, descent_started);

            if stack.app_len() == 0 {
                profile_stack_counter!(stack_ready_checks);
                let ready_started = stack_phase_start!();
                if let Some((next, reductions)) =
                    self.finish_ready_stack_frame(&mut stack, current)?
                {
                    record_stack_time!(stack_ready_frame_nanos, ready_started);
                    profile_stack_counter!(stack_ready_successes);
                    steps += reductions;
                    self.reductions += reductions;
                    if steps >= limit {
                        return Err(EvalError::StepLimit { limit });
                    }
                    current = next;
                    continue;
                }
                record_stack_time!(stack_ready_frame_nanos, ready_started);
            }

            let step_started = stack_phase_start!();
            let step = self.stack_eval_step(
                current,
                &mut stack,
                &mut scratch_args,
                limit - steps,
                profile_resolve,
            )?;
            record_stack_time!(stack_eval_step_nanos, step_started);
            profile_stack_step_result!(step);

            match step {
                StackStep::Reduced { node, reductions } => {
                    steps += reductions;
                    self.reductions += reductions;
                    if steps >= limit {
                        return Err(EvalError::StepLimit { limit });
                    }
                    current = node;
                }
                StackStep::Whnf {
                    node,
                    head,
                    reductions,
                } => {
                    steps += reductions;
                    self.reductions += reductions;
                    if steps >= limit {
                        return Err(EvalError::StepLimit { limit });
                    }
                    let whnf_started = stack_phase_start!();
                    let value = if stack.has_frame_below_apps() {
                        self.rethread_stack_app_segment(&mut stack, head)?
                    } else {
                        node
                    };
                    let Some((next, reductions)) =
                        self.finish_whnf_stack_frame(&mut stack, value)?
                    else {
                        record_stack_time!(stack_whnf_finish_nanos, whnf_started);
                        return Ok((value, steps));
                    };
                    record_stack_time!(stack_whnf_finish_nanos, whnf_started);
                    steps += reductions;
                    self.reductions += reductions;
                    if steps >= limit {
                        return Err(EvalError::StepLimit { limit });
                    }
                    current = next;
                }
                StackStep::Fallback {
                    root,
                    head,
                    reductions,
                } => {
                    steps += reductions;
                    self.reductions += reductions;
                    if steps >= limit {
                        return Err(EvalError::StepLimit { limit });
                    }
                    if self.profile.is_some() {
                        self.profile_persistent_fallback();
                    }
                    let root = if stack.app_len() == 0 {
                        root
                    } else {
                        self.rethread_stack_app_segment(&mut stack, head)?
                    };
                    let Some(step) = self.eval_loop_step(
                        root,
                        limit - steps,
                        &mut eval_spine,
                        &mut scratch_args,
                        &mut scratch_apps,
                        &mut fallback_frame_stack,
                        false,
                    )?
                    else {
                        if stack.top_is_frame() {
                            let Some((next, reductions)) =
                                self.finish_whnf_stack_frame(&mut stack, root)?
                            else {
                                return Ok((root, steps));
                            };
                            steps += reductions;
                            self.reductions += reductions;
                            if steps >= limit {
                                return Err(EvalError::StepLimit { limit });
                            }
                            current = next;
                            continue;
                        }
                        return Ok((root, steps));
                    };
                    debug_assert!(
                        fallback_frame_stack.peek().is_none(),
                        "strict_markers=false fallback step should not keep frames"
                    );
                    steps += step.reductions;
                    self.reductions += step.reductions;
                    if steps >= limit {
                        return Err(EvalError::StepLimit { limit });
                    }
                    current = step.node;
                    eval_spine.clear();
                }
            }
        }
        Err(EvalError::StepLimit { limit })
    }
}
