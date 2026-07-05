//! Eval-spine collection and graph rewrite helpers.
use super::*;

impl Program {
    pub(in crate::runtime) fn fill_eval_spine(
        &mut self,
        mut node: NodeId,
        spine: &mut EvalSpine,
    ) -> Result<NodeId, EvalError> {
        spine.clear();
        while let Some((fun, arg)) = self.cell(node).app_fields() {
            spine.push_desc(arg, node);
            node = self.resolve_profiled(fun)?;
        }
        Ok(node)
    }

    pub(in crate::runtime) fn apply_eval_spine_rewrite(
        &mut self,
        root: NodeId,
        spine: &EvalSpine,
        used: usize,
        mut node: NodeId,
    ) -> NodeId {
        let len = spine.len();
        debug_assert!(used <= len);
        if used == 0 && len == 0 {
            if node != root {
                self.set_cell_at(root.index(), Cell::indir(Some(node)));
            }
            return node;
        }
        if used > 0 {
            let redex = spine.app(used - 1);
            if node != redex {
                self.set_app_cell_at(redex.index(), Cell::indir(Some(node)));
            }
        }
        for head_idx in used..len {
            let app = spine.app(head_idx);
            let arg = spine.arg(head_idx);
            self.set_app_cell_at(app.index(), Cell::app(node, arg));
            node = app;
        }
        node
    }

    pub(in crate::runtime) fn apply_eval_spine_app(
        &mut self,
        root: NodeId,
        spine: &EvalSpine,
        used: usize,
        fun: NodeId,
        arg: NodeId,
    ) -> NodeId {
        let len = spine.len();
        debug_assert!(used <= len);
        let mut node = if used == 0 {
            self.app(fun, arg)
        } else {
            let redex = spine.app(used - 1);
            self.set_app_cell_at(redex.index(), Cell::app(fun, arg));
            redex
        };
        if used == 0 && len == 0 {
            if node != root {
                self.set_cell_at(root.index(), Cell::indir(Some(node)));
            }
            return node;
        }
        for head_idx in used..len {
            let app = spine.app(head_idx);
            let arg = spine.arg(head_idx);
            self.set_app_cell_at(app.index(), Cell::app(node, arg));
            node = app;
        }
        node
    }

    pub(in crate::runtime) fn strict_redex_from_eval_spine(
        &mut self,
        root: NodeId,
        used: usize,
        spine: &EvalSpine,
        scratch_apps: &mut Vec<NodeId>,
    ) -> StrictRedex {
        if spine.len() == used {
            return StrictRedex::Root(root);
        }
        spine.write_apps_head_order(scratch_apps);
        StrictRedex::Spine {
            root,
            used,
            apps: scratch_apps.clone(),
        }
    }
}
