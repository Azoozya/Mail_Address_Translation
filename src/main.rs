use crate::test::full_test;
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
    // Open the connection
    let path = String::from("data/data.db");
    full_test(path);

    Ok(())
}
// ###########################################   MAIN   ######################################

