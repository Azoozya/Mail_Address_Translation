#[macro_use]
extern crate rocket;

use crate::webapi::{index,index_post,clean_get};
use crate::webapi::user::{
    get_list_address,
    post_list_address,
    get_submit_user,
    post_submit_user,
};
use crate::webapi::domain::{
    get_submit_domain,
    post_submit_domain,
};
use crate::webapi::alias::{
    get_new_alias,
    post_new_alias,
};

use crate::sql::sqlite::base::DBase;
use crate::sql::sqlite::row::MATRow;
//use crate::sqlite::table::MATable;


use rusqlite;
mod sql; // contains base,table,row
mod webapi;
mod test;
mod error;
use rocket::fs::{relative, FileServer};
//use rocket::{Build, Rocket};

use lazy_static::lazy_static;

lazy_static! {
    static ref SQLITE_FILE: String = String::from("data/data.db");
}
// ###########################################   MAIN   ######################################
// #[rocket::main] + async + .launch().await + Error handling
// OR
// #[launch] + fn rocket() -> _

#[rocket::main]
async fn main() -> Result<(), rusqlite::Error> {
    let path = SQLITE_FILE.clone();
    let conn = match rusqlite::Connection::open(&path) {
        Err(e) => {
            println!("{}", e);
            return Err(e);
        }
        Ok(co) => co,
    };

    DBase::init(&path, &conn); //To be sure there is table
                               //DBase::release(&mut metadb); // to clear

    if let Err((_, e)) = conn.close() {
        println!("{}", e);
    };

    match rocket::build()
        .mount("/", FileServer::from(relative!("static/forms")))
        .mount(
            "/",
            routes![
                index,
                index_post,
                clean_get,

                get_list_address,
                post_list_address,
                get_submit_user,
                post_submit_user,

                get_submit_domain,
                post_submit_domain,

                get_new_alias,
                post_new_alias,
            ],
        )
        .launch()
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{}", e);
            Ok(())
        }
    }
}

// ###########################################   MAIN   ######################################

/*
#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::from(relative!("static/forms")))
        .mount("/", routes![index])
}*/
