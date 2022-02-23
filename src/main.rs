use crate::base::DBase;
use crate::table::MATable;
use crate::row::MATRow;

use rusqlite;

mod test;
mod table;
mod row;
mod base;



/*
#[derive(Debug)]
enum Err {
    Aled,
    DBerr(rusqlite::ErrorCode),
}
*/



// ###########################################   MAIN   ######################################
fn main() -> Result<(), rusqlite::Error> {
    let path = String::from("data/data.db");


    Ok(())
}
// ###########################################   MAIN   ######################################

