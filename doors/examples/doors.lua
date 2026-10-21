-- Original Faction example. Edit this Lua menu to give your board its own arcade.
bbs.title("After Hours Arcade")
bbs.text("Two quick games. No downloads, no wallets, just one more turn.")
bbs.text("Choose a door with arrows + Enter, or press its hotkey.")
bbs.option("H", "Signal Hunt", "game:signal-hunt")
bbs.option("N", "Neon Salvage", "game:neon-salvage")
bbs.option("B", "Back to the BBS", "back")
