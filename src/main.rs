use crate::base::DBase;
use crate::row::MATRow;
use crate::table::MATable;

use rusqlite;

mod base;
mod row;
mod table;
mod test;

// ###########################################   MAIN   ######################################
fn main() -> Result<(), rusqlite::Error> {
    let path = String::from("data/data.db");

    Ok(())
}
// ###########################################   MAIN   ######################################
