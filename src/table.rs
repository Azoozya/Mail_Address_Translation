use crate::base::DBase;
use crate::row::MATRow;

pub enum MATable {
    Users,
    Aliases,
    Domains,
    Address,
}

/*

    Forge and transmit a request SELECT *what* FROM *self* *filter*.
    Return a Vector which contain every results and the size of the vector, doesn't handle error
    select(&self, db: &DBase, what: String, filter: String) -> Result<(Vec<MATRow>, usize),rusqlite::Error>

    Forge a request DELETE targeting by id/(alias,domain).
    Return the _delete result
    delete_by_id(&self, db: &DBase, value: &MATRow) -> bool

    Forge a request DELETE targeting by name/user.
    Return the _delete result
    delete_by_name(&self, db: &DBase, value: &MATRow) -> bool

    Forge and transmit a request CREATE.
    Return false if error occured, otherwise true
    create(&self, db: &DBase) -> bool

    Forge and transmit a request DROP.
    Return false if error occured, otherwise true
    drop(&self, db: &DBase) -> bool

    Check if the *value* is already in base, forge a request INSERT , retrieve id's autoincremented.
    Return false if value already in base (except for MATable::Address) , if _execute_no_param return false , if _retrieve_id returned an error , otherwise true;
    insert(&self, db: &DBase, value: &mut MATRow) -> bool

    Return the name of the table corresponding to *row*
    get(row: &MATable) -> &'static str

============================================================================================

    Execute the request *req* and handle error.
    Return true if success , otherwise false
    _execute_no_param(&self, db: &DBase, req: &str) -> bool

    Forge the request to test existence in base.
    Return the opposite of _exist result and handle error (return false)
    _unique_name(&self, db: &DBase, name: &String) -> bool

    Called by _unique_name to transmit the request.
    Return true if the request returned something, false if nothing , doesn't handle errors
    _exist(&self, db: &DBase, what: String, filter: String) -> Result<bool, rusqlite::Error>

    Return the name of the table corresponding to self
    _get(&self) -> &str

    Forge and transmit a requeset SELECT.
    Return the id/user if found one, error if none
    _retrieve_id(&self, db: &DBase, entity: &MATRow) -> Result<i32, rusqlite::Error>


    Called by delete_by_*. Transmit the request DELETE FROM *self* *filter*.
    Return true if success , otherwise false
    _delete(&self, db: &DBase, filter: String) -> bool

*/

impl MATable {
    // Get table name
    fn _get(&self) -> &str {
        match self {
            MATable::Users => "Users",
            MATable::Aliases => "Aliases",
            MATable::Domains => "Domains",
            MATable::Address => "Address",
        }
    }

    pub fn get(row: MATable) -> &'static str {
        match row {
            MATable::Users => "Users",
            MATable::Aliases => "Aliases",
            MATable::Domains => "Domains",
            MATable::Address => "Address",
        }
    }

    // Input the plaintext req (no args)
    // Output sucess ?
    // Do not work for "select"
    fn _execute_no_param(&self, db: &DBase, req: &str) -> bool {
        //println!("{}",req);
        match db.conn.execute(req, rusqlite::params![]) {
            Ok(_) => true,
            Err(err) => {
                println!("{}", err);
                false
            }
        }
    }

    fn _exist(&self, db: &DBase, what: String, filter: String) -> Result<bool, rusqlite::Error> {
        let tabl = self._get();
        let req = format!("select {} from {} {}", what, tabl, filter);

        let mut stmt = db.conn.prepare(&req)?;
        let mut rows = stmt.query([])?;
        match rows.next()? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    fn _unique_name(&self, db: &DBase, name: &String) -> bool {
        match self._exist(
            db,
            String::from("`name`"),
            format!("where {} = '{}'", String::from("`name`"), name),
        ) {
            Ok(b) => !b,
            Err(err) => {
                if cfg!(debug_assertions) {
                    println!("{:#?}", err);
                }
                return false;
            }
        }
    }

    fn _retrieve_id(&self, db: &DBase, entity: &MATRow) -> Result<i32, rusqlite::Error> {

        let tabl = self._get();
        let req: String = match entity {
            MATRow::User {
                id: _,
                name,
                pass: _,
            } => format!(
                "select `id` from {} where `name` = '{}'",
                MATable::get(MATable::Users),
                name
            ),
            MATRow::Alias { id: _, name } => format!(
                "select `id` from {} where `name` = '{}'",
                MATable::get(MATable::Aliases),
                name
            ),
            MATRow::Domain {
                id: _,
                name,
                nb_ref: _,
            } => format!(
                "select `id` from {} where `name` = '{}'",
                MATable::get(MATable::Domains),
                name
            ),
            MATRow::Address {
                user: _,
                alias,
                domain,
            } => format!(
                "select `user` from {} where `alias` = '{}' and `domain` = {}",
                MATable::get(MATable::Address),
                alias,
                domain
            ),
        };

        println!("{}", req);
        let mut stmt = db.conn.prepare(&req)?;
        let mut rows = stmt.query([])?;
        match rows.next()? {
            Some(row) => {
                let id = row.get(0)?;
                Ok(id)
            }
            None => Err(rusqlite::Error::QueryReturnedNoRows),
        }
    }

    // find and retrieve id by name
    pub fn find(&self, db: &DBase, entity: &MATRow) -> i32 {
        if db.up == false {
            return -1;
        }

        let id = match self._retrieve_id(db,entity)
        {
            Err(e) => {
                if cfg!(debug_assertions) {
                    println!("{:#?}", e);
                } ; -1
            },
            Ok(id) => id,
        };
        id
    }

    // Create
    pub fn create(&self, db: &DBase) -> bool {
        if db.up == false {
            return false;
        }

        let tabl = self._get();
        let pattern = match self {
			MATable::Users => "`id` INTEGER PRIMARY KEY AUTOINCREMENT, `name` TEXT NOT NULL, `pass` TEXT NOT NULL",
    		MATable::Aliases => "`id` INTEGER PRIMARY KEY AUTOINCREMENT, `name` TEXT NOT NULL",
    		MATable::Domains => "`id` INTEGER PRIMARY KEY AUTOINCREMENT, `name` TEXT NOT NULL, `nb_ref` INTEGER NOT NULL",
			// ########################################################################################################################
			MATable::Address => "`user` INTEGER ,`alias` INTEGER , `domain` INTEGER , FOREIGN KEY(`user`) REFERENCES Users(`id`) , FOREIGN KEY(`alias`) REFERENCES Aliases(`id`) , FOREIGN KEY(`domain`) REFERENCES Domains(`id`) ,  PRIMARY KEY(`alias`,`domain`)",
			// ########################################################################################################################
		};
        let req = format!("create table {} ( {} )", tabl, pattern);
        self._execute_no_param(db, &req)
    }

    // Drop
    pub fn drop(&self, db: &DBase) -> bool {
        if db.up == false {
            return false;
        }

        let tabl = self._get();
        let req = format!("drop table {}", tabl);
        self._execute_no_param(db, &req)
    }

    // Insert
    pub fn insert(&self, db: &DBase, value: &mut MATRow) -> bool {
        if db.up == false {
            return false;
        }

        let tabl = self._get();
        let (pattern, values) = match value {
            MATRow::User { id: _, name, pass } => {
                if !self._unique_name(db, name) {
                    return false;
                }
                (
                    String::from("name, pass"),
                    format!("'{}', '{}'", name, pass),
                )
            }
            MATRow::Alias { id: _, name } => {
                if !self._unique_name(db, name) {
                    return false;
                }
                (String::from("name"), format!("'{}'", name))
            }
            MATRow::Domain {
                id: _,
                name,
                nb_ref,
            } => {
                if !self._unique_name(db, name) {
                    return false;
                }
                (
                    String::from("name, nb_ref"),
                    format!("'{}', {}", name, nb_ref),
                )
            }
            MATRow::Address {
                user,
                alias,
                domain,
            } => {
                // if !unique ?  ####################################################
                (
                    String::from("user, alias, domain"),
                    format!("{}, {}, {}", user, alias, domain),
                )
            }
        };
        let req = format!("insert into {} ( {} ) values ( {} )", tabl, pattern, values);

        if self._execute_no_param(db, &req) {
            match self._retrieve_id(db, value) {
                Err(_) => false,
                Ok(id_inside) => match value {
                    MATRow::Alias { id, name: _ } => {
                        *id = id_inside;
                        true
                    }
                    MATRow::User {
                        id,
                        name: _,
                        pass: _,
                    } => {
                        *id = id_inside;
                        true
                    }
                    MATRow::Domain {
                        id,
                        name: _,
                        nb_ref: _,
                    } => {
                        *id = id_inside;
                        true
                    }
                    MATRow::Address {
                        user: _,
                        alias: _,
                        domain: _,
                    } => true,
                },
            }
        } else {
            false
        }
    }

    pub fn delete_by_id(&self, db: &DBase, value: &MATRow) -> bool {
        if db.up == false {
            return false;
        }
        let filter: String = match value {
            MATRow::User {
                id,
                name: _,
                pass: _,
            } => format!("where `id` = {}", id),
            MATRow::Alias { id, name: _ } => format!("where `id` = {}", id),
            MATRow::Domain {
                id,
                name: _,
                nb_ref: _,
            } => format!("where `id` = {}", id),
            MATRow::Address {
                user: _,
                alias,
                domain,
            } => format!("where `alias` = {} and `domain` = {}", alias, domain),
        };

        self._delete(&db, filter)
    }

    pub fn delete_by_name(&self, db: &DBase, value: &MATRow) -> bool {
        if db.up == false {
            return false;
        }
        let filter: String = match value {
            MATRow::User {
                id: _,
                name,
                pass: _,
            } => format!("where `name` = '{}'", name.clone()),
            MATRow::Alias { id: _, name } => format!("where `name` = '{}'", name.clone()),
            MATRow::Domain {
                id: _,
                name,
                nb_ref: _,
            } => format!("where `name` = '{}'", name.clone()),
            MATRow::Address {
                user,
                alias: _,
                domain: _,
            } => format!("where `user` = '{}'", user),
        };

        self._delete(&db, filter)
    }

    pub fn update_by_id(&self, db: &DBase, value: &MATRow) -> bool {
        if db.up == false {
            return false;
        }
        // UPDATE {table} SET {column} = {column} + {value} WHERE {condition}
        true
    }

    pub fn update_by_name(&self, db: &DBase, value: &MATRow) -> bool { false }


    // Delete
    fn _delete(&self, db: &DBase, filter: String) -> bool {
        let req = format!("delete from {} {}", self._get(), filter);
        self._execute_no_param(db, &req)
    }

    // Update
    fn _update(&self, db: &DBase, operation: String, filter: String) -> bool {
        let req = format!("update {} set {} where {}", self._get(), operation, filter);
        self._execute_no_param(db, &req)
    }

    // Select
    pub fn select(
        &self,
        db: &DBase,
        what: String,
        filter: String,
    ) -> Result<(Vec<MATRow>, usize), rusqlite::Error> {
        if db.up == false {
            return Ok((Vec::<MATRow>::new(), 0));
        }

        let tabl = self._get();
        let req = format!("select {} from {} {}", what, tabl, filter);

        let mut stmt = db.conn.prepare(&req)?;
        let mut rows = stmt.query([])?;

        let mut ret = Vec::<crate::MATRow>::new();
        let mut nb = 0;

        loop {
            match rows.next()? {
                Some(row) => {
                    let entity = match self {
                        MATable::Users => MATRow::User {
                            id: row.get(0)?,
                            name: row.get(1)?,
                            pass: row.get(2)?,
                        },
                        MATable::Aliases => MATRow::Alias {
                            id: row.get(0)?,
                            name: row.get(1)?,
                        },
                        MATable::Domains => MATRow::Domain {
                            id: row.get(0)?,
                            name: row.get(1)?,
                            nb_ref: row.get(2)?,
                        },
                        MATable::Address => MATRow::Address {
                            user: row.get(0)?,
                            alias: row.get(1)?,
                            domain: row.get(2)?,
                        },
                    };
                    nb += 1;
                    ret.push(entity);
                }
                None => break,
            };
        }
        Ok((ret, nb))
    }
}
