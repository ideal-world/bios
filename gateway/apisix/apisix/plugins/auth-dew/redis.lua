local core = require("apisix.core")
local redis_new = require("resty.redis").new

local redis_client = redis_new()
local CACHES = {}

local function init(host, port, database, timeout, password)
    core.log.info("Init redis connection, host:", host, " port: ", port, " db: ", database)
    redis_client:set_timeouts(timeout, timeout, timeout)
    local _, conn_err = redis_client:connect(host, port)
    if conn_err then
        error("Redis connection failure:" .. conn_err)
    end
    if password and password ~= '' then
        local _, err = redis_client:auth(password)
        if err then
            error("Redis connection failure:" .. err)
        end
    end
    if database ~= 0 then
        local _, err = redis_client:select(database)
        if err then
            error("Redis change db failure:" .. err)
        end
    end
    return true
end

local function set(key, value, cache_sec)
    redis_client:set(key, value)
    if cache_sec and cache_sec > 0 then
        redis_client:expire(key, cache_sec)
    end
end

local function hset(key, field, value)
    redis_client:hset(key, field, value)
end

local function get(key, cache_sec)
    if cache_sec and cache_sec > 0 and CACHES[key] and CACHES[key][1] > os.time() then
        return CACHES[key][2]
    else
        local value, err = redis_client:get(key)
        if err then
            error("Redis operation failure [get]:" .. err)
        end
        if value == ngx.null then
            return nil
        end
        if cache_sec and cache_sec > 0 then
            CACHES[key] = { os.time() + cache_sec, value }
        end
        return value
    end
end

local function hscan(key, field, max_number, func)
    local cursor = "0"
    repeat
        local value, err = redis_client:hscan(key, cursor, "count", max_number, "match", field)
        if err then
            error("Redis operation failure [hscan]:" .. err)
        end
        local data
        cursor, data = unpack(value)
        if next(data) then
            local key = nil
            for _, v in pairs(data) do
                if key == nil then
                    key = v
                else
                    func(key, v)
                    key = nil
                end
            end
        end
    until cursor == "0"
end

return {
    init = init,
    set = set,
    get = get,
    hset = hset,
    hscan = hscan,
}