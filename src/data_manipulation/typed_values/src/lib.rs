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

use bigdecimal::{BigDecimal, ToPrimitive};
use data_binary::repr::{Datum, ToDatum};
use std::fmt::{self, Display, Formatter};
use types::SqlTypeFamily;

#[derive(Debug, PartialEq, Clone)]
pub enum TypedValue {
    Num {
        value: BigDecimal,
        type_family: SqlTypeFamily,
    },
    String(String),
    Bool(bool),
}

impl TypedValue {
    pub fn type_family(&self) -> Option<SqlTypeFamily> {
        match self {
            TypedValue::Num { type_family, .. } => Some(*type_family),
            TypedValue::String(_) => Some(SqlTypeFamily::String),
            TypedValue::Bool(_) => Some(SqlTypeFamily::Bool),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn as_to_datum(self) -> Box<dyn ToDatum> {
        Box::new(self)
    }
}

impl Display for TypedValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TypedValue::Num { value, .. } => write!(f, "{}", value),
            TypedValue::String(value) => write!(f, "{}", value),
            TypedValue::Bool(value) => write!(f, "{}", value),
        }
    }
}

impl ToDatum for TypedValue {
    fn convert(&self) -> Datum {
        match self {
            TypedValue::Num {
                value,
                type_family: SqlTypeFamily::SmallInt,
            } => Datum::from_i16(value.to_i16().unwrap()),
            TypedValue::Num {
                value,
                type_family: SqlTypeFamily::Integer,
            } => Datum::from_i32(value.to_i32().unwrap()),
            TypedValue::Num {
                value,
                type_family: SqlTypeFamily::BigInt,
            } => Datum::from_i64(value.to_i64().unwrap()),
            TypedValue::String(str) => Datum::from_string(str.clone()),
            TypedValue::Bool(boolean) => Datum::from_bool(*boolean),
            _ => unreachable!(),
        }
    }
}
