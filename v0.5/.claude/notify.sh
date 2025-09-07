#!/bin/bash

# Check if Music app is playing and pause if needed
MUSIC_WAS_PLAYING=$(osascript -e 'tell application "System Events"
    if exists (processes whose name is "Music") then
        tell application "Music"
            if player state is playing then
                pause
                return "yes"
            end if
        end tell
    end if
    return "no"
end tell' 2>/dev/null)

# Save current volume (don't change it)
CURRENT_VOLUME=$(osascript -e 'output volume of (get volume settings)')

# Play notification at current volume
say "Claude is done and waiting for your next task!"
afplay /System/Library/Sounds/Glass.aiff

# Resume music if it was playing
if [ "$MUSIC_WAS_PLAYING" = "yes" ]; then
    osascript -e 'tell application "Music" to play'
fi