// Copyright 2020 - present Alex Dukhno
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bigdecimal::BigDecimal;
use catalog::SqlTable;
use data_binary::{
    repr::{Datum, ToDatum},
    Binary,
};
use data_manipulation_query_result::QueryExecutionError;
use data_manipulation_typed_tree::StaticTypedTree;
use data_manipulation_typed_values::TypedValue;
use types::SqlTypeFamily;

pub trait Flow {
    type Output;

    fn next_tuple(&mut self) -> Result<Option<Self::Output>, QueryExecutionError>;
}

pub struct StaticValues(Box<dyn Iterator<Item = Vec<Option<StaticTypedTree>>>>);

impl StaticValues {
    pub fn from(values: Vec<Vec<Option<StaticTypedTree>>>) -> Box<StaticValues> {
        Box::new(StaticValues(Box::new(values.into_iter())))
    }
}

impl Flow for StaticValues {
    type Output = Vec<Option<StaticTypedTree>>;

    fn next_tuple(&mut self) -> Result<Option<Self::Output>, QueryExecutionError> {
        Ok(self.0.next())
    }
}

pub struct StaticExpressionEval {
    source: Box<dyn Flow<Output = Vec<Option<StaticTypedTree>>>>,
}

impl StaticExpressionEval {
    pub fn new(source: Box<dyn Flow<Output = Vec<Option<StaticTypedTree>>>>) -> Box<StaticExpressionEval> {
        Box::new(StaticExpressionEval { source })
    }
}

impl Flow for StaticExpressionEval {
    type Output = Vec<Option<TypedValue>>;

    fn next_tuple(&mut self) -> Result<Option<Self::Output>, QueryExecutionError> {
        if let Ok(Some(tuple)) = self.source.next_tuple() {
            let mut next_tuple = vec![];
            for value in tuple {
                let typed_value = match value {
                    None => None,
                    Some(value) => match value.eval() {
                        Err(error) => return Err(error),
                        Ok(value) => Some(value),
                    },
                };
                next_tuple.push(typed_value);
            }
            Ok(Some(next_tuple))
        } else {
            Ok(None)
        }
    }
}

pub struct ConstraintValidator {
    source: Box<dyn Flow<Output = Vec<Option<TypedValue>>>>,
    column_types: Vec<(String, SqlTypeFamily)>,
}

impl ConstraintValidator {
    pub fn new(
        source: Box<dyn Flow<Output = Vec<Option<TypedValue>>>>,
        column_types: Vec<(String, SqlTypeFamily)>,
    ) -> Box<ConstraintValidator> {
        Box::new(ConstraintValidator { source, column_types })
    }
}

impl Flow for ConstraintValidator {
    type Output = Vec<Option<Box<dyn ToDatum>>>;

    fn next_tuple(&mut self) -> Result<Option<Self::Output>, QueryExecutionError> {
        if let Some(tuple) = self.source.next_tuple()? {
            let mut data = vec![];
            for (index, value) in tuple.into_iter().enumerate() {
                let value = match (value, self.column_types[index].1) {
                    (None, _) => None,
                    (Some(value), type_family) => match value.type_family() {
                        None => unimplemented!(),
                        Some(value_type) => match value_type.compare(&type_family) {
                            Ok(wide_type_family) => {
                                log::debug!("{:?} {:?} {:?}", value, wide_type_family, type_family);
                                match (value.clone(), type_family) {
                                    (TypedValue::Num { value, .. }, SqlTypeFamily::SmallInt) => {
                                        if !(BigDecimal::from(i16::MIN)..=BigDecimal::from(i16::MAX)).contains(&value) {
                                            return Err(QueryExecutionError::most_specific_type_mismatch(
                                                value,
                                                type_family,
                                                self.column_types[index].0.as_str(),
                                                index,
                                            ));
                                        }
                                    }
                                    (TypedValue::Num { value, .. }, SqlTypeFamily::Integer) => {
                                        if !(BigDecimal::from(i32::MIN)..=BigDecimal::from(i32::MAX)).contains(&value) {
                                            return Err(QueryExecutionError::most_specific_type_mismatch(
                                                value,
                                                type_family,
                                                self.column_types[index].0.as_str(),
                                                index,
                                            ));
                                        }
                                    }
                                    (TypedValue::Num { value, .. }, SqlTypeFamily::BigInt) => {
                                        if !(BigDecimal::from(i64::MIN)..=BigDecimal::from(i64::MAX)).contains(&value) {
                                            return Err(QueryExecutionError::most_specific_type_mismatch(
                                                value,
                                                type_family,
                                                self.column_types[index].0.as_str(),
                                                index,
                                            ));
                                        }
                                    }
                                    (TypedValue::String(_), _) => {}
                                    (TypedValue::Bool(_), _) => {}
                                    _ => unimplemented!(),
                                }
                                Some(value.as_to_datum())
                            }
                            Err(_) => {
                                return Err(QueryExecutionError::most_specific_type_mismatch(
                                    value,
                                    type_family,
                                    self.column_types[index].0.as_str(),
                                    index,
                                ))
                            }
                        },
                    },
                };
                data.push(value);
            }
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
}

pub struct TableInPlan {
    source: Box<dyn Flow<Output = Vec<Option<Box<dyn ToDatum>>>>>,
    table: Box<dyn SqlTable>,
}

impl TableInPlan {
    pub fn new(source: Box<dyn Flow<Output = Vec<Option<Box<dyn ToDatum>>>>>, table: Box<dyn SqlTable>) -> TableInPlan {
        TableInPlan { source, table }
    }

    pub fn execute(mut self) -> Result<usize, QueryExecutionError> {
        let mut len = 0;
        while let Some(data) = self.source.next_tuple()? {
            self.table.write(Binary::pack(
                &data
                    .into_iter()
                    .map(|v| v.map(|v| v.convert()).unwrap_or_else(Datum::from_null))
                    .collect::<Vec<Datum>>()
                    .as_slice(),
            ));
            len += 1;
        }
        Ok(len)
    }
}
