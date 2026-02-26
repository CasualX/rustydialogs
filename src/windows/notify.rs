use std::{fs, process};
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
	let (accent_color, accent_soft, icon_symbol) = match p.icon {
		MessageIcon::Info => ("#2563EB", "#DBEAFE", "ℹ"),
		MessageIcon::Warning => ("#D97706", "#FEF3C7", "⚠"),
		MessageIcon::Error => ("#DC2626", "#FEE2E2", "✖"),
		MessageIcon::Question => ("#7C3AED", "#EDE9FE", "?"),
	};
	let close_script = if p.timeout > 0 {
		format!("idTimer = window.setTimeout(\"window.Close\", {}, \"VBScript\")", p.timeout)
	} else {
		String::new()
	};

	let hta = format!(
		r##"<html>
<head>
<meta charset="utf-8">
<title>{title}</title>
<HTA:APPLICATION
	SysMenu = "yes"
	ID = "rustydialogsHTA"
	APPLICATIONNAME = "rustydialogs_notifyPopup"
	MINIMIZEBUTTON = "no"
	MAXIMIZEBUTTON = "no"
	BORDER = "dialog"
	SCROLL = "no"
	SINGLEINSTANCE = "yes"
	WINDOWSTATE = "hidden">
<script language="VBScript">
Dim g_path
g_path = document.location.pathname

Sub Window_onLoad()
	Dim minW, maxW
	minW = 280
	maxW = 480

	' First pass: measure at generous width
	ResizeTo maxW, 400

	Dim cardEl
	Set cardEl = document.getElementById("notifyCard")

	' Pick width: clamp between min and max
	Dim contentW
	contentW = cardEl.scrollWidth + 20
	If contentW < minW Then contentW = minW
	If contentW > maxW Then contentW = maxW

	' Second pass: resize to chosen width so text reflows
	ResizeTo contentW, 400

	' Measure final height including all padding
	Dim intHeight
	intHeight = cardEl.scrollHeight + 20
	If intHeight < 72 Then intHeight = 72

	ResizeTo contentW, intHeight
	MoveTo Screen.Width - contentW - 16, Screen.Height - intHeight - 48
	{close_script}
End Sub

Sub Window_onUnload()
	On Error Resume Next
	Dim fso
	Set fso = CreateObject("Scripting.FileSystemObject")
	' Strip leading slash from /C:/...
	Dim p
	p = g_path
	If Left(p, 1) = "/" Then p = Mid(p, 2)
	p = Replace(p, "/", "\")
	fso.DeleteFile p, True
End Sub
</script>
</head>
<body style="margin:0; padding:0; background:transparent; font-family:'Segoe UI',Tahoma,sans-serif; overflow:hidden;">
<div id="notifyCard" style="background:#FFFFFF; border:1px solid #E5E7EB; border-left:6px solid {accent_color}; border-radius:12px; box-shadow:0 4px 16px #0002; padding:12px 14px 12px 14px;">
  <!-- icon + text row -->
  <table border="0" cellpadding="0" cellspacing="0" width="100%">
    <tr>
      <td valign="top" width="38">
        <div id="notifyBadge" style="width:28px; height:28px; line-height:28px; text-align:center; border-radius:999px; font-size:16px; font-weight:700; color:{accent_color}; background:{accent_soft};">{icon_symbol}</div>
      </td>
      <td valign="top">
        <p id="notifyTitle" style="margin:0 0 3px 0; font-size:14px; line-height:1.25; font-weight:600; color:#111827;">{title}</p>
        <p id="notifyMsg" style="margin:0; font-size:12px; line-height:1.4; color:#374151; word-wrap:break-word;">{message}</p>
      </td>
    </tr>
  </table>
  <!-- bottom spacer -->
  <div style="height:10px;"></div>
</div>
</body>
</html>
"##
	);

	if fs::write(&path, hta).is_err() {
		return;
	}

	let _ = process::Command::new("mshta.exe").arg(&path).spawn();
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
