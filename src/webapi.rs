use std::thread;
use urlencoding::decode;

use rocket::fs::NamedFile;
use rand::{thread_rng, Rng};
use rocket::http::Status; // https://api.rocket.rs/v0.4/rocket/http/struct.Status.html#structfield.reason
use rocket::response::status::NotFound;
use rocket::form::Form;

use crate::sql::sqlite::base::DBase;
use crate::sql::sqlite::row::MATRow;
use crate::sql::sqlite::table::MATable;

use crate::webapi::user::User;
use crate::webapi::domain::Domain;
//use crate::webapi::address::Address;

use crate::error::MATError;
use crate::SQLITE_FILE;

pub mod user;
pub mod address;
pub mod domain;

fn base32(number: usize) -> String {
    let alphabet = "abcdefghijklmnopqrstuvwxyz234567";
    let mut ret = String::from("");
    // Detect if usize correspond to u32 or u64
    let archi_64 = cfg!(target_pointer_width = "64");
    let mut number = number;
    let mut tmp: usize;

    // base32 => 5 bits , 64 =  12*5 + 4 / 32 = 6 * 5 + 2
    for _ in 0..(if archi_64 { 12 } else { 6 }) {
        // get 5 bits from left
        tmp = number % 32;
        // update number value (equivalent to / 32)
        number = number >> 5;
        // make conversion into base32 by indexing our custom alphabet without padding
        match alphabet.char_indices().nth(tmp) {
            Some(c) => ret.push(c.1),
            None => ret.push('0'),
        };
    }

    ret
}

fn generate_random_b32_string() -> (String, String) {
    let mut rng = thread_rng();

    let left: usize = rng.gen();
    let right: usize = rng.gen();

    let left = thread::spawn(move || base32(left));
    let right = thread::spawn(move || base32(right));

    let left = left.join().unwrap();
    let right = right.join().unwrap();

    (left, right)
}

//TLS

// Login , Cookies / GET  / POST
#[get("/")]
pub async fn index() -> Result<NamedFile, NotFound<String>> {
    NamedFile::open("static/forms/index.html")
        .await
        .map_err(|e| NotFound(e.to_string()))
}


#[post("/", data = "<args>")]
pub fn index_post(args: String) -> String {
    args
}

#[get("/clean")]
pub fn clean_get() -> String {
    /* init conn */
    let path = SQLITE_FILE.clone();
    let conn = match rusqlite::Connection::open(&path) {
        Err(e) => {
            if cfg!(debug_assertions) {
                println!("{:#?}", e);
            }
            return String::from("Nothing to drop !");
        }
        Ok(co) => co,
    };

    /* interact with db*/
    let mut db = DBase::new(&path, &conn);

    DBase::release(&mut db);
    DBase::init(&path, &conn);

    /* delete conn */
    if let Err((_, e)) = conn.close() {
        if cfg!(debug_assertions) {
            println!("{:#?}", e);
        }
    };

    String::from("Table dropped !")
}
