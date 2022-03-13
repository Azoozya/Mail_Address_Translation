use crate::webapi::DBase;
use crate::webapi::MATRow;
use crate::webapi::MATable;

use crate::webapi::SQLITE_FILE;

use crate::webapi::Form;
use crate::webapi::NamedFile;
use crate::webapi::NotFound;
use crate::webapi::Status;

use crate::webapi::MATError;
use crate::webapi::decode;

// #############################    DOMAIN    #############################

#[derive(FromForm)]
pub struct Domain {
    domain: String,
}

impl Domain {
    pub fn new(domain: String) -> Domain {
        Domain { domain }
    }

    pub fn domain(&self) -> String {
        self.domain.clone()
    }

    fn decode_value(encoded: String) -> String {
        match decode(&encoded) {
            Err(_) => String::from(""),
            Ok(s) => s.to_string(),
        }
    }

    fn cut_head(input: String) -> Result<String, MATError> {
        let mut iter = input.split(".");
        let cnt = iter.clone().count();

        let mut domain = String::from("");
        if cnt < 2 {
            return Err(MATError::NotAnURI);
        }
        // limit to three.two.one (www.camel.lama)
        else if cnt == 2 {
            if let Some(s) = iter.nth(cnt - 2) {
                if s.is_empty() {
                    return Err(MATError::NotAnURI);
                }
                domain.push_str(s);
            }
        } else if cnt > 2 {
            if let Some(s) = iter.nth(cnt - 3) {
                if s.is_empty() {
                    return Err(MATError::NotAnURI);
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

    fn cut_tail(input: String) -> Result<String, MATError> {
        let mut iter = input.split("/");

        match iter.next() {
            Some(s) => {
                if s.is_empty() {
                    Err(MATError::NotAnURI)
                } else {
                    Ok(String::from(s))
                }
            }
            None => Err(MATError::NotAnURI),
        }
    }

    pub fn clean(&mut self) -> Result<(), MATError> {
        let mut domain = self.domain();

        domain = Domain::decode_value(domain);
        if domain.is_empty() {
            return Err(MATError::URLEnconding);
        }
        domain = Domain::cut_tail(domain)?;
        domain = Domain::cut_head(domain)?;

        self.domain = domain;
        Ok(())
    }

    pub fn clone(&self) -> Domain {
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
            return Err(MATError::DBError.to_status());
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
        return Err(MATError::DBAlreadyIn.to_status());
    }

    /* delete conn */
    if let Err((_, e)) = conn.close() {
        if cfg!(debug_assertions) {
            println!("{:#?}", e);
        }
    };
    Ok(format!("{} added in base !", dmn.name()))
}
