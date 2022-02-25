use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::response::status::NotFound;
use rocket::form::Form;
use urlencoding::decode;
use rand::{ Rng , thread_rng };
use std::thread;


#[derive(Debug)]
pub enum WebError {
	Empty,
	NotAnURI,
	URLEnconding,
}

impl WebError {

	pub fn to_status(&self) -> Status {
		match self {
			WebError::Empty => Status::BadRequest,
			WebError::NotAnURI => Status::BadRequest,
			WebError::URLEnconding => Status::BadRequest,
		}
	}
}

fn base32(number: usize) -> String {
	let alphabet = "abcdefghijklmnopqrstuvwxyz234567";
	let mut ret = String::from("");
	// Detect if usize correspond to u32 or u64	
	let archi_64 = cfg!(target_pointer_width = "64");
	let mut number = number ;	
	let mut tmp: usize;


	// base32 => 5 bits , 64 =  12*5 + 4 / 32 = 6 * 5 + 2
	for _ in 0..(if archi_64 {12} else {6}) {
		// get 5 bits from left
		tmp = number%32;
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

fn extract_value(input: String) -> String{
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

fn generate_random_b32_string() -> (String,String){
	let mut rng = thread_rng();

	let left: usize = rng.gen();
	let right: usize = rng.gen();

	let left = thread::spawn(move || { base32(left) });
	let right = thread::spawn(move || { base32(right) });

	let left = left.join().unwrap();
	let right = right.join().unwrap();

	(left,right)
}

//TLS

// Login , Cookies / GET  / POST
#[get("/")]
pub async fn index() -> Result<NamedFile, NotFound<String>> {
    NamedFile::open("static/forms/index.html").await.map_err(|e| NotFound(e.to_string()))
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

	fn clean(&mut self) -> Result<(),WebError>{
		let name = self.name();
		/*
			Work with it. url decode ? Squeeze special char ?
		*/
		// If empty return "Debug" User
		if name.is_empty() { Err(WebError::Empty) } else { self.name = name; Ok(()) }
	}

}


// New user : new_user GET/POST
#[get("/submit_user")]
pub async fn get_submit_user() -> Result<NamedFile, NotFound<String>> {
	NamedFile::open("static/forms/submit_user.html").await.map_err(|e| NotFound(e.to_string()))
}

#[post("/submit_user",data = "<username>")]
pub async fn post_submit_user(mut username: Form<User>) -> Result<String,Status> {
	match username.clean() {
		Err(e) => Err(e.to_status()),
		Ok(_) => Ok(username.name()),
	}
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

	fn decode_value(encoded: String) -> String{
		match decode(&encoded) {
			Err(_) => String::from(""),
			Ok(s) => s.to_string(),
		}
	}

	fn cut_head(input: String) -> Result<String,WebError>{
		let mut iter = input.split(".");
		let cnt = iter.clone().count();
		
		let mut domain = String::from("");		
		if cnt < 2 { return Err(WebError::NotAnURI); }
		
		// limit to three.two.one (www.camel.lama)
		else if cnt == 2 {
			if let Some(s) = iter.nth(cnt-2) {
				if s.is_empty() { return Err(WebError::NotAnURI); }
				domain.push_str(s);
			}	
		}

		else if cnt > 2 {		
						
			if let Some(s) = iter.nth(cnt-3) {
				if s.is_empty() { return Err(WebError::NotAnURI); }
				domain.push_str(s);
			}		
		}

		// Will be one or two iteration max
		loop {
			match iter.next() {
				Some(s) => { domain.push('.'); domain.push_str(s) },
				None => break,
			};		
		}
		
		Ok(domain)
	}

	fn cut_tail(input: String) -> Result<String,WebError>{
		let mut iter = input.split("/");
		 
		match iter.next() {
				Some(s) => if s.is_empty() { Err(WebError::NotAnURI) } else { Ok(String::from(s)) }
				None => Err(WebError::NotAnURI),
		}	
	}

	fn clean(&mut self) -> Result<(),WebError>{
		let mut domain = self.domain();
		
		domain = Domain::decode_value(domain);
		if domain.is_empty() { return Err(WebError::URLEnconding); }
		domain = Domain::cut_tail(domain)?;			
		domain = Domain::cut_head(domain)?;

		self.domain = domain;
		Ok(())
	}
}


//[*.]<maybe.>domain.root[/*]
// New domain : new_domain POST
#[get("/submit_domain")]
pub async fn get_submit_domain() -> Result<NamedFile, NotFound<String>> {
	NamedFile::open("static/forms/submit_domain.html").await.map_err(|e| NotFound(e.to_string()))
}

#[post("/submit_domain",data = "<domain>")]
pub async fn post_submit_domain(mut domain: Form<Domain>) -> Result<String,Status> {
	match domain.clean() {
		Err(e) => Err(e.to_status()),
		Ok(_) => Ok(domain.domain()),
	}	

}


// #############################    ALIAS    #############################


#[derive(FromForm)]
pub struct Alias {
	user: User,
	domain: Domain,
}

impl Alias {

	pub fn clean(&mut self)-> Result<(),WebError>{
		self.domain.clean()?;
		self.user.clean()?;
		Ok(())
	}

}

// Generate alias : gen_alias GET
// Request for an alias (submit client/domain) new_alias POST
#[get("/new_alias")]
pub async fn get_new_alias() -> Result<NamedFile, NotFound<String>> {
	NamedFile::open("static/forms/new_alias.html").await.map_err(|e| NotFound(e.to_string()))
}

#[post("/new_alias",data = "<args_alias>")]
pub async fn post_new_alias(mut args_alias: Form<Alias>) -> Result<String,Status> {
	if let Err(e) = args_alias.clean() {
		return Err(e.to_status());
	}	

	let (left,right) = generate_random_b32_string();
	
	Ok(format!("{}.{}",left,right))
}



// List address (submit client , hide pass) get_address POST
pub fn handle_list_address() -> () {

}



#[post("/", data = "<args>")]
pub fn index_post(args: String) -> String {
    args
}

