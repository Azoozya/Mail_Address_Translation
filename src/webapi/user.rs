use crate::webapi::DBase;
use crate::webapi::MATRow;
use crate::webapi::MATable;

use crate::webapi::SQLITE_FILE;

use crate::webapi::Form;
use crate::webapi::NamedFile;
use crate::webapi::NotFound;
use crate::webapi::Status;

use crate::webapi::MATError;
// #############################    USER    #############################

#[derive(FromForm)]
pub struct User {
    name: String,
}

impl User {
    #[allow(dead_code)]
    pub fn new(name: String) -> User {
        User { name }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn clean(&mut self) -> Result<(), MATError> {
        let name = self.name();
        /*
            Work with it. url decode ? Squeeze special char ?
        */
        // If empty return "Debug" User
        if name.is_empty() {
            Err(MATError::Empty)
        } else {
            self.name = name;
            Ok(())
        }
    }

    #[allow(dead_code)]
    pub fn clone(&self) -> User {
        User { name: self.name() }
    }
}

// New user : new_user GET/POST
#[get("/submit_user")]
pub async fn get_submit_user() -> Result<NamedFile, NotFound<String>> {
    NamedFile::open("static/forms/submit_user.html")
        .await
        .map_err(|e| NotFound(e.to_string()))
}

#[post("/submit_user", data = "<username>")]
pub async fn post_submit_user(mut username: Form<User>) -> Result<String, Status> {
    let usr = match username.clean() {
        Err(e) => {
            if cfg!(debug_assertions) {
                println!("{:#?}", e);
            }
            return Err(e.to_status());
        }
        Ok(_) => username.name(),
    };

    // Request db
    /* init conn */
    let path = SQLITE_FILE.clone();
    let conn = match rusqlite::Connection::open(&path) {
        Err(e) => {
            if cfg!(debug_assertions) {
                println!("{:#?}", e);
            }
            return Err(MATError::DBError.to_status());
        }
        Ok(co) => co,
    };

    /* interact with db*/
    let db = DBase::new(&path, &conn);
    let tabl = MATable::Users;

    let mut usr = MATRow::User {
        id: 0,
        name: usr,
        pass: String::from("000000"),
    };

    // Return !true (= false) if success , false otherwise (already in or any problem)
    if !tabl.insert(&db, &mut usr) {
        return Err(MATError::DBAlreadyIn.to_status());
    }

    /* delete conn */
    if let Err((_, e)) = conn.close() {
        if cfg!(debug_assertions) {
            println!("{:#?}", e);
        }
    };
    Ok(format!("{} added in base !", usr.name()))
}

// List address (submit client , hide pass) get_address POST
#[get("/list_address")]
pub async fn get_list_address() -> Result<NamedFile, NotFound<String>> {
    NamedFile::open("static/forms/list_address.html")
        .await
        .map_err(|e| NotFound(e.to_string()))
}

#[post("/list_address", data = "<args_user>")]
pub async fn post_list_address(mut args_user: Form<User>) -> Result<String, Status> {
    // Retrieve user input
    let usr = match args_user.clean() {
        Err(e) => {
            if cfg!(debug_assertions) {
                println!("{:#?}", e);
            }
            return Err(e.to_status());
        }
        Ok(_) => args_user.name(),
    };

    // Request db
    /* init conn */
    let path = SQLITE_FILE.clone();
    let conn = match rusqlite::Connection::open(&path) {
        Err(e) => {
            if cfg!(debug_assertions) {
                println!("{:#?}", e);
            }
            return Err(MATError::DBError.to_status());
        }
        Ok(co) => co,
    };

    /* interact with db*/
    let db = DBase::new(&path, &conn);
    // get id corresponding to user
    let tabl = MATable::Users;
    let usr = MATRow::User{
        id: 0,
        name: usr,
        pass: String::from("000000"),
    };

    // If not found return error/custom message
    let usr_id = tabl.find(&db,&usr);
    if usr_id == -1 {
        return Err(MATError::DBNotFound.to_status())
    }
    // let (answ, len) =
    //     match tabl.select(&db, String::from("id"), format!("WHERE `name` = '{}'", usr)) {
    //         Err(e) => {
    //             if cfg!(debug_assertions) {
    //                 println!("{:#?}", e);
    //             }
    //             return Err(MATError::DBError.to_status());
    //         }
    //         Ok((answ, len)) => (answ, len),
    //     };
    //
    // if len != 1 {
    //     if len == 0 {
    //         return Err(MATError::DBNotFound.to_status());
    //     } else {
    //         // Multiple time same user , should not exist
    //         return Err(MATError::DBError.to_status());
    //     }
    // }

    // get list of address associated to the id just retrieved
    let tabl = MATable::Address;
    let (answ, len) = match tabl.select(
        &db,
        format!("WHERE `user` = {}", usr_id),
    ) {
        Err(e) => {
            if cfg!(debug_assertions) {
                println!("{:#?}", e);
            }
            return Err(MATError::DBError.to_status());
        }
        Ok((answ, len)) => (answ, len),
    };

    let mut ret = String::from("");
    let tabla = MATable::Aliases;
    let mut als_id;
    let tabld = MATable::Domains;
    let mut dmn_id;
    /* ##########################        Template         ####################### */
    for i in 0..len {
        if let MATRow::Address { user:_, alias, domain } = &answ[i] {
            als_id = alias;
            dmn_id = domain;
        }
        else { return Err(MATError::DBError.to_status());}

        let vec = match tabla.select(
            &db,
            format!("where `id` = {}",als_id),
        ) {
            Err(e) => {
                if cfg!(debug_assertions) {
                    println!("{:#?}", e);
                }
                return Err(MATError::DBError.to_status());
            }
            Ok((v, _)) => v,
        };
        ret.push_str(&vec[0].name());
        ret.push('@');

        let vec = match tabld.select(
            &db,
            format!("where `id` = {}",dmn_id),
        ) {
            Err(e) => {
                if cfg!(debug_assertions) {
                    println!("{:#?}", e);
                }
                return Err(MATError::DBError.to_status());
            }
            Ok((v, _)) => v,
        };
        ret.push_str(&vec[0].name());
        ret.push('\n');
    }
    /* ##########################        Template          ####################### */

    /* delete conn */
    if let Err((_, e)) = conn.close() {
        if cfg!(debug_assertions) {
            println!("{:#?}", e);
        }
    };

    Ok(ret)
}
