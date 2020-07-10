use serde_derive::{Deserialize, Serialize};
use clipboard::{ClipboardContext, ClipboardProvider};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
	let mut view = web_view::builder()
		.title("Emote Picker")
		.user_data(())
		.invoke_handler(|_wv, arg| {
			handle_view_cmd(arg)
				.map_err(|e| web_view::Error::custom(e.to_string()))
		})
		.content(web_view::Content::Html(include_str!("main.html")))
		.size(400, 300)
		.build()?;

	view.send_message(ViewMessage::Init)?;

	let entries = get_emoji_info().unwrap();

	let handle = view.handle();
	std::thread::spawn(move || {
		let msg = ViewMessage::Update { entries: &entries };
		let json = serde_json::to_string(&msg).unwrap();
		let json = format!("on_message({})", json);

		handle.dispatch(move |vw| {
			vw.eval(&json)
				.map_err(|e| web_view::Error::custom(e.to_string()))
		}).unwrap();
	});


	while let Some(()) = view.step().transpose()? {

	}

	Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ViewCommand {
	Debug { text: String },
	CopyToClipboard { text: String },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ViewMessage<'e> {
	Init,
	Update {
		entries: &'e [EmoteEntry],
	}
}

#[derive(Debug, Deserialize, Serialize)]
struct EmoteEntry {
	text: String,
	name: String,
	group: String,
	tags: Vec<String>,
}


fn get_emoji_info() -> Result<Vec<EmoteEntry>, Box<dyn Error>> {
	let mut emote_list_path = dirs::config_dir()
		.expect("Couldn't get config dir");
	emote_list_path.push("emote-picker/emotes.json");

	println!("{:?}", emote_list_path);

	let emote_list = std::fs::read_to_string(emote_list_path)
		.unwrap_or("[]".into());

	serde_json::from_str(&emote_list)
		.map_err(Into::into)
}



trait WebViewExt {
	fn send_message(&mut self, msg: ViewMessage) -> Result<(), Box<dyn Error>>;
}

impl<T> WebViewExt for web_view::WebView<'_, T> {
	fn send_message(&mut self, msg: ViewMessage) -> Result<(), Box<dyn Error>> {
		let json = serde_json::to_string(&msg)?;
		self.eval(&format!("on_message({})", json))?;
		Ok(())
	}
}


fn handle_view_cmd(arg: &str) -> Result<(), Box<dyn Error>> {
	let msg: ViewCommand = serde_json::from_str(arg)?;

	match msg {
		ViewCommand::Debug{text} => {
			println!("[dbg] {}", text);
		}

		ViewCommand::CopyToClipboard{text} => {
			println!("copy {}", text);
			let mut clip = ClipboardContext::new()?;
			clip.set_contents(text)?;
		}
	}

	Ok(())
}
