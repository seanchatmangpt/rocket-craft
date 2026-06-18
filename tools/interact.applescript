tell application "Google Chrome"
    activate
    open location "file:///Users/sac/rocket-craft/pwa-staff/manufactured/Brm-HTML5-Shipping.html"
    delay 8 -- Wait for engine load and initialization
end tell

tell application "System Events"
    tell process "Google Chrome"
        set frontmost to true
        repeat 3 times
            keystroke "w"
            delay 1
            keystroke "a"
            delay 1
            keystroke "s"
            delay 1
            keystroke "d"
            delay 1
            key code 49 -- Space
            delay 1
        end repeat
    end tell
end tell
