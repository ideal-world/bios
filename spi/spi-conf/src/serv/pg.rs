pub mod conf_pg_initializer;

mod conf_pg_config_serv;
pub use conf_pg_config_serv::*;
mod conf_pg_namespace_serv;
pub use conf_pg_namespace_serv::*;
mod conf_pg_config_history_serv;
pub use conf_pg_config_history_serv::*;
mod conf_pg_nacos_mocker;

use tardis::db::sea_orm::Value;

fn gen_insert_sql_stmt<'a>(fields_and_values: impl IntoIterator<Item = (&'a str, Value)>) -> (String, String, Vec<Value>) {
    let mut init = (String::new(), String::new());
    let mut values = vec![];
    for (idx, (column, value)) in fields_and_values.into_iter().enumerate() {
        if idx > 0 {
            init.0.push_str(", ");
            init.1.push_str(", ");
        }
        init.0.push_str(column);
        init.1.push('$');
        init.1.push_str((idx + 1).to_string().as_str());
        values.push(value)
    }
    (init.0, init.1, values)
}

fn gen_select_sql_stmt<'a>(keys: impl IntoIterator<Item = (&'a str, &'a str, Value)>) -> (Option<String>, Vec<Value>) {
    let mut init = String::new();
    let mut values = vec![];
    for (idx, (column, op, value)) in keys.into_iter().enumerate() {
        if idx > 0 {
            init.push_str(", ");
        }
        init.push_str(column);
        init.push_str(op);
        init.push('$');
        init.push_str((idx + 1).to_string().as_str());
        values.push(value)
    }
    ((!init.is_empty()).then_some(init), values)
}

/// return set caluse, where caluse, values
fn gen_update_sql_stmt<'a>(
    fields_and_values: impl IntoIterator<Item = (&'a str, Value)>,
    key_and_values: impl IntoIterator<Item = (&'a str, Value)>,
) -> (String, String, Vec<Value>) {
    let (mut set_caluse, mut where_caluse) = (String::new(), String::new());
    let mut values = vec![];
    let mut placeholder_idx = 1;
    for (idx, (column, value)) in fields_and_values.into_iter().enumerate() {
        if idx > 0 {
            set_caluse.push_str(", ");
        }
        set_caluse.push_str(column);
        set_caluse.push_str(" = $");
        set_caluse.push_str((placeholder_idx).to_string().as_str());
        values.push(value);
        placeholder_idx += 1;
    }
    for (idx, (column, value)) in key_and_values.into_iter().enumerate() {
        if idx > 0 {
            where_caluse.push_str(" and ");
        }
        where_caluse.push_str(column);
        where_caluse.push_str(" = $");
        where_caluse.push_str((placeholder_idx).to_string().as_str());
        values.push(value);
        placeholder_idx += 1;
    }
    (set_caluse, where_caluse, values)
}
