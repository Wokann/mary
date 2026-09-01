use std::collections::{HashMap, HashSet};

use crate::{
    ast::{ConstVal, NameAccess, NameRef},
    ir::{CallId, CallableShape, ValueType},
};

#[derive(Clone, Debug, Default)]
pub struct ConstScope {
    constants: HashMap<String, ConstVal>,
    callables: HashMap<String, (CallId, CallableShape)>,
    user_types: HashMap<String, usize>,
    typed_constants: HashMap<(usize, i64), String>,
    constant_value_types: HashMap<String, usize>,
    variable_value_types: HashMap<i64, usize>,
    dependent_callable_return_types: HashMap<(String, usize, i64), usize>,
    dependent_callable_parameter_types: HashMap<(String, usize, i64, usize), usize>,
    related_callable_return_types: HashMap<(String, i64, String), usize>,
    type_subsets: HashSet<(usize, usize)>,
    typed_identities: HashMap<String, usize>,
    type_identity_names: HashMap<usize, String>,
}

impl ConstScope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_const(&mut self, name: String, val: ConstVal) {
        self.constants.insert(name, val);
    }

    pub fn add_func(&mut self, name: String, id: CallId, parameter_types: Vec<ValueType>) {
        let to_insert = (id, CallableShape::new_func(parameter_types));
        self.callables.insert(name, to_insert);
    }

    pub fn add_typed_func(
        &mut self,
        name: String,
        id: CallId,
        return_type: ValueType,
        parameter_types: Vec<ValueType>,
    ) {
        let to_insert = (
            id,
            CallableShape::new_typed_func(return_type, parameter_types),
        );
        self.callables.insert(name, to_insert);
    }

    pub fn add_proc(&mut self, name: String, id: CallId, parameter_types: Vec<ValueType>) {
        let to_insert = (id, CallableShape::new_proc(parameter_types));
        self.callables.insert(name, to_insert);
    }

    pub fn add_or_get_user_type(&mut self, name: String) -> usize {
        let would_be_next = self.user_types.len();
        *self.user_types.entry(name).or_insert(would_be_next)
    }

    pub fn user_type(&self, name: &str) -> Option<usize> {
        self.user_types.get(name).copied()
    }

    pub fn add_type_subset(&mut self, subset: &str, superset: &str) -> Option<Result<bool, ()>> {
        let pair = (self.user_type(subset)?, self.user_type(superset)?);
        if pair.0 == pair.1 || self.is_type_subset(pair.1, pair.0) {
            return Some(Err(()));
        }
        Some(Ok(self.type_subsets.insert(pair)))
    }

    fn is_type_subset(&self, subset: usize, superset: usize) -> bool {
        if subset == superset {
            return true;
        }
        let mut pending = vec![subset];
        let mut visited = HashSet::new();
        while let Some(current) = pending.pop() {
            if !visited.insert(current) {
                continue;
            }
            for &(child, parent) in &self.type_subsets {
                if child == current {
                    if parent == superset {
                        return true;
                    }
                    pending.push(parent);
                }
            }
        }
        false
    }

    pub fn narrower_value_type(&self, left: ValueType, right: ValueType) -> Option<ValueType> {
        if left == right {
            return Some(left);
        }
        match (left, right) {
            (ValueType::UserType(_), ValueType::Integer) => return Some(left),
            (ValueType::Integer, ValueType::UserType(_)) => return Some(right),
            _ => {}
        }
        let (ValueType::UserType(left), ValueType::UserType(right)) = (left, right) else {
            return None;
        };
        if self.is_type_subset(left, right) {
            Some(ValueType::UserType(left))
        } else if self.is_type_subset(right, left) {
            Some(ValueType::UserType(right))
        } else {
            None
        }
    }

    pub fn add_typed_int_const(&mut self, type_name: &str, name: String, value: i64) {
        let type_id = self.add_or_get_user_type(type_name.to_owned());
        self.constants.insert(name.clone(), ConstVal::Int(value));
        self.constant_value_types.insert(name.clone(), type_id);
        self.typed_constants.insert((type_id, value), name);
    }

    /// Adds an input-compatible alias without replacing the canonical name
    /// selected when an integer of this type is decorated during decompilation.
    pub fn add_typed_int_alias(&mut self, name: String, value: i64, type_id: usize) -> bool {
        if self.constants.contains_key(&name) {
            return false;
        }
        self.constants.insert(name.clone(), ConstVal::Int(value));
        self.constant_value_types.insert(name, type_id);
        true
    }

    pub fn typed_int_const_name(&self, type_id: usize, value: i64) -> Option<&str> {
        self.typed_constants
            .get(&(type_id, value))
            .map(String::as_str)
    }

    pub fn const_int_value(&self, name: &str) -> Option<i64> {
        match self.constants.get(name) {
            Some(ConstVal::Int(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn constant_value_type(&self, name: &str) -> Option<ValueType> {
        self.constant_value_types
            .get(name)
            .copied()
            .map(ValueType::UserType)
    }

    pub fn add_typed_identity(&mut self, type_name: &str, name: String) -> Option<bool> {
        let type_id = self.user_type(type_name)?;
        if self.typed_identities.contains_key(&name)
            || self.type_identity_names.contains_key(&type_id)
            || self.constants.contains_key(&name)
            || self.callables.contains_key(&name)
        {
            return Some(false);
        }
        self.typed_identities.insert(name.clone(), type_id);
        self.type_identity_names.insert(type_id, name);
        Some(true)
    }

    pub fn typed_identity_name(&self, type_id: usize) -> Option<&str> {
        self.type_identity_names.get(&type_id).map(String::as_str)
    }

    pub fn typed_identity_type(&self, name: &str) -> Option<ValueType> {
        self.typed_identities
            .get(name)
            .copied()
            .map(ValueType::UserType)
    }

    pub fn add_variable_value_type(&mut self, variable_id: i64, type_name: &str) -> Option<()> {
        let type_id = self.user_type(type_name)?;
        self.variable_value_types.insert(variable_id, type_id);
        Some(())
    }

    pub fn has_variable_value_type(&self, variable_id: i64) -> bool {
        self.variable_value_types.contains_key(&variable_id)
    }

    pub fn variable_value_type(&self, variable_id: i64) -> Option<ValueType> {
        self.variable_value_types
            .get(&variable_id)
            .copied()
            .map(ValueType::UserType)
    }

    pub fn add_dependent_callable_return_type(
        &mut self,
        callable_name: &str,
        parameter_index: usize,
        parameter_value: i64,
        type_name: &str,
    ) -> Option<bool> {
        let type_id = self.user_type(type_name)?;
        let key = (callable_name.to_owned(), parameter_index, parameter_value);
        if self.dependent_callable_return_types.contains_key(&key) {
            return Some(false);
        }
        self.dependent_callable_return_types.insert(key, type_id);
        Some(true)
    }

    pub fn dependent_callable_return_type(
        &self,
        callable_name: &str,
        parameter_index: usize,
        parameter_value: i64,
    ) -> Option<ValueType> {
        self.dependent_callable_return_types
            .get(&(callable_name.to_owned(), parameter_index, parameter_value))
            .copied()
            .map(ValueType::UserType)
    }

    pub fn add_dependent_callable_parameter_type(
        &mut self,
        callable_name: &str,
        discriminator_parameter_index: usize,
        discriminator_value: i64,
        target_parameter_index: usize,
        type_name: &str,
    ) -> Option<bool> {
        let type_id = self.user_type(type_name)?;
        let key = (
            callable_name.to_owned(),
            discriminator_parameter_index,
            discriminator_value,
            target_parameter_index,
        );
        if self.dependent_callable_parameter_types.contains_key(&key) {
            return Some(false);
        }
        self.dependent_callable_parameter_types.insert(key, type_id);
        Some(true)
    }

    pub fn dependent_callable_parameter_type(
        &self,
        callable_name: &str,
        discriminator_parameter_index: usize,
        discriminator_value: i64,
        target_parameter_index: usize,
    ) -> Option<ValueType> {
        self.dependent_callable_parameter_types
            .get(&(
                callable_name.to_owned(),
                discriminator_parameter_index,
                discriminator_value,
                target_parameter_index,
            ))
            .copied()
            .map(ValueType::UserType)
    }

    pub fn add_related_callable_return_type(
        &mut self,
        discriminator_callable: &str,
        discriminator_value: i64,
        value_callable: &str,
        type_name: &str,
    ) -> Option<bool> {
        let type_id = self.user_type(type_name)?;
        let key = (
            discriminator_callable.to_owned(),
            discriminator_value,
            value_callable.to_owned(),
        );
        if self.related_callable_return_types.contains_key(&key) {
            return Some(false);
        }
        self.related_callable_return_types.insert(key, type_id);
        Some(true)
    }

    pub fn related_callable_return_types(
        &self,
        discriminator_callable: &str,
        discriminator_value: i64,
    ) -> Vec<(&str, ValueType)> {
        self.related_callable_return_types
            .iter()
            .filter_map(|((discriminator, value, value_callable), type_id)| {
                (discriminator == discriminator_callable && *value == discriminator_value)
                    .then_some((value_callable.as_str(), ValueType::UserType(*type_id)))
            })
            .collect()
    }

    pub fn validate_callable_type_metadata(&self) -> Result<(), String> {
        for (callable_name, parameter_index, _) in self.dependent_callable_return_types.keys() {
            let Some((_, shape)) = self.callables.get(callable_name) else {
                return Err(format!(
                    "conditional return type references undeclared callable '{callable_name}'"
                ));
            };
            if *parameter_index >= shape.num_parameters() {
                return Err(format!(
                    "conditional return type for '{callable_name}' references parameter {parameter_index}, but the callable has {} parameters",
                    shape.num_parameters()
                ));
            }
        }
        for (callable_name, discriminator_index, _, target_index) in
            self.dependent_callable_parameter_types.keys()
        {
            let Some((_, shape)) = self.callables.get(callable_name) else {
                return Err(format!(
                    "conditional parameter type references undeclared callable '{callable_name}'"
                ));
            };
            for (role, parameter_index) in [
                ("discriminator", discriminator_index),
                ("target", target_index),
            ] {
                if *parameter_index >= shape.num_parameters() {
                    return Err(format!(
                        "conditional parameter type for '{callable_name}' references {role} parameter {parameter_index}, but the callable has {} parameters",
                        shape.num_parameters()
                    ));
                }
            }
        }
        for (discriminator_callable, _, value_callable) in self.related_callable_return_types.keys()
        {
            if !self.callables.contains_key(discriminator_callable) {
                return Err(format!(
                    "related return type references undeclared discriminator callable '{discriminator_callable}'"
                ));
            }
            if !self.callables.contains_key(value_callable) {
                return Err(format!(
                    "related return type references undeclared value callable '{value_callable}'"
                ));
            }
        }
        Ok(())
    }

    /// Callable declarations, exposed for table migration and diagnostics.
    pub fn callable_map(&self) -> &HashMap<String, (CallId, CallableShape)> {
        &self.callables
    }
}

impl NameAccess for ConstScope {
    fn lookup_name(&self, name: &str) -> Option<NameRef<'_>> {
        if let Some(val) = self.constants.get(name) {
            Some(NameRef::Const(val))
        } else {
            match self.callables.get(name) {
                Some((id, shape)) => {
                    if shape.is_func() {
                        Some(NameRef::Func(*id))
                    } else {
                        Some(NameRef::Proc(*id))
                    }
                }

                None => None,
            }
        }
    }

    fn typed_identity_type(&self, name: &str) -> Option<ValueType> {
        ConstScope::typed_identity_type(self, name)
    }
}
