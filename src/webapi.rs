use rand::{thread_rng, Rng};
use rocket::form::Form;
use rocket::fs::NamedFile;
use rocket::http::Status; // https://api.rocket.rs/v0.4/rocket/http/struct.Status.html#structfield.reason
use rocket::response::status::NotFound;
use std::thread;
use urlencoding::decode;

use crate::base::DBase;
use crate::row::MATRow;
use crate::table::MATable;

use crate::SQLITE_FILE;

#[derive(Debug)]
pub enum WebError {
    Empty,
    NotAnURI,
    URLEnconding,
    DBError,
    DBNotFound,
    DBAlreadyIn,
}

impl WebError {
    pub fn to_status(&self) -> Status {
        match self {
            WebError::Empty => Status::BadRequest,
            WebError::NotAnURI => Status::BadRequest,
            WebError::URLEnconding => Status::BadRequest,
            WebError::DBError => Status::InternalServerError,
            WebError::DBNotFound => Status::NoContent,
            WebError::DBAlreadyIn => Status::Conflict,
        }
    }
}

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

fn extract_value(input: String) -> String {
    let mut iter = input.split("=");
    iter.next(); // skip the attribute name
    let mut value = String::from("");
    // From browser '=' in the value field will be urlencoded, we let them like this
    // If someone try to submit '=' it will just not be restored
    loop {
        match iter.next() {
            Some(s) => value.push_str(s),
            None => break,
        };
    }
    value
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

// #############################    USER    #############################

#[derive(FromForm)]
pub struct User {
    name: String,
}

impl User {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn clean(&mut self) -> Result<(), WebError> {
        let name = self.name();
        /*
            Work with it. url decode ? Squeeze special char ?
        */
        // If empty return "Debug" User
        if name.is_empty() {
            Err(WebError::Empty)
        } else {
            self.name = name;
            Ok(())
        }
    }

    fn clone(&self) -> User {
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
            return Err(WebError::DBError.to_status());
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
        return Err(WebError::DBAlreadyIn.to_status());
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
            return Err(WebError::DBError.to_status());
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
        return Err(WebError::DBNotFound.to_status())
    }
    // let (answ, len) =
    //     match tabl.select(&db, String::from("id"), format!("WHERE `name` = '{}'", usr)) {
    //         Err(e) => {
    //             if cfg!(debug_assertions) {
    //                 println!("{:#?}", e);
    //             }
    //             return Err(WebError::DBError.to_status());
    //         }
    //         Ok((answ, len)) => (answ, len),
    //     };
    //
    // if len != 1 {
    //     if len == 0 {
    //         return Err(WebError::DBNotFound.to_status());
    //     } else {
    //         // Multiple time same user , should not exist
    //         return Err(WebError::DBError.to_status());
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
            return Err(WebError::DBError.to_status());
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
        if let MATRow::Address {user,alias,domain} = &answ[i] {
            als_id = alias;
            dmn_id = domain;
        }
        else { return Err(WebError::DBError.to_status());}

        let vec = match tabla.select(
            &db,
            format!("where `id` = {}",als_id),
        ) {
            Err(e) => {
                if cfg!(debug_assertions) {
                    println!("{:#?}", e);
                }
                return Err(WebError::DBError.to_status());
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
                return Err(WebError::DBError.to_status());
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

// #############################    DOMAIN    #############################

#[derive(FromForm)]
pub struct Domain {
    domain: String,
}

impl Domain {
    fn domain(&self) -> String {
        self.domain.clone()
    }

    fn decode_value(encoded: String) -> String {
        match decode(&encoded) {
            Err(_) => String::from(""),
            Ok(s) => s.to_string(),
        }
    }

    fn cut_head(input: String) -> Result<String, WebError> {
        let mut iter = input.split(".");
        let cnt = iter.clone().count();

        let mut domain = String::from("");
        if cnt < 2 {
            return Err(WebError::NotAnURI);
        }
        // limit to three.two.one (www.camel.lama)
        else if cnt == 2 {
            if let Some(s) = iter.nth(cnt - 2) {
                if s.is_empty() {
                    return Err(WebError::NotAnURI);
                }
                domain.push_str(s);
            }
        } else if cnt > 2 {
            if let Some(s) = iter.nth(cnt - 3) {
                if s.is_empty() {
                    return Err(WebError::NotAnURI);
                }
                domain.push_str(s);
            }
        }

        // Will be one or two iteration max
        loop {
            match iter.next() {
                Some(s) => {
                    domain.push('.');
                    domain.push_str(s)
                }
                None => break,
            };
        }

        Ok(domain)
    }

    fn cut_tail(input: String) -> Result<String, WebError> {
        let mut iter = input.split("/");

        match iter.next() {
            Some(s) => {
                if s.is_empty() {
                    Err(WebError::NotAnURI)
                } else {
                    Ok(String::from(s))
                }
            }
            None => Err(WebError::NotAnURI),
        }
    }

    fn clean(&mut self) -> Result<(), WebError> {
        let mut domain = self.domain();

        domain = Domain::decode_value(domain);
        if domain.is_empty() {
            return Err(WebError::URLEnconding);
        }
        domain = Domain::cut_tail(domain)?;
        domain = Domain::cut_head(domain)?;

        self.domain = domain;
        Ok(())
    }

    fn clone(&self) -> Domain {
        Domain {
            domain: self.domain(),
        }
    }
}

//[*.]<maybe.>domain.root[/*]
// New domain : new_domain POST
#[get("/submit_domain")]
pub async fn get_submit_domain() -> Result<NamedFile, NotFound<String>> {
    NamedFile::open("static/forms/submit_domain.html")
        .await
        .map_err(|e| NotFound(e.to_string()))
}

#[post("/submit_domain", data = "<domain>")]
pub async fn post_submit_domain(mut domain: Form<Domain>) -> Result<String, Status> {
    let dmn = match domain.clean() {
        Err(e) => {
            if cfg!(debug_assertions) {
                println!("{:#?}", e);
            }
            return Err(e.to_status());
        }
        Ok(_) => domain.domain(),
    };

    // Request db
    /* init conn */
    let path = SQLITE_FILE.clone();
    let conn = match rusqlite::Connection::open(&path) {
        Err(e) => {
            if cfg!(debug_assertions) {
                println!("{:#?}", e);
            }
            return Err(WebError::DBError.to_status());
        }
        Ok(co) => co,
    };

    /* interact with db*/
    let db = DBase::new(&path, &conn);
    let tabl = MATable::Domains;

    let mut dmn = MATRow::Domain {
        id: 0,
        name: dmn,
        nb_ref: 0,
    };

    // Return !true (= false) if success , false otherwise (already in or any problem)
    if !tabl.insert(&db, &mut dmn) {
        return Err(WebError::DBAlreadyIn.to_status());
    }

    /* delete conn */
    if let Err((_, e)) = conn.close() {
        if cfg!(debug_assertions) {
            println!("{:#?}", e);
        }
    };
    Ok(format!("{} added in base !", dmn.name()))
}

// #############################    ALIAS    #############################

#[derive(FromForm)]
pub struct Address {
    user: User,
    domain: Domain,
}

impl Address {
    pub fn clean(&mut self) -> Result<(), WebError> {
        self.domain.clean()?;
        self.user.clean()?;
        Ok(())
    }

    pub fn new(usr: &String, dmn: &String) -> Address {
        Address {
            user: User { name: usr.clone() },
            domain: Domain {
                domain: dmn.clone(),
            },
        }
    }

    pub fn user(&self) -> User {
        self.user.clone()
    }

    pub fn domain(&self) -> Domain {
        self.domain.clone()
    }

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
            return Err(WebError::DBError.to_status());
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
        return Err(WebError::DBNotFound.to_status())
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
                return Err(WebError::DBNotFound.to_status())
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
        return Err(WebError::DBError.to_status());
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
        return Err(WebError::DBError.to_status());
    };
    // insert alias , using ids


    // increment domain nb_ref
    match tabld.updt_ref(&db,dmn_id,1) {
        Err(e) => {
            tabl.delete_by_id(&db,&als);
            tabl.delete_by_id(&db,&adr);
            return Err(WebError::DBError.to_status());
        },
        Ok(b) => if !b { return  Err(WebError::DBNotFound.to_status()); },
    };

    /* delete conn */
    if let Err((_, e)) = conn.close() {
        if cfg!(debug_assertions) {
            println!("{:#?}", e);
        }
    };

    Ok(format!("{}.{}@{}", left, right,dmn.name()))
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
