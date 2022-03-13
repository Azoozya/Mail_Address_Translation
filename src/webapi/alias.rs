use crate::webapi::DBase;
use crate::webapi::MATRow;
use crate::webapi::MATable;

use crate::webapi::Domain;
use crate::webapi::User;

use crate::webapi::SQLITE_FILE;

use crate::webapi::Form;
use crate::webapi::NamedFile;
use crate::webapi::NotFound;
use crate::webapi::Status;

use crate::webapi::MATError;

use crate::webapi::generate_random_b32_string;

// #############################    ALIAS    #############################

#[derive(FromForm)]
pub struct Address {
    user: User,
    domain: Domain,
}

impl Address {
    pub fn clean(&mut self) -> Result<(), MATError> {
        self.domain.clean()?;
        self.user.clean()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn new(usr: String, dmn: String) -> Address {
        Address {
            user: User::new(usr),
            domain: Domain::new(dmn),
        }
    }

    #[allow(dead_code)]
    pub fn user(&self) -> User {
        self.user.clone()
    }

    #[allow(dead_code)]
    pub fn domain(&self) -> Domain {
        self.domain.clone()
    }

    #[allow(dead_code)]
    pub fn clone(&self) -> Address {
        Address {
            user: self.user(),
            domain: self.domain(),
        }
    }
}

// Generate alias : gen_alias GET
// Request for an alias (submit client/domain) new_alias POST
#[get("/new_alias")]
pub async fn get_new_alias() -> Result<NamedFile, NotFound<String>> {
    NamedFile::open("static/forms/new_alias.html")
        .await
        .map_err(|e| NotFound(e.to_string()))
}

#[post("/new_alias", data = "<args_alias>")]
pub async fn post_new_alias(mut args_alias: Form<Address>) -> Result<String, Status> {
    if let Err(e) = args_alias.clean() {
        if cfg!(debug_assertions) {
            println!("{:#?}", e);
        }
        return Err(e.to_status());
    }

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
    // Check if user exist
    let usr = MATRow::User{
        id: 0,
        name: args_alias.user.name(),
        pass: String::from("000000"),
    };

    // If not found return error/custom message
    let usr_id = tabl.find(&db,&usr);
    if usr_id == -1 {
        return Err(MATError::DBNotFound.to_status())
    }
    // retrieve id
    // Check if domain exist
    let tabld = MATable::Domains;
    let mut dmn = MATRow::Domain{
        id: 0,
        name: args_alias.domain.domain(),
        nb_ref: 0,
    };

    let dmn_id;

    // if doesn't exist it create
    if !tabld.insert(&db,&mut dmn){
        // 2 possibilities , it doesn't exist OR an undefined error occurred
        dmn_id = tabld.find(&db,&dmn);
        if dmn_id == -1 {
                // If not found that mean it's an undefined error
                return Err(MATError::DBNotFound.to_status())
        }
    }
    else {
        dmn_id = dmn.id();
    }
    let (left, right) = generate_random_b32_string();

    let tabl = MATable::Aliases;
    let mut als = MATRow::Alias {
        id: 0,
        name: format!("{}.{}", left, right),
    };

    if !tabl.insert(&db,&mut als){
        return Err(MATError::DBError.to_status());
    };

    let als_id = als.id();
    let table = MATable::Address;
    let mut adr = MATRow::Address {
        user: usr_id,
        alias: als_id,
        domain: dmn_id,
    };

    if !table.insert(&db,&mut adr){
        tabl.delete_by_id(&db,&als);
        return Err(MATError::DBError.to_status());
    };
    // insert alias , using ids


    // increment domain nb_ref
    match tabld.updt_ref(&db,dmn_id,1) {
        Err(_) => {
            tabl.delete_by_id(&db,&als);
            tabl.delete_by_id(&db,&adr);
            return Err(MATError::DBError.to_status());
        },
        Ok(b) => if !b { return  Err(MATError::DBNotFound.to_status()); },
    };

    /* delete conn */
    if let Err((_, e)) = conn.close() {
        if cfg!(debug_assertions) {
            println!("{:#?}", e);
        }
    };

    Ok(format!("{}.{}@{}", left, right,dmn.name()))
}
