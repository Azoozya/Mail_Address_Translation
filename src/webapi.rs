use rocket::fs::NamedFile;
use rocket::response::status::NotFound;
use urlencoding::decode;
use rand::Rng;

struct User {
	name: String,
}

struct Domain {
	domain: String,
}

/*
fn base32(number: u32) -> String {
	let alphabet = "abcdefghijklmnopqrstuvwxyz234567";
	let mut tmp = 0;

	alphabet

}*/

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

impl User {

	fn name(&self) -> String {
		self.name.clone()
	}

	fn new(input: String) -> User{
		let name = extract_value(input);
		// If empty return "Debug" User
		if name.is_empty() { User{ name: String::from("Debug")} } else { User{name} }
	}

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

	fn cut_head(input: String) -> String{
		let mut iter = input.split(".");
		let cnt = iter.clone().count();
		
		let mut domain = String::from("");		
		
		if cnt < 2 { return String::from("camel.lama"); }
		
		// limit to three.two.one (www.camel.lama)
		else if cnt == 2 {
			if let Some(s) = iter.nth(cnt-2) {
				domain.push_str(s);
			}	
		}

		else if cnt > 2 {		
						
			if let Some(s) = iter.nth(cnt-3) {
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
		
		domain
	}

	fn cut_tail(input: String) -> String{
		let mut iter = input.split("/");
		 
		match iter.next() {
				Some(s) => if s.is_empty() { String::from("cama.camel.lama") } else { String::from(s) }
				None => String::from("cama.camel.lama"),
		}	
	}

	fn new(input: String) -> Domain{
		let mut domain = extract_value(input);
		
		domain = Domain::decode_value(domain);
		domain = Domain::cut_tail(domain);			
		domain = Domain::cut_head(domain);

		Domain{domain}
	}
}


//TLS

// Login , Cookies / GET  / POST
#[get("/")]
pub async fn index() -> Result<NamedFile, NotFound<String>> {
    NamedFile::open("static/forms/index.html").await.map_err(|e| NotFound(e.to_string()))
}




// New user : new_user GET/POST
#[get("/submit_user")]
pub async fn get_submit_user() -> Result<NamedFile, NotFound<String>> {
	NamedFile::open("static/forms/submit_user.html").await.map_err(|e| NotFound(e.to_string()))
}

#[post("/submit_user",data = "<username>")]
pub async fn post_submit_user(username: String) -> String {
	let user = User::new(username);
	user.name()
}





//[*.]<maybe.>domain.root[/*]
// New domain : new_domain POST
#[get("/submit_domain")]
pub async fn get_submit_domain() -> Result<NamedFile, NotFound<String>> {
	NamedFile::open("static/forms/submit_domain.html").await.map_err(|e| NotFound(e.to_string()))
}

#[post("/submit_domain",data = "<domain>")]
pub async fn post_submit_domain(domain: String) -> String {
	let domain = Domain::new(domain);
	domain.domain()
}





// Generate alias : gen_alias GET
// Request for an alias (submit client/domain) new_alias POST
#[get("/new_alias")]
pub async fn get_new_alias() -> Result<NamedFile, NotFound<String>> {
	NamedFile::open("static/forms/new_alias.html").await.map_err(|e| NotFound(e.to_string()))
}

#[post("/new_alias",data = "<username>")]
pub async fn post_submit_alias(username: String) -> String {
	String::from("")
}



// List address (submit client , hide pass) get_address POST
pub fn handle_list_address() -> () {

}



#[post("/", data = "<args>")]
pub fn index_post(args: String) -> String {
    args
}

