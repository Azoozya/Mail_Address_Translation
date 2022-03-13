use rocket::http::Status; // https://api.rocket.rs/v0.4/rocket/http/struct.Status.html#structfield.reason


#[derive(Debug)]
pub enum MATError {
    Empty,
    NotAnURI,
    URLEnconding,
    DBError,
    DBNotFound,
    DBAlreadyIn,
}

impl MATError {
    pub fn to_status(&self) -> Status {
        match self {
            MATError::Empty => Status::BadRequest,
            MATError::NotAnURI => Status::BadRequest,
            MATError::URLEnconding => Status::BadRequest,
            MATError::DBError => Status::InternalServerError,
            MATError::DBNotFound => Status::NoContent,
            MATError::DBAlreadyIn => Status::Conflict,
        }
    }
}
