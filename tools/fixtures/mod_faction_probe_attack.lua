-- SPDX-License-Identifier: BSD-2-Clause
-- Copyright (c) 2026 Faction contributors
-- Disposable #209 acceptance fixture only. Never enable on a real home.
-- Counters observe public addressing metadata; no payload, key or credential logs.
local st = require "prosody.util.stanza";
local directory = module:get_option_string("faction_probe_attack_directory");
if directory ~= "/home/mailb/faction209-attack" or module.host ~= "homeb.test" then
    error("owned disposable attack fixture required");
end
local function read(path, limit)
    local file = io.open(path, "rb");
    if not file then return nil; end
    local value = file:read(limit + 1);
    file:close();
    if not value or #value > limit then return nil; end
    return value;
end
local marker = read("/etc/faction-disposable-test", 1024);
if not marker or not marker:find("209", 1, true) then
    error("disposable native marker required");
end
local omemo = "urn:xmpp:omemo:2";
local pubsub = "http://jabber.org/protocol/pubsub";
local modes = { ["replaced-device"] = true; ["added-device"] = true; ["changed-identity"] = true };
local active;
local function bare(jid) return jid and jid:match("^[^/]+"); end
local function control()
    local raw = read(directory .. "/mode", 1024);
    if not raw then return nil; end
    local token, mode, target, sender, device, added = raw:match(
        "^faction209%-v1\n([0-9a-f]+)\n([a-z%-]+)\n([^\n]+)\n([^\n]+)\n([0-9]+)\n([0-9]+)\n$");
    if not token or #token ~= 32 or not modes[mode]
        or not target:match("^[a-z0-9._%-]+@homeb%.test$")
        or not sender:match("^[a-z0-9._%-]+@homea%.test$")
        or tonumber(device) < 1 or tonumber(device) > 2147483647
        or tonumber(added) < 1 or tonumber(added) > 2147483647 or device == added then
        return nil;
    end
    if not active or active.token ~= token then
        active = { token = token; mode = mode; target = target; sender = sender;
            device = device; added = added; devices = 0; bundles = 0; messages = 0; unexpected = 0 };
    end
    if active.mode ~= mode or active.target ~= target or active.sender ~= sender
        or active.device ~= device or active.added ~= added then return nil; end
    return active;
end
local function receipt(c)
    local file = assert(io.open(directory .. "/receipt", "wb"));
    file:write("faction209-v1\n", c.token, "\n", c.mode, "\n",
        tostring(c.devices), "\n", tostring(c.bundles), "\n",
        tostring(c.messages), "\n", tostring(c.unexpected), "\n");
    file:close();
end
local function increment(c, field)
    c[field] = math.min(9999, c[field] + 1);
end
local function matches(c, stanza)
    return c and bare(stanza.attr.to) == c.target and bare(stanza.attr.from) == c.sender;
end
local function result(stanza, node, id)
    return st.reply(stanza):tag("pubsub", { xmlns = pubsub })
        :tag("items", { node = node }):tag("item", { id = id });
end
module:hook("iq/bare/" .. pubsub .. ":pubsub", function(event)
    local stanza = event.stanza;
    local c = control();
    if stanza.attr.type ~= "get" or not matches(c, stanza) then return; end
    local query = stanza:get_child("pubsub", pubsub);
    local items = query and query:get_child("items");
    if not items then return; end
    if items.attr.node == omemo .. ":devices" then
        increment(c, "devices");
        receipt(c);
        if c.mode == "changed-identity" then return; end
        local reply = result(stanza, items.attr.node, "current")
            :tag("devices", { xmlns = omemo });
        if c.mode == "added-device" then reply:tag("device", { id = c.device }):up(); end
        reply:tag("device", { id = c.added });
        event.origin.send(reply:reset());
        return true;
    elseif items.attr.node == omemo .. ":bundles" then
        increment(c, "bundles");
        local item = items:get_child("item");
        if not item or item.attr.id ~= c.device then increment(c, "unexpected"); end
        receipt(c);
        if c.mode ~= "changed-identity" then return; end
        -- Public zero vectors deliberately substitute the verified identity.
        -- The native client must reject that identity before cryptographic use.
        local zero32 = string.rep("A", 43) .. "=";
        local zero64 = string.rep("A", 86) .. "==";
        local reply = result(stanza, items.attr.node, c.device)
            :tag("bundle", { xmlns = omemo })
            :tag("spk", { id = "1" }):text(zero32):up()
            :tag("spks"):text(zero64):up()
            :tag("ik"):text(zero32):up():tag("prekeys");
        for id = 1, 25 do reply:tag("pk", { id = tostring(id) }):text(zero32):up(); end
        event.origin.send(reply:reset());
        return true;
    end
end, 1000);
module:hook("message/bare", function(event)
    local stanza = event.stanza;
    local c = control();
    if not matches(c, stanza) then return; end
    increment(c, "messages");
    local encrypted = stanza:get_child("encrypted", omemo);
    local header = encrypted and encrypted:get_child("header");
    local key_count = 0;
    if header then
        for keys in header:childtags("keys") do
            if keys.attr.jid ~= c.target then increment(c, "unexpected"); end
            for key in keys:childtags("key") do
                key_count = key_count + 1;
                if key.attr.rid ~= c.device then increment(c, "unexpected"); end
            end
        end
    end
    if key_count ~= 1 then increment(c, "unexpected"); end
    receipt(c);
    -- Preserve ordinary routing; observe only. Payload text is never inspected.
end, 1000);
