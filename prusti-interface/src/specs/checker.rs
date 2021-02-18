use log::{info, debug};
use rustc_middle::{
    hir::map::Map,
    ty::TyCtxt,
};
use rustc_span::{Span, MultiSpan};
use rustc_hir::{
    self as hir,
    intravisit::{self, Visitor},
    itemlikevisit::ItemLikeVisitor,
    def_id::{DefId, LocalDefId},
};

use std::collections::HashMap;

use crate::{
    environment::Environment,
    utils::has_prusti_attr,
    PrustiError,
};


pub struct SpecChecker<'tcx> {
    tcx: TyCtxt<'tcx>,

    predicates: HashMap<hir::HirId, Span>,
    // span of call and definition of predicates illegally called
    pred_calls: Vec<(Span, Span)>,
}

// shallow visitor, just to collect all function items that originate from
// predicates
impl<'tcx> ItemLikeVisitor<'tcx> for SpecChecker<'tcx> {
    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) {
        // collect DefIds for all predicate function items
        debug!("Predicate search item: {:#?}", item);
        if has_prusti_attr(item.attrs, "pred_spec_id_ref") {
            self.predicates.insert(item.hir_id, item.span);
            debug!("Predicate search item found: {:?}", item.hir_id);
        }
    }

    fn visit_trait_item(&mut self, _: &'tcx rustc_hir::TraitItem<'tcx>) {
        // nothing here
    }

    fn visit_impl_item(&mut self, _: &'tcx rustc_hir::ImplItem<'tcx>) {
        // nothing here
    }

    fn visit_foreign_item(&mut self, _: &'tcx rustc_hir::ForeignItem<'tcx>) {
        // nothing here
    }
}

// deep visitor, check any calls
impl<'tcx> Visitor<'tcx> for SpecChecker<'tcx> {
    type Map = Map<'tcx>;

    fn nested_visit_map(&mut self) -> intravisit::NestedVisitorMap<Self::Map> {
        intravisit::NestedVisitorMap::All(self.tcx.hir())
    }

    fn visit_expr(&mut self, ex: &'tcx rustc_hir::Expr<'tcx>) {
        debug!("spec-checking expr {:#?}", ex);
        if let rustc_hir::ExprKind::Call(ref callee_expr, _) = ex.kind {
            // TODO: we need to do more here, see external.rs:104ff and external.rs:130ff
            if let Some(pred_def_span) = self.predicates.get(&callee_expr.hir_id) {
                info!("found predicate call");
                self.pred_calls.push((ex.span, *pred_def_span));
            }
        }

        intravisit::walk_expr(self, ex);
    }

    fn visit_fn(
        &mut self,
        fk: intravisit::FnKind<'tcx>,
        fd: &'tcx hir::FnDecl<'tcx>,
        b: hir::BodyId,
        s: Span,
        id: hir::HirId,
    ) {
        // Stop checking inside `prusti::spec_only` functions
        if has_prusti_attr(fk.attrs(), "spec_only") {
            return;
        }

        intravisit::walk_fn(self, fk, fd, b, s, id);
    }
}

impl<'tcx> SpecChecker<'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        Self {
            tcx,
            predicates: HashMap::new(),
            pred_calls: Vec::new(),
        }
    }

    pub fn report_errors(&self, env: &Environment<'tcx>) {
        debug!("Predicate funcs: {:?}", self.predicates);
        debug!("Predicate calls: {:?}", self.pred_calls);
        for &(call_span, def_span) in &self.pred_calls {
            PrustiError::incorrect(
                "call to predicate is not allowed".to_string(),
                MultiSpan::from_spans(vec![call_span, def_span])
            ).emit(env);
        }
    }
}
