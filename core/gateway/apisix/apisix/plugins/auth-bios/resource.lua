local core = require("apisix.core")
local m_utils = require("apisix.plugins.auth-bios.utils")
local url = require("net.url")

local RESOURCES = {}

local _M = {}

function _M.add_res(res_action, res_uri, auth_info)
    res_action = string.lower(res_action)
    core.log.info("Add resource [" .. res_action .. "][" .. res_uri .. "]")
    local items = parse_uri(res_uri)
    local resources = RESOURCES
    for _, item in pairs(items) do
        if item == "$" then
            if resources["$"] == nil then
                resources["$"] = {}
            end
            resources["$"][res_action] = {
                action = res_action,
                uri = res_uri,
                auth = auth_info
            }
        else
            if resources[item] == nil then
                resources[item] = {}
            end
            resources = resources[item]
        end
    end
end

function _M.remove_res(res_action, res_uri)
    res_action = string.lower(res_action)
    core.log.info("Remove resource [" .. res_action .. "][" .. res_uri .. "]")
    local items = parse_uri(res_uri)
    local resources = RESOURCES
    for _, item in pairs(items) do
        if resources[item] ~= nil then
            resources = resources[item]
        else
            return
        end
        if resources ~= nil then
            resources[res_action] = nil
        end
    end
    do_remove_res(RESOURCES, items)
end

function do_remove_res(res, items)
    if m_utils.table_length(res) == 0 or m_utils.table_length(items) == 0 then
        return
    end
    local curr_item = table.remove(items, 1)
    do_remove_res(res[curr_item], items)
    if m_utils.table_length(res[curr_item]) == 0 then
        res[curr_item] = nil
    end
end

function _M.match_res(res_action, req_uri)
    res_action = string.lower(res_action)
    local items = parse_uri(req_uri)
    local matched_uris = {}
    table.remove(items)
    do_match_res(res_action, RESOURCES, items, matched_uris, false)
    return matched_uris
end

function do_match_res(res_action, res, items, matched_uris, multi_wildcard)
    if res["$"] ~= nil and (m_utils.table_length(items) == 0 or multi_wildcard) then
        -- matched
        local match_info = res["$"][res_action]
        if match_info.auth["_start"] <= ngx.time() and match_info.auth["_end"] >= ngx.time() then
            table.insert(matched_uris, match_info)
        end
        return
    end
    if (m_utils.table_length(items) == 0) then
        -- un-matched
        return
    end
    local curr_items = { table.unpack(items, 2) }
    if res[items[1]] ~= nil then
        do_match_res(res_action, res[items[1]], curr_items, matched_uris, false)
    end
    if res["*"] ~= nil then
        do_match_res(res_action, res["*"], curr_items, matched_uris, false)
    end
    if res["**"] ~= nil then
        do_match_res(res_action, res["**"], curr_items, matched_uris, true)
    end
    if multi_wildcard then
        do_match_res(res_action, res, curr_items, matched_uris, true)
    end
end

function _M.get_res()
    return RESOURCES
end

function parse_uri(res_uri)
    local res = url.parse(res_uri)
    local items = {}
    table.insert(items, string.lower(res.scheme))
    local host = m_utils.split(res.host, ".")
    table.insert(items, string.lower(host[1]))
    table.insert(items, string.lower(host[2]))
    local paths = m_utils.split(res.path, "/")
    for _, p in pairs(paths) do
        table.insert(items, string.lower(p))
    end
    local query = res.query
    if query ~= nil and m_utils.table_length(query) ~= 0 then
        table.insert(items, "?")
        table.insert(items, string.lower(m_utils.sort_query(query)))
    end
    table.insert(items, "$")
    return items
end

return _M