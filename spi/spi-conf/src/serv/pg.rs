pub mod conf_pg_initializer;
mod conf_pg_config_serv;
pub use conf_pg_config_serv::*;
mod conf_pg_namespace_serv; 
pub use conf_pg_namespace_serv::*;
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
        init.1.push_str((idx+1).to_string().as_str());
        values.push(value)
    };
    (init.0, init.1, values)
}