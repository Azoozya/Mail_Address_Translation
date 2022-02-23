use crate::base::DBase;
use crate::row::MATRow;

pub enum MATable {
    Users,
    Aliases,
    Domains,
    Address,
}


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

    pub fn get(row: &MATable) -> &'static str{
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
                println!("{}", err);
                return false;
            }
        }
    }

    fn _retrieve_id(&self, db: &DBase, entity: &MATRow) -> Result<i32, rusqlite::Error> {
        let tabl = self._get();
        let entity_name: &String = match entity {
            MATRow::User {
                id: _,
                name,
                pass: _,
            } => name,
            MATRow::Alias { id: _, name } => name,
            MATRow::Domain {
                id: _,
                name,
                nb_ref: _,
            } => name,
            MATRow::Address {
                user: _,
                alias: _,
                domain: _,
            } => {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }
        };

        let req = format!("select `id` from {} where `name` = '{}'", tabl, entity_name);
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
                Err(_) => {
                    if let MATRow::Address {
                        user: _,
                        alias: _,
                        domain: _,
                    } = value
                    {
                        return true;
                    } else {
                        return false;
                    }
                }
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
        let filter: String = match value {
            MATRow::User {
                id,
                name: _,
                pass: _,
            } => format!("where id = {}", id),
            MATRow::Alias { id, name: _ } => format!("where id = {}", id),
            MATRow::Domain {
                id,
                name: _,
                nb_ref: _,
            } => format!("where id = {}", id),
            MATRow::Address {
                user: _,
                alias,
                domain,
            } => format!("where alias = {} and domain = {}", alias, domain),
        };

        self._delete(&db, filter)
    }

    pub fn delete_by_name(&self, db: &DBase, value: &MATRow) -> bool {
        let filter: String = match value {
            MATRow::User {
                id: _,
                name,
                pass: _,
            } => format!("where name = '{}'", name.clone()),
            MATRow::Alias { id: _, name } => format!("where name = '{}'", name.clone()),
            MATRow::Domain {
                id: _,
                name,
                nb_ref: _,
            } => format!("where name = '{}'", name.clone()),
            MATRow::Address {
                user,
                alias: _,
                domain: _,
            } => format!("where user = '{}'", user),
        };

        self._delete(&db, filter)
    }

    // Delete
    fn _delete(&self, db: &DBase, filter: String) -> bool {
        let req = format!("delete from {} {}", self._get(), filter);
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
