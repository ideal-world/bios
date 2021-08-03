local core = require("apisix.core")
local m_utils = require("apisix.plugins.auth-dew.utils")
local m_redis = require("apisix.plugins.auth-dew.redis")
local m_resource = require("apisix.plugins.auth-dew.resource")
local json = require("cjson")

local _M = {}

function _M.init(cache_resources, cache_change_resources, cache_change_resources_timer_sec)
    m_redis.hscan(cache_resources, "*", 100, function(k, v)
        local res = m_utils.split(k, "##")
        m_resource.add_res(res[2], res[1], json.decode(v))
    end)
    ngx.timer.every(cache_change_resources_timer_sec, function()
        core.log.debug("Fetch changed resources")
        m_redis.hscan(cache_change_resources, "*", 10, function(k, v)
            local res = m_utils.split(k, "##")
            if (v ~= "") then
                m_resource.add_res(res[2], res[1], json.decode(v))
            else
                m_resource.remove_res(res[2], res[1])
            end
        end)
    end)
end

return _M