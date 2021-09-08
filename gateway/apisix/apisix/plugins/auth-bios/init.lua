local core = require("apisix.core")
local m_utils = require("apisix.plugins.auth-bios.utils")
local m_redis = require("apisix.plugins.auth-bios.redis")
local m_resource = require("apisix.plugins.auth-bios.resource")
local json = require("cjson")

local _M = {}

function _M.init(cache_resources, cache_change_resources, cache_change_resources_timer_sec)
    m_redis.hscan(cache_resources, "*", 100, function(k, v)
        local res = m_utils.split(k, "##")
        m_resource.add_res(res[2], res[1], json.decode(v))
    end)
    ngx.timer.every(cache_change_resources_timer_sec, function()
        core.log.debug("Fetch changed resources")
        m_redis.scan(cache_change_resources, 10, function(_, v)
            local res_value, err = m_redis.hget(cache_resources, v)
            if err then
                error("Fetch changed resource [" .. v .. "] failure" .. err)
            end
            local res_keys = m_utils.split(v, "##")
            if res_value == nil then
                m_resource.remove_res(res_keys[2], res_keys[1])
            else
                m_resource.add_res(res_keys[2], res_keys[1], json.decode(res_value))
            end
        end)
    end)
end

return _M