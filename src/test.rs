#[cfg(test)]
pub mod tests {

    use crate::sql::sqlite::base::DBase;
    use crate::sql::sqlite::row::MATRow;
    use crate::sql::sqlite::table::MATable;

    // use crate::webapi::domain::Domain;
    // use crate::webapi::user::User;
    // use crate::webapi::address::Address;

    // https://api.rocket.rs/v0.5-rc/rocket/local/blocking/struct.Client.html
    use rocket::local::blocking::Client;
    //https://api.rocket.rs/v0.5-rc/rocket/http/struct.ContentType.html#associatedconstant.Form
    use rocket::http::{ContentType,Status};

    #[test]
    pub fn full_test_sql() -> () {
        let path = String::from("./test.db");
        let conn = match rusqlite::Connection::open(&path) {
            Err(e) => {
                println!("{}", e);
                return;
            }
            Ok(co) => co,
        };

        let mut metadb = DBase::init(&path, &conn);
        assert_eq!(test_create_drop(&metadb, false), true);

        metadb = DBase::init(&path, &conn);
        assert_eq!(test_insert_delete(&metadb, true), true);
        DBase::release(&mut metadb);

        metadb = DBase::init(&path, &conn);
        assert_eq!(test_select(&metadb, false), true);
        DBase::release(&mut metadb);

        if let Err((_, e)) = conn.close() {
            println!("{}", e);
        };
    }

    fn _test_insert_delete_user(db: &DBase, verbose: bool) -> bool {
        // Set data
        let tabl = MATable::Users;

        let mut usr = MATRow::User {
            id: 0,
            name: String::from("Aa"),
            pass: String::from("Aa"),
        };
        let mut uusr = MATRow::User {
            id: 0,
            name: String::from("bB"),
            pass: String::from("Bb"),
        };
        let mut uuusr = MATRow::User {
            id: 0,
            name: String::from("cC"),
            pass: String::from("cC"),
        };
        let mut uuuusr = MATRow::User {
            id: 0,
            name: String::from("Dd"),
            pass: String::from("dD"),
        };

        let mut result = true;

        // Tries of insert
        result &= tabl.insert(db, &mut usr);
        result &= tabl.insert(db, &mut uusr);
        result &= tabl.insert(db, &mut uuusr);
        result &= tabl.insert(db, &mut uuuusr);
        result &= !tabl.insert(db, &mut uuuusr); // Insert should return false cuz this user has already been inserted

        // Tries of delete
        result &= tabl.delete_by_name(db, &mut usr);
        result &= tabl.delete_by_name(db, &mut uusr);
        result &= tabl.delete_by_name(db, &mut uuusr);
        result &= tabl.delete_by_name(db, &mut uuuusr);

        // Print and return
        if verbose {
            println!(
                "[Insert/Delete: Users] {}",
                if result { "Success" } else { "Failed" }
            );
        }
        result
    }

    fn _test_insert_delete_alias(db: &DBase, verbose: bool) -> bool {
        // Set data
        let tabl = MATable::Aliases;

        let mut als = MATRow::Alias {
            id: 0,
            name: String::from("Aa"),
        };
        let mut aals = MATRow::Alias {
            id: 0,
            name: String::from("bB"),
        };
        let mut aaals = MATRow::Alias {
            id: 0,
            name: String::from("Cc"),
        };
        let mut aaaals = MATRow::Alias {
            id: 0,
            name: String::from("dD"),
        };

        let mut result = true;

        // Tries of insert
        result &= tabl.insert(db, &mut als);
        result &= tabl.insert(db, &mut aals);
        result &= tabl.insert(db, &mut aaals);
        result &= tabl.insert(db, &mut aaaals);
        result &= !tabl.insert(db, &mut aaaals); // Insert should return false cuz this user has already been inserted

        // Tries of delete
        result &= tabl.delete_by_name(db, &mut als);
        result &= tabl.delete_by_name(db, &mut aals);
        result &= tabl.delete_by_name(db, &mut aaals);
        result &= tabl.delete_by_name(db, &mut aaaals);

        // Print and return
        if verbose {
            println!(
                "[Insert/Delete: Aliases] {}",
                if result { "Success" } else { "Failed" }
            );
        }
        result
    }

    fn _test_insert_delete_domain(db: &DBase, verbose: bool) -> bool {
        // Set data
        let tabl = MATable::Domains;

        let mut dmn = MATRow::Domain {
            id: 0,
            name: String::from("aa"),
            nb_ref: -423654,
        };
        let mut ddmn = MATRow::Domain {
            id: 0,
            name: String::from("BB"),
            nb_ref: 0xFFFF,
        };
        let mut dddmn = MATRow::Domain {
            id: 0,
            name: String::from("CC"),
            nb_ref: 0,
        };
        let mut ddddmn = MATRow::Domain {
            id: 0,
            name: String::from("dd"),
            nb_ref: 0o100,
        };

        let mut result = true;

        // Tries of insert
        result &= tabl.insert(db, &mut dmn);
        result &= tabl.insert(db, &mut ddmn);
        result &= tabl.insert(db, &mut dddmn);
        result &= tabl.insert(db, &mut ddddmn);
        result &= !tabl.insert(db, &mut ddddmn); // Insert should return false cuz this user has already been inserted

        // Tries of delete
        result &= tabl.delete_by_id(db, &mut dmn);
        result &= tabl.delete_by_id(db, &mut ddmn);
        result &= tabl.delete_by_id(db, &mut dddmn);
        result &= tabl.delete_by_id(db, &mut ddddmn);

        // Print and return
        if verbose {
            println!(
                "[Insert/Delete: Domains] {}",
                if result { "Success" } else { "Failed" }
            );
        }
        result
    }

    fn _test_insert_delete_address(db: &DBase, verbose: bool) -> bool {
        // Set data
        let mut tabl = MATable::Users;

        let mut usr = MATRow::User {
            id: 0,
            name: String::from("Aa"),
            pass: String::from("Aa"),
        };
        let mut uusr = MATRow::User {
            id: 0,
            name: String::from("bB"),
            pass: String::from("Bb"),
        };
        tabl.insert(db, &mut usr);
        tabl.insert(db, &mut uusr);

        tabl = MATable::Aliases;

        let mut als = MATRow::Alias {
            id: 0,
            name: String::from("aA"),
        };
        let mut aals = MATRow::Alias {
            id: 0,
            name: String::from("Bb"),
        };
        tabl.insert(db, &mut als);
        tabl.insert(db, &mut aals);

        tabl = MATable::Domains;

        let mut dmn = MATRow::Domain {
            id: 0,
            name: String::from("AA"),
            nb_ref: 0xFFFF,
        };
        let mut ddmn = MATRow::Domain {
            id: 0,
            name: String::from("bb"),
            nb_ref: -423654,
        };
        tabl.insert(db, &mut dmn);
        tabl.insert(db, &mut ddmn);

        tabl = MATable::Address;

        let mut adr = MATRow::Address {
            user: usr.id(),
            alias: als.id(),
            domain: dmn.id(),
        };
        let mut aadr = MATRow::Address {
            user: uusr.id(),
            alias: aals.id(),
            domain: dmn.id(),
        };
        let mut aaadr = MATRow::Address {
            user: usr.id(),
            alias: als.id(),
            domain: ddmn.id(),
        };
        let mut aaaadr = MATRow::Address {
            user: uusr.id(),
            alias: aals.id(),
            domain: ddmn.id(),
        };

        let mut result = true;

        // Tries of insert
        result &= tabl.insert(db, &mut adr);
        result &= tabl.insert(db, &mut aadr);
        result &= tabl.insert(db, &mut aaadr);
        result &= tabl.insert(db, &mut aaaadr);
        //result &= !tabl.insert(db,&mut aaaadr); // Insert should return false cuz this user has already been inserted

        // Tries of delete
        result &= tabl.delete_by_id(db, &mut adr);
        result &= tabl.delete_by_id(db, &mut aadr);
        result &= tabl.delete_by_name(db, &mut aaadr);
        result &= tabl.delete_by_name(db, &mut aaaadr);

        // Print and return
        if verbose {
            println!(
                "[Insert/Delete: Address] {}",
                if result { "Success" } else { "Failed" }
            );
        }
        result
    }

    fn test_insert_delete(db: &DBase, verbose: bool) -> bool {
        let mut result = true;

        result &= _test_insert_delete_user(&db, verbose);
        result &= _test_insert_delete_alias(&db, verbose);
        result &= _test_insert_delete_domain(&db, verbose);
        result &= _test_insert_delete_address(&db, verbose);

        println!(
            "[Insert/Delete: table] {}",
            if result { "Success" } else { "Failed" }
        );
        result
    }

    fn _test_create_drop(db: &DBase, tabl: MATable, verbose: bool) -> bool {
        tabl.drop(&db);
        let result = tabl.create(&db) & tabl.drop(&db);

        if verbose {
            println!(
                "[Create/Drop: {}] {}",
                MATable::get(tabl),
                if result { "Success" } else { "Failed" }
            );
        }
        result
    }

    fn test_create_drop(db: &DBase, verbose: bool) -> bool {
        let mut result = true;

        result &= _test_create_drop(&db, MATable::Users, verbose);
        result &= _test_create_drop(&db, MATable::Aliases, verbose);
        result &= _test_create_drop(&db, MATable::Domains, verbose);
        result &= _test_create_drop(&db, MATable::Address, verbose);

        println!(
            "[Create/Drop: table] {}",
            if result { "Success" } else { "Failed" }
        );
        result
    }

    fn _test_select(db: &DBase, tabl: MATable, goal: usize, verbose: bool) -> bool {
        let (v, nb) = match tabl.select(&db, String::from("")) {
            Err(e) => {
                println!("{}", e);
                return false;
            }
            Ok((v, nb)) => (v, nb),
        };
        if verbose {
            for i in 0..nb {
                println!("{:#?}", v[i]);
            }
        }
        if verbose {
            println!(
                "[Select: {}] {}",
                MATable::get(tabl),
                if nb == goal { "Success" } else { "Failed" }
            );
        }
        nb == goal
    }

    //Call it after test_insert
    fn test_select(db: &DBase, verbose: bool) -> bool {
        // Set data

        let mut tabl = MATable::Users;

        let mut usr = MATRow::User {
            id: 0,
            name: String::from("Aa"),
            pass: String::from("Aa"),
        };
        let mut uusr = MATRow::User {
            id: 0,
            name: String::from("bB"),
            pass: String::from("Bb"),
        };
        tabl.insert(db, &mut usr);
        tabl.insert(db, &mut uusr);

        tabl = MATable::Aliases;

        let mut als = MATRow::Alias {
            id: 0,
            name: String::from("aA"),
        };
        let mut aals = MATRow::Alias {
            id: 0,
            name: String::from("Bb"),
        };
        tabl.insert(db, &mut als);
        tabl.insert(db, &mut aals);

        tabl = MATable::Domains;

        let mut dmn = MATRow::Domain {
            id: 0,
            name: String::from("AA"),
            nb_ref: 0xFFFF,
        };
        let mut ddmn = MATRow::Domain {
            id: 0,
            name: String::from("bb"),
            nb_ref: -423654,
        };
        tabl.insert(db, &mut dmn);
        tabl.insert(db, &mut ddmn);

        tabl = MATable::Address;

        let mut adr = MATRow::Address {
            user: usr.id(),
            alias: als.id(),
            domain: dmn.id(),
        };
        let mut aadr = MATRow::Address {
            user: uusr.id(),
            alias: aals.id(),
            domain: dmn.id(),
        };
        tabl.insert(db, &mut adr);
        tabl.insert(db, &mut aadr);

        let mut result = true;

        // Tries of select
        result &= _test_select(&db, MATable::Users, 2, verbose);
        result &= _test_select(&db, MATable::Aliases, 2, verbose);
        result &= _test_select(&db, MATable::Domains, 2, verbose);
        result &= _test_select(&db, MATable::Address, 2, verbose);

        println!(
            "[Select: table] {}",
            if result { "Success" } else { "Failed" }
        );
        result
    }

    //https://rocket.rs/v0.5-rc/guide/testing/
    #[test]
    pub fn full_test_webapi() -> (){
        assert_eq!(_clean(), true);
        assert_eq!(test_user(), true);
        assert_eq!(test_domain(), true);
        assert_eq!(test_address(), true);
        _clean();
    }


    fn _clean() -> bool {
        let client = Client::tracked(crate::rocket()).expect("valid rocket instance");
        let response = client.get("/clean").dispatch();
        response.status() == Status::Ok
    }

    fn test_user() -> bool {
        let client = Client::tracked(crate::rocket()).expect("valid rocket instance");

        let get = client.get("/submit_user").dispatch();
        if get.status() != Status::Ok
        {
            println!("[GET] test user : {}",get.status());
            return false;
        }

        let post = client.post("/submit_user").header(ContentType::Form).body("name=lama").dispatch();
        if post.status() != Status::Ok
        {
            println!("[POST] test user : {}",post.status());
            return false;
        }

        // Should cause an error, due to conflict it creates in db
        // # Good choice ? -> Sherlock
        let repost = client.post("/submit_user").header(ContentType::Form).body("name=lama").dispatch();
        if repost.status() != Status::Conflict
        {
            println!("[RE-POST] test user : {}",repost.status());
            return false;
        }

        true
    }

    fn test_domain() -> bool {
        let client = Client::tracked(crate::rocket()).expect("valid rocket instance");
        
        let get = client.get("/submit_domain").dispatch();
        if get.status() != Status::Ok
        {
            println!("[GET] test domain : {}",get.status());
            return false;
        }

        let post = client.post("/submit_domain").header(ContentType::Form).body("domain=lama.fr").dispatch();
        if post.status() != Status::Ok
        {
            println!("[POST] test domain : {}",post.status());
            return false;
        }

        let repost = client.post("/submit_domain").header(ContentType::Form).body("domain=lama.fr").dispatch();
        if repost.status() != Status::Conflict
        {
            println!("[RE-POST] test domain : {}",repost.status());
            return false;
        }

        true
    }

    fn test_address() -> bool {
        let client = Client::tracked(crate::rocket()).expect("valid rocket instance");

        let get = client.get("/new_address").dispatch();
        if get.status() != Status::Ok
        {
            println!("[GET] test address : {}",get.status());
            return false;
        }

        // Correct way
        let post = client.post("/new_address").header(ContentType::Form).body("user.name=lama&domain.domain=lama.fr").dispatch();
        if post.status() != Status::Ok
        {
            println!("[POST] test address : {}",post.status());
            return false;
        }

        let repost = client.post("/new_address").header(ContentType::Form).body("user.name=lama&domain.domain=lama.fr").dispatch();
        if repost.status() != Status::Ok
        {
            println!("[RE-POST] test address : {}",repost.status());
            return false;
        }

        // If user is missing in db
        let inv_user = client.post("/new_address").header(ContentType::Form).body("user.name=UNKNOWN&domain.domain=lama.fr").dispatch();
        if inv_user.status() == Status::Ok
        {
            println!("[User invalid POST] test address : {}",inv_user.status());
            return false;
        }

        // If domain is missing in db, should return true cuz it create the domain
        let inv_domain = client.post("/new_address").header(ContentType::Form).body("user.name=lama&domain.domain=UN.KNOWN").dispatch();
        if inv_domain.status() != Status::Ok
        {
            println!("[Domain invalid POST] test address : {}",inv_domain.status());
            return false;
        }

        let inv_domain_repost = client.post("/submit_domain").header(ContentType::Form).body("domain=UN.KNOWN").dispatch();
        if inv_domain_repost.status() != Status::Conflict
        {
            println!("[Domain invalid RE-POST] test address : {}",inv_domain_repost.status());
            return false;
        }

        // Listing

        let listing = client.post("/list_address").header(ContentType::Form).body("name=lama").dispatch();
        if listing.status() != Status::Ok
        {
            println!("[Listing POST] test address : {}",listing.status());
            return false;
        }

        //if user doesn't exist
        let inv_listing = client.post("/list_address").header(ContentType::Form).body("name=UNKNOWN").dispatch();
        if inv_listing.status() != Status::NoContent
        {
            println!("[Invalid listing POST] test address : {}",inv_listing.status());
            return false;
        }

        true
    }

}
