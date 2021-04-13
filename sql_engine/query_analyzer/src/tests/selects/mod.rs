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

#[cfg(test)]
mod expressions;
#[cfg(test)]
mod general_cases;

fn select_with_columns(schema_name: &str, table_name: &str, select_items: Vec<SelectItem>) -> Query {
    Query::Select(SelectStatement {
        projection_items: select_items,
        schema_name: schema_name.to_owned(),
        table_name: table_name.to_owned(),
        where_clause: None,
    })
}

fn select(schema_name: &str, table_name: &str) -> Query {
    select_with_columns(schema_name, table_name, vec![SelectItem::Wildcard])
}
