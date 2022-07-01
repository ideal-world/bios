local core = require("apisix.core")

local function start_with(str, prefix)
    return string.sub(str, 1, string.len(prefix)) == prefix
end

local function end_with(str, suffix)
    return suffix == '' or string.sub(str, -string.len(suffix)) == suffix
end

local function split(str, sep)
    local splits = {}
    if sep == nil then
        table.insert(splits, str)
    elseif sep == "" then
        local len = #str
        for i = 1, len do
            table.insert(splits, str:sub(i, i))
        end
    else
        local pattern = "[^" .. sep .. "]+"
        for s in string.gmatch(str, pattern) do
            table.insert(splits, s)
        end
    end
    return splits
end

local function contain(str, char)
    return string.find(str, char, 1, true) ~= nil
end

local function table_length(t)
    local count = 0
    for _ in pairs(t) do
        count = count + 1
    end
    return count
end

local function sort_query(query)
    if not query then
        return ""
    end
    local ordered_keys = {}
    for k in pairs(query) do
        table.insert(ordered_keys, k)
    end
    table.sort(ordered_keys, function(a, b)
        return string.lower(a) < string.lower(b)
    end)
    local sorted_query = ""
    local len = table_length(ordered_keys)
    for i = 1, len do
        if i ~= len then
            sorted_query = sorted_query .. ordered_keys[i] .. "=" .. query[ordered_keys[i]] .. "&"
        else
            sorted_query = sorted_query .. ordered_keys[i] .. "=" .. query[ordered_keys[i]]
        end
    end
    return sorted_query
end

return {
    start_with = start_with,
    end_with = end_with,
    split = split,
    contain = contain,
    table_length = table_length,
    sort_query = sort_query,
}

