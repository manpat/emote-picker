use std::fs::File;
use std::error::Error;
use std::net::TcpStream;
use std::io::{Write, Read};

use serde_derive::Serialize;


fn main() -> Result<(), Box<dyn Error>> {
	let time = std::time::Instant::now();

	let emoji_test_data = fetch("unicode.org", "/Public/emoji/13.0/emoji-test.txt")?;
	// let emoji_test_data = fetch("unicode.org", "/Public/emoji/13.0/emoji-sequences.txt"))?; // TODO
	// let emoji_test_data = fetch("unicode.org", "/Public/emoji/13.0/emoji-zwj-sequences.txt"))?; // TODO

	let info = parse_emoji_info(&emoji_test_data);
	let data = serde_json::to_string_pretty(&info)?;
	let mut f = File::create("emotes.json")?;
	write!(&mut f, "{}", data)?;

	println!("Done {}ms", time.elapsed().as_millis());

	Ok(())
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

	println!("Parsing...");

	let group_re = Regex::new(r"# group: ").unwrap();
	let subgroup_re = Regex::new(r"# subgroup: ").unwrap();
	let emoji_re = Regex::new(r"([\dA-F ]+)\s*; ([\w\-]+)\s*#.*E\d+.\d\s([\w ]+)").unwrap();

	let mut entries = Vec::new();

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
				let qualification = entry.get(2).unwrap().as_str();
				match qualification {
					"minimally-qualified" | "component" => { continue }
					_ => {}
				}

				let text: String = entry.get(1).unwrap().as_str()
					.split_whitespace()
					.map(|s| u32::from_str_radix(s, 16).ok().and_then(std::char::from_u32).unwrap())
					.collect();

				let name = entry.get(3).unwrap().as_str().to_owned();
				
				entries.push(EmoteEntry {
					text,
					name,
					group: group_name.to_owned(),
					tags: vec![subgroup_name.to_owned()],
				});
			}
		}
	}

	entries
}



fn fetch(host: &str, resource: &str) -> Result<String, Box<dyn Error>> {
	println!("Fetching {}...", resource);

	let mut stream = TcpStream::connect(format!("{}:80", host))?;
	write!(&mut stream, "GET {} HTTP/1.1\r\n", resource)?;
	write!(&mut stream, "Host: {}\r\n", host)?;
	write!(&mut stream, "\r\n")?;

	let mut response = String::new();
	stream.read_to_string(&mut response)?;

	if response.find("200 OK").is_none() {
		return Err(GenericError::new("Server didn't send the stuff"))
	}

	if let Some(start) = response.find("\r\n\r\n") {
		Ok(response[start..].to_owned())

	} else {
		Err(GenericError::new("Response missing body"))
	}
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