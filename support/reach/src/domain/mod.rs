pub mod reach_message;
pub mod reach_message_log;
pub mod reach_message_signature;
pub mod reach_message_template;
pub mod reach_trigger_global_config;
#[macro_export]
macro_rules! fill_by_mod_req {
    ($req:expr => {
        $($field: ident $(:$clone_stg:tt)?),*$(,)?
    } $active_model:expr ) => {
        use sea_orm::entity::ActiveValue;
        $(
            if let Some($field) = &$req.$field {
                $active_model.$field = ActiveValue::Set(fill_by_mod_req!(@clone $field $(,$clone_stg)?));
            }
        )*
    };
    /*
        clone or copy a reference value
    */ 
    // deref to copy 
    (@clone $value: expr, Copy) => {
        *$value
    };
    // call clone() to clone
    (@clone $value: expr, Clone) => {
        $value.clone()
    };
    // default for clone
    (@clone $value: expr) => {
        fill_by_mod_req!(@clone $value, Clone)
    }
}

#[macro_export]
macro_rules! fill_by_add_req {
    ($req:expr => {
        $($field: ident $(:$clone_stg:tt)?),*$(,)?
    } $active_model:expr ) => {
        use sea_orm::entity::ActiveValue;
        $(
            $active_model.$field = ActiveValue::Set(fill_by_add_req!(@clone $req.$field $(,$clone_stg)?));
        )*
    };
    /*
        clone or copy a reference value
    */ 
    // deref to copy 
    (@clone $value: expr, Copy) => {
        *$value
    };
    // call clone() to clone
    (@clone $value: expr, Clone) => {
        $value.clone()
    };
    // default for clone
    (@clone $value: expr) => {
        fill_by_add_req!(@clone $value, Clone)
    }
}
