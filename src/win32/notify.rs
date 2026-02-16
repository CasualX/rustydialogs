use std::fs;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

pub fn notify_popup(p: &NotifyPopup<'_>) {
	let mut path = std::env::temp_dir();
	path.push(format!(
		"rustydialogs-notify-{}-{}.hta",
		process::id(),
		SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_nanos())
	));

	let title = html_escape(p.title);
	let message = html_escape(p.message)
		.replace('\n', "<br>")
		.replace('\t', "&nbsp;&nbsp;&nbsp;&nbsp;");
	let icon_label = match p.icon {
		MessageIcon::Info => "Info",
		MessageIcon::Warning => "Warning",
		MessageIcon::Error => "Error",
		MessageIcon::Question => "Question",
	};
	let close_script = if p.timeout > 0 {
		format!(
			"Sub Window_onLoad\n\tidTimer = window.setTimeout(\"window.Close\", {}, \"VBScript\")\nEnd Sub",
			p.timeout
		)
	} else {
		String::new()
	};

	let hta = format!(
		r#"<html>
<head>
<meta charset="utf-8">
<title>{title}</title>
<HTA:APPLICATION
	SysMenu = "no"
	ID = "rustydialogsHTA"
	APPLICATIONNAME = "rustydialogs_notifyPopup"
	MINIMIZEBUTTON = "no"
	MAXIMIZEBUTTON = "no"
	BORDER = "dialog"
	SCROLL = "no"
	SINGLEINSTANCE = "yes"
	WINDOWSTATE = "hidden">
<script language="VBScript">
intWidth = Screen.Width/4
intHeight = Screen.Height/10
ResizeTo intWidth, intHeight
MoveTo Screen.Width * .7, Screen.Height * .8
{close_script}
</script>
</head>
<body style="background-color:#EEEEEE; font-family:Arial; margin:12px;">
<div><strong>{icon_label}</strong></div>
<div>{message}</div>
</body>
</html>
"#
	);

	if fs::write(&path, hta).is_err() {
		return;
	}

	let _ = process::Command::new("mshta.exe")
		.arg(&path)
		.spawn();
}

fn html_escape(value: &str) -> String {
	let mut result = String::with_capacity(value.len());
	for ch in value.chars() {
		match ch {
			'&' => result.push_str("&amp;"),
			'<' => result.push_str("&lt;"),
			'>' => result.push_str("&gt;"),
			'"' => result.push_str("&quot;"),
			'\'' => result.push_str("&#39;"),
			_ => result.push(ch),
		}
	}
	result
}
