// Copyright 2020 - 2021 Alex Dukhno
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

use super::*;
use query_ast::Value;
use types::Str;

#[test]
fn string_literal() {
    assert_eq!(
        TreeBuilder::insert_position(Expr::Value(Value::String("literal".to_owned())), SqlType::Str { len: 20, kind: Str::Var }),
        Ok(UntypedTree::UnOp {
            op: UnOperator::Cast(SqlType::Str { len: 20, kind: Str::Var }),
            item: Box::new(UntypedTree::Item(UntypedItem::Const(UntypedValue::Literal("literal".to_owned()))))
        })
    );
}

#[test]
fn integer() {
    assert_eq!(
        TreeBuilder::insert_position(Expr::Value(Value::Int(1)), SqlType::small_int()),
        Ok(UntypedTree::UnOp {
            op: UnOperator::Cast(SqlType::small_int()),
            item: Box::new(UntypedTree::Item(UntypedItem::Const(UntypedValue::Int(1))))
        })
    );
}

#[test]
fn bigint() {
    assert_eq!(
        TreeBuilder::insert_position(Expr::Value(Value::Number("2147483648".to_owned())), SqlType::big_int()),
        Ok(UntypedTree::UnOp {
            op: UnOperator::Cast(SqlType::big_int()),
            item: Box::new(UntypedTree::Item(UntypedItem::Const(UntypedValue::BigInt(2147483648))))
        })
    );
    assert_eq!(
        TreeBuilder::insert_position(Expr::Value(Value::Number("-2147483649".to_owned())), SqlType::big_int()),
        Ok(UntypedTree::UnOp {
            op: UnOperator::Cast(SqlType::big_int()),
            item: Box::new(UntypedTree::Item(UntypedItem::Const(UntypedValue::BigInt(-2147483649))))
        })
    );
    assert_eq!(
        TreeBuilder::insert_position(Expr::Value(Value::Number("9223372036854775807".to_owned())), SqlType::big_int()),
        Ok(UntypedTree::UnOp {
            op: UnOperator::Cast(SqlType::big_int()),
            item: Box::new(UntypedTree::Item(UntypedItem::Const(UntypedValue::BigInt(9223372036854775807))))
        })
    );
    assert_eq!(
        TreeBuilder::insert_position(Expr::Value(Value::Number("-9223372036854775808".to_owned())), SqlType::big_int()),
        Ok(UntypedTree::UnOp {
            op: UnOperator::Cast(SqlType::big_int()),
            item: Box::new(UntypedTree::Item(UntypedItem::Const(UntypedValue::BigInt(-9223372036854775808))))
        })
    );
}

#[test]
fn numeric() {
    assert_eq!(
        TreeBuilder::insert_position(Expr::Value(Value::Number("-9223372036854775809".to_owned())), SqlType::big_int()),
        Ok(UntypedTree::UnOp {
            op: UnOperator::Cast(SqlType::big_int()),
            item: Box::new(UntypedTree::Item(UntypedItem::Const(UntypedValue::Number(
                BigDecimal::from_str("-9223372036854775809").unwrap()
            ))))
        })
    );
    assert_eq!(
        TreeBuilder::insert_position(Expr::Value(Value::Number("9223372036854775808".to_owned())), SqlType::big_int()),
        Ok(UntypedTree::UnOp {
            op: UnOperator::Cast(SqlType::big_int()),
            item: Box::new(UntypedTree::Item(UntypedItem::Const(UntypedValue::Number(
                BigDecimal::from_str("9223372036854775808").unwrap()
            ))))
        })
    );
}
