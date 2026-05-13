on maybeAllowScreenPrompt()
	log "screen-recording: waiting for Allow prompt"
	repeat 20 times
		tell application "System Events"
			repeat with proc in every process
				try
					if exists (button "Allow" of window 1 of proc) then
						click button "Allow" of window 1 of proc
						log "screen-recording: clicked Allow"
						return "clicked Allow"
					end if
				end try
			end repeat
		end tell
		delay 1
	end repeat
	log "screen-recording: no Allow prompt found"
	return "no Allow prompt found"
end maybeAllowScreenPrompt

return maybeAllowScreenPrompt()
