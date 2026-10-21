-- Original Faction game: find a pirate radio frequency in seven guesses.
local secret, tries
if state == "" then
    secret = bbs.random(99)
    tries = 7
else
    local a, b = string.match(state, "^(%d+):(%d+)$")
    secret, tries = tonumber(a), tonumber(b)
end
bbs.title("Signal Hunt // pirate radio after midnight")
if input == "Q" or input == "q" then
    bbs.text("You pack up the antenna. The signal is still out there.")
elseif input ~= "" then
    local guess = tonumber(input)
    if guess == nil or guess < 1 or guess > 99 or guess % 1 ~= 0 then
        bbs.text("Tune a whole frequency from 1 to 99.")
    elseif guess == secret then
        bbs.text("LOCKED ON! The underground broadcast is yours.")
        tries = 0
    else
        tries = tries - 1
        if guess < secret then bbs.text("Too low. Tune higher.")
        else bbs.text("Too high. Tune lower.") end
    end
end
if tries > 0 and input ~= "Q" and input ~= "q" then
    bbs.text(string.format("Guesses left: %d. Frequencies: 1 through 99.", tries))
    bbs.state(string.format("%d:%d", secret, tries))
    bbs.prompt("Frequency (1-99), or Q to leave: ")
else
    if tries == 0 then bbs.text(string.format("Tonight's frequency was %d.", secret)) end
    bbs.option("P", "Hunt another signal", "game:signal-hunt")
    bbs.option("L", "Arcade lobby", "lobby")
    bbs.option("B", "Back to the BBS", "back")
end
