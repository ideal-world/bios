use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Display, Debug, Deserialize, Serialize)]
pub enum DataTypeKind {
    #[display(fmt = "STRING")]
    STRING,
    #[display(fmt = "NUMBER")]
    NUMBER,
    #[display(fmt = "BOOLEAN")]
    BOOLEAN,
    #[display(fmt = "DATE")]
    DATE,
    #[display(fmt = "DATETIME")]
    DATETIME,
    #[display(fmt = "JSON")]
    JSON,
    #[display(fmt = "STRINGS")]
    STRINGS,
    #[display(fmt = "NUMBERS")]
    NUMBERS,
    #[display(fmt = "BOOLEANS")]
    BOOLEANS,
    #[display(fmt = "DATES")]
    DATES,
    #[display(fmt = "DATETIMES")]
    DATETIMES,
    #[display(fmt = "ARRAY")]
    ARRAY,
}
