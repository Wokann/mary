use std::{
    collections::{HashMap, HashSet, VecDeque},
    iter::zip,
};

use crate::{
    ast::{AssignOperation, Expr, Invoke, Stmt, SwitchCase},
    const_scope::ConstScope,
    ir::{IntValue, StrValue, ValueType},
};

use super::error::DecompileError;

#[derive(Clone, Copy, Debug, PartialEq)]
enum TypeRequirement {
    Type(ValueType),
    Conflict,
}

type TypeRequirements = HashMap<String, TypeRequirement>;

fn constant_integer_value(const_scope: &ConstScope, expr: &Expr) -> Option<i64> {
    match expr {
        Expr::Int(value) => Some(*value),
        Expr::Name(name) => const_scope.const_int_value(name),
        Expr::OpAdd(pair) => constant_integer_value(const_scope, &pair.0)?
            .checked_add(constant_integer_value(const_scope, &pair.1)?),
        Expr::OpSub(pair) => constant_integer_value(const_scope, &pair.0)?
            .checked_sub(constant_integer_value(const_scope, &pair.1)?),
        Expr::OpMul(pair) => constant_integer_value(const_scope, &pair.0)?
            .checked_mul(constant_integer_value(const_scope, &pair.1)?),
        Expr::OpDiv(pair) => constant_integer_value(const_scope, &pair.0)?
            .checked_div(constant_integer_value(const_scope, &pair.1)?),
        Expr::OpMod(pair) => constant_integer_value(const_scope, &pair.0)?
            .checked_rem(constant_integer_value(const_scope, &pair.1)?),
        Expr::OpNeg(inner) => constant_integer_value(const_scope, inner)?.checked_neg(),
        _ => None,
    }
}

fn invoke_parameter_types(
    const_scope: &ConstScope,
    invoke: &Invoke,
    static_parameter_types: &[ValueType],
) -> Vec<ValueType> {
    static_parameter_types
        .iter()
        .copied()
        .enumerate()
        .map(|(target_parameter_index, static_type)| {
            invoke
                .args
                .iter()
                .enumerate()
                .find_map(|(discriminator_parameter_index, argument)| {
                    let discriminator_value = constant_integer_value(const_scope, argument)?;
                    const_scope.dependent_callable_parameter_type(
                        &invoke.func,
                        discriminator_parameter_index,
                        discriminator_value,
                        target_parameter_index,
                    )
                })
                .unwrap_or(static_type)
        })
        .collect()
}

/// Propagates a type required by a later use back to the local definitions
/// which can reach that use. This is deliberately separate from the forward
/// decorator: a VM local often receives a raw integer in a switch branch and
/// only acquires a semantic domain when it is passed to a typed callable much
/// later.
struct LocalTypeBackpropagator<'a> {
    const_scope: &'a ConstScope,
    definition_hints: HashMap<String, VecDeque<Option<TypeRequirement>>>,
    explicit_local_types: &'a HashMap<String, ValueType>,
}

impl<'a> LocalTypeBackpropagator<'a> {
    fn new(
        const_scope: &'a ConstScope,
        explicit_local_types: &'a HashMap<String, ValueType>,
    ) -> Self {
        Self {
            const_scope,
            definition_hints: HashMap::new(),
            explicit_local_types,
        }
    }

    fn add_requirement(
        &self,
        requirements: &mut TypeRequirements,
        name: &str,
        value_type: ValueType,
    ) {
        if !matches!(value_type, ValueType::UserType(_)) {
            return;
        }
        requirements
            .entry(name.to_owned())
            .and_modify(|requirement| {
                if let TypeRequirement::Type(existing) = *requirement {
                    *requirement = self
                        .const_scope
                        .narrower_value_type(existing, value_type)
                        .map_or(TypeRequirement::Conflict, TypeRequirement::Type);
                }
            })
            .or_insert(TypeRequirement::Type(value_type));
    }

    fn merge(&self, paths: impl IntoIterator<Item = TypeRequirements>) -> TypeRequirements {
        let mut merged = TypeRequirements::new();
        for path in paths {
            for (name, requirement) in path {
                match requirement {
                    TypeRequirement::Type(value_type) => {
                        self.add_requirement(&mut merged, &name, value_type);
                    }
                    TypeRequirement::Conflict => {
                        merged.insert(name, TypeRequirement::Conflict);
                    }
                }
            }
        }
        merged
    }

    fn expr_type(&self, expr: &Expr) -> ValueType {
        match expr {
            Expr::Call(invoke) if invoke.func == "VarGet" => invoke
                .args
                .first()
                .and_then(|argument| match argument {
                    Expr::Int(variable_id) => Some(*variable_id),
                    Expr::Name(name) => self.const_scope.const_int_value(name),
                    _ => None,
                })
                .and_then(|variable_id| self.const_scope.variable_value_type(variable_id))
                .unwrap_or(ValueType::Integer),
            Expr::Name(name) => self
                .const_scope
                .constant_value_type(name)
                .unwrap_or(ValueType::Undefined),
            Expr::Int(_) => ValueType::Integer,
            Expr::Str(_) => ValueType::String,
            Expr::Call(invoke) => {
                for (parameter_index, argument) in invoke.args.iter().enumerate() {
                    let parameter_value = match argument {
                        Expr::Int(value) => Some(*value),
                        Expr::Name(name) => self.const_scope.const_int_value(name),
                        _ => None,
                    };
                    if let Some(value_type) = parameter_value.and_then(|parameter_value| {
                        self.const_scope.dependent_callable_return_type(
                            &invoke.func,
                            parameter_index,
                            parameter_value,
                        )
                    }) {
                        return value_type;
                    }
                }
                self.const_scope
                    .callable_map()
                    .get(&invoke.func)
                    .map_or(ValueType::Undefined, |(_, shape)| shape.return_type())
            }
            _ => ValueType::Undefined,
        }
    }

    fn require_expr(
        &self,
        expr: &mut Expr,
        value_type: ValueType,
        requirements: &mut TypeRequirements,
    ) {
        match expr {
            Expr::Name(name) if self.const_scope.const_int_value(name).is_none() => {
                self.add_requirement(requirements, name, value_type);
            }
            Expr::Int(value) => {
                if let ValueType::UserType(type_id) = value_type {
                    if let Some(name) = self.const_scope.typed_int_const_name(type_id, *value) {
                        *expr = Expr::Name(name.to_owned());
                    } else if let Some(wrapper) = self.const_scope.typed_identity_name(type_id) {
                        *expr =
                            Expr::Call(Invoke::new(wrapper.to_owned(), vec![Expr::Int(*value)]));
                    }
                }
            }
            Expr::OpNeg(inner) => {
                if let ValueType::UserType(type_id) = value_type {
                    let positive = match &**inner {
                        Expr::Int(value) => Some(*value),
                        Expr::Name(name) => self.const_scope.const_int_value(name),
                        _ => None,
                    };
                    if let Some(value) = positive.and_then(i64::checked_neg) {
                        if let Some(name) = self.const_scope.typed_int_const_name(type_id, value) {
                            *expr = Expr::Call(Invoke::new(
                                "mary_negated_int".to_owned(),
                                vec![Expr::Name(name.to_owned())],
                            ));
                            return;
                        } else if let Some(wrapper) = self.const_scope.typed_identity_name(type_id)
                        {
                            *expr = Expr::Call(Invoke::new(
                                wrapper.to_owned(),
                                vec![Expr::Call(Invoke::new(
                                    "mary_negated_int".to_owned(),
                                    vec![Expr::Int(value)],
                                ))],
                            ));
                            return;
                        }
                    }
                }
                self.visit_expr(expr, requirements);
            }
            _ => self.visit_expr(expr, requirements),
        }
    }

    fn visit_invoke(&self, invoke: &mut Invoke, requirements: &mut TypeRequirements) {
        let Some((_, shape)) = self.const_scope.callable_map().get(&invoke.func) else {
            for argument in &mut invoke.args {
                self.visit_expr(argument, requirements);
            }
            return;
        };
        let parameter_types =
            invoke_parameter_types(self.const_scope, invoke, shape.parameter_types());
        for (argument, parameter_type) in zip(&mut invoke.args, parameter_types) {
            self.require_expr(argument, parameter_type, requirements);
        }
    }

    fn visit_expr(&self, expr: &mut Expr, requirements: &mut TypeRequirements) {
        match expr {
            Expr::Call(invoke) => self.visit_invoke(invoke, requirements),
            Expr::CmpEq(pair)
            | Expr::CmpNe(pair)
            | Expr::CmpLt(pair)
            | Expr::CmpLe(pair)
            | Expr::CmpGe(pair)
            | Expr::CmpGt(pair) => {
                let left_type = self.expr_type(&pair.0);
                let right_type = self.expr_type(&pair.1);
                self.require_expr(&mut pair.0, right_type, requirements);
                self.require_expr(&mut pair.1, left_type, requirements);
            }
            Expr::OpAdd(pair)
            | Expr::OpSub(pair)
            | Expr::OpMul(pair)
            | Expr::OpDiv(pair)
            | Expr::OpMod(pair)
            | Expr::OpOr(pair)
            | Expr::OpAnd(pair) => {
                self.visit_expr(&mut pair.0, requirements);
                self.visit_expr(&mut pair.1, requirements);
            }
            Expr::OpNeg(inner) | Expr::OpNot(inner) => self.visit_expr(inner, requirements),
            Expr::Name(_)
            | Expr::Int(_)
            | Expr::Str(_)
            | Expr::PostIncrement(_)
            | Expr::PreIncrement(_)
            | Expr::PostDecrement(_)
            | Expr::PreDecrement(_) => {}
        }
    }

    fn transfer_definition(
        &mut self,
        name: &str,
        expr: &mut Expr,
        requirements: &mut TypeRequirements,
    ) {
        let requirement = self
            .explicit_local_types
            .get(name)
            .copied()
            .map(TypeRequirement::Type)
            .or_else(|| requirements.remove(name))
            .or_else(|| {
                let value_type = self.expr_type(expr);
                matches!(value_type, ValueType::UserType(_))
                    .then_some(TypeRequirement::Type(value_type))
            });
        self.definition_hints
            .entry(name.to_owned())
            .or_default()
            .push_front(requirement);
        if let Some(TypeRequirement::Type(value_type)) = requirement {
            self.require_expr(expr, value_type, requirements);
        } else {
            self.visit_expr(expr, requirements);
        }
    }

    fn visit_stmts(
        &mut self,
        stmts: &mut [Stmt],
        mut requirements: TypeRequirements,
    ) -> TypeRequirements {
        for stmt in stmts.iter_mut().rev() {
            requirements = self.visit_stmt(stmt, requirements);
        }
        requirements
    }

    fn visit_stmt(
        &mut self,
        stmt: &mut Stmt,
        mut requirements: TypeRequirements,
    ) -> TypeRequirements {
        match stmt {
            Stmt::Vars(items) => {
                for (name, initializer) in items.iter_mut().rev() {
                    if let Some(initializer) = initializer {
                        self.transfer_definition(name, initializer, &mut requirements);
                    } else {
                        requirements.remove(name);
                    }
                }
            }
            Stmt::Consts(_) | Stmt::Ir(_) => {}
            Stmt::Assign(operation, name, expr) | Stmt::AssignNoDisc(operation, name, expr) => {
                if *operation == AssignOperation::None {
                    self.transfer_definition(name, expr, &mut requirements);
                } else {
                    requirements.remove(name);
                    self.visit_expr(expr, &mut requirements);
                }
            }
            Stmt::Expr(expr) => self.visit_expr(expr, &mut requirements),
            Stmt::Call(invoke) => self.visit_invoke(invoke, &mut requirements),
            Stmt::If(condition, body) => {
                let entered = self.visit_stmts(body, requirements.clone());
                requirements = self.merge([requirements, entered]);
                self.visit_expr(condition, &mut requirements);
            }
            Stmt::IfElse(condition, yes, no) => {
                let no = self.visit_stmts(no, requirements.clone());
                let yes = self.visit_stmts(yes, requirements);
                requirements = self.merge([yes, no]);
                self.visit_expr(condition, &mut requirements);
            }
            Stmt::Switch(selector, cases, _, _) => {
                let mut case_type = None;
                let mut case_type_conflict = false;
                for case in cases.iter() {
                    let values = match case {
                        SwitchCase::Case(values, _) | SwitchCase::Fallthrough(values, _) => values,
                        _ => continue,
                    };
                    for value in values {
                        let value_type = self.expr_type(value);
                        if !matches!(value_type, ValueType::UserType(_)) {
                            continue;
                        }
                        match case_type {
                            None => case_type = Some(value_type),
                            Some(existing) if existing == value_type => {}
                            Some(existing) => {
                                if let Some(narrower) =
                                    self.const_scope.narrower_value_type(existing, value_type)
                                {
                                    case_type = Some(narrower);
                                } else {
                                    case_type_conflict = true;
                                }
                            }
                        }
                    }
                }
                let has_default = cases.iter().any(|case| {
                    matches!(
                        case,
                        SwitchCase::Default(_)
                            | SwitchCase::DefaultFallthrough(_)
                            | SwitchCase::ImplicitDefault(_)
                    )
                });
                let mut paths = Vec::with_capacity(cases.len() + usize::from(!has_default));
                if !has_default {
                    paths.push(requirements.clone());
                }
                for case in cases.iter_mut().rev() {
                    paths.push(self.visit_stmts(case.stmts_mut(), requirements.clone()));
                }
                requirements = self.merge(paths);
                if !case_type_conflict {
                    if let Some(value_type) = case_type {
                        self.require_expr(selector, value_type, &mut requirements);
                    }
                }
                self.visit_expr(selector, &mut requirements);
            }
            Stmt::For(elements) => {
                // One conservative reverse iteration is sufficient to decorate
                // definitions inside the loop without asserting a fixed-point
                // type for values which may arrive through the back edge.
                let update = self.visit_stmt(&mut elements.2, requirements.clone());
                let mut iterated = self.visit_stmts(&mut elements.3, update);
                self.visit_expr(&mut elements.0, &mut iterated);
                requirements = self.merge([requirements, iterated]);
                requirements = self.visit_stmt(&mut elements.1, requirements);
            }
            Stmt::DoWhile(condition, body) => {
                self.visit_expr(condition, &mut requirements);
                requirements = self.visit_stmts(body, requirements);
            }
            Stmt::Exit => requirements.clear(),
            Stmt::JumpNext | Stmt::Break => {}
        }
        requirements
    }
}

struct StringDecorateVisitor<'a> {
    constant_names: Vec<String>,
    next_to_place: IntValue,
    const_scope: &'a ConstScope,
    local_types: HashMap<String, ValueType>,
    local_origins: HashMap<String, String>,
    local_values: HashMap<String, i64>,
    callable_return_overrides: HashMap<String, ValueType>,
    definition_hints: HashMap<String, VecDeque<Option<TypeRequirement>>>,
    stable_hint_types: HashMap<String, ValueType>,
}

impl<'a> StringDecorateVisitor<'a> {
    fn new(
        constant_names: Vec<String>,
        const_scope: &'a ConstScope,
        definition_hints: HashMap<String, VecDeque<Option<TypeRequirement>>>,
        explicit_local_types: &HashMap<String, ValueType>,
    ) -> Self {
        let mut stable_hint_types: HashMap<String, ValueType> = definition_hints
            .iter()
            .filter_map(|(name, hints)| {
                let mut common = None;
                for hint in hints {
                    let Some(TypeRequirement::Type(value_type)) = hint else {
                        return None;
                    };
                    common = Some(match common {
                        None => *value_type,
                        Some(existing) => const_scope.narrower_value_type(existing, *value_type)?,
                    });
                }
                common.map(|value_type| (name.clone(), value_type))
            })
            .collect();
        stable_hint_types.extend(
            explicit_local_types
                .iter()
                .map(|(name, value_type)| (name.clone(), *value_type)),
        );
        Self {
            const_scope,
            constant_names,
            next_to_place: 0,
            local_types: HashMap::new(),
            local_origins: HashMap::new(),
            local_values: HashMap::new(),
            callable_return_overrides: HashMap::new(),
            definition_hints,
            stable_hint_types,
        }
    }

    fn definition_type(&mut self, name: &str, expr: &Expr) -> ValueType {
        let expression_type = self.expr_type(expr);
        match self
            .definition_hints
            .get_mut(name)
            .and_then(VecDeque::pop_front)
            .flatten()
        {
            Some(TypeRequirement::Type(value_type)) => self
                .const_scope
                .narrower_value_type(value_type, expression_type)
                .unwrap_or_else(|| {
                    if expression_type == ValueType::Undefined {
                        value_type
                    } else {
                        ValueType::Integer
                    }
                }),
            Some(TypeRequirement::Conflict) => ValueType::Integer,
            None => expression_type,
        }
    }

    fn stringify_expr(&mut self, expr: &mut Expr) -> Result<(), ()> {
        match expr {
            Expr::Int(int_value) => {
                let int_value = *int_value;

                if int_value == self.next_to_place {
                    /* OK: we found where next_to_place is placed first, increment it */
                    self.next_to_place += 1;
                } else if int_value > self.next_to_place {
                    /* ERROR: we couldn't place the value we need to place, and found
                     * an instance of the one after that */
                    return Err(());
                }

                *expr = Expr::Name(self.constant_names[int_value as usize].clone());
                Ok(())
            }

            _ => Err(()),
        }
    }

    fn visit_invoke(&mut self, invoke: &mut Invoke) -> Result<(), DecompileError> {
        if invoke.func == "mary_negated_int"
            || self.const_scope.typed_identity_type(&invoke.func).is_some()
        {
            return Ok(());
        }
        let (_, shape) = &self.const_scope.callable_map()[&invoke.func];

        assert_eq!(shape.num_parameters(), invoke.args.len());

        let mut error = false;
        let variable_value_type = if invoke.func == "VarSet" {
            invoke
                .args
                .first()
                .and_then(|argument| self.integer_value(argument))
                .and_then(|variable_id| self.const_scope.variable_value_type(variable_id))
        } else {
            None
        };

        let parameter_types = shape
            .parameter_types()
            .iter()
            .copied()
            .enumerate()
            .map(|(target_parameter_index, static_type)| {
                invoke
                    .args
                    .iter()
                    .enumerate()
                    .find_map(|(discriminator_parameter_index, argument)| {
                        let discriminator_value = self.integer_value(argument)?;
                        self.const_scope.dependent_callable_parameter_type(
                            &invoke.func,
                            discriminator_parameter_index,
                            discriminator_value,
                            target_parameter_index,
                        )
                    })
                    .unwrap_or(static_type)
            })
            .collect::<Vec<_>>();
        for (expr, param_type) in zip(&mut invoke.args, parameter_types) {
            match param_type {
                ValueType::String => {
                    if self.stringify_expr(expr).is_err() {
                        error = true;
                    }
                }

                ValueType::UserType(type_id) => {
                    let constant_value = match &*expr {
                        Expr::Int(value) => Some(*value),
                        Expr::Name(name) => self.const_scope.const_int_value(name),
                        _ => None,
                    };
                    if let Some(value) = constant_value {
                        if let Some(name) = self.const_scope.typed_int_const_name(type_id, value) {
                            *expr = Expr::Name(name.to_owned());
                        } else if let Some(wrapper) = self.const_scope.typed_identity_name(type_id)
                        {
                            *expr =
                                Expr::Call(Invoke::new(wrapper.to_owned(), vec![Expr::Int(value)]));
                        }
                    } else {
                        // Preserve the original VM arithmetic while still
                        // recovering the typed base of expressions such as
                        // `mine_floor_0 + runtime_or_literal_offset`.
                        self.decorate_typed_int(expr, param_type);
                        if self.visit_expr(expr).is_err() {
                            error = true;
                        }
                    }
                }

                _ => {
                    if self.visit_expr(expr).is_err() {
                        error = true;
                    }
                }
            }
        }

        if let (Some(value_type), Some(value)) = (variable_value_type, invoke.args.get_mut(1)) {
            self.decorate_typed_int(value, value_type);
        }

        if error {
            Err(DecompileError::NotEnoughStringConsumers)
        } else {
            Ok(())
        }
    }

    fn expr_type(&self, expr: &Expr) -> ValueType {
        match expr {
            Expr::Call(invoke) if invoke.func == "VarGet" => invoke
                .args
                .first()
                .and_then(|argument| self.integer_value(argument))
                .and_then(|variable_id| self.const_scope.variable_value_type(variable_id))
                .unwrap_or(ValueType::Integer),
            Expr::Call(invoke) => {
                if let Some(value_type) = self.const_scope.typed_identity_type(&invoke.func) {
                    return value_type;
                }
                for (parameter_index, argument) in invoke.args.iter().enumerate() {
                    let parameter_value = self.integer_value(argument);
                    if let Some(value_type) = parameter_value.and_then(|parameter_value| {
                        self.const_scope.dependent_callable_return_type(
                            &invoke.func,
                            parameter_index,
                            parameter_value,
                        )
                    }) {
                        return value_type;
                    }
                }
                if let Some(value_type) = self.callable_return_overrides.get(&invoke.func) {
                    return *value_type;
                }
                self.const_scope
                    .callable_map()
                    .get(&invoke.func)
                    .map_or(ValueType::Undefined, |(_, shape)| shape.return_type())
            }
            Expr::Name(name) => self
                .local_types
                .get(name)
                .copied()
                .or_else(|| self.stable_hint_types.get(name).copied())
                .or_else(|| self.const_scope.constant_value_type(name))
                .unwrap_or(ValueType::Undefined),
            Expr::Int(_) => ValueType::Integer,
            Expr::Str(_) => ValueType::String,
            _ => ValueType::Undefined,
        }
    }

    fn decorate_typed_int(&self, expr: &mut Expr, value_type: ValueType) {
        let ValueType::UserType(type_id) = value_type else {
            return;
        };
        // Preserve the enum-valued base of an offset expression. This is used
        // by native scripts for ranges such as `map < mine_floor_0 + 256`.
        // The right operand remains an ordinary numeric offset; treating both
        // operands as enum values would be incorrect for dense ID domains.
        if let Expr::OpAdd(pair) | Expr::OpSub(pair) = expr {
            self.decorate_typed_int(&mut pair.0, value_type);
            return;
        }
        if let Expr::Int(value) = &*expr {
            if let Some(name) = self.const_scope.typed_int_const_name(type_id, *value) {
                // Switch case values live in case-table metadata rather than
                // executable push instructions, so their symbolic spelling
                // has no direct-vs-negated encoding distinction.
                *expr = Expr::Name(name.to_owned());
            } else if let Some(wrapper) = self.const_scope.typed_identity_name(type_id) {
                *expr = Expr::Call(Invoke::new(wrapper.to_owned(), vec![Expr::Int(*value)]));
            }
            return;
        }
        let value = match &*expr {
            Expr::OpNeg(inner) => match &**inner {
                Expr::Int(value) => value.checked_neg(),
                Expr::Name(name) => self
                    .const_scope
                    .const_int_value(name)
                    .and_then(i64::checked_neg),
                _ => None,
            },
            _ => None,
        };
        if let Some(name) =
            value.and_then(|value| self.const_scope.typed_int_const_name(type_id, value))
        {
            *expr = Expr::Call(Invoke::new(
                "mary_negated_int".to_owned(),
                vec![Expr::Name(name.to_owned())],
            ));
        } else if let Some(value) = value {
            if let Some(wrapper) = self.const_scope.typed_identity_name(type_id) {
                *expr = Expr::Call(Invoke::new(
                    wrapper.to_owned(),
                    vec![Expr::Call(Invoke::new(
                        "mary_negated_int".to_owned(),
                        vec![Expr::Int(value)],
                    ))],
                ));
            }
        }
    }

    fn integer_value(&self, expr: &Expr) -> Option<i64> {
        match expr {
            Expr::Int(value) => Some(*value),
            Expr::Name(name) => self
                .const_scope
                .const_int_value(name)
                .or_else(|| self.local_values.get(name).copied()),
            Expr::OpAdd(pair) => self
                .integer_value(&pair.0)?
                .checked_add(self.integer_value(&pair.1)?),
            Expr::OpSub(pair) => self
                .integer_value(&pair.0)?
                .checked_sub(self.integer_value(&pair.1)?),
            Expr::OpMul(pair) => self
                .integer_value(&pair.0)?
                .checked_mul(self.integer_value(&pair.1)?),
            Expr::OpDiv(pair) => self
                .integer_value(&pair.0)?
                .checked_div(self.integer_value(&pair.1)?),
            Expr::OpMod(pair) => self
                .integer_value(&pair.0)?
                .checked_rem(self.integer_value(&pair.1)?),
            Expr::OpNeg(inner) => self.integer_value(inner)?.checked_neg(),
            Expr::Call(invoke)
                if self.const_scope.typed_identity_type(&invoke.func).is_some()
                    && invoke.args.len() == 1 =>
            {
                self.integer_value(&invoke.args[0])
            }
            _ => None,
        }
    }

    fn expr_origin(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Call(invoke) => Some(invoke.func.clone()),
            Expr::Name(name) => self.local_origins.get(name).cloned(),
            _ => None,
        }
    }

    fn apply_related_refinements(&mut self, condition: &Expr, condition_is_true: bool) {
        let (pair, equality_holds) = match condition {
            Expr::CmpEq(pair) => (pair, condition_is_true),
            Expr::CmpNe(pair) => (pair, !condition_is_true),
            _ => return,
        };
        if !equality_holds {
            return;
        }
        let extract = |discriminator: &Expr, value: &Expr| {
            let discriminator_value = self.integer_value(value)?;
            let discriminator_callable = match discriminator {
                Expr::Name(name) => self.local_origins.get(name).cloned(),
                Expr::Call(invoke) => Some(invoke.func.clone()),
                _ => None,
            }?;
            Some((discriminator_callable, discriminator_value))
        };
        let Some((discriminator_callable, discriminator_value)) =
            extract(&pair.0, &pair.1).or_else(|| extract(&pair.1, &pair.0))
        else {
            return;
        };
        let refinements = self
            .const_scope
            .related_callable_return_types(&discriminator_callable, discriminator_value);
        for (value_callable, value_type) in refinements {
            self.callable_return_overrides
                .insert(value_callable.to_owned(), value_type);
            for (name, origin) in &self.local_origins {
                if origin == value_callable {
                    self.local_types.insert(name.clone(), value_type);
                }
            }
        }
    }

    fn join_local_types<T: PartialEq>(
        paths: impl IntoIterator<Item = HashMap<String, T>>,
    ) -> HashMap<String, T> {
        let mut paths = paths.into_iter();
        let Some(mut joined) = paths.next() else {
            return HashMap::new();
        };

        for path in paths {
            joined.retain(|name, value_type| path.get(name) == Some(value_type));
        }

        joined
    }

    fn join_local_value_types(
        &self,
        left_types: &HashMap<String, ValueType>,
        left_values: &HashMap<String, i64>,
        right_types: &HashMap<String, ValueType>,
        right_values: &HashMap<String, i64>,
    ) -> HashMap<String, ValueType> {
        let paths = [(left_types, left_values), (right_types, right_values)];
        let names = paths
            .iter()
            .flat_map(|(types, _)| types.keys().cloned())
            .collect::<HashSet<_>>();
        let mut joined = HashMap::new();

        for name in names {
            let first = paths[0].0.get(&name).copied();
            let second = paths[1].0.get(&name).copied();
            if let Some(value_type) = first.filter(|_| first == second) {
                joined.insert(name, value_type);
                continue;
            }

            let mut candidate = None;
            let mut conflict = false;
            for value_type in [first, second]
                .into_iter()
                .flatten()
                .filter(|value_type| matches!(value_type, ValueType::UserType(_)))
            {
                candidate = match candidate {
                    None => Some(value_type),
                    Some(existing) => self.const_scope.narrower_value_type(existing, value_type),
                };
                if candidate.is_none() {
                    conflict = true;
                    break;
                }
            }
            if conflict {
                continue;
            }
            let Some(ValueType::UserType(type_id)) = candidate else {
                continue;
            };

            let compatible = paths.iter().all(|(types, values)| match types.get(&name) {
                Some(ValueType::UserType(path_type_id)) => self
                    .const_scope
                    .narrower_value_type(
                        ValueType::UserType(type_id),
                        ValueType::UserType(*path_type_id),
                    )
                    .is_some(),
                Some(ValueType::Integer) | None => values.get(&name).is_some_and(|value| {
                    self.const_scope
                        .typed_int_const_name(type_id, *value)
                        .is_some()
                        || self.const_scope.typed_identity_name(type_id).is_some()
                }),
                Some(ValueType::String | ValueType::Undefined) => false,
            });
            if compatible {
                joined.insert(name, ValueType::UserType(type_id));
            }
        }

        joined
    }

    fn collect_assigned_locals(stmts: &[Stmt], names: &mut HashSet<String>) {
        for stmt in stmts {
            match stmt {
                Stmt::Vars(items) => names.extend(items.iter().map(|(name, _)| name.clone())),
                Stmt::Assign(_, name, _) | Stmt::AssignNoDisc(_, name, _) => {
                    names.insert(name.clone());
                }
                Stmt::If(_, body) | Stmt::DoWhile(_, body) => {
                    Self::collect_assigned_locals(body, names);
                }
                Stmt::IfElse(_, yes, no) => {
                    Self::collect_assigned_locals(yes, names);
                    Self::collect_assigned_locals(no, names);
                }
                Stmt::For(elements) => {
                    Self::collect_assigned_locals(std::slice::from_ref(&elements.1), names);
                    Self::collect_assigned_locals(std::slice::from_ref(&elements.2), names);
                    Self::collect_assigned_locals(&elements.3, names);
                }
                Stmt::Switch(_, cases, _, _) => {
                    for case in cases {
                        Self::collect_assigned_locals(case.stmts(), names);
                    }
                }
                _ => {}
            }
        }
    }

    fn visit_expr(&mut self, expr: &mut Expr) -> Result<(), DecompileError> {
        match expr {
            Expr::Name(_)
            | Expr::Int(_)
            | Expr::Str(_)
            | Expr::PostIncrement(_)
            | Expr::PreIncrement(_)
            | Expr::PostDecrement(_)
            | Expr::PreDecrement(_) => Ok(()),

            Expr::OpAdd(exprs)
            | Expr::OpSub(exprs)
            | Expr::OpMul(exprs)
            | Expr::OpDiv(exprs)
            | Expr::OpMod(exprs) => {
                self.visit_expr(&mut exprs.0)?;
                self.visit_expr(&mut exprs.1)
            }

            Expr::OpAnd(exprs) => {
                self.visit_expr(&mut exprs.0)?;
                let outer_types = self.local_types.clone();
                let outer_origins = self.local_origins.clone();
                let outer_values = self.local_values.clone();
                let outer_overrides = self.callable_return_overrides.clone();
                self.apply_related_refinements(&exprs.0, true);
                let result = self.visit_expr(&mut exprs.1);
                self.local_types = outer_types;
                self.local_origins = outer_origins;
                self.local_values = outer_values;
                self.callable_return_overrides = outer_overrides;
                result
            }

            Expr::OpOr(exprs) => {
                self.visit_expr(&mut exprs.0)?;
                let outer_types = self.local_types.clone();
                let outer_origins = self.local_origins.clone();
                let outer_values = self.local_values.clone();
                let outer_overrides = self.callable_return_overrides.clone();
                self.apply_related_refinements(&exprs.0, false);
                let result = self.visit_expr(&mut exprs.1);
                self.local_types = outer_types;
                self.local_origins = outer_origins;
                self.local_values = outer_values;
                self.callable_return_overrides = outer_overrides;
                result
            }

            Expr::CmpEq(exprs)
            | Expr::CmpNe(exprs)
            | Expr::CmpLt(exprs)
            | Expr::CmpLe(exprs)
            | Expr::CmpGe(exprs)
            | Expr::CmpGt(exprs) => {
                let left_type = self.expr_type(&exprs.0);
                let right_type = self.expr_type(&exprs.1);
                self.decorate_typed_int(&mut exprs.1, left_type);
                self.decorate_typed_int(&mut exprs.0, right_type);
                self.visit_expr(&mut exprs.0)?;
                self.visit_expr(&mut exprs.1)
            }

            Expr::OpNeg(inner) => {
                if let Expr::Int(magnitude) = &**inner {
                    if let Some(value) = magnitude.checked_neg() {
                        *expr = Expr::Call(Invoke::new(
                            "mary_negated_int".to_owned(),
                            vec![Expr::Int(value)],
                        ));
                        return Ok(());
                    }
                }
                self.visit_expr(inner)
            }

            Expr::OpNot(expr) => self.visit_expr(expr),

            Expr::Call(invoke) => self.visit_invoke(invoke),
        }
    }

    fn visit_stmt(&mut self, stmt: &mut Stmt) -> Result<(), DecompileError> {
        match stmt {
            Stmt::Vars(items) => {
                for (name, option_expr) in items {
                    if let Some(expr) = option_expr {
                        let value_type = self.definition_type(name, expr);
                        let origin = self.expr_origin(expr);
                        let integer_value = self.integer_value(expr);
                        self.visit_expr(expr)?;
                        if value_type != ValueType::Undefined {
                            self.local_types.insert(name.clone(), value_type);
                        }
                        if let Some(origin) = origin {
                            self.local_origins.insert(name.clone(), origin);
                        }
                        if let Some(integer_value) = integer_value {
                            self.local_values.insert(name.clone(), integer_value);
                        } else {
                            self.local_values.remove(name);
                        }
                    }
                }

                Ok(())
            }

            Stmt::Consts(items) => {
                for (_, expr) in items {
                    self.visit_expr(expr)?;
                }

                Ok(())
            }

            Stmt::Assign(operation, name, expr) | Stmt::AssignNoDisc(operation, name, expr) => {
                let value_type = if *operation == AssignOperation::None {
                    self.definition_type(name, expr)
                } else {
                    self.expr_type(expr)
                };
                let origin = self.expr_origin(expr);
                let integer_value = self.integer_value(expr);
                self.visit_expr(expr)?;
                if *operation == AssignOperation::None && value_type != ValueType::Undefined {
                    self.local_types.insert(name.clone(), value_type);
                } else {
                    self.local_types.remove(name);
                }
                if *operation == AssignOperation::None {
                    if let Some(origin) = origin {
                        self.local_origins.insert(name.clone(), origin);
                    } else {
                        self.local_origins.remove(name);
                    }
                } else {
                    self.local_origins.remove(name);
                }
                if *operation == AssignOperation::None {
                    if let Some(integer_value) = integer_value {
                        self.local_values.insert(name.clone(), integer_value);
                    } else {
                        self.local_values.remove(name);
                    }
                } else {
                    self.local_values.remove(name);
                }
                Ok(())
            }
            Stmt::Expr(expr) => self.visit_expr(expr),

            Stmt::Call(invoke) => self.visit_invoke(invoke),

            Stmt::If(expr, stmts) => {
                self.visit_expr(expr)?;
                let outer_types = self.local_types.clone();
                let outer_origins = self.local_origins.clone();
                let outer_values = self.local_values.clone();
                let outer_overrides = self.callable_return_overrides.clone();
                self.apply_related_refinements(expr, true);
                self.visit_stmts(stmts)?;
                let then_types = self.local_types.clone();
                let then_origins = self.local_origins.clone();
                let then_values = self.local_values.clone();
                self.local_types = self.join_local_value_types(
                    &outer_types,
                    &outer_values,
                    &then_types,
                    &then_values,
                );
                self.local_origins = Self::join_local_types([outer_origins, then_origins]);
                self.local_values = Self::join_local_types([outer_values, then_values]);
                self.callable_return_overrides = outer_overrides;
                Ok(())
            }

            Stmt::IfElse(expr, stmts_then, stmts_else) => {
                self.visit_expr(expr)?;
                let outer_types = self.local_types.clone();
                let outer_origins = self.local_origins.clone();
                let outer_values = self.local_values.clone();
                let outer_overrides = self.callable_return_overrides.clone();
                self.apply_related_refinements(expr, true);
                self.visit_stmts(stmts_then)?;
                let then_types = self.local_types.clone();
                let then_origins = self.local_origins.clone();
                let then_values = self.local_values.clone();
                self.local_types = outer_types.clone();
                self.local_origins = outer_origins.clone();
                self.local_values = outer_values.clone();
                self.callable_return_overrides = outer_overrides.clone();
                self.apply_related_refinements(expr, false);
                self.visit_stmts(stmts_else)?;
                let else_types = self.local_types.clone();
                let else_origins = self.local_origins.clone();
                let else_values = self.local_values.clone();
                self.local_types = self.join_local_value_types(
                    &then_types,
                    &then_values,
                    &else_types,
                    &else_values,
                );
                self.local_origins = Self::join_local_types([then_origins, else_origins]);
                self.local_values = Self::join_local_types([then_values, else_values]);
                self.callable_return_overrides = outer_overrides;
                Ok(())
            }

            Stmt::For(for_elems) => {
                self.visit_stmt(&mut for_elems.1)?;
                let entry_types = self.local_types.clone();
                let entry_origins = self.local_origins.clone();
                let entry_values = self.local_values.clone();
                let mut loop_assigned = HashSet::new();
                Self::collect_assigned_locals(&for_elems.3, &mut loop_assigned);
                Self::collect_assigned_locals(
                    std::slice::from_ref(&for_elems.2),
                    &mut loop_assigned,
                );
                for name in loop_assigned {
                    self.local_types.remove(&name);
                    self.local_origins.remove(&name);
                    self.local_values.remove(&name);
                }
                self.visit_expr(&mut for_elems.0)?;
                self.visit_stmts(&mut for_elems.3)?;
                self.visit_stmt(&mut for_elems.2)?;
                let iterated_types = self.local_types.clone();
                let iterated_origins = self.local_origins.clone();
                let iterated_values = self.local_values.clone();
                self.local_types = Self::join_local_types([entry_types, iterated_types]);
                self.local_origins = Self::join_local_types([entry_origins, iterated_origins]);
                self.local_values = Self::join_local_types([entry_values, iterated_values]);
                Ok(())
            }

            Stmt::DoWhile(expr, stmts) => {
                let mut loop_assigned = HashSet::new();
                Self::collect_assigned_locals(stmts, &mut loop_assigned);
                for name in loop_assigned {
                    self.local_types.remove(&name);
                    self.local_origins.remove(&name);
                    self.local_values.remove(&name);
                }
                self.visit_stmts(stmts)?;
                self.visit_expr(expr)?;
                Ok(())
            }

            Stmt::Switch(expr, switch_cases, _, _) => {
                let switch_type = self.expr_type(expr);
                let switch_origin = match expr {
                    Expr::Name(name) => self.local_origins.get(name).cloned(),
                    Expr::Call(invoke) => Some(invoke.func.clone()),
                    _ => None,
                };
                self.visit_expr(expr)?;
                let outer_types = self.local_types.clone();
                let outer_origins = self.local_origins.clone();
                let outer_values = self.local_values.clone();
                let outer_overrides = self.callable_return_overrides.clone();
                let has_default = switch_cases.iter().any(|switch_case| {
                    matches!(
                        switch_case,
                        SwitchCase::Default(_)
                            | SwitchCase::DefaultFallthrough(_)
                            | SwitchCase::ImplicitDefault(_)
                    )
                });
                let mut exit_types = Vec::with_capacity(switch_cases.len() + 1);
                let mut exit_origins = Vec::with_capacity(switch_cases.len() + 1);
                let mut exit_values = Vec::with_capacity(switch_cases.len() + 1);
                let mut fallthrough_types = None;
                let mut fallthrough_origins = None;
                let mut fallthrough_values = None;
                let mut fallthrough_overrides = None;
                if !has_default {
                    exit_types.push(outer_types.clone());
                    exit_origins.push(outer_origins.clone());
                    exit_values.push(outer_values.clone());
                }

                for switch_case in switch_cases {
                    self.local_types = outer_types.clone();
                    self.local_origins = outer_origins.clone();
                    self.local_values = outer_values.clone();
                    self.callable_return_overrides = outer_overrides.clone();
                    match switch_case {
                        SwitchCase::Case(values, _) | SwitchCase::Fallthrough(values, _) => {
                            let mut label_states = Vec::with_capacity(values.len());
                            for value in values {
                                self.local_types = outer_types.clone();
                                self.local_origins = outer_origins.clone();
                                self.local_values = outer_values.clone();
                                self.callable_return_overrides = outer_overrides.clone();
                                if let (Some(discriminator), Some(discriminator_value)) = (
                                    switch_origin.as_deref(),
                                    match &*value {
                                        Expr::Int(value) => Some(*value),
                                        Expr::Name(name) => self.const_scope.const_int_value(name),
                                        _ => None,
                                    },
                                ) {
                                    for (value_callable, value_type) in
                                        self.const_scope.related_callable_return_types(
                                            discriminator,
                                            discriminator_value,
                                        )
                                    {
                                        self.callable_return_overrides
                                            .insert(value_callable.to_owned(), value_type);
                                        for (name, origin) in &self.local_origins {
                                            if origin == value_callable {
                                                self.local_types.insert(name.clone(), value_type);
                                            }
                                        }
                                    }
                                }
                                self.decorate_typed_int(value, switch_type);
                                self.visit_expr(value)?;
                                label_states.push((
                                    self.local_types.clone(),
                                    self.local_origins.clone(),
                                    self.local_values.clone(),
                                    self.callable_return_overrides.clone(),
                                ));
                            }
                            if let Some((types, origins, values, overrides)) =
                                label_states.into_iter().reduce(
                                    |(types, origins, values, overrides),
                                     (next_types, next_origins, next_values, next_overrides)| {
                                        (
                                            self.join_local_value_types(
                                                &types,
                                                &values,
                                                &next_types,
                                                &next_values,
                                            ),
                                            Self::join_local_types([origins, next_origins]),
                                            Self::join_local_types([values, next_values]),
                                            Self::join_local_types([overrides, next_overrides]),
                                        )
                                    },
                                )
                            {
                                self.local_types = types;
                                self.local_origins = origins;
                                self.local_values = values;
                                self.callable_return_overrides = overrides;
                            }
                        }
                        _ => {}
                    }

                    if let (Some(types), Some(origins), Some(values), Some(overrides)) = (
                        fallthrough_types.take(),
                        fallthrough_origins.take(),
                        fallthrough_values.take(),
                        fallthrough_overrides.take(),
                    ) {
                        self.local_types = self.join_local_value_types(
                            &self.local_types,
                            &self.local_values,
                            &types,
                            &values,
                        );
                        self.local_origins = Self::join_local_types([
                            std::mem::take(&mut self.local_origins),
                            origins,
                        ]);
                        self.local_values = Self::join_local_types([
                            std::mem::take(&mut self.local_values),
                            values,
                        ]);
                        self.callable_return_overrides = Self::join_local_types([
                            std::mem::take(&mut self.callable_return_overrides),
                            overrides,
                        ]);
                    }

                    self.visit_stmts(switch_case.stmts_mut())?;
                    if matches!(
                        switch_case,
                        SwitchCase::Fallthrough(_, _)
                            | SwitchCase::DefaultFallthrough(_)
                            | SwitchCase::ImplicitDefault(_)
                            | SwitchCase::DeadJump(_)
                    ) {
                        fallthrough_types = Some(self.local_types.clone());
                        fallthrough_origins = Some(self.local_origins.clone());
                        fallthrough_values = Some(self.local_values.clone());
                        fallthrough_overrides = Some(self.callable_return_overrides.clone());
                    } else {
                        exit_types.push(self.local_types.clone());
                        exit_origins.push(self.local_origins.clone());
                        exit_values.push(self.local_values.clone());
                    }
                }

                if let (Some(types), Some(origins), Some(values)) =
                    (fallthrough_types, fallthrough_origins, fallthrough_values)
                {
                    exit_types.push(types);
                    exit_origins.push(origins);
                    exit_values.push(values);
                }

                self.local_types = Self::join_local_types(exit_types);
                self.local_origins = Self::join_local_types(exit_origins);
                self.local_values = Self::join_local_types(exit_values);
                self.callable_return_overrides = outer_overrides;

                Ok(())
            }

            Stmt::Ir(_) => Ok(()),

            Stmt::JumpNext | Stmt::Break | Stmt::Exit => Ok(()),
        }
    }

    fn visit_stmts(&mut self, stmts: &mut [Stmt]) -> Result<(), DecompileError> {
        let mut result = Ok(());

        for stmt in stmts {
            if let Err(err) = self.visit_stmt(stmt) {
                result = Err(err);
            }
        }

        result
    }
}

pub(super) fn decorate_stmts_with_strings(
    stmts: &mut Vec<Stmt>,
    strings: &[StrValue],
    const_scope: &ConstScope,
    script_name: Option<&str>,
    text_names: Option<&[Option<String>]>,
    explicit_local_types: &HashMap<String, ValueType>,
) -> Result<(), DecompileError> {
    let mut string_constants = vec![];
    let mut string_constant_stmts = vec![];

    for (i, string) in strings.iter().enumerate() {
        let name = text_names
            .and_then(|names| names.get(i))
            .cloned()
            .flatten()
            .unwrap_or_else(|| {
                script_name.map_or_else(
                    || format!("MESSAGE_{i}"),
                    |script_name| format!("gText_{script_name}_{i:03}"),
                )
            });

        string_constants.push(name.clone());
        string_constant_stmts.push(Stmt::Consts(vec![(name, Expr::Str(string.clone()))]));
    }

    string_constant_stmts.append(stmts);
    *stmts = string_constant_stmts;

    let mut assigned_locals = HashSet::new();
    StringDecorateVisitor::collect_assigned_locals(stmts, &mut assigned_locals);
    if let Some(name) = explicit_local_types
        .keys()
        .find(|name| !assigned_locals.contains(*name))
    {
        return Err(DecompileError::UnknownLocalTypeHint(name.clone()));
    }

    let mut backpropagator = LocalTypeBackpropagator::new(const_scope, explicit_local_types);
    backpropagator.visit_stmts(stmts, TypeRequirements::new());

    let mut visitor = StringDecorateVisitor::new(
        string_constants,
        const_scope,
        backpropagator.definition_hints,
        explicit_local_types,
    );
    visitor.visit_stmts(stmts)?;

    if (visitor.next_to_place as usize) < strings.len() {
        Err(DecompileError::NotEnoughStringConsumers)
    } else {
        Ok(())
    }
}
