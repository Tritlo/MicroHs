//! Persistent-spine reducer used by the WHNF driver.
use super::*;

impl Program {
    pub(in crate::runtime) fn strict_redex_from_persistent_spine(
        &mut self,
        root: NodeId,
        used: usize,
        spine: &PersistentSpine,
    ) -> StrictRedex {
        if spine.len() == used {
            return StrictRedex::Root(root);
        }
        if self.profile.is_some() {
            self.profile_strict_redex_snapshot(spine.len());
        }
        StrictRedex::Spine {
            root,
            used,
            apps: spine.apps.iter().copied().collect(),
        }
    }

    pub(in crate::runtime) fn fill_persistent_spine(
        &mut self,
        mut node: NodeId,
        spine: &mut PersistentSpine,
        profile_resolve: bool,
    ) -> Result<NodeId, EvalError> {
        while let Some((fun, _)) = self.cell(node).app_fields() {
            spine.push_front(node);
            node = self.resolve_for_whnf(fun, profile_resolve)?;
        }
        Ok(node)
    }

    pub(in crate::runtime) fn persistent_eval_step(
        &mut self,
        head: NodeId,
        spine: &mut PersistentSpine,
        frame_stack: &mut EvalFrameStack,
        scratch_args: &mut Vec<NodeId>,
        budget: usize,
    ) -> Result<PersistentStep, EvalError> {
        let args_len = spine.len();
        let profile_head = if self.profile.is_some() {
            self.profile_step(head, args_len, false)
        } else {
            None
        };

        macro_rules! arg {
            ($idx:expr) => {
                spine.arg(&self.nodes, $idx)?
            };
        }
        macro_rules! app_site {
            ($key:literal, $fun:expr, $arg:expr) => {
                self.app_with_site($key, $fun, $arg)
            };
        }
        macro_rules! finish_reduction {
            ($node:expr, $reductions:expr) => {{
                if profile_head.is_some() {
                    self.profile_reduction(profile_head, $reductions);
                }
                return Ok(PersistentStep::Reduced {
                    node: $node,
                    reductions: $reductions,
                });
            }};
        }
        macro_rules! rewrite_step {
            ($used:expr, $node:expr, $reductions:expr) => {{
                let node = $node;
                if $used > 0 {
                    let redex = spine.app($used - 1);
                    if node != redex {
                        self.set_app_cell_at(redex.index(), Cell::indir(Some(node)));
                    }
                    if $used < spine.len() {
                        if self.profile.is_some() {
                            self.profile_remaining_app_scan(spine.len() - $used);
                        }
                    }
                    if $used < spine.len() && !spine.remaining_apps_contain($used, node) {
                        let app = spine.app($used);
                        let arg = spine.arg(&self.nodes, $used)?;
                        self.set_app_cell_at(app.index(), Cell::app(node, arg));
                    }
                }
                spine.consume($used);
                finish_reduction!(node, $reductions);
            }};
        }
        macro_rules! app_step_reductions {
            ($used:expr, $fun:expr, $arg:expr, $reductions:expr) => {{
                let fun = $fun;
                let arg = $arg;
                let node = if $used > 0 {
                    let redex = spine.app($used - 1);
                    self.set_app_cell_at(redex.index(), Cell::app(fun, arg));
                    redex
                } else {
                    app_site!("persistent_app_step_result", fun, arg)
                };
                spine.consume($used);
                finish_reduction!(node, $reductions);
            }};
        }
        macro_rules! app_step {
            ($used:expr, $fun:expr, $arg:expr) => {{
                app_step_reductions!($used, $fun, $arg, 1);
            }};
        }
        macro_rules! force_step {
            ($used:expr, $variant:ident, $frame:ident, $kind:expr, $next:expr) => {{
                let redex_root = spine.outer_root(head);
                let kind = $kind;
                let next = $next;
                let redex = self.strict_redex_from_persistent_spine(redex_root, $used, spine);
                if self.profile.is_some() {
                    self.profile_persistent_force();
                    self.profile_eval_frame_push(stringify!($variant));
                }
                frame_stack.push(EvalFrame::$variant($frame {
                    redex,
                    profile_head,
                    kind,
                }));
                return Ok(PersistentStep::Force { node: next });
            }};
        }
        macro_rules! force_int64_shift_step {
            ($used:expr, $op:expr, $x:expr, $next:expr) => {{
                let redex_root = spine.outer_root(head);
                let x = $x;
                let next = $next;
                let redex = self.strict_redex_from_persistent_spine(redex_root, $used, spine);
                if self.profile.is_some() {
                    self.profile_persistent_force();
                    self.profile_eval_frame_push("Int64Shift");
                }
                frame_stack.push(EvalFrame::Int64Shift(Int64ShiftFrame {
                    redex,
                    profile_head,
                    op: $op,
                    x,
                }));
                return Ok(PersistentStep::Force { node: next });
            }};
        }

        let head_dispatch = match self.cell(head).prim() {
            Some(Prim::Known(known)) => PersistentHead::Known(known),
            Some(Prim::Runtime(runtime)) => PersistentHead::Other(runtime.strict_action(args_len)),
            None if args_len > 0 => match self.cold_node(head) {
                Some(Node::Ffi(name)) => PersistentHead::Ffi(name.to_string()),
                Some(Node::JsCall(call)) => PersistentHead::JsCall {
                    tags: call.tags.clone(),
                    body: call.body.clone(),
                },
                Some(Node::JsWrap { tags }) => PersistentHead::JsWrap {
                    tags: tags.to_string(),
                },
                _ => PersistentHead::Whnf,
            },
            None => PersistentHead::Whnf,
        };

        let known = match head_dispatch {
            PersistentHead::Ffi(name) => {
                if self.profile.is_some() {
                    self.profile_arg_materialization(args_len);
                }
                spine.write_args_head_order(&self.nodes, scratch_args)?;
                let Some((used, node)) = self.ffi_call(&name, scratch_args.as_slice())? else {
                    return Ok(PersistentStep::Whnf {
                        node: spine.outer_root(head),
                    });
                };
                rewrite_step!(used, node, 1);
            }
            PersistentHead::JsCall { tags, body } => {
                if self.profile.is_some() {
                    self.profile_arg_materialization(args_len);
                }
                spine.write_args_head_order(&self.nodes, scratch_args)?;
                let Some((used, node)) = self.js_call(&tags, &body, scratch_args.as_slice())?
                else {
                    return Ok(PersistentStep::Whnf {
                        node: spine.outer_root(head),
                    });
                };
                rewrite_step!(used, node, 1);
            }
            PersistentHead::JsWrap { tags } => {
                if self.profile.is_some() {
                    self.profile_arg_materialization(args_len);
                }
                spine.write_args_head_order(&self.nodes, scratch_args)?;
                let Some((used, node)) = self.js_wrap(&tags, scratch_args.as_slice())? else {
                    return Ok(PersistentStep::Whnf {
                        node: spine.outer_root(head),
                    });
                };
                rewrite_step!(used, node, 1);
            }
            PersistentHead::Known(known) => known,
            PersistentHead::Other(action) => {
                if self.profile.is_some() {
                    self.profile_strict_primitive_dispatch(args_len, action);
                }
                match action {
                    StrictPrimitiveAction::IntBin(op) => {
                        let x = arg!(0);
                        force_step!(2, Int, IntFrame, IntFrameKind::BinSecond { op, x }, arg!(1));
                    }
                    StrictPrimitiveAction::IntUn(op) => {
                        force_step!(1, Int, IntFrame, IntFrameKind::Un { op }, arg!(0));
                    }
                    StrictPrimitiveAction::Int64Bin(op) => {
                        if op.rhs_is_shift() {
                            force_int64_shift_step!(2, op, arg!(0), arg!(1));
                        } else if op.driver_marker_safe() {
                            force_step!(
                                2,
                                Int64,
                                Int64Frame,
                                Int64FrameKind::BinSecond { op, x: arg!(0) },
                                arg!(1)
                            );
                        }
                    }
                    StrictPrimitiveAction::Int64Un(op) => {
                        force_step!(1, Int64, Int64Frame, Int64FrameKind::Un { op }, arg!(0));
                    }
                    StrictPrimitiveAction::Float64Bin(op) => {
                        force_step!(
                            2,
                            Float64,
                            Float64Frame,
                            Float64FrameKind::BinSecond { op, x: arg!(0) },
                            arg!(1)
                        );
                    }
                    StrictPrimitiveAction::Float64Un(op) => {
                        force_step!(
                            1,
                            Float64,
                            Float64Frame,
                            Float64FrameKind::Un { op },
                            arg!(0)
                        );
                    }
                    StrictPrimitiveAction::Float32Bin(op) => {
                        force_step!(
                            2,
                            Float32,
                            Float32Frame,
                            Float32FrameKind::BinSecond { op, x: arg!(0) },
                            arg!(1)
                        );
                    }
                    StrictPrimitiveAction::Float32Un(op) => {
                        force_step!(
                            1,
                            Float32,
                            Float32Frame,
                            Float32FrameKind::Un { op },
                            arg!(0)
                        );
                    }
                    StrictPrimitiveAction::BytesBin(op) => {
                        force_step!(
                            2,
                            Bytes,
                            BytesFrame,
                            BytesFrameKind::BinSecond { op, x: arg!(0) },
                            arg!(1)
                        );
                    }
                    StrictPrimitiveAction::Conversion(kind) => {
                        force_step!(1, Conversion, ConversionFrame, kind, arg!(0));
                    }
                    StrictPrimitiveAction::None => {
                        return Ok(PersistentStep::Fallback {
                            root: spine.outer_root(head),
                        });
                    }
                }
                return Ok(PersistentStep::Fallback {
                    root: spine.outer_root(head),
                });
            }
            PersistentHead::Whnf => {
                return Ok(PersistentStep::Whnf {
                    node: spine.outer_root(head),
                });
            }
        };
        use KnownPrim::*;

        match known {
            IoStrict if args_len >= 2 => {
                force_step!(
                    2,
                    Whnf,
                    WhnfFrame,
                    WhnfFrameKind::IoStrict {
                        action: arg!(0),
                        value: arg!(1),
                    },
                    arg!(1)
                );
            }
            Seq if args_len >= 2 => {
                force_step!(
                    2,
                    Whnf,
                    WhnfFrame,
                    WhnfFrameKind::Seq { result: arg!(1) },
                    arg!(0)
                );
            }
            IsInt if args_len >= 1 => {
                force_step!(1, Whnf, WhnfFrame, WhnfFrameKind::IsInt, arg!(0));
            }
            _ => {}
        }

        match known {
            IoPerformIo if args_len >= 1 => {
                let world = self.world();
                let k = self.prim("K");
                let io = arg!(0);
                let action = app_site!("IO.performIO.action", io, world);
                app_step!(1, action, k);
            }
            IoBind if args_len >= 3 => {
                let io = arg!(0);
                let k = arg!(1);
                let world = arg!(2);
                let action = app_site!("IO.bind.action", io, world);
                app_step!(3, action, k);
            }
            IoThen if args_len >= 3 && budget >= 2 => {
                let k = self.prim("K");
                let io = arg!(0);
                let y = arg!(1);
                let world = arg!(2);
                let then = app_site!("IO.then.k", k, y);
                let action = app_site!("IO.then.action", io, world);
                app_step_reductions!(3, action, then, 2);
            }
            IoThen if args_len >= 2 => {
                let bind = self.prim("IO.>>=");
                let io = arg!(0);
                let y = arg!(1);
                let bind_action = app_site!("IO.then.bind_action", bind, io);
                let k = self.prim("K");
                let then = app_site!("IO.then.k", k, y);
                app_step!(2, bind_action, then);
            }
            IoReturn if args_len >= 3 => {
                let x = arg!(0);
                let world = arg!(1);
                let k = arg!(2);
                let kx = app_site!("IO.return.kx", k, x);
                app_step!(3, kx, world);
            }
            I | Ord | Chr if args_len >= 1 => {
                let mut used = 1;
                let mut reductions = 1;
                let mut node = arg!(0);
                let mut alias_shortcuts = 0;
                while reductions < budget && used < args_len && self.is_identity_alias_node(node)? {
                    node = arg!(used);
                    used += 1;
                    reductions += 1;
                    alias_shortcuts += 1;
                }
                self.profile_shortcut("identity_alias_chain", alias_shortcuts);
                rewrite_step!(used, node, reductions);
            }
            K if args_len >= 2 => rewrite_step!(2, arg!(0), 1),
            A if args_len >= 2 => rewrite_step!(2, arg!(1), 1),
            U if args_len >= 2 => {
                app_step!(2, arg!(1), arg!(0));
            }
            S if args_len >= 3 => {
                let x = arg!(2);
                let left = app_site!("S.left", arg!(0), x);
                let right = app_site!("S.right", arg!(1), x);
                app_step!(3, left, right);
            }
            SPrime if args_len >= 4 => {
                let yw = app_site!("S'.yw", arg!(1), arg!(3));
                let zw = app_site!("S'.zw", arg!(2), arg!(3));
                let left = app_site!("S'.left", arg!(0), yw);
                app_step!(4, left, zw);
            }
            B if args_len >= 3 => {
                let yz = app_site!("B.yz", arg!(1), arg!(2));
                app_step!(3, arg!(0), yz);
            }
            BPrime if args_len >= 4 => {
                let zw = app_site!("B'.zw", arg!(2), arg!(3));
                let xy = app_site!("B'.xy", arg!(0), arg!(1));
                app_step!(4, xy, zw);
            }
            BPrime if args_len >= 2 => {
                let xy = app_site!("B'.xy_under", arg!(0), arg!(1));
                let b = self.prim("B");
                app_step!(2, b, xy);
            }
            Z if args_len >= 3 => {
                app_step!(3, arg!(0), arg!(1));
            }
            Z if args_len >= 2 => {
                let xy = app_site!("Z.xy_under", arg!(0), arg!(1));
                let k = self.prim("K");
                app_step!(2, k, xy);
            }
            J if args_len >= 3 => {
                app_step!(3, arg!(2), arg!(0));
            }
            L if args_len >= 3 => {
                app_step!(3, arg!(1), arg!(0));
            }
            KK if args_len >= 3 => rewrite_step!(3, arg!(1), 1),
            KA if args_len >= 3 => rewrite_step!(3, arg!(2), 1),
            C if args_len >= 3 => {
                let xz = app_site!("C.xz", arg!(0), arg!(2));
                app_step!(3, xz, arg!(1));
            }
            CPrime if args_len >= 4 => {
                let yw = app_site!("C'.yw", arg!(1), arg!(3));
                let xyw = app_site!("C'.xyw", arg!(0), yw);
                app_step!(4, xyw, arg!(2));
            }
            P if args_len >= 3 => {
                let zx = app_site!("P.zx", arg!(2), arg!(0));
                app_step!(3, zx, arg!(1));
            }
            R if args_len >= 3 => {
                let yz = app_site!("R.yz", arg!(1), arg!(2));
                app_step!(3, yz, arg!(0));
            }
            R if args_len >= 2 => {
                let c = self.prim("C");
                let cy = app_site!("R.cy_under", c, arg!(1));
                app_step!(2, cy, arg!(0));
            }
            O if args_len >= 4 => {
                let wx = app_site!("O.wx", arg!(3), arg!(0));
                app_step!(4, wx, arg!(1));
            }
            K2 if args_len >= 3 => rewrite_step!(3, arg!(0), 1),
            K2 if args_len >= 2 => {
                let k = self.prim("K");
                app_step!(2, k, arg!(0));
            }
            K3 if args_len >= 4 => rewrite_step!(4, arg!(0), 1),
            K3 if args_len >= 2 => {
                let k2 = self.prim("K2");
                app_step!(2, k2, arg!(0));
            }
            K4 if args_len >= 5 => rewrite_step!(5, arg!(0), 1),
            K4 if args_len >= 2 => {
                let k3 = self.prim("K3");
                app_step!(2, k3, arg!(0));
            }
            CPrimeB if args_len >= 4 => {
                let yw = app_site!("C'B.yw", arg!(1), arg!(3));
                let xz = app_site!("C'B.xz", arg!(0), arg!(2));
                app_step!(4, xz, yw);
            }
            CPrimeB if args_len >= 3 => {
                let xz = app_site!("C'B.xz_under", arg!(0), arg!(2));
                let b = self.prim("B");
                let bxz = app_site!("C'B.bxz_under", b, xz);
                app_step!(3, bxz, arg!(1));
            }
            Y if args_len >= 1 => {
                app_step!(1, arg!(0), spine.app(0));
            }
            Tag(tag) if args_len >= 2 => {
                let tag = self.int(i64::from(tag));
                let ytag = app_site!("Tag.ytag", arg!(1), tag);
                app_step!(2, ytag, arg!(0));
            }
            Tuple(fields) if args_len > usize::from(fields) => {
                let fields = usize::from(fields);
                let mut n = arg!(fields);
                for idx in 0..fields - 1 {
                    let arg = arg!(idx);
                    n = app_site!("Tuple.prefix", n, arg);
                }
                let last = arg!(fields - 1);
                app_step!(fields + 1, n, last);
            }
            I | Ord | Chr | K | A | U | S | SPrime | B | BPrime | Z | J | L | KK | KA | C
            | CPrime | P | R | O | K2 | K3 | K4 | CPrimeB | Y | Tag(_) | Tuple(_) | IoPerformIo
            | IoBind | IoThen | IoReturn => Ok(PersistentStep::Whnf {
                node: spine.outer_root(head),
            }),
            _ => Ok(PersistentStep::Fallback {
                root: spine.outer_root(head),
            }),
        }
    }
}
