#[macro_use] extern crate rocket;
use crate::webapi::{index,get_submit_user,post_submit_user,get_submit_domain,post_submit_domain};

use crate::base::DBase;
use crate::row::MATRow;
use crate::table::MATable;

use rusqlite;
mod base;
mod row;
mod table;
mod test;
mod webapi;
use rocket::{Rocket,Build};
use rocket::fs::{FileServer, relative};


// ###########################################   MAIN   ######################################
// #[rocket::main] + async + .launch().await + Error handling
// OR
// #[launch] + fn rocket() -> _

#[rocket::main]
async fn main() -> Result<(), rusqlite::Error> {
    let path = String::from("data/data.db");
	match rocket::build()
        .mount("/", FileServer::from(relative!("static/forms")))
        .mount("/", routes![index,get_submit_user,post_submit_user,get_submit_domain,post_submit_domain])
		.launch()
		.await {
		Ok(_) => Ok(()),
		Err(e) => { println!("{}",e); Ok(()) },
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
