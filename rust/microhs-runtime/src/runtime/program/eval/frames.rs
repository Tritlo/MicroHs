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

    pub(in crate::runtime) fn int64_result_value_node(result: Int64Result) -> Node {
        match result {
            Int64Result::Int64(n) => Node::Int64(n),
            Int64Result::Bool(b) => Self::bool_value_node(b),
            Int64Result::Ordering(ord) => Self::ordering_value_node(ord),
        }
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
        debug_assert!(used > 0);
        debug_assert!(used <= app_end);
        let redex_index = app_end - used;
        let redex = stack.app_unchecked(redex_index);
        let wrote_indirection = node != redex;
        if wrote_indirection {
            self.set_app_cell_at(redex.index(), Cell::indir(Some(node)));
        }
        if self.profiling_enabled() {
            self.profile_stack_rewrite(used, wrote_indirection);
        }
        stack.apps.truncate(redex_index);
        node
    }

    pub(in crate::runtime) fn apply_stack_frame_value(
        &mut self,
        stack: &mut EvalStack,
        app_end: usize,
        used: usize,
        value: Node,
    ) -> NodeId {
        debug_assert!(used > 0);
        debug_assert!(used <= app_end);
        let redex_index = app_end - used;
        let redex = stack.app_unchecked(redex_index);
        self.set_app_node_at(redex.index(), value);
        if self.profiling_enabled() {
            self.profile_stack_rewrite(used, false);
        }
        stack.apps.truncate(redex_index);
        redex
    }

    pub(in crate::runtime) fn apply_stack_redex_value(
        &mut self,
        redex: NodeId,
        used: usize,
        value: Node,
    ) -> NodeId {
        self.set_app_node_at(redex.index(), value);
        if self.profiling_enabled() {
            self.profile_stack_rewrite(used, false);
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
        if self.profiling_enabled() {
            self.profile_stack_app_update(used);
        }
        if used == 0 {
            self.app(fun, arg)
        } else {
            let app_end = stack.apps.len();
            debug_assert!(used <= app_end);
            let redex_index = app_end - used;
            let redex = stack.app_unchecked(redex_index);
            self.set_app_cell_at(redex.index(), Cell::app(fun, arg));
            stack.apps.truncate(redex_index);
            redex
        }
    }

    pub(in crate::runtime) fn rethread_stack_app_segment(
        &mut self,
        stack: &mut EvalStack,
        mut node: NodeId,
    ) -> Result<NodeId, EvalError> {
        let base = stack.app_base();
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

    /// Follow `App` links from `current`, pushing each application node onto the
    /// active spine until a non-application head is reached.
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
                self.profile_reduction(frame.profile_head, 1);
                let wrote_indirection = result != frame.redex;
                if wrote_indirection {
                    self.set_app_cell_at(frame.redex.index(), Cell::indir(Some(result)));
                }
                if self.profiling_enabled() {
                    self.profile_stack_rewrite(frame.used, wrote_indirection);
                }
                return Ok((result, 1));
            }
            WhnfFrameKind::IoStrict { action, value } => {
                self.profile_reduction(frame.profile_head, 1);
                if self.profiling_enabled() {
                    self.profile_stack_app_update(frame.used);
                }
                self.set_app_cell_at(frame.redex.index(), Cell::app(action, value));
                return Ok((frame.redex, 1));
            }
            WhnfFrameKind::IsInt => {
                let value = self.resolve(value)?;
                let n = self.cell_int_value(value).unwrap_or(-1);
                self.apply_stack_redex_value(frame.redex, frame.used, Node::Int(n))
            }
        };

        self.profile_reduction(frame.profile_head, 1);
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
                        if self.profiling_enabled() {
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
                self.profile_reduction(frame.profile_head, 1);
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
                        if self.profiling_enabled() {
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
                                self.profile_reduction(frame.profile_head, 1);
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
                self.profile_reduction(frame.profile_head, 1);
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
                if self.profiling_enabled() {
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
                        if self.profiling_enabled() {
                            self.profile_eval_frame_push("Float64");
                        }
                        return Ok(Some((x, 0)));
                    }
                    Float64FrameKind::BinFirst { op, y } => op.apply(value, y),
                    Float64FrameKind::Un { op } => Float64Result::Float(op.apply(value)),
                };
                self.profile_reduction(frame.profile_head, 1);
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
                        if self.profiling_enabled() {
                            self.profile_eval_frame_push("Float32");
                        }
                        return Ok(Some((x, 0)));
                    }
                    Float32FrameKind::BinFirst { op, y } => op.apply(value, y),
                    Float32FrameKind::Un { op } => Float32Result::Float(op.apply(value)),
                };
                self.profile_reduction(frame.profile_head, 1);
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
                        if self.profiling_enabled() {
                            self.profile_eval_frame_push("Bytes");
                        }
                        return Ok(Some((x, 0)));
                    }
                    BytesFrameKind::BinFirst { op, y } => {
                        self.bytes_bin_result_node(op, current, y)?
                    }
                };
                self.profile_reduction(frame.profile_head, 1);
                let node = self.apply_stack_frame_rewrite(stack, frame.app_end, frame.used, node);
                (node, 1)
            }
            (StackFrame::Conversion(frame), ReadyFrame::Conversion(value)) => {
                self.profile_reduction(frame.profile_head, 1);
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
                // Raw float bits are an unsigned 32-bit word; zero-extend so the sign bit
                // (-0.0 = 0x8000_0000) survives the Word comparison in isNegZeroFloat32.
                Node::Int(i64::from(n.to_bits()))
            }
            _ => unreachable!("conversion frame kind and value mismatch"),
        }
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
    ) -> Result<(NodeId, usize), EvalError> {
        self.reduce_depth += 1;
        let result = self.reduce_whnf_from_stack(root, limit, profile_resolve);
        self.reduce_depth -= 1;
        result
    }

    pub(in crate::runtime) fn reduce_whnf_from_stack(
        &mut self,
        mut current: NodeId,
        limit: usize,
        profile_resolve: bool,
    ) -> Result<(NodeId, usize), EvalError> {
        let mut steps = 0;
        let mut stack = EvalStack::default();
        let mut eval_spine = EvalSpine::default();
        let mut scratch_args = Vec::new();
        let mut scratch_apps = Vec::new();
        let profiling = self.profiling_enabled();

        while steps < limit {
            // A fork (or other scheduler event) asked the current thread to yield so an
            // otherwise-unbounded single-thread slice can return to the scheduler. This
            // is a normal step boundary, so re-reducing later resumes from the frontier.
            if self.reschedule_now {
                return Err(EvalError::StepLimit { limit });
            }
            if steps > 0 {
                self.check_pending_async_exception(false)?;
            }
            self.maybe_collect_garbage_between_steps(
                current,
                &eval_spine,
                &scratch_args,
                &scratch_apps,
                Some(&stack),
            )?;

            current = self.resolve_for_whnf(current, profile_resolve)?;

            if stack.app_len() == 0 {
                if let Some((next, reductions)) =
                    self.finish_ready_stack_frame(&mut stack, current)?
                {
                    steps += reductions;
                    self.reductions += reductions;
                    if steps >= limit {
                        return Err(EvalError::StepLimit { limit });
                    }
                    current = next;
                    continue;
                }
            }

            while let Some(fun) = self.app_fun_trusted(current) {
                if profiling {
                    self.profile_stack_descent_push();
                }
                stack.push_app(current);
                current = self.resolve_for_whnf(fun, profile_resolve)?;
            }

            if stack.app_len() == 0 {
                if let Some((next, reductions)) =
                    self.finish_ready_stack_frame(&mut stack, current)?
                {
                    steps += reductions;
                    self.reductions += reductions;
                    if steps >= limit {
                        return Err(EvalError::StepLimit { limit });
                    }
                    current = next;
                    continue;
                }
            }

            let step = self.stack_eval_step(
                current,
                &mut stack,
                &mut scratch_args,
                limit - steps,
                profile_resolve,
            )?;

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
                    let value = if stack.has_frame_below_apps() {
                        self.rethread_stack_app_segment(&mut stack, head)?
                    } else {
                        node
                    };
                    let Some((next, reductions)) =
                        self.finish_whnf_stack_frame(&mut stack, value)?
                    else {
                        return Ok((value, steps));
                    };
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
                    if self.profiling_enabled() {
                        self.profile_fallback_entry();
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
