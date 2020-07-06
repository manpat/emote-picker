use std::fs::File;
use std::error::Error;
use std::net::TcpStream;
use std::io::{Write, Read};

use serde_derive::Serialize;

const UNICODE_VERSION: &str = "13.0";

fn main() -> Result<(), Box<dyn Error>> {
	let mut stream = TcpStream::connect("unicode.org:80")?;
	write!(&mut stream, "GET /Public/emoji/{}/emoji-test.txt HTTP/1.1\r\n", UNICODE_VERSION)?;
	write!(&mut stream, "Host: unicode.org\r\n")?;
	write!(&mut stream, "\r\n")?;

	let mut response = String::new();
	stream.read_to_string(&mut response)?;

	if response.find("200 OK").is_none() {
		return Err(GenericError::new("Server didn't send the stuff"))
	}

	if let Some(start) = response.find("\r\n\r\n") {
		let info = parse_emoji_info(&response[start..]);
		let data = serde_json::to_string_pretty(&info)?;
		let mut f = File::create("emotes.json")?;
		write!(&mut f, "{}", data)?;
		Ok(())

	} else {
		Err(GenericError::new("Malformed response"))
	}
}


#[derive(Debug, Serialize)]
struct EmoteEntry {
	text: String,
	name: String,
	group: String,
	tags: Vec<String>,
}


fn parse_emoji_info(raw: &str) -> Vec<EmoteEntry> {
	use regex::Regex;

	let group_re = Regex::new(r"# group: ").unwrap();
	let subgroup_re = Regex::new(r"# subgroup: ").unwrap();
	let emoji_re = Regex::new(r"([\dA-F ]+)\s*;.*E\d+.\d\s([\w ]+)").unwrap();

	let mut entries = Vec::new();

	let time = std::time::Instant::now();

	for group_chunk in group_re.split(raw).skip(1) {
		let name = group_chunk.lines()
			.filter(|l| !l.starts_with('#') && !l.trim().is_empty())
			.next();

		let group_name = if let Some(n) = name { n.trim() } else { continue };

		for subgroup_chunk in subgroup_re.split(group_chunk).skip(1) {
			let name = subgroup_chunk.lines()
				.filter(|l| !l.starts_with('#') && !l.trim().is_empty())
				.next();

			let subgroup_name = if let Some(n) = name { n.trim() } else { continue };

			for entry in emoji_re.captures_iter(subgroup_chunk) {
				let text: String = entry.get(1).unwrap().as_str()
					.split_whitespace()
					.map(|s| u32::from_str_radix(s, 16).ok().and_then(std::char::from_u32).unwrap())
					.collect();

				let name = entry.get(2).unwrap().as_str().to_owned();
				
				entries.push(EmoteEntry {
					text,
					name,
					group: group_name.to_owned(),
					tags: vec![subgroup_name.to_owned()],
				});
			}
		}
	}

	println!("get_emoji_info {}ms", time.elapsed().as_millis());

	entries
}



#[derive(Debug)]
pub struct GenericError(String);

impl GenericError {
	pub fn new<S>(s: S) -> Box<dyn Error>
		where S: Into<String>
	{
		Box::new(GenericError(s.into()))
	}
}

impl Error for GenericError {}
impl std::fmt::Display for GenericError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}