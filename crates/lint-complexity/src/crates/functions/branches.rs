use syn::visit::{self, Visit};
use syn::{BinOp, Block, ExprBinary, ExprForLoop, ExprIf, ExprMatch, ExprTry, ExprWhile, Item};

use newtype::Counter;

use super::super::score::Score;

pub(super) struct Branches {
    score: Score,
}

impl Branches {
    pub(super) fn scored(body: &Block) -> Score {
        let mut scan = Branches { score: Score::new(1) };
        scan.visit_block(body);
        scan.score
    }

    fn counted(&mut self, extra: Score) {
        self.score = self.score.plus(extra);
    }
}

impl<'ast> Visit<'ast> for Branches {
    fn visit_item(&mut self, _nested: &'ast Item) {}

    fn visit_expr_if(&mut self, found: &'ast ExprIf) {
        self.counted(Score::new(1));
        visit::visit_expr_if(self, found);
    }

    fn visit_expr_while(&mut self, found: &'ast ExprWhile) {
        self.counted(Score::new(1));
        visit::visit_expr_while(self, found);
    }

    fn visit_expr_for_loop(&mut self, found: &'ast ExprForLoop) {
        self.counted(Score::new(1));
        visit::visit_expr_for_loop(self, found);
    }

    fn visit_expr_try(&mut self, found: &'ast ExprTry) {
        self.counted(Score::new(1));
        visit::visit_expr_try(self, found);
    }

    fn visit_expr_match(&mut self, found: &'ast ExprMatch) {
        self.counted(Score::new(found.arms.len().saturating_sub(1)));
        visit::visit_expr_match(self, found);
    }

    fn visit_expr_binary(&mut self, found: &'ast ExprBinary) {
        self.counted(Score::new(usize::from(matches!(found.op, BinOp::And(_) | BinOp::Or(_)))));
        visit::visit_expr_binary(self, found);
    }
}
