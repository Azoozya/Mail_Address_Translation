#[derive(Debug)]
pub enum MATRow {
    User { id: i32, name: String, pass: String },
    Alias { id: i32, name: String },
    Domain { id: i32, name: String, nb_ref: i32 },
    Address { user: i32, alias: i32, domain: i32 },
}

impl MATRow {
    pub fn id(&self) -> i32 {
        match self {
            MATRow::User {
                id,
                name: _,
                pass: _,
            } => *id,
            MATRow::Alias { id, name: _ } => *id,
            MATRow::Domain {
                id,
                name: _,
                nb_ref: _,
            } => *id,
            MATRow::Address {
                user: _,
                alias: _,
                domain: _,
            } => -1,
        }
    }

    pub fn name(&self) -> String {
        match self {
            MATRow::User {
                id: _,
                name,
                pass: _,
            } => format!("{}", name.clone()),
            MATRow::Alias { id: _, name } => format!("{}", name.clone()),
            MATRow::Domain {
                id: _,
                name,
                nb_ref: _,
            } => format!("{}", name.clone()),
            MATRow::Address {
                user: _,
                alias: _,
                domain: _,
            } => String::from(""),
        }
    }

    pub fn nb_ref(&self) -> i32 {
        match self {
            MATRow::User {
                id:_,
                name: _,
                pass: _,
            } => -1,
            MATRow::Alias { id: _, name: _ } => -1,
            MATRow::Domain {
                id: _,
                name: _,
                nb_ref,
            } => *nb_ref,
            MATRow::Address {
                user: _,
                alias: _,
                domain: _,
            } => -1,
        }
    }

    #[allow(dead_code)]
    pub fn pass(&self) -> String {
        match self {
            MATRow::User {
                id: _,
                name: _,
                pass,
            } => format!("{}", pass.clone()),
            MATRow::Alias { id: _, name: _ } => String::from(""),
            MATRow::Domain {
                id: _,
                name: _,
                nb_ref: _,
            } => String::from(""),
            MATRow::Address {
                user: _,
                alias: _,
                domain: _,
            } => String::from(""),
        }
    }

}
