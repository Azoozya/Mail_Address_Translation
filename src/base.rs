use crate::table::MATable;

#[derive(Debug)]
pub struct DBase<'a, 'b> {
    pub fname: &'a String,
    pub conn: &'b rusqlite::Connection,
    pub up: bool,
}

impl DBase<'_, '_> {
    // Check

    // Create
    pub fn init<'a, 'b>(fname: &'a String, conn: &'b rusqlite::Connection) -> DBase<'a, 'b> {
        let metadb = DBase::new(&fname, conn);
        println!("\tOpen db {}", metadb.fname);

        let tmp = MATable::Users;
        tmp.create(&metadb);
        let tmp = MATable::Aliases;
        tmp.create(&metadb);
        let tmp = MATable::Domains;
        tmp.create(&metadb);
        let tmp = MATable::Address;
        tmp.create(&metadb);

        return metadb;
    }

    // Delete
    // Drop tables and close the connection
    pub fn release(metadb: &mut DBase) -> () {
        let tmp = MATable::Users;
        tmp.drop(&metadb);
        let tmp = MATable::Aliases;
        tmp.drop(&metadb);
        let tmp = MATable::Domains;
        tmp.drop(&metadb);
        let tmp = MATable::Address;
        tmp.drop(&metadb);

        println!("\tClose db {}", metadb.fname);
        metadb.close();
    }

    // Register the connection
    pub fn new<'a, 'b>(fname: &'a String, conn: &'b rusqlite::Connection) -> DBase<'a, 'b> {
        DBase {
            fname,
            conn,
            up: true,
        }
    }

    pub fn close(&mut self) -> () {
        self.up = false;
    }
}
