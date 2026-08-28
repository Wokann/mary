use std::collections::{HashMap, HashSet};

use super::error::DecompileError;
use super::error::DecompileErrorExtra;
use super::DecompileState;
use super::DecompileToken;

use crate::ir::SwitchId;
use crate::{
    ast::{AssignOperation, Expr, Invoke, Stmt, SwitchCase, SwitchLayout},
    ir::{CallId, CallableShape, CaseEnum, Ins, JumpId, VarId},
};

pub(super) struct InsDecompiler<'a, 'b> {
    pub(super) state: DecompileState<'a>,
    variables: HashMap<VarId, String>,

    /// maps JumpId to the instruction location
    /// used for merging switch cases, which need to break to the same
    /// location
    jump_to_idx: HashMap<JumpId, usize>,
    switch_break_ids: HashSet<JumpId>,

    known_callables: &'b HashMap<CallId, (String, CallableShape)>,
}

impl<'a, 'b> InsDecompiler<'a, 'b> {
    pub(super) fn new(
        instructions: &'a [Ins],
        known_callables: &'b HashMap<CallId, (String, CallableShape)>,
    ) -> Self {
        Self {
            state: DecompileState::new(instructions),
            variables: HashMap::new(),
            jump_to_idx: prepare_jump_to_idx(instructions),
            switch_break_ids: prepare_switch_break_ids(instructions),
            known_callables,
        }
    }

    pub(super) fn remaining(&self) -> usize {
        self.state.input.len()
    }

    pub(super) fn at_end(&self) -> bool {
        self.remaining() == 0
    }

    /// Helper
    fn back(&self) -> &[DecompileToken] {
        &self.state.stack[..]
    }

    fn is_expr(&self, token: &DecompileToken) -> bool {
        match token {
            DecompileToken::Expr(_)
            | DecompileToken::FunctionCall(_)
            | DecompileToken::PushVar(_)
            | DecompileToken::PushInt(_) => true,
            _ => false,
        }
    }

    fn variable_name(&mut self, var_id: VarId) -> String {
        // TODO: represent ast variables with IDS and not names!!

        self.variables
            .entry(var_id)
            .or_insert(format!("var_{0}", var_id.0))
            .clone()
    }

    fn pop_expr(&mut self) -> Expr {
        match self.state.stack.pop().unwrap() {
            DecompileToken::Expr(expr) => expr,
            DecompileToken::FunctionCall(invoke) => Expr::Call(invoke),
            DecompileToken::PushVar(var_id) => Expr::Name(self.variable_name(var_id)),
            DecompileToken::PushInt(int_value) => Expr::Int(int_value),
            _ => unreachable!(),
        }
    }

    fn pop_assign_params(&mut self) -> (VarId, Expr) {
        /* pop 2 expr, left is target var number, right is value */

        let rhs = self.pop_expr();
        let lhs = self.state.stack.pop().unwrap();

        match (lhs, rhs) {
            (DecompileToken::PushInt(int_var_id), expr) => (VarId(int_var_id as usize), expr),
            (_, _) => unreachable!("should be handled by match"),
        }
    }

    fn pop_binop_params(&mut self) -> (Expr, Expr) {
        let rhs = self.pop_expr();
        let lhs = self.pop_expr();

        (lhs, rhs)
    }

    fn push_stmt(&mut self, stmt: Stmt) {
        if let Some(DecompileToken::Stmts(stmts)) = self.state.stack.last_mut() {
            stmts.push(stmt);
        } else {
            /* TODO: there's those bugged assign expr we need to take care of!!! */
            self.state.stack.push(DecompileToken::Stmts(vec![stmt]));
        }
    }

    fn match_at_cmp(&mut self) -> Result<(), DecompileError> {
        /* we are just after a cmp instruction. */

        /* we expect the following sequence:
         * expr | BRANCH(k) PUSH[0] JMP(l) LABEL(k) PUSH[1] LABEL(l) */

        // check for expression
        match self.back() {
            [.., check_expr] if self.is_expr(check_expr) => {}

            _ => {
                return Err(DecompileError::CouldntReduce);
            }
        }

        use Ins::{Cmp, Jmp, Label, PushInt};

        match &self.state.input[..] {
            [Cmp, check_branch, PushInt(0), Jmp(jump_l), Label(label_k), PushInt(1), Label(label_l), ..]
                if check_branch
                    .branch_target()
                    .map_or(false, |branch_k| branch_k == *label_k)
                    && *jump_l == *label_l =>
            {
                // make sure we got the size (7) of the input head to remove right
                assert!(matches!(self.state.input[6], Label(l) if (*jump_l == l)));
                self.state.input = &self.state.input[7..];

                let binop_exprs = self.pop_binop_params();

                self.state
                    .stack
                    .push(DecompileToken::Expr(match check_branch {
                        Ins::Blt(_) => Expr::CmpLt(Box::new(binop_exprs)),
                        Ins::Ble(_) => Expr::CmpLe(Box::new(binop_exprs)),
                        Ins::Beq(_) => Expr::CmpEq(Box::new(binop_exprs)),
                        Ins::Bne(_) => Expr::CmpNe(Box::new(binop_exprs)),
                        Ins::Bge(_) => Expr::CmpGe(Box::new(binop_exprs)),
                        Ins::Bgt(_) => Expr::CmpGt(Box::new(binop_exprs)),
                        _ => unreachable!(),
                    }));

                Ok(())
            }

            _ => Err(DecompileError::UnexpectedLookahead),
        }
    }

    fn match_at_dupe_inc(&mut self) -> Result<(), DecompileError> {
        // also checks that we indeed started with a PushVar
        let pushed_var_id = match self.back() {
            [.., DecompileToken::PushVar(var_id)] => *var_id,

            _ => {
                return Err(DecompileError::CouldntReduce);
            }
        };

        match &self.state.input[..] {
            /* post-increment (i++) */
            [Ins::Dupe, Ins::Inc, Ins::PopVar(pop_var_id), ..] if pushed_var_id == *pop_var_id => {
                self.state.input = &self.state.input[3..];

                let var_name = self.variable_name(pushed_var_id);
                let token = DecompileToken::Expr(Expr::PostIncrement(var_name));

                self.state.stack.pop();
                self.state.stack.push(token);

                Ok(())
            }

            /* post-decrement (i--) */
            [Ins::Dupe, Ins::Dec, Ins::PopVar(pop_var_id), ..] if pushed_var_id == *pop_var_id => {
                self.state.input = &self.state.input[3..];

                let var_name = self.variable_name(pushed_var_id);
                let token = DecompileToken::Expr(Expr::PostDecrement(var_name));

                self.state.stack.pop();
                self.state.stack.push(token);

                Ok(())
            }

            /* pre-increment (++i) */
            [Ins::Inc, Ins::Dupe, Ins::PopVar(pop_var_id), ..] if pushed_var_id == *pop_var_id => {
                self.state.input = &self.state.input[3..];

                let var_name = self.variable_name(pushed_var_id);
                let token = DecompileToken::Expr(Expr::PreIncrement(var_name));

                self.state.stack.pop();
                self.state.stack.push(token);

                Ok(())
            }

            /* pre-decrement (--i) */
            [Ins::Dec, Ins::Dupe, Ins::PopVar(pop_var_id), ..] if pushed_var_id == *pop_var_id => {
                self.state.input = &self.state.input[3..];

                let var_name = self.variable_name(pushed_var_id);
                let token = DecompileToken::Expr(Expr::PreDecrement(var_name));

                self.state.stack.pop();
                self.state.stack.push(token);

                Ok(())
            }

            _ => Err(DecompileError::UnexpectedLookahead),
        }
    }

    fn match_at_label(&mut self, last_label_id: JumpId) -> Result<(), DecompileError> {
        use DecompileToken::{Beq, Bne, Jump, Label, Stmts};

        /* All of these patterns are hideous. Not only that, but if I put comments
         * anywhere within them, rustfmt craps out and silently fails to format the whole match */

        match self.back() {
            /* If statement:
             * expr BEQ:k stmts | LABEL:k */
            [.., check_expr, Beq(beq_id), Stmts(_)]
                if self.is_expr(check_expr) && *beq_id == last_label_id =>
            {
                self.apply_if(true);
                Ok(())
            }

            /* If statement, body-less variant:
             * expr BEQ:k stmts | LABEL:k */
            [.., check_expr, Beq(beq_id)]
                if self.is_expr(check_expr) && *beq_id == last_label_id =>
            {
                self.apply_if(false);
                Ok(())
            }

            /* If-Else statement:
             * expr BEQ:k stmts JMP:l LABEL:k stmts | LABEL:l */
            [.., check_expr, Beq(beq_k), Stmts(_), Jump(jump_l), Label(label_k), Stmts(_)]
                if self.is_expr(check_expr) && *beq_k == *label_k && *jump_l == last_label_id =>
            {
                self.apply_if_else(true, true);
                Ok(())
            }

            /* If-Else statement, then-less variant:
             * expr BEQ:k JMP:l LABEL:k stmts | LABEL:l */
            [.., check_expr, Beq(beq_k), Jump(jump_l), Label(label_k), Stmts(_)]
                if self.is_expr(check_expr) && *beq_k == *label_k && *jump_l == last_label_id =>
            {
                self.apply_if_else(false, true);
                Ok(())
            }

            /* If-Else statement, else-less variant:
             * expr BEQ:k stmts JMP:l LABEL:k | LABEL:l */
            [.., check_expr, Beq(beq_k), Stmts(_), Jump(jump_l), Label(label_k)]
                if self.is_expr(check_expr) && *beq_k == *label_k && *jump_l == last_label_id =>
            {
                self.apply_if_else(true, false);
                Ok(())
            }

            /* If-Else statement, body-less variant:
             * expr BEQ:k JMP:l LABEL:k | LABEL:l */
            [.., check_expr, Beq(beq_k), Jump(jump_l), Label(label_k)]
                if self.is_expr(check_expr) && *beq_k == *label_k && *jump_l == last_label_id =>
            {
                self.apply_if_else(false, false);
                Ok(())
            }

            /* For statement:
             * LABEL:k expr BNE:m JMP:n
             * LABEL:l stmts JMP:k
             * LABEL:m stmts JMP:l
             * | LABEL:n */
            [.., Stmts(_), Label(label_k), check_expr, Bne(bne_m), Jump(jump_n), Label(label_l), Stmts(it_statement), Jump(jump_k), Label(label_m), Stmts(_), Jump(jump_l)]
                if self.is_expr(check_expr)
                    && jump_k == label_k
                    && *bne_m == *label_m
                    && *jump_l == *label_l
                    && it_statement.len() == 1
                    && *jump_n == last_label_id =>
            {
                self.apply_for(true);
                Ok(())
            }

            /* For statement, body-less variant:
             * LABEL:k expr BNE:m JMP:n
             * LABEL:l stmts JMP:k
             * LABEL:m JMP:l
             * | LABEL:n */
            [.., Stmts(_), Label(label_k), check_expr, Bne(bne_m), Jump(jump_n), Label(label_l), Stmts(it_statement), Jump(jump_k), Label(label_m), Jump(jump_l)]
                if self.is_expr(check_expr)
                    && jump_k == label_k
                    && *bne_m == *label_m
                    && *jump_l == *label_l
                    && it_statement.len() == 1
                    && *jump_n == last_label_id =>
            {
                self.apply_for(false);
                Ok(())
            }

            /* A reachable jump to the immediately following label has no
             * semantic branch, but remains observable in bytecode. */
            [.., Jump(jump_id)] if *jump_id == last_label_id => {
                self.state.stack.pop();
                self.push_stmt(Stmt::JumpNext);
                self.state.input = &self.state.input[1..];
                Ok(())
            }

            _ => Err(DecompileError::CouldntReduce),
        }
    }

    fn match_at_bne(&mut self, bne_jump_id: JumpId) -> Result<(), DecompileError> {
        use DecompileToken::{Case, Label, Stmts};

        match self.back() {
            /* Do-While statement:
             * LABEL:k stmts expr BNE:k */
            [.., Label(label_id), Stmts(_), check_expr]
                if self.is_expr(check_expr) && *label_id == bne_jump_id =>
            {
                self.apply_do_while(true);
                Ok(())
            }

            /* A case marker has no runtime footprint and may be placed between
             * the backward loop label and its body. Preserve the case outside
             * the reconstructed do-while. */
            [.., Label(label_id), Case(_, _), Stmts(_), check_expr]
                if self.is_expr(check_expr) && *label_id == bne_jump_id =>
            {
                self.apply_do_while_in_case();
                Ok(())
            }

            /* body-less variant
             * LABEL:k expr BNE:k */
            [.., Label(label_id), check_expr]
                if self.is_expr(check_expr) && *label_id == bne_jump_id =>
            {
                self.apply_do_while(false);
                Ok(())
            }

            _ => Err(DecompileError::CouldntReduce),
        }
    }

    fn check_match_case_breaks(
        &self,
        switch_cases: &[(SwitchCase, Option<JumpId>)],
        jump_id: JumpId,
    ) -> bool {
        let idx = self.jump_to_idx[&jump_id];

        for (_, break_id) in switch_cases {
            if let Some(break_id) = break_id {
                if idx != self.jump_to_idx[break_id] {
                    return false;
                }
            }
        }

        true
    }

    fn switch_tail_label_count(
        &self,
        switch_cases: &[(SwitchCase, Option<JumpId>)],
        past_switch_jump_id: JumpId,
    ) -> Option<usize> {
        let target_idx = self.jump_to_idx[&past_switch_jump_id];
        let labels: Vec<JumpId> = self.state.input[1..]
            .iter()
            .take_while(|ins| {
                matches!(ins, Ins::Label(id) if self.jump_to_idx.get(id) == Some(&target_idx))
            })
            .filter_map(|ins| match ins {
                Ins::Label(id) => Some(*id),
                _ => None,
            })
            .collect();

        let mut last = labels.iter().position(|id| *id == past_switch_jump_id)?;
        for (_, break_id) in switch_cases {
            if let Some(break_id) = break_id {
                last = last.max(labels.iter().position(|id| id == break_id)?);
            }
        }
        Some(last + 1)
    }

    fn append_switch_case(
        &mut self,
        switch_id: SwitchId,
        switch_case: SwitchCase,
        break_id: Option<JumpId>,
    ) {
        match self.state.stack.last_mut() {
            Some(DecompileToken::SwitchCases(existing_id, switch_cases))
                if *existing_id == switch_id =>
            {
                switch_cases.push((switch_case, break_id));
            }
            _ => self.state.stack.push(DecompileToken::SwitchCases(
                switch_id,
                vec![(switch_case, break_id)],
            )),
        }
    }

    fn match_at_jump(&mut self, jump_id: Option<JumpId>) -> Result<(), DecompileError> {
        use DecompileToken::{Case, Stmts, SwitchCases};

        /* the only full pattern that ends with a jump is a switch case block */

        match self.back() {
            [.., Case(case_switch_id, _), Stmts(_)] | [.., Case(case_switch_id, _)] => {
                let case_switch_id = *case_switch_id;
                let switch_case = self.reduce_switch_case(true, false);
                self.append_switch_case(case_switch_id, switch_case, jump_id);

                Ok(())
            }

            /* Unreachable high-level statements may be retained between the
             * final real case and the switch dispatch block. */
            [.., SwitchCases(case_switch_id, _), Stmts(_)]
                if matches!(
                    self.state.input.get(1..3),
                    Some([Ins::Label(_), Ins::Switch(id)]) if id == case_switch_id
                ) =>
            {
                let case_switch_id = *case_switch_id;
                let stmts = match self.state.stack.pop() {
                    Some(Stmts(stmts)) => stmts,
                    _ => unreachable!(),
                };
                self.append_switch_case(case_switch_id, SwitchCase::DeadJump(stmts), jump_id);
                self.state.input = &self.state.input[1..];
                Ok(())
            }

            /* Some vanilla scripts contain an unreachable jump in the middle
             * of the case sequence. Preserve it explicitly; an empty Case
             * silently loses the distinction when source is printed. */
            [.., SwitchCases(_, _)] => match &self.state.input[1..] {
                /* TODO: in case of jump -> jump, need to check if those map to the same target */
                [Ins::Case(_, _), ..] | [Ins::Jmp(_), ..] => match self.state.stack.last_mut() {
                    Some(SwitchCases(_, switch_cases)) => {
                        self.state.input = &self.state.input[1..];
                        switch_cases.push((SwitchCase::DeadJump(vec![]), jump_id));
                        Ok(())
                    }

                    _ => unreachable!(),
                },

                _ => Err(DecompileError::CouldntReduce),
            },

            _ => Err(DecompileError::CouldntReduce),
        }
    }

    fn match_at_assign(&mut self) -> Result<(), DecompileError> {
        use DecompileToken::PushInt;

        match self.back() {
            [.., PushInt(_), check_expr] if self.is_expr(check_expr) => {
                self.apply_assign();
                Ok(())
            }

            _ => Err(DecompileError::CouldntReduce),
        }
    }

    fn match_at_binop(&mut self) -> Result<(), DecompileError> {
        match self.back() {
            [.., check_lexpr, check_rexpr]
                if self.is_expr(check_lexpr) && self.is_expr(check_rexpr) =>
            {
                self.apply_binop();
                Ok(())
            }

            _ => Err(DecompileError::CouldntReduce),
        }
    }

    fn match_at_unop(&mut self) -> Result<(), DecompileError> {
        match self.back() {
            [.., check_expr] if self.is_expr(check_expr) => {
                self.apply_unop();
                Ok(())
            }

            _ => Err(DecompileError::CouldntReduce),
        }
    }

    fn match_at_discard(&mut self) -> Result<(), DecompileError> {
        use DecompileToken::AssignExpr;

        match self.back() {
            [.., DecompileToken::FunctionCall(_)] => {
                self.apply_function_call_stmt();
                Ok(())
            }

            [.., check_expr] if self.is_expr(check_expr) => {
                self.apply_expr_stmt();
                Ok(())
            }

            [.., AssignExpr(_, _, _)] => {
                self.apply_assign_stmt();
                Ok(())
            }

            _ => Err(DecompileError::CouldntReduce),
        }
    }

    fn match_at_switch(&mut self) -> Result<(), DecompileError> {
        /*
         * expr JMP:k
         * cases:s,e
         * JMP:e LABEL:k | SWITCH:s LABEL:e
         */

        // check for SWITCH:s LABEL:e sequence at input head
        let (switch_id, past_switch_jump_id) = match &self.state.input[..] {
            [Ins::Switch(switch_id), Ins::Label(jump_id), ..] => (*switch_id, *jump_id),

            _ => {
                return Err(DecompileError::UnexpectedLookahead);
            }
        };
        use DecompileToken::{Jump, Label, SwitchCases};

        match self.back() {
            [.., check_expr, Jump(jump_k), SwitchCases(sc_switch_id, switch_cases), Jump(jump_e), Label(label_k)]
                if *jump_k == *label_k
                    && self.check_match_case_breaks(switch_cases, past_switch_jump_id)
                    && *jump_e == past_switch_jump_id
                    && *sc_switch_id == switch_id
                    && self.is_expr(check_expr) =>
            {
                let tail_label_count = self
                    .switch_tail_label_count(switch_cases, past_switch_jump_id)
                    .ok_or(DecompileError::UnexpectedLookahead)?;
                self.apply_switch(tail_label_count, SwitchLayout::Standard);
                Ok(())
            }

            /* Compact switch: the final case break also serves as the tail
             * jump, so there is no additional JMP:e before LABEL:k. */
            [.., check_expr, Jump(jump_k), SwitchCases(sc_switch_id, switch_cases), Label(label_k)]
                if *jump_k == *label_k
                    && self.check_match_case_breaks(switch_cases, past_switch_jump_id)
                    && *sc_switch_id == switch_id
                    && self.is_expr(check_expr) =>
            {
                let tail_label_count = self
                    .switch_tail_label_count(switch_cases, past_switch_jump_id)
                    .ok_or(DecompileError::UnexpectedLookahead)?;
                self.apply_switch(tail_label_count, SwitchLayout::Compact);
                Ok(())
            }

            _ => Err(DecompileError::CouldntReduce),
        }
    }

    fn match_at_call(&mut self, call_id: CallId) -> Result<(), DecompileError> {
        let (name, shape) = match self.known_callables.get(&call_id) {
            Some((name, shape)) => (name, shape),

            None => {
                return Err(DecompileError::UnknownCallableID(call_id));
            }
        };

        if shape.num_parameters() > self.back().len() {
            return Err(DecompileError::CallableExpectsTooManyParameters(
                call_id,
                self.back().len(),
            ));
        }

        // stack needs to have shape.num_parameters() expression at its tail
        for i in 0..shape.num_parameters() {
            let token = &self.back()[self.back().len() - i - 1];

            if !self.is_expr(token) {
                return Err(DecompileError::CallableExpectsTooManyParameters(call_id, i));
            }
        }

        /* OK */

        self.state.input = &self.state.input[1..]; // consume CALL

        // pop invoke arguments
        let args = {
            let mut args = vec![];

            for _ in 0..shape.num_parameters() {
                args.push(self.pop_expr());
            }

            // args is [top ... bottom], but the arg order should be reverse that
            args.reverse();

            args
        };

        let invoke = Invoke::new(name.clone(), args);

        if shape.is_func() {
            /* this is either part of an expression or directly followed by DISC */
            self.state.stack.push(DecompileToken::FunctionCall(invoke));
        } else {
            /* procedure calls are never expressions */
            self.push_stmt(Stmt::Call(invoke));
        }

        Ok(())
    }

    fn apply_expr_stmt(&mut self) {
        /* expr | DISC */

        self.state.input = &self.state.input[1..]; // consume disc
        let expr = self.pop_expr();

        self.push_stmt(Stmt::Expr(expr));
    }

    fn apply_function_call_stmt(&mut self) {
        self.state.input = &self.state.input[1..]; // consume disc

        let invoke = match self.state.stack.pop() {
            Some(DecompileToken::FunctionCall(invoke)) => invoke,
            _ => unreachable!(),
        };

        self.push_stmt(Stmt::Call(invoke));
    }

    fn apply_assign_stmt(&mut self) {
        /* assignment_expr | DISC */

        self.state.input = &self.state.input[1..]; // consume disc
        let assign_expr = self.state.stack.pop().unwrap();

        let (var_id, assign_operation, expr) = match assign_expr {
            DecompileToken::AssignExpr(var_id, assign_operation, expr) => {
                (var_id, assign_operation, expr)
            }

            _ => unreachable!(),
        };

        let var_name = self.variable_name(var_id);
        self.push_stmt(Stmt::Assign(assign_operation, var_name, expr));
    }

    fn apply_if(&mut self, has_body: bool) {
        self.state.input = &self.state.input[1..]; // consume upcoming label

        let stmts = if has_body {
            self.state.stack.pop().unwrap()
        } else {
            DecompileToken::Stmts(vec![])
        };

        self.state.stack.pop(); // discard the beq

        let expr = self.pop_expr();

        let stmts = match stmts {
            DecompileToken::Stmts(stmts) => stmts,
            _ => unreachable!("should have been checked at the head match"),
        };

        // A jump at the end of the true branch to the immediately following
        // false label is the bytecode shape produced by an explicit empty
        // `else`.  Raising it here keeps the source fully structured while
        // preserving the otherwise redundant jump during recompilation.
        if matches!(stmts.last(), Some(Stmt::JumpNext)) {
            let mut then_stmts = stmts;
            then_stmts.pop();
            self.push_stmt(Stmt::IfElse(expr, then_stmts, vec![]));
        } else {
            self.push_stmt(Stmt::If(expr, stmts));
        }
    }

    fn apply_if_else(&mut self, has_then_block: bool, has_else_block: bool) {
        /* expr BEQ:k stmts JMP:l LABEL:k stmts | LABEL:l */

        use DecompileToken::Stmts;

        self.state.input = &self.state.input[1..]; // consume upcoming label

        let else_stmts = if has_else_block {
            self.state.stack.pop().unwrap()
        } else {
            DecompileToken::Stmts(vec![])
        };

        self.state.stack.pop(); // discard the label
        self.state.stack.pop(); // discard the jump

        let then_stmts = if has_then_block {
            self.state.stack.pop().unwrap()
        } else {
            DecompileToken::Stmts(vec![])
        };

        self.state.stack.pop(); // discard the beq
        let expr = self.pop_expr();

        let (then_stmts, else_stmts) = match (then_stmts, else_stmts) {
            (Stmts(then_stmts), Stmts(else_stmts)) => (then_stmts, else_stmts),
            _ => unreachable!("should have been checked at the head match"),
        };

        self.push_stmt(Stmt::IfElse(expr, then_stmts, else_stmts));
    }

    fn apply_for(&mut self, has_body: bool) {
        /* stmts LABEL expr BNE JMP LABEL stmts JMP LABEL stmts JMP | LABEL */

        use DecompileToken::*;

        self.state.input = &self.state.input[1..]; // consume upcoming label

        self.state.stack.pop(); // discard the jump(l)

        let body_stmts = if has_body {
            self.state.stack.pop().unwrap()
        } else {
            DecompileToken::Stmts(vec![])
        };

        self.state.stack.pop(); // discard the label(m)
        self.state.stack.pop(); // discard the jump(k)
        let it_stmts = self.state.stack.pop().unwrap();
        self.state.stack.pop(); // discard the label(l)
        self.state.stack.pop(); // discard the jump(n)
        self.state.stack.pop(); // discard the bne(m)
        let expr = self.pop_expr();
        self.state.stack.pop(); // discard the label(k)
        let prev_stmts = self.state.stack.pop().unwrap();

        if let (Stmts(mut prev_stmts), Stmts(mut it_stmts), Stmts(body_stmts)) =
            (prev_stmts, it_stmts, body_stmts)
        {
            let for_head = prev_stmts.pop().unwrap();
            let for_tail = it_stmts.pop().unwrap();

            prev_stmts.push(Stmt::For(Box::new((expr, for_head, for_tail, body_stmts))));
            self.state.stack.push(Stmts(prev_stmts));
        } else {
            unreachable!("should have been checked at the head match")
        }
    }

    fn apply_do_while(&mut self, has_body: bool) {
        /* LABEL:k stmts expr | BNE:k */

        self.state.input = &self.state.input[1..]; // consume upcoming bne

        let expr = self.pop_expr();

        let stmts = if has_body {
            self.state.stack.pop().unwrap()
        } else {
            DecompileToken::Stmts(vec![])
        };

        self.state.stack.pop(); // discard the label

        let stmts = match stmts {
            DecompileToken::Stmts(stmts) => stmts,
            _ => unreachable!("should have been checked at the head match"),
        };

        self.push_stmt(Stmt::DoWhile(expr, stmts));
    }

    fn apply_do_while_in_case(&mut self) {
        self.state.input = &self.state.input[1..];
        let expr = self.pop_expr();
        let stmts = match self.state.stack.pop() {
            Some(DecompileToken::Stmts(stmts)) => stmts,
            _ => unreachable!(),
        };
        let case = self.state.stack.pop().unwrap();
        self.state.stack.pop(); // loop label
        self.state.stack.push(case);
        self.push_stmt(Stmt::DoWhile(expr, stmts));
    }

    fn apply_switch(&mut self, tail_label_count: usize, layout: SwitchLayout) {
        /* expr JMP:k cases:s,e JMP:e LABEL:k | SWITCH:s LABEL:e... */

        self.state.input = &self.state.input[1 + tail_label_count..];

        self.state.stack.pop(); // discard LABEL:k
        if layout == SwitchLayout::Standard {
            self.state.stack.pop(); // discard JMP:e
        }
        let switch_cases = self.state.stack.pop().unwrap();
        self.state.stack.pop(); // discard JMP:k
        let expr = self.pop_expr();

        let cases = match switch_cases {
            DecompileToken::SwitchCases(_, cases) => {
                cases.into_iter().map(|(case, _)| case).collect()
            }

            _ => unreachable!(),
        };

        self.push_stmt(Stmt::Switch(expr, cases, SwitchId(0), layout));
    }

    fn reduce_switch_case(&mut self, consume_jump: bool, fallthrough: bool) -> SwitchCase {
        /* CASE:s stmts | JMP:e */

        if consume_jump {
            self.state.input = &self.state.input[1..];
        }

        /* TODO: has_stmts */
        let stmts = if let Some(DecompileToken::Stmts(_)) = self.state.stack.last() {
            self.state.stack.pop().unwrap()
        } else {
            DecompileToken::Stmts(vec![])
        };

        let last_case = self.state.stack.pop().unwrap();

        let case_enum = match last_case {
            DecompileToken::Case(_, case_enum) => case_enum,
            _ => unreachable!(),
        };

        let stmts = match stmts {
            DecompileToken::Stmts(stmts) => stmts,
            _ => unreachable!(),
        };

        match case_enum {
            CaseEnum::Val(case_value) => {
                /* handle multiple cases */
                let mut case_exprs = vec![Expr::Int(case_value)];

                /* TODO: check for jump */
                while let [.., DecompileToken::Case(_, CaseEnum::Val(case_value))] = self.back() {
                    case_exprs.push(Expr::Int(*case_value));
                    self.state.stack.pop();
                }

                case_exprs.reverse();

                if fallthrough {
                    SwitchCase::Fallthrough(case_exprs, stmts)
                } else {
                    SwitchCase::Case(case_exprs, stmts)
                }
            }

            CaseEnum::Default if fallthrough => SwitchCase::DefaultFallthrough(stmts),
            CaseEnum::Default => SwitchCase::Default(stmts),
        }
    }

    fn apply_binop(&mut self) {
        let binop_ins = self.state.input[0];
        self.state.input = &self.state.input[1..];

        let rexpr = self.pop_expr();
        let lexpr = self.pop_expr();

        let operands = Box::new((lexpr, rexpr));

        self.state.stack.push(DecompileToken::Expr(match binop_ins {
            Ins::Add => Expr::OpAdd(operands),
            Ins::Sub => Expr::OpSub(operands),
            Ins::Mul => Expr::OpMul(operands),
            Ins::Div => Expr::OpDiv(operands),
            Ins::Mod => Expr::OpMod(operands),
            Ins::LogicalAnd => Expr::OpAnd(operands),
            Ins::LogicalOr => Expr::OpOr(operands),
            _ => unreachable!(),
        }));
    }

    fn apply_unop(&mut self) {
        let unop_ins = self.state.input[0];
        self.state.input = &self.state.input[1..];

        let expr = self.pop_expr();
        let expr = Box::new(expr);

        self.state.stack.push(DecompileToken::Expr(match unop_ins {
            Ins::Neg => Expr::OpNeg(expr),
            Ins::LogicalNot => Expr::OpNot(expr),
            _ => unreachable!(),
        }));
    }

    fn apply_assign(&mut self) {
        /* PUSHI expr | ANY_ASSIGN */

        let assign_operation = match self.state.input[0] {
            Ins::Assign => AssignOperation::None,
            Ins::AssignAdd => AssignOperation::Add,
            Ins::AssignSub => AssignOperation::Sub,
            Ins::AssignMul => AssignOperation::Mul,
            Ins::AssignDiv => AssignOperation::Div,
            Ins::AssignMod => AssignOperation::Mod,
            _ => unreachable!(),
        };

        self.state.input = &self.state.input[1..]; // consume assign instruction
        let (var_id, expr) = self.pop_assign_params();

        let assign_expr = DecompileToken::AssignExpr(var_id, assign_operation, expr);
        self.state.stack.push(assign_expr);
    }

    pub(super) fn next(&mut self) -> Result<(), DecompileError> {
        assert!(!self.at_end());

        let ins = &self.state.input[0];

        /* ====================================
         * = INSTRUCTIONS THAT ARE KEPT AS IS =
         * ==================================== */
        match *ins {
            Ins::Assign
            | Ins::AssignAdd
            | Ins::AssignSub
            | Ins::AssignMul
            | Ins::AssignDiv
            | Ins::AssignMod => self.match_at_assign(),

            Ins::Add
            | Ins::Sub
            | Ins::Mul
            | Ins::Div
            | Ins::Mod
            | Ins::LogicalAnd
            | Ins::LogicalOr => self.match_at_binop(),

            Ins::Neg | Ins::LogicalNot => self.match_at_unop(),

            Ins::Cmp => self.match_at_cmp(),

            // any of these can only be part of a pre/post increment/decrement
            Ins::Dupe | Ins::Inc | Ins::Dec => self.match_at_dupe_inc(),

            Ins::PushVar(var_id) => {
                self.state.stack.push(DecompileToken::PushVar(var_id));
                self.state.input = &self.state.input[1..];

                Ok(())
            }

            Ins::PushInt(int_value) => {
                self.state.stack.push(DecompileToken::PushInt(int_value));
                self.state.input = &self.state.input[1..];

                Ok(())
            }

            /* tail of pre/post increment/decrement
             * should have been forward matched */
            Ins::PopVar(_) => Err(DecompileError::UnexpectedLookahead),

            Ins::Discard => self.match_at_discard(),

            Ins::Jmp(jump_id) => {
                /* JMP is either:
                 * - within a if-else statement, we need to keep it for later
                 * - within a for statement, we need to keep it for later
                 * - within a switch statement, we need to keep it for later
                 * - at the tail of a switch case
                 * - within a conditional expression, which is matched by a forward pattern at CMP */

                if let Err(DecompileError::CouldntReduce) = self.match_at_jump(Some(jump_id)) {
                    if self.switch_break_ids.contains(&jump_id)
                        && matches!(self.state.input.get(1), Some(Ins::Jmp(_)))
                    {
                        self.push_stmt(Stmt::Break);
                    } else {
                        self.state.stack.push(DecompileToken::Jump(jump_id));
                    }
                    self.state.input = &self.state.input[1..];
                }

                Ok(())
            }

            Ins::Beq(jump_id) => {
                /* BEQ is either:
                 * - within a if statement, we need to keep it for later
                 * - within a if-else statement, we need to keep it for later
                 * - within a conditional expression, which is matched by a forward pattern at CMP */

                self.state.stack.push(DecompileToken::Beq(jump_id));
                self.state.input = &self.state.input[1..];

                Ok(())
            }

            Ins::Bne(jump_id) => {
                /* BNE is either:
                 * - within a for statement, we need to keep it for later
                 * - at the tail of a do-while statement
                 * - within a conditional expression, which is matched by a forward pattern at CMP */

                if let Err(_) = self.match_at_bne(jump_id) {
                    self.state.stack.push(DecompileToken::Bne(jump_id));
                    self.state.input = &self.state.input[1..];
                }

                Ok(())
            }

            /* branches that are only part of forward-matched patterns */
            Ins::Blt(_) | Ins::Ble(_) | Ins::Bge(_) | Ins::Bgt(_) => {
                Err(DecompileError::UnexpectedLookahead)
            }

            Ins::Call(call_id) => self.match_at_call(call_id),

            // switch id check is handled within match_at_switch
            Ins::Switch(_) => self.match_at_switch(),

            Ins::Case(switch_id, case_enum) => {
                /* TODO: merge successive cases */

                let implicit_default_break = match self.back() {
                    [.., check_expr, DecompileToken::Jump(_), DecompileToken::Stmts(_), DecompileToken::Jump(break_id)]
                        if self.is_expr(check_expr) =>
                    {
                        Some(*break_id)
                    }
                    _ => None,
                };
                if let Some(break_id) = implicit_default_break {
                    self.state.stack.pop();
                    let stmts = match self.state.stack.pop() {
                        Some(DecompileToken::Stmts(stmts)) => stmts,
                        _ => unreachable!(),
                    };
                    self.append_switch_case(
                        switch_id,
                        SwitchCase::ImplicitDefault(stmts),
                        Some(break_id),
                    );
                }

                let interleaved_default = match self.back() {
                    [.., DecompileToken::SwitchCases(existing_id, _), DecompileToken::Stmts(_), DecompileToken::Jump(break_id)]
                        if *existing_id == switch_id =>
                    {
                        Some(*break_id)
                    }
                    _ => None,
                };
                if let Some(break_id) = interleaved_default {
                    self.state.stack.pop();
                    let stmts = match self.state.stack.pop() {
                        Some(DecompileToken::Stmts(stmts)) => stmts,
                        _ => unreachable!(),
                    };
                    self.append_switch_case(
                        switch_id,
                        SwitchCase::ImplicitDefault(stmts),
                        Some(break_id),
                    );
                }

                if matches!(
                    self.back(),
                    [.., DecompileToken::Case(_, _), DecompileToken::Stmts(_)]
                ) {
                    let previous_switch_id = match self.back()[self.back().len() - 2] {
                        DecompileToken::Case(id, _) => id,
                        _ => unreachable!(),
                    };
                    let switch_case = self.reduce_switch_case(false, true);
                    self.append_switch_case(previous_switch_id, switch_case, None);
                }

                let token = DecompileToken::Case(switch_id, case_enum);

                self.state.stack.push(token);
                self.state.input = &self.state.input[1..];

                Ok(())
            }

            Ins::Exit => {
                self.push_stmt(Stmt::Exit);

                if let Err(DecompileError::CouldntReduce) = self.match_at_jump(None) {
                    self.state.input = &self.state.input[1..];
                }

                Ok(())
            }

            Ins::Label(jump_id) => {
                /* LABEL is either:
                 * - within a lot of control structures (keep it)
                 * - at the tail of a if, if-else, for or switch statement
                 * - within a conditional expression, which is matched by a forward pattern at CMP */

                if let Err(DecompileError::CouldntReduce) = self.match_at_label(jump_id) {
                    self.state.stack.push(DecompileToken::Label(jump_id));
                    self.state.input = &self.state.input[1..];
                }

                Ok(())
            }
        }
    }
}

/// This is used to generate InsDecompiler::jump_to_idx
fn prepare_jump_to_idx(instructions: &[Ins]) -> HashMap<JumpId, usize> {
    let mut result = HashMap::new();
    let mut idx = 0;

    for i in 0..instructions.len() {
        match instructions[i] {
            Ins::Label(jump_id) => {
                result.insert(jump_id, idx);
            }

            /* don't increment idx here (this is not a real ins) */
            Ins::Case(_, _) => {}

            _ => {
                idx += 1;
            }
        }
    }

    result
}

fn prepare_switch_break_ids(instructions: &[Ins]) -> HashSet<JumpId> {
    let mut result = HashSet::new();
    for (index, ins) in instructions.iter().enumerate() {
        if let Ins::Switch(switch_id) = ins {
            let tail_ids: HashSet<JumpId> = instructions[index + 1..]
                .iter()
                .map_while(|tail| match tail {
                    Ins::Label(id) => Some(*id),
                    _ => None,
                })
                .collect();
            let Some(case_start) = instructions[..index]
                .iter()
                .position(|candidate| matches!(candidate, Ins::Case(id, _) if id == switch_id))
            else {
                continue;
            };
            for candidate in &instructions[case_start..index] {
                if let Ins::Jmp(target) = candidate {
                    if tail_ids.contains(target) {
                        result.insert(*target);
                    }
                }
            }
        }
    }
    result
}

pub(crate) fn decompile_instructions<'a>(
    instructions: &'a [Ins],
    known_callables: &HashMap<CallId, (String, CallableShape)>,
) -> Result<Vec<Stmt>, DecompileErrorExtra<'a>> {
    let mut ins_decompiler = InsDecompiler::new(instructions, known_callables);

    while !ins_decompiler.at_end() {
        let remaining_before = ins_decompiler.remaining();

        match ins_decompiler.next() {
            Ok(_) => {}
            Err(err) => return Err(DecompileErrorExtra(err, ins_decompiler.state)),
        }

        assert_ne!(
            remaining_before,
            ins_decompiler.remaining(),
            "{0}",
            ins_decompiler.state
        );
    }

    let mut result_state = ins_decompiler.state;

    match &result_state.stack[..] {
        [DecompileToken::Stmts(_)]
        | [DecompileToken::AssignExpr(_, _, _), DecompileToken::Stmts(_)] => {
            if matches!(
                result_state.stack.first(),
                Some(DecompileToken::AssignExpr(_, _, _))
            ) {
                let assignment = result_state.stack.remove(0);
                let mut stmts = match result_state.stack.remove(0) {
                    DecompileToken::Stmts(stmts) => stmts,
                    _ => unreachable!(),
                };
                let (var_id, op, expr) = match assignment {
                    DecompileToken::AssignExpr(var_id, op, expr) => (var_id, op, expr),
                    _ => unreachable!(),
                };
                let var_name = ins_decompiler.variables[&var_id].clone();
                stmts.insert(0, Stmt::AssignNoDisc(op, var_name, expr));
                result_state.stack.push(DecompileToken::Stmts(stmts));
            }

            match result_state.stack.remove(0) {
                DecompileToken::Stmts(mut stmts) => {
                    /* add variable declarations */

                    let num_vars = match ins_decompiler.variables.keys().max() {
                        Some(var_id) => var_id.0 + 1,
                        None => 0,
                    };

                    // let mut keys: Vec<&VarId> = ins_decompiler.variables.keys().collect();
                    // keys.sort();

                    if num_vars > 0 {
                        let mut vars = vec![];

                        for var_id in 0..num_vars {
                            let var_id = &VarId(var_id);

                            if ins_decompiler.variables.contains_key(var_id) {
                                vars.push((ins_decompiler.variables[var_id].clone(), None));
                            } else {
                                vars.push((format!("unused_{0}", var_id.0), None));
                            }
                        }

                        stmts.insert(0, Stmt::Vars(vars));
                    }

                    Ok(stmts)
                }

                _ => unreachable!(),
            }
        }

        /* empty script */
        [] => Ok(vec![]),

        _ => Err(DecompileErrorExtra(
            DecompileError::CouldntReduceAtEnd,
            result_state,
        )),
    }
}
