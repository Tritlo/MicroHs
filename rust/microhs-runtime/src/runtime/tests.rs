//! Runtime parity and regression tests.
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::runtime::{
        BFile, BFileKind, BytesBinOp, BytesFrame, BytesFrameKind, EvalFrame, EvalFrameStack,
        EvalSpine, EvalStack, FORCE_REDUCTION_LIMIT, IGNORED_IO_SHORTCUT_RECURSION_LIMIT, MpzValue,
        PersistentSpine, StackFrame, StackWhnfFrame, StrictRedex, WeakNode, WhnfFrameKind,
        bwt_decode, bwt_encode, lz77_compress, lz77_decompress, lzma_compress_payload,
        lzma_decompress_payload, serialize_bytes_quoted,
    };
    use crate::{EvalError, KnownPrim, Node, NodeId, ParseError, Prim, Program, parse_program};

    fn whnf(input: &[u8]) -> String {
        let mut program = parse_program(input).unwrap();
        let (root, _) = program.reduce_whnf(100).unwrap();
        program.render(root)
    }

    fn collect_for_test(program: &mut Program, root: NodeId) {
        program
            .collect_garbage_between_steps(
                root,
                &EvalFrameStack::default(),
                &EvalSpine::default(),
                &PersistentSpine::default(),
                &[],
                &[],
                None,
            )
            .unwrap();
    }

    #[test]
    fn moving_gc_remapper_rewrites_node_id_holders() {
        fn id(index: usize) -> NodeId {
            NodeId::from_index(index)
        }

        fn shifted(id: NodeId) -> NodeId {
            NodeId::from_index(id.index() + 1000)
        }

        let mut labels = HashMap::new();
        labels.insert(0, id(12));
        let mut program = Program::new(
            vec![
                Node::App(id(1), id(2)),
                Node::Indir(Some(id(3))),
                Node::bytes_view(id(4), 0, 1),
                Node::MVar(Some(id(5))),
                Node::array(vec![id(6), id(7)]),
                Node::Weak(Box::new(WeakNode {
                    key: Some(id(8)),
                    value: Some(id(9)),
                    finalizer: Some(id(10)),
                })),
                Node::Free(Some(id(11))),
                Node::Int(0),
            ],
            id(0),
            labels,
        );
        program.stable_ptrs.push(Some(id(13)));
        program.pending_weak_finalizers.push(id(14));
        program.bfiles.push(Some(BFile {
            kind: BFileKind::ReadOnlyMemoryView {
                base: id(15),
                offset: 0,
                len: 1,
                pos: 0,
            },
            readable: true,
            writable: false,
        }));
        program.arg_ref_array = Some(id(16));
        program.world = Some(id(17));
        program.prim_cache.a = Some(id(18));
        program.compound_cache.fst = Some(id(19));
        program.small_ints[0] = Some(id(20));
        program.node_pointers = vec![id(21), id(22)];
        program.node_pointer_slots.insert(id(21), 0);
        program.node_pointer_slots.insert(id(22), 1);
        program.gc_mark_work.push(id(23));

        let mut current_root = id(24);
        let mut frame_stack = EvalFrameStack::default();
        frame_stack.push(EvalFrame::Bytes(BytesFrame {
            redex: StrictRedex::Spine {
                root: id(25),
                used: 1,
                apps: vec![id(26)],
            },
            profile_head: Some(id(27)),
            kind: BytesFrameKind::BinFirst {
                op: BytesBinOp::Append,
                y: id(28),
            },
        }));
        let mut eval_spine = EvalSpine::default();
        eval_spine.push_desc(id(29), id(30));
        let mut persistent_spine = PersistentSpine::default();
        persistent_spine.push_front(id(31));
        let mut scratch_args = vec![id(32)];
        let mut scratch_apps = vec![id(33)];
        let mut machine_stack = EvalStack::default();
        machine_stack.apps.extend([id(34), id(35)]);
        machine_stack.frames.push(StackFrame::Whnf(StackWhnfFrame {
            prev_app_base: 0,
            redex: id(36),
            used: 0,
            profile_head: Some(id(37)),
            kind: WhnfFrameKind::IoStrict {
                action: id(38),
                value: id(39),
            },
        }));

        program.remap_node_ids_for_moving_gc(
            &mut current_root,
            &mut frame_stack,
            &mut eval_spine,
            &mut persistent_spine,
            &mut scratch_args,
            &mut scratch_apps,
            Some(&mut machine_stack),
            shifted,
        );

        assert_eq!(program.root, shifted(id(0)));
        assert_eq!(current_root, shifted(id(24)));
        assert_eq!(program.labels[&0], shifted(id(12)));
        assert_eq!(program.stable_ptrs[1], Some(shifted(id(13))));
        assert_eq!(program.pending_weak_finalizers, vec![shifted(id(14))]);
        assert_eq!(program.arg_ref_array, Some(shifted(id(16))));
        assert_eq!(program.world, Some(shifted(id(17))));
        assert_eq!(program.prim_cache.a, Some(shifted(id(18))));
        assert_eq!(program.compound_cache.fst, Some(shifted(id(19))));
        assert_eq!(program.small_ints[0], Some(shifted(id(20))));
        assert_eq!(program.gc_mark_work, vec![shifted(id(23))]);

        assert_eq!(
            program.cell(id(0)).app_fields(),
            Some((shifted(id(1)), shifted(id(2))))
        );
        assert_eq!(program.cell(id(1)).option_id_word1(), Some(shifted(id(3))));
        assert_eq!(program.cell(id(6)).option_id_word1(), Some(id(11)));

        match program.node_for_debug(id(2)) {
            Node::BytesView(view) => assert_eq!(view.base, shifted(id(4))),
            other => panic!("expected BytesView, got {other:?}"),
        }
        match program.node_for_debug(id(3)) {
            Node::MVar(value) => assert_eq!(value, Some(shifted(id(5)))),
            other => panic!("expected MVar, got {other:?}"),
        }
        match program.node_for_debug(id(4)) {
            Node::Array(items) => assert_eq!(&*items, &[shifted(id(6)), shifted(id(7))]),
            other => panic!("expected Array, got {other:?}"),
        }
        match program.node_for_debug(id(5)) {
            Node::Weak(weak) => {
                assert_eq!(weak.key, Some(shifted(id(8))));
                assert_eq!(weak.value, Some(shifted(id(9))));
                assert_eq!(weak.finalizer, Some(shifted(id(10))));
            }
            other => panic!("expected Weak, got {other:?}"),
        }

        let bfile = program.bfiles[0].as_ref().unwrap();
        match &bfile.kind {
            BFileKind::ReadOnlyMemoryView { base, .. } => assert_eq!(*base, shifted(id(15))),
            other => panic!("expected ReadOnlyMemoryView, got {other:?}"),
        }
        assert_eq!(
            program.node_pointers,
            vec![shifted(id(21)), shifted(id(22))]
        );
        assert_eq!(program.node_pointer_slots.get(&shifted(id(21))), Some(&0));
        assert_eq!(program.node_pointer_slots.get(&shifted(id(22))), Some(&1));
        assert!(!program.node_pointer_slots.contains_key(&id(21)));

        match frame_stack.top.as_ref().unwrap() {
            EvalFrame::Bytes(frame) => {
                match &frame.redex {
                    StrictRedex::Spine { root, apps, .. } => {
                        assert_eq!(*root, shifted(id(25)));
                        assert_eq!(apps, &[shifted(id(26))]);
                    }
                    StrictRedex::Root(_) => panic!("expected spine redex"),
                }
                assert_eq!(frame.profile_head, Some(shifted(id(27))));
                match &frame.kind {
                    BytesFrameKind::BinFirst { y, .. } => assert_eq!(*y, shifted(id(28))),
                    BytesFrameKind::BinSecond { .. } => panic!("expected BinFirst"),
                }
            }
            _ => panic!("expected Bytes frame"),
        }
        assert_eq!(eval_spine.desc_arg(0), shifted(id(29)));
        assert_eq!(eval_spine.desc_app(0), shifted(id(30)));
        assert_eq!(persistent_spine.app(0), shifted(id(31)));
        assert_eq!(scratch_args, vec![shifted(id(32))]);
        assert_eq!(scratch_apps, vec![shifted(id(33))]);
        assert_eq!(machine_stack.apps, vec![shifted(id(34)), shifted(id(35))]);
        match &machine_stack.frames[0] {
            StackFrame::Whnf(frame) => {
                assert_eq!(frame.redex, shifted(id(36)));
                assert_eq!(frame.profile_head, Some(shifted(id(37))));
                match &frame.kind {
                    WhnfFrameKind::IoStrict { action, value } => {
                        assert_eq!(*action, shifted(id(38)));
                        assert_eq!(*value, shifted(id(39)));
                    }
                    _ => panic!("expected IoStrict frame"),
                }
            }
            _ => panic!("expected Whnf stack frame"),
        }
    }

    #[test]
    fn moving_gc_evacuates_marked_heap_and_remaps_roots() {
        fn id(index: usize) -> NodeId {
            NodeId::from_index(index)
        }

        let mut labels = HashMap::new();
        labels.insert(1, id(5));
        labels.insert(2, id(10));
        let mut program = Program::new(
            vec![
                Node::App(id(2), id(5)),
                Node::Int(999),
                Node::bytes_view(id(4), 0, 2),
                Node::Int(777),
                Node::Bytes(Box::new(vec![b'a', b'b'])),
                Node::Array(Box::new(vec![id(2), id(4), id(7)])),
                Node::Free(None),
                Node::App(id(4), id(8)),
                Node::Prim(Prim::Known(KnownPrim::I)),
                Node::Bytes(Box::new(vec![b'z'])),
                Node::Int(513),
            ],
            id(0),
            labels,
        );
        program.stable_ptrs.push(Some(id(7)));
        program.bfiles.push(Some(BFile {
            kind: BFileKind::ReadOnlyMemoryView {
                base: id(4),
                offset: 0,
                len: 2,
                pos: 0,
            },
            readable: true,
            writable: false,
        }));
        program.node_pointers = vec![id(9)];
        program.node_pointer_slots.insert(id(9), 0);

        let mut marked = vec![false; program.node_count()];
        for live in [0, 2, 4, 5, 7, 8, 9] {
            marked[live] = true;
        }

        let mut current_root = id(0);
        let mut frame_stack = EvalFrameStack::default();
        let mut eval_spine = EvalSpine::default();
        eval_spine.push_desc(id(2), id(7));
        let mut persistent_spine = PersistentSpine::default();
        persistent_spine.push_front(id(5));
        let mut scratch_args = vec![id(4)];
        let mut scratch_apps = vec![id(7)];
        let mut machine_stack = EvalStack::default();
        machine_stack.apps.push(id(0));

        let freed = program.evacuate_marked_heap_for_moving_gc(
            &marked,
            &mut current_root,
            &mut frame_stack,
            &mut eval_spine,
            &mut persistent_spine,
            &mut scratch_args,
            &mut scratch_apps,
            Some(&mut machine_stack),
        );

        let new_root = id(0);
        let new_view = id(1);
        let new_bytes = id(2);
        let new_array = id(3);
        let new_app = id(4);
        let new_i = id(5);
        let new_pointer_target = id(6);

        assert_eq!(freed, 4);
        assert_eq!(program.node_count(), 7);
        assert_eq!(program.root(), new_root);
        assert_eq!(current_root, new_root);
        assert_eq!(program.free_head, None);
        assert_eq!(program.free_nodes, 0);
        assert_eq!(program.labels.get(&1), Some(&new_array));
        assert!(!program.labels.contains_key(&2));

        assert_eq!(
            program.cell(new_root).app_fields(),
            Some((new_view, new_array))
        );
        match program.node_for_debug(new_view) {
            Node::BytesView(view) => assert_eq!(view.base, new_bytes),
            other => panic!("expected BytesView, got {other:?}"),
        }
        match program.node_for_debug(new_array) {
            Node::Array(items) => assert_eq!(&*items, &[new_view, new_bytes, new_app]),
            other => panic!("expected Array, got {other:?}"),
        }
        assert_eq!(program.cell(new_app).app_fields(), Some((new_bytes, new_i)));
        match program.node_for_debug(new_pointer_target) {
            Node::Bytes(bytes) => assert_eq!(&*bytes, b"z"),
            other => panic!("expected Bytes, got {other:?}"),
        }

        assert_eq!(program.stable_ptrs[1], Some(new_app));
        match &program.bfiles[0].as_ref().unwrap().kind {
            BFileKind::ReadOnlyMemoryView { base, .. } => assert_eq!(*base, new_bytes),
            other => panic!("expected ReadOnlyMemoryView, got {other:?}"),
        }
        assert_eq!(program.node_pointers, vec![new_pointer_target]);
        assert_eq!(
            program.node_pointer_slots.get(&new_pointer_target),
            Some(&0)
        );
        assert!(!program.node_pointer_slots.contains_key(&id(9)));
        assert_eq!(eval_spine.desc_arg(0), new_view);
        assert_eq!(eval_spine.desc_app(0), new_app);
        assert_eq!(persistent_spine.app(0), new_array);
        assert_eq!(scratch_args, vec![new_bytes]);
        assert_eq!(scratch_apps, vec![new_app]);
        assert_eq!(machine_stack.apps, vec![new_root]);
        assert!(program.cold_nodes.iter().all(Option::is_some));
    }

    fn deserialize_memory(program: &mut Program, bytes: Vec<u8>) -> (NodeId, i64) {
        let ptr = program
            .alloc_bfile(BFile {
                kind: BFileKind::Memory { bytes, pos: 0 },
                readable: true,
                writable: false,
            })
            .unwrap();
        let ptr_node = program.push_node(Node::Ptr(ptr));
        let world = program.push_node(Node::Int(99_999));
        let deserialize = program.prim("IO.deserialize");
        let deserialize_ptr = program.app(deserialize, ptr_node);
        let action = program.app(deserialize_ptr, world);
        let pair = program
            .reduce_node_whnf(action, FORCE_REDUCTION_LIMIT)
            .unwrap();
        let (left, returned_world) = program.cell(pair).app_fields().unwrap();
        assert_eq!(returned_world, world);
        let (head, value) = program.cell(left).app_fields().unwrap();
        assert!(matches!(
            program.cell(head).prim(),
            Some(Prim::Known(KnownPrim::P))
        ));
        (value, ptr)
    }

    #[test]
    fn reduces_identity() {
        assert_eq!(whnf(b"v8.4\n0\nI #42 @ }"), "42");
    }

    #[test]
    fn reduces_skk_identity() {
        assert_eq!(whnf(b"v8.4\n0\nS K @ K @ #7 @ }"), "7");
    }

    #[test]
    fn reduces_optimizer_combinators() {
        assert_eq!(whnf(b"v8.4\n0\nS' K @ K @ K @ #5 @ #0 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nB' K @ #5 @ K @ #0 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nZ K @ #5 @ #0 @ #1 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nJ #5 @ #0 @ I @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nL #5 @ I @ #0 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nKK #0 @ #5 @ #1 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nKA #0 @ #1 @ #5 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nC' A @ K @ #5 @ #0 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nR #0 @ K @ #5 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nO #5 @ #0 @ #1 @ K @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nC'B K @ K @ #5 @ #0 @ }"), "5");
    }

    #[test]
    fn reduces_partial_arity_specializations() {
        assert_eq!(whnf(b"v8.4\n0\nB' I @ I @ #9 @ }"), "((B (I I)) 9)");
        assert_eq!(whnf(b"v8.4\n0\nZ K @ #5 @ }"), "(K (K 5))");
        assert_eq!(whnf(b"v8.4\n0\nR #1 @ #2 @ }"), "((C 2) 1)");
        assert_eq!(whnf(b"v8.4\n0\nK2 #1 @ #2 @ }"), "(K 1)");
        assert_eq!(whnf(b"v8.4\n0\nK3 #1 @ #2 @ #3 @ }"), "(K 1)");
        assert_eq!(whnf(b"v8.4\n0\nC'B K @ I @ #9 @ }"), "((B (K 9)) I)");
    }

    #[test]
    fn reduces_constructor_tags_and_tuples() {
        assert_eq!(whnf(b"v8.4\n0\nTAG3 #99 @ K @ }"), "3");
        assert_eq!(whnf(b"v8.4\n0\nTAG10 #99 @ A @ }"), "99");
        assert_eq!(whnf(b"v8.4\n0\nT3 #1 @ #2 @ #3 @ K3 @ #0 @ }"), "1");
        assert_eq!(whnf(b"v8.4\n0\nT4 #1 @ #2 @ #3 @ #4 @ K4 @ #0 @ }"), "1");
    }

    #[test]
    fn resolves_shared_labels() {
        assert_eq!(whnf(b"v8.4\n1\nA #42 :0 @ _0 @ }"), "42");
    }

    #[test]
    fn serializes_quoted_bytestring_escapes_like_c() {
        let mut special = Vec::new();
        serialize_bytes_quoted(b"?^|\\\"", &mut special);
        assert_eq!(special, b"\"?\\^\\|\\\\\\\"\"");

        for byte in 0u8..=255 {
            let mut input = b"v8.4\n0\n".to_vec();
            serialize_bytes_quoted(&[byte], &mut input);
            input.extend_from_slice(b" }\n");

            let program = parse_program(&input).unwrap();
            match program.node_for_debug(program.root()) {
                Node::Bytes(bytes) => assert_eq!(
                    bytes.as_slice(),
                    &[byte],
                    "byte {byte:#04x} encoded as {}",
                    String::from_utf8_lossy(&input)
                ),
                other => panic!(
                    "byte {byte:#04x} parsed as {other:?} from {}",
                    String::from_utf8_lossy(&input)
                ),
            }
        }
    }

    #[test]
    fn serializes_bigints_with_c_wire_format() {
        let program = parse_program(b"v8.4\n0\n%-123456789\" }\n").unwrap();
        assert_eq!(
            program.serialize_program(program.root()).unwrap(),
            b"v8.4\n0\n%-123456789\" }\n"
        );

        let legacy = parse_program(b"v8.4\n0\n%\"123456789\" }\n").unwrap();
        match legacy.node_for_debug(legacy.root()) {
            Node::BigInt(bytes) => assert_eq!(bytes.as_slice(), b"123456789"),
            other => panic!("legacy bigint parsed as {other:?}"),
        }
    }

    #[test]
    fn mpz_get_d_uses_decimal_rounding_like_c() {
        let decimal = b"-299228957055072645483636";
        let value = MpzValue::parse_decimal(decimal).unwrap();
        let expected = std::str::from_utf8(decimal)
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert_eq!(value.to_f64().to_bits(), expected.to_bits());
    }

    #[test]
    fn serializes_shared_apps_with_labels() {
        let program = parse_program(b"v8.4\n1\nK #1 @ :0 _0 @ }\n").unwrap();
        let serialized = program.serialize_program(program.root()).unwrap();
        assert!(serialized.starts_with(b"v8.4\n1\n"), "{serialized:?}");
        assert!(
            serialized
                .windows(b":2 ".len())
                .any(|window| window == b":2 "),
            "{}",
            String::from_utf8_lossy(&serialized)
        );
        assert!(
            serialized
                .windows(b"_2 ".len())
                .any(|window| window == b"_2 "),
            "{}",
            String::from_utf8_lossy(&serialized)
        );

        let round_trip = parse_program(&serialized).unwrap();
        assert_eq!(
            round_trip.serialize_program(round_trip.root()).unwrap(),
            serialized
        );
    }

    #[test]
    fn serializes_cyclic_apps_with_labels() {
        let program = parse_program(b"v8.4\n1\nK _0 @ :0 }\n").unwrap();
        let serialized = program.serialize_program(program.root()).unwrap();
        assert!(serialized.starts_with(b"v8.4\n1\n"), "{serialized:?}");
        assert!(
            serialized
                .windows(b":2 ".len())
                .any(|window| window == b":2 "),
            "{}",
            String::from_utf8_lossy(&serialized)
        );
        assert!(
            serialized
                .windows(b"_2 ".len())
                .any(|window| window == b"_2 "),
            "{}",
            String::from_utf8_lossy(&serialized)
        );
        parse_program(&serialized).unwrap();
    }

    #[test]
    fn io_deserialize_reads_one_comb_from_bfile() {
        let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
        let (value, ptr) = deserialize_memory(&mut program, b"v8.4\n0\n#42 }tail".to_vec());
        assert_eq!(program.render(value), "42");
        assert_eq!(program.read_bfile_bytes(ptr, 4).unwrap(), b"tail");
    }

    #[test]
    fn io_deserialize_preserves_shared_cycles() {
        let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
        let (value, _) = deserialize_memory(&mut program, b"v8.4\n1\nK _0 @ :0 }\n".to_vec());
        let serialized = program.serialize_program(value).unwrap();
        assert!(serialized.starts_with(b"v8.4\n1\n"), "{serialized:?}");
        assert!(
            serialized.contains(&b'_'),
            "{}",
            String::from_utf8_lossy(&serialized)
        );
        assert!(
            serialized.contains(&b':'),
            "{}",
            String::from_utf8_lossy(&serialized)
        );
        parse_program(&serialized).unwrap();
    }

    #[test]
    fn reduces_integer_arithmetic() {
        assert_eq!(whnf(b"v8.4\n0\n+ #40 @ #2 @ }"), "42");
        assert_eq!(whnf(b"v8.4\n0\nsubtract #10 @ #3 @ }"), "-7");
        assert_eq!(whnf(b"v8.4\n0\nquot #22 @ #5 @ }"), "4");
        assert_eq!(whnf(b"v8.4\n0\nrem #22 @ #5 @ }"), "2");
    }

    #[test]
    fn reduces_integer_bit_ops() {
        assert_eq!(whnf(b"v8.4\n0\nand #6 @ #3 @ }"), "2");
        assert_eq!(whnf(b"v8.4\n0\nor #4 @ #1 @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nshl #3 @ #2 @ }"), "12");
        assert_eq!(whnf(b"v8.4\n0\npopcount #7 @ }"), "3");
    }

    #[test]
    fn reduces_integer_comparisons_to_microhs_bools() {
        assert_eq!(whnf(b"v8.4\n0\n== #2 @ #2 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\n< #2 @ #1 @ }"), "K");
        assert_eq!(whnf(b"v8.4\n0\nu> #-1 @ #1 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\nicmp #1 @ #2 @ }"), "K2");
        assert_eq!(whnf(b"v8.4\n0\nucmp #-1 @ #1 @ }"), "KA");
    }

    #[test]
    fn reduces_int64_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nI+ ##40 @ ##2 @ }"), "42i64");
        assert_eq!(whnf(b"v8.4\n0\nIsubtract ##10 @ ##3 @ }"), "-7i64");
        assert_eq!(whnf(b"v8.4\n0\nIquot ##22 @ ##5 @ }"), "4i64");
        assert_eq!(whnf(b"v8.4\n0\nIand ##6 @ ##3 @ }"), "2i64");
        assert_eq!(whnf(b"v8.4\n0\nIshl ##3 @ #2 @ }"), "12i64");
        assert_eq!(whnf(b"v8.4\n0\nIpopcount ##7 @ }"), "3");
        assert_eq!(whnf(b"v8.4\n0\nI== ##2 @ ##2 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\nIu> ##-1 @ ##1 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\nIicmp ##1 @ ##2 @ }"), "K2");
        assert_eq!(whnf(b"v8.4\n0\nIucmp ##-1 @ ##1 @ }"), "KA");
        assert_eq!(whnf(b"v8.4\n0\nitoI #7 @ }"), "7i64");
        assert_eq!(whnf(b"v8.4\n0\nItoi ##7 @ }"), "7");
    }

    #[test]
    fn reduces_float64_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nd+ &1.5 @ &2.25 @ }"), "3.75");
        assert_eq!(whnf(b"v8.4\n0\nd* &3 @ &2.5 @ }"), "7.5");
        assert_eq!(whnf(b"v8.4\n0\ndneg &1.5 @ }"), "-1.5");
        assert_eq!(whnf(b"v8.4\n0\nd< &1.5 @ &2.25 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\nd== &1.5 @ &2.25 @ }"), "K");
        assert_eq!(whnf(b"v8.4\n0\nitod #7 @ }"), "7.0");
        assert_eq!(whnf(b"v8.4\n0\nItod ##7 @ }"), "7.0");
        assert_eq!(whnf(b"v8.4\n0\ndtoi &7.75 @ }"), "7");
        assert_eq!(whnf(b"v8.4\n0\nd> utod #-1 @ @ &1000 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\ntoDbl fromDbl &1.5 @ @ }"), "1.5");
    }

    #[test]
    fn reduces_float32_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nf+ &&1.5 @ &&2.25 @ }"), "3.75f");
        assert_eq!(whnf(b"v8.4\n0\nf* &&3 @ &&2.5 @ }"), "7.5f");
        assert_eq!(whnf(b"v8.4\n0\nfneg &&1.5 @ }"), "-1.5f");
        assert_eq!(whnf(b"v8.4\n0\nf< &&1.5 @ &&2.25 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\nf== &&1.5 @ &&2.25 @ }"), "K");
        assert_eq!(whnf(b"v8.4\n0\nitof #7 @ }"), "7.0f");
        assert_eq!(whnf(b"v8.4\n0\nItof ##7 @ }"), "7.0f");
        assert_eq!(whnf(b"v8.4\n0\nftoi &&7.75 @ }"), "7");
        assert_eq!(whnf(b"v8.4\n0\nf> utof #-1 @ @ &&1000 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\nftod dtof &1.5 @ @ }"), "1.5");
        assert_eq!(whnf(b"v8.4\n0\ntoFlt fromFlt &&1.5 @ @ }"), "1.5f");
    }

    #[test]
    fn reduces_bytestring_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nbs++ \"foo\" @ \"bar\" @ }"), "\"foobar\"");
        assert_eq!(whnf(b"v8.4\n0\nbs++. \"foo\" @ \"bar\" @ }"), "\"foo.bar\"");
        assert_eq!(whnf(b"v8.4\n0\nbs== \"x\" @ \"x\" @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\nbs< \"abc\" @ \"abd\" @ }"), "A");
        assert_eq!(whnf(b"v8.4\n0\nbscmp \"abd\" @ \"abc\" @ }"), "KA");
        assert_eq!(whnf(b"v8.4\n0\nbslength \"hello\" @ }"), "5");
        assert_eq!(whnf(b"v8.4\n0\nbsreplicate #3 @ #65 @ }"), "\"AAA\"");
        assert_eq!(whnf(b"v8.4\n0\nbsindex \"ABC\" @ #1 @ }"), "66");
        assert_eq!(
            whnf(b"v8.4\n0\nbssubstr \"abcdef\" @ #2 @ #3 @ }"),
            "\"cde\""
        );
        assert_eq!(whnf(b"v8.4\n0\nheadUTF8 $2 \xc3\xa5 @ }"), "229");
        assert_eq!(whnf(b"v8.4\n0\ntailUTF8 $3 \xc3\xa5x @ }"), "\"x\"");
        assert_eq!(whnf(b"v8.4\n0\nbsunpack \"AB\" @ #0 @ K @ }"), "65");
        assert_eq!(
            whnf(b"v8.4\n0\nbsunpack \"AB\" @ #0 @ A @ #0 @ K @ }"),
            "66"
        );
        assert_eq!(whnf(b"v8.4\n0\nfromUTF8 $3 \xc3\xa5x @ #0 @ K @ }"), "229");
        assert_eq!(
            whnf(b"v8.4\n0\nfromUTF8 $3 \xc3\xa5x @ #0 @ A @ #0 @ K @ }"),
            "120"
        );
        assert_eq!(whnf(b"v8.4\n0\nfromUTF8 $2 \xc0\x80 @ #0 @ K @ }"), "0");
        assert_eq!(whnf(b"v8.4\n0\nfromUTF8 $1 \xc3 @ }"), "K");

        let mut program = parse_program(b"v8.4\n0\nfromUTF8 $2 \xc1\x81 @ }").unwrap();
        assert!(matches!(
            program.reduce_whnf(100),
            Err(EvalError::InvalidByteString)
        ));
    }

    #[test]
    fn reduces_mutable_bytestring_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nbsnew #2 @ #4 @ }"), "\"\\x00\\x00\"");
        assert_eq!(
            whnf(b"v8.4\n1\nseq bswrite bsnew #2 @ #4 @ :0 @ #1 @ #65 @ @ bsread _0 @ #1 @ @ }"),
            "65"
        );
        assert_eq!(
            whnf(b"v8.4\n1\nseq bsappbyte bsnew #0 @ #0 @ :0 @ #65 @ @ bsfreeze _0 @ @ }"),
            "\"A\""
        );
        assert_eq!(
            whnf(b"v8.4\n1\nseq bsappchar bsnew #0 @ #0 @ :0 @ #229 @ @ bsfreeze _0 @ @ }"),
            "\"\\xc3\\xa5\""
        );
        assert_eq!(
            whnf(b"v8.4\n1\nseq bswrite \"abc\" :0 @ #1 @ #88 @ @ bsread _0 @ #1 @ @ }"),
            "88"
        );
    }

    #[test]
    fn reduces_strict_alias_and_probe_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nord #65 @ }"), "65");
        assert_eq!(whnf(b"v8.4\n0\nchr #65 @ }"), "65");
        assert_eq!(whnf(b"v8.4\n0\nseq + #1 @ #2 @ @ #9 @ }"), "9");
        assert_eq!(whnf(b"v8.4\n0\nisint #7 @ }"), "7");
        assert_eq!(whnf(b"v8.4\n0\nisint \"x\" @ }"), "-1");
    }

    #[test]
    fn rejects_unknown_primitives() {
        assert!(matches!(
            parse_program(b"v8.4\n0\nnot-a-prim }"),
            Err(ParseError::UnknownPrim(name)) if name == "not-a-prim"
        ));

        let mut unsupported = parse_program(b"v8.4\n0\nIO.fork #1 @ }").unwrap();
        assert!(matches!(
            unsupported.reduce_whnf(10),
            Err(EvalError::UnknownPrim(name)) if name == "IO.fork"
        ));
    }

    #[test]
    fn reduces_rnf_and_exception_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nrnf #0 @ O #1 @ #2 @ @ }"), "I");
        assert_eq!(whnf(b"v8.4\n0\nrnf #1 @ raise #7 @ @ }"), "I");

        let mut program = parse_program(b"v8.4\n0\nraise #7 @ }").unwrap();
        assert!(matches!(
            program.reduce_whnf(100),
            Err(EvalError::Raised(_))
        ));

        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO catch IO.return #5 @ @ K IO.return #42 @ @ @ @ }"),
            "5"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO catch raise #7 @ @ K IO.return #42 @ @ @ @ }"),
            "42"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO catch IO.strict IO.return @ quot #1 @ #0 @ @ @ IO.return @ @ }"),
            "4"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO catch IO.strict IO.return @ + #9223372036854775807 @ #1 @ @ @ IO.return @ @ }"),
            "7"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO catch IO.strict IO.return @ Iuquot ##1 @ ##0 @ @ @ IO.return @ @ }"),
            "4"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO catch IO.strict IO.return @ Ineg ##-9223372036854775808 @ @ @ IO.return @ @ }"),
            "7"
        );

        let mut program = parse_program(b"v8.4\n0\nshl #1 @ #64 @ }").unwrap();
        assert!(matches!(
            program.reduce_whnf(100),
            Err(EvalError::InvalidShift(64))
        ));
    }

    #[test]
    fn formats_uncaught_rts_exceptions_like_c() {
        let mut program = parse_program(b"v8.4\n0\nraise #4 @ }").unwrap();
        let Err(EvalError::Raised(exn)) = program.reduce_whnf(100) else {
            panic!("raise did not produce an exception");
        };
        assert_eq!(
            program.uncaught_exception_message_bytes(exn).unwrap(),
            b"DivideByZero"
        );
    }

    #[test]
    fn reduces_stable_pointer_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nSPnew #42 @ }"), "1");
        assert_eq!(whnf(b"v8.4\n0\nSPderef SPnew #42 @ @ }"), "42");
        assert_eq!(
            whnf(b"v8.4\n2\nseq SPfree SPnew #1 @ :0 @ @ SPnew #2 @ :1 @ }"),
            "1"
        );
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO SPnew #42 @ @ }"), "1");
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO SPderef SPnew #42 @ @ @ }"),
            "42"
        );
    }

    #[test]
    fn reduces_pointer_conversion_primitives() {
        assert_eq!(whnf(b"v8.4\n0\ntoPtr #42 @ }"), "Ptr#42");
        assert_eq!(whnf(b"v8.4\n0\ntoInt toPtr #42 @ @ }"), "42");
        assert_eq!(whnf(b"v8.4\n0\ntoFunPtr #7 @ }"), "FunPtr#7");
        assert_eq!(whnf(b"v8.4\n0\ntoInt toFunPtr #7 @ @ }"), "7");
        assert_eq!(whnf(b"v8.4\n0\ntoInt toPtr toFunPtr #9 @ @ @ }"), "9");
    }

    #[test]
    fn reduces_foreign_pointer_primitives() {
        assert_eq!(
            whnf(b"v8.4\n0\nfp2bs bs2fp \"abcdef\" @ @ #3 @ }"),
            "\"abc\""
        );
        assert_eq!(
            whnf(b"v8.4\n0\nfp2bs fp+ bs2fp \"abcdef\" @ @ #2 @ @ #3 @ }"),
            "\"cde\""
        );
        assert_eq!(
            whnf(b"v8.4\n1\n== toInt fp2p bs2fp \"abc\" :0 @ @ @ @ toInt fp2p bs2fp _0 @ @ @ @ }"),
            "A"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO fpnew toPtr #42 @ @ @ }"),
            "ForeignPtr#42"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nseq fpfin toFunPtr #0 @ @ bs2fp \"abc\" @ @ @ #7 @ }"),
            "7"
        );
    }

    #[test]
    fn reduces_weak_pointer_primitives() {
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.lazyBind Wknew #0 @ #42 @ @ Wkderef @ @ #0 @ I @ }"),
            "42"
        );
        assert_eq!(
            whnf(b"v8.4\n1\nseq Wkfinal Wknew #0 @ #42 @ :0 @ @ IO.performIO Wkderef _0 @ @ #0 @ I @ @ }"),
            "42"
        );
    }

    #[test]
    fn weak_gc_clears_value_when_key_unreachable() {
        let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
        let key = program.push_node(Node::Int(123_456));
        let value = program.push_node(Node::Int(42));
        let weak = program.new_weak_ptr(key, value, None);

        collect_for_test(&mut program, weak);

        match program.cold_node(weak) {
            Some(Node::Weak(weak)) => {
                assert_eq!(weak.key, None);
                assert_eq!(weak.value, None);
            }
            other => panic!("expected weak node, got {other:?}"),
        }
        let deref = program.deref_weak_ptr(weak).unwrap();
        assert_eq!(program.render(deref), "K");
    }

    #[test]
    fn weak_gc_keeps_value_when_key_reachable() {
        let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
        let key = program.push_node(Node::Int(123_456));
        let value = program.push_node(Node::Int(42));
        let weak = program.new_weak_ptr(key, value, None);
        let root = program.app(weak, key);

        collect_for_test(&mut program, root);

        match program.cold_node(weak) {
            Some(Node::Weak(weak)) => {
                assert_eq!(weak.key, Some(key));
                assert_eq!(weak.value, Some(value));
            }
            other => panic!("expected weak node, got {other:?}"),
        }
        let deref = program.deref_weak_ptr(weak).unwrap();
        assert_eq!(
            program.cell(deref).app_fields().map(|(_, arg)| arg),
            Some(value)
        );
    }

    #[test]
    fn reduces_mvar_primitives() {
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.lazyBind IO.newmvar @ IO.trytakemvar @ @ #0 @ I @ }"),
            "0"
        );
        assert_eq!(
            whnf(b"v8.4\n1\nseq IO.performIO IO.newmvar @ :0 @ IO.performIO IO.>> IO.putmvar _0 @ #42 @ @ IO.takemvar _0 @ @ @ @ }"),
            "42"
        );
        assert_eq!(
            whnf(b"v8.4\n1\nseq IO.performIO IO.newmvar @ :0 @ IO.performIO IO.>> IO.putmvar _0 @ #42 @ @ IO.readmvar _0 @ @ @ @ }"),
            "42"
        );
        assert_eq!(
            whnf(b"v8.4\n1\nseq IO.performIO IO.newmvar @ :0 @ IO.performIO IO.tryputmvar _0 @ #42 @ @ @ }"),
            "A"
        );
    }

    #[test]
    fn reduces_array_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nA.alloc #3 @ #7 @ }"), "[7, 7, 7]");
        assert_eq!(whnf(b"v8.4\n0\nA.size A.alloc #3 @ #7 @ @ }"), "3");
        assert_eq!(whnf(b"v8.4\n0\nA.read A.alloc #3 @ #7 @ @ #1 @ }"), "7");
        assert_eq!(whnf(b"v8.4\n1\nA.== #0 [1] :0 @ _0 @ }"), "A");
        assert_eq!(whnf(b"v8.4\n1\nA.== #0 [1] :0 @ A.copy _0 @ @ }"), "K");
        assert_eq!(
            whnf(b"v8.4\n1\nseq A.write #0 #0 #0 [3] :0 @ #1 @ #42 @ @ A.read _0 @ #1 @ @ }"),
            "42"
        );
        assert_eq!(
            whnf(b"v8.4\n1\nseq A.trunc #0 #0 #0 [3] :0 @ #1 @ @ A.size _0 @ @ }"),
            "1"
        );
        assert_eq!(whnf(b"v8.4\n1\nA.== A.alloc #1 @ #0 @ :0 @ _0 @ }"), "A");
        assert_eq!(
            whnf(b"v8.4\n1\nseq A.write A.alloc #3 @ #0 @ :0 @ #1 @ #42 @ @ A.read _0 @ #1 @ @ }"),
            "42"
        );
    }

    #[test]
    fn reduces_io_control_primitives() {
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO IO.return #5 @ @ }"), "5");
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.>> IO.return #1 @ @ IO.return #7 @ @ @ }"),
            "7"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.>>= IO.return #1 @ @ K IO.return #7 @ @ @ @ }"),
            "7"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.lazyBind IO.return #3 @ @ IO.return @ @ }"),
            "3"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.strict IO.return #4 @ @ @ }"),
            "4"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.atomic IO.return #6 @ @ @ }"),
            "6"
        );
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO IO.gc #0 @ @ }"), "I");
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO IO.yield @ }"), "I");
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO IO.getmaskingstate @ }"), "0");
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.>> IO.setmaskingstate #2 @ @ IO.getmaskingstate @ @ }"),
            "2"
        );
        assert_eq!(whnf(b"v8.4\n0\nthnum IO.performIO IO.thid @ @ }"), "1");
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO IO.threadstatus IO.performIO IO.thid @ @ @ }"),
            "0"
        );
    }

    #[test]
    fn ignored_io_shortcut_has_depth_cap() {
        fn ignored_chain(program: &mut Program, len: usize) -> NodeId {
            let unit = program.prim("I");
            let ret = program.prim("IO.return");
            let unit_action = program.app(ret, unit);
            let result = program.push_node(Node::Int(7));
            let ret = program.prim("IO.return");
            let mut action = program.app(ret, result);
            for _ in 0..len {
                let then = program.prim("IO.>>");
                let left = program.app(then, unit_action);
                action = program.app(left, action);
            }
            action
        }

        let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
        let bounded = ignored_chain(&mut program, IGNORED_IO_SHORTCUT_RECURSION_LIMIT / 2);
        assert!(
            program
                .ignored_io_action_reductions(bounded, usize::MAX)
                .unwrap()
                .is_some()
        );

        let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
        let over_cap = ignored_chain(&mut program, IGNORED_IO_SHORTCUT_RECURSION_LIMIT + 1);
        assert_eq!(
            program
                .ignored_io_action_reductions(over_cap, usize::MAX)
                .unwrap(),
            None
        );

        let perform_io = program.prim("IO.performIO");
        program.root = program.app(perform_io, over_cap);
        let (root, _) = program.reduce_whnf(20_000).unwrap();
        assert_eq!(program.render(root), "7");
    }

    #[test]
    fn reduces_builtin_ffi_calls() {
        let is_linux = if cfg!(target_os = "linux") { "1" } else { "0" };
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^islinux @ }"), is_linux);
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^GETRAW @ }"), "-1");
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^putchar #10 @ @ }"), "I");
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^sizeof_char @ }"), "1");
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO ^sizeof_int @ }"),
            std::mem::size_of::<std::os::raw::c_int>().to_string()
        );
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^want_gmp @ }"), "0");
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^want_imath @ }"), "1");
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO dynsym \"islinux\" @ @ }"),
            is_linux
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO dynsym O #105 @ O #115 @ O #108 @ O #105 @ O #110 @ O #117 @ O #120 @ K @ @ @ @ @ @ @ @ @ }"),
            is_linux
        );

        let mut program = parse_program(b"v8.4\n0\nIO.performIO ^does_not_exist @ }").unwrap();
        assert!(matches!(
            program.reduce_whnf(100),
            Err(EvalError::UnknownFfi(name)) if name == "does_not_exist"
        ));

        let mut program = parse_program(b"v8.4\n0\nIO.performIO ^GETTIMEMICRO @ }").unwrap();
        let (root, _) = program.reduce_whnf(100).unwrap();
        let root = program.resolve(root).unwrap();
        match program.node_for_debug(root) {
            Node::Int(n) => assert!(n >= 0),
            _ => panic!("GETTIMEMICRO did not return an Int"),
        }
    }

    #[test]
    fn lz77c_ffi_compresses_to_guest_buffer() {
        let mut program = parse_program(b"v8.4\n0\nI }\n").unwrap();
        let input = b"AAAAAAAAAAAAAAAAzzzzzzzzzzzzzzzz";
        let src = program.alloc_memory(input.len()).unwrap();
        program.write_pointer_bytes(src, input).unwrap();
        let out_ptr = program.alloc_memory(8).unwrap();

        let src_node = program.push_node(Node::Ptr(src));
        let len_node = program.push_node(Node::Int(input.len() as i64));
        let out_ptr_node = program.push_node(Node::Ptr(out_ptr));
        let world = program.prim("I");
        let Some((used, pair)) = program
            .ffi_call("lz77c", &[src_node, len_node, out_ptr_node, world])
            .unwrap()
        else {
            panic!("lz77c did not reduce");
        };
        assert_eq!(used, 4);

        let Some((compressed_len, returned_world)) = program.pair_fields(pair).unwrap() else {
            panic!("lz77c did not return a pair");
        };
        assert_eq!(returned_world, world);
        let compressed_len = match program.node_for_debug(compressed_len) {
            Node::Int(n) => usize::try_from(n).unwrap(),
            other => panic!("lz77c length returned {other:?}"),
        };
        let compressed_ptr = program.peek_signed(out_ptr, 8).unwrap();
        let compressed = program
            .read_pointer_bytes(compressed_ptr, compressed_len)
            .unwrap();
        assert_eq!(lz77_decompress(&compressed).unwrap(), input);
    }

    #[test]
    fn decodes_c_runtime_compression_fixtures() {
        const INPUT: &[u8] = b"AAAAAAAAAAAAAAAABABABABABABA\xff\0end";
        const C_LZ77: &[u8] = &[
            76, 90, 49, 16, 0, 0, 0, 0, 65, 224, 0, 6, 0, 66, 224, 1, 2, 4, 255, 0, 101, 110, 100,
        ];
        const C_BWT: &[u8] = &[
            66, 87, 49, 33, 0, 0, 0, 1, 0, 0, 0, 255, 100, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 66, 66, 66, 66, 66, 66, 65, 65, 65, 65, 65, 65, 110, 0, 101, 65,
        ];
        const C_LZMA: &[u8] = &[
            76, 90, 50, 28, 0, 0, 0, 93, 0, 0, 0, 1, 33, 0, 0, 0, 0, 0, 0, 0, 0, 32, 237, 68, 84,
            65, 127, 132, 12, 164, 143, 145, 248, 248, 0,
        ];

        assert_eq!(&C_LZ77[..3], b"LZ1");
        let lz77_len = u32::from_le_bytes(C_LZ77[3..7].try_into().unwrap()) as usize;
        assert_eq!(lz77_len, C_LZ77.len() - 7);
        assert_eq!(lz77_decompress(&C_LZ77[7..]).unwrap(), INPUT);
        assert_eq!(
            lz77_decompress(&lz77_compress(INPUT).unwrap()).unwrap(),
            INPUT
        );

        assert_eq!(&C_BWT[..3], b"BW1");
        let bwt_len = u32::from_le_bytes(C_BWT[3..7].try_into().unwrap()) as usize;
        let bwt_zero = u32::from_le_bytes(C_BWT[7..11].try_into().unwrap()) as usize;
        assert_eq!(bwt_len, C_BWT.len() - 11);
        assert_eq!(bwt_decode(&C_BWT[11..], bwt_zero).unwrap(), INPUT);
        let (rust_bwt_zero, rust_bwt_last) = bwt_encode(INPUT).unwrap();
        assert_eq!(bwt_decode(&rust_bwt_last, rust_bwt_zero).unwrap(), INPUT);

        assert_eq!(&C_LZMA[..3], b"LZ2");
        let lzma_len = u32::from_le_bytes(C_LZMA[3..7].try_into().unwrap()) as usize;
        assert_eq!(lzma_len, C_LZMA.len() - 7);
        assert_eq!(lzma_decompress_payload(&C_LZMA[7..]).unwrap(), INPUT);
        assert_eq!(
            lzma_decompress_payload(&lzma_compress_payload(INPUT).unwrap()).unwrap(),
            INPUT
        );
    }

    #[test]
    fn reduces_math_ffi_calls() {
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^sqrt &9 @ @ }"), "3.0");
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^pow &2 @ &8 @ @ }"), "256.0");
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO ^scalbn &1.5 @ #2 @ @ }"),
            "6.0"
        );
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO ^sqrtf &&9 @ @ }"), "3.0f");
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO ^powf &&2 @ &&8 @ @ }"),
            "256.0f"
        );
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO ^scalbnf &&1.5 @ #2 @ @ }"),
            "6.0f"
        );
    }

    #[test]
    fn reduces_array_primitives_as_io_actions() {
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO A.alloc #3 @ #7 @ @ }"),
            "[7, 7, 7]"
        );
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO A.size #0 #0 [2] @ @ }"), "2");
        assert_eq!(whnf(b"v8.4\n0\nIO.performIO A.copy #0 [1] @ @ }"), "[0]");
        assert_eq!(
            whnf(b"v8.4\n1\nIO.performIO IO.>> A.write #0 #0 #0 [3] :0 @ #1 @ #42 @ @ A.read _0 @ #1 @ @ @ }"),
            "42"
        );
    }

    #[test]
    fn reduces_mutable_bytestring_primitives_as_io_actions() {
        assert_eq!(
            whnf(b"v8.4\n0\nIO.performIO bsnew #2 @ #4 @ @ }"),
            "\"\\x00\\x00\""
        );
        assert_eq!(
            whnf(b"v8.4\n1\nIO.performIO IO.>> bswrite bsnew #2 @ #4 @ :0 @ #1 @ #65 @ @ bsread _0 @ #1 @ @ @ }"),
            "65"
        );
        assert_eq!(
            whnf(b"v8.4\n1\nIO.performIO IO.>> bsappbyte bsnew #0 @ #0 @ :0 @ #65 @ @ bsfreeze _0 @ @ @ }"),
            "\"A\""
        );
    }
}
