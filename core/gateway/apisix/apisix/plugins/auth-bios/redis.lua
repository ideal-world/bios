local core = require("apisix.core")
local redis_new = require("resty.redis").new

local redis_client = {}
local CACHES = {}

local function init(host, port, database, timeout, password)
    core.log.info("Init redis connection, host:", host, " port: ", port, " db: ", database)
    redis_client.client = redis_new()
    redis_client.client:set_timeouts(timeout, timeout, timeout)
    local _, conn_err = redis_client.client:connect(host, port)
    if conn_err then
        if conn_err == "already connected" then
            core.log.warn("Redis reconnection, host:", host, " port: ", port, " db: ", database)
            redis_client.client:close()
            init(host, port, database, timeout, password)
        else
            error("Redis connection failure:" .. conn_err)
        end
    end
    if password and password ~= '' then
        local _, err = redis_client.client:auth(password)
        if err then
            error("Redis connection failure:" .. err)
        end
    end
    if database ~= 0 then
        local _, err = redis_client.client:select(database)
        if err then
            error("Redis change db failure:" .. err)
        end
    end
    redis_client.host = host
    redis_client.port = port
    redis_client.database = database
    redis_client.timeout = timeout
    redis_client.password = password
    return true
end

local function close()
    if redis_client and redis_client.client then
        core.log.info("Close redis connection, host:", redis_client.host, " port: ", redis_client.port, " db: ", redis_client.database)
        local _, err = redis_client.client:close()
        if err then
            error("Redis operation failure [close]:" .. err)
        end
    end
end

local function reconnect(err)
    if err == "closed" then
        core.log.warn("Redis reconnect, host:", redis_client.host, " port: ", redis_client.port, " db: ", redis_client.database)
        return init(redis_client.host, redis_client.port, redis_client.database, redis_client.timeout, redis_client.password)
    end
    return false
end

local function set(key, value, cache_sec, retry)
    local _, err = redis_client.client:set(key, value)
    if err then
        if retry and reconnect(err) then
            set(key, value, cache_sec, false)
        else
            error("Redis operation failure [set]:" .. err)
        end
    end
    if cache_sec and cache_sec > 0 then
        local _, exp_err = redis_client.client:expire(key, cache_sec)
        if exp_err then
            if retry and reconnect(exp_err) then
                redis_client.client:expire(key, cache_sec)
            else
                error("Redis operation failure [expire]:" .. exp_err)
            end
        end
    end
end

-- TODO auto remove local caches
local function get(key, cache_sec, retry)
    if cache_sec and cache_sec > 0 and CACHES[key] and CACHES[key][1] > os.time() then
        return CACHES[key][2]
    else
        local value, err = redis_client.client:get(key)
        if err then
            if retry and reconnect(err) then
                return get(key, cache_sec, false)
            else
                error("Redis operation failure [get]:" .. err)
            end
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

local function del(key, retry)
    local _, err = redis_client.client:del(key)
    if err then
        if retry and reconnect(err) then
            del(key, false)
        else
            error("Redis operation failure [del]:" .. err)
        end
    end
end

local function hset(key, field, value, retry)
    local _, err = redis_client.client:hset(key, field, value)
    if err then
        if retry and reconnect(err) then
            hset(key, field, value, false)
        else
            error("Redis operation failure [hset]:" .. err)
        end
    end
end

local function hdel(key, field, retry)
    local _, err = redis_client.client:hdel(key, field)
    if err then
        if retry and reconnect(err) then
            hdel(key, field, false)
        else
            error("Redis operation failure [hdel]:" .. err)
        end
    end
end

local function hget(key, field, retry)
    local value, err = redis_client.client:hget(key, field)
    if err then
        if retry and reconnect(err) then
            return hget(key, field, false)
        else
            error("Redis operation failure [hget]:" .. err)
        end
    end
    if value == ngx.null then
        return nil
    end
    return value
end

local function lpush(key, value, retry)
    local _, err = redis_client.client:lpush(key, value)
    if err then
        if retry and reconnect(err) then
            lpush(key, value, false)
        else
            error("Redis operation failure [lpush]:" .. err)
        end
    end
end

local function lrangeall(key, retry)
    local value, err = redis_client.client:lrange(key, 0, -1)
    if err then
        if retry and reconnect(err) then
            return lrangeall(key, false)
        else
            error("Redis operation failure [lrangeall]:" .. err)
        end
    end
    return value
end

local function hscan(key, field, max_number, func, retry)
    local cursor = "0"
    repeat
        local value, err = redis_client.client:hscan(key, cursor, "count", max_number, "match", field)
        if err then
            if retry and reconnect(err) then
                return hscan(key, field, max_number, func, false)
            else
                error("Redis operation failure [hscan]:" .. err)
            end
        end
        local data
        cursor, data = unpack(value)
        if next(data) then
            local k
            for _, v in pairs(data) do
                if k == nil then
                    k = v
                else
                    func(k, v)
                    k = nil
                end
            end
        end
    until cursor == "0"
end

local function scan(key, max_number, func, retry)
    local cursor = "0"
    repeat
        local value, err = redis_client.client:scan(cursor, "match", key .. "*", "count", max_number)
        if err then
            if retry and reconnect(err) then
                return scan(key, max_number, func, false)
            else
                error("Redis operation failure [scan]:" .. err)
            end
        end
        local data
        cursor, data = unpack(value)
        if next(data) then
            for _, k in pairs(data) do
                func(k, redis_client.client:get(k))
            end
        end
    until cursor == "0"
end

return {
    init = init,
    close = close,
    set = function(key, value, cache_sec)
        set(key, value, cache_sec, true)
    end,
    get = function(key, cache_sec)
        return get(key, cache_sec, true)
    end,
    del = function(key)
        del(key, true)
    end,
    lpush = function(key, value)
        lpush(key, value, true)
    end,
    lrangeall = function(key)
        return lrangeall(key, true)
    end,
    hset = function(key, field, value)
        hset(key, field, value, true)
    end,
    hdel = function(key, field)
        hdel(key, field, true)
    end,
    hget = function(key, field)
        return hget(key, field, true)
    end,
    hscan = function(key, field, max_number, func)
        hscan(key, field, max_number, func, true)
    end,
    scan = function(key, max_number, func)
        scan(key, max_number, func, true)
    end,
}