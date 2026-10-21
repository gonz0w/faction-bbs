-- Original Faction game: a five-sector salvage run, local points only.
local sector, energy, score = 1, 4, 0
if state ~= "" then
    local a, b, c = string.match(state, "^(%d+):(%d+):(%d+)$")
    sector, energy, score = tonumber(a), tonumber(b), tonumber(c)
end
bbs.title("Neon Salvage // five sectors before dawn")
if input == "S" or input == "H" then
    local loot = bbs.random(20)
    if input == "H" then
        energy = energy - 1
        if bbs.random(4) == 1 then
            energy = energy - 1
            bbs.text("ICE bites back. Your deck loses two energy.")
        else loot = loot + 20; bbs.text("Access granted. You crack a sealed data cache.") end
    else bbs.text("You pull salvage from a derelict relay.") end
    score = score + loot
    bbs.text(string.format("Recovered %d credits. Local game points only.", loot))
    sector = sector + 1
end
bbs.text(string.format("Sector %d / 5    Energy %d    Score %d", sector, energy, score))
if sector > 5 or energy < 1 or input == "L" then
    bbs.text(string.format("You dock before dawn with %d points. Run again?", score))
    bbs.option("P", "Another salvage run", "game:neon-salvage")
    bbs.option("L", "Arcade lobby", "lobby")
else
    bbs.text("Safe salvage pays a little. Hacking pays more, but drains energy.")
    bbs.state(string.format("%d:%d:%d", sector, energy, score))
    bbs.option("S", "Salvage the relay", "input:S")
    bbs.option("H", "Hack the data cache", "input:H")
    bbs.option("L", "Land and count your score", "input:L")
end
bbs.option("B", "Back to the BBS", "back")
