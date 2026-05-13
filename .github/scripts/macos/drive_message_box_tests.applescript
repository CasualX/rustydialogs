on waitForDialogToClose(proc, dialogWindow, stepLabel)
	repeat 40 times
		tell application "System Events"
			try
				if not (exists dialogWindow) then
					log "message-box-tests: dialog closed for " & stepLabel
					return true
				end if
			end try
		end tell
		delay 0.1
	end repeat
	error "Dialog did not close for " & stepLabel
end waitForDialogToClose

on withTestDialog(stepLabel, actionName, buttonName)
	log "message-box-tests: waiting for dialog for " & stepLabel
	repeat 240 times
		tell application "System Events"
			repeat with proc in every process
				try
					repeat with dialogWindow in windows of proc
						set dialogTitle to name of dialogWindow
						if dialogTitle starts with "[tests] MessageBox - " then
							log "message-box-tests: found dialog in process " & (name of proc) & " with title " & dialogTitle
							set frontmost of proc to true
							if actionName is "click" then
								click button buttonName of dialogWindow
							else if actionName is "dismiss" then
								keystroke "." using command down
							else
								error "Unknown action: " & actionName
							end if
							waitForDialogToClose(proc, dialogWindow, stepLabel)
							return true
						end if
					end repeat
				end try
			end repeat
		end tell
		delay 0.25
	end repeat
	error "Timed out waiting for message box dialog for " & stepLabel
end withTestDialog

on clickButton(buttonName, stepLabel)
	log "message-box-tests: " & stepLabel & " -> clicking " & buttonName
	withTestDialog(stepLabel, "click", buttonName)
	delay 0.4
end clickButton

on dismissDialog(stepLabel)
	log "message-box-tests: " & stepLabel & " -> dismissing dialog"
	withTestDialog(stepLabel, "dismiss", "")
	delay 0.4
end dismissDialog

set actions to {"OK", "DISMISS", "OK", "Cancel", "DISMISS", "Yes", "No", "DISMISS", "Yes", "No", "Cancel", "DISMISS", "OK", "DISMISS", "OK", "Cancel", "DISMISS", "Yes", "No", "DISMISS", "Yes", "No", "Cancel", "DISMISS", "OK", "DISMISS", "OK", "Cancel", "DISMISS", "Yes", "No", "DISMISS", "Yes", "No", "Cancel", "DISMISS", "OK", "DISMISS", "OK", "Cancel", "DISMISS", "Yes", "No", "DISMISS", "Yes", "No", "Cancel", "DISMISS"}

	log "message-box-tests: starting " & (count actions) & " scripted actions"

repeat with actionIndex from 1 to count actions
	set actionName to item actionIndex of actions
	set stepLabel to "step " & actionIndex & "/" & (count actions)
	if actionName is "DISMISS" then
		dismissDialog(stepLabel)
	else
		clickButton(actionName as text, stepLabel)
	end if
end repeat

log "message-box-tests: completed scripted actions"
