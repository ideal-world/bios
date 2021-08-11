local core = require("apisix.core")
local m_redis = require("apisix.plugins.auth-dew.redis")
local m_init = require("apisix.plugins.auth-dew.init")
local m_ident = require("apisix.plugins.auth-dew.ident")
local m_auth = require("apisix.plugins.auth-dew.auth")
local json = require("cjson")
local ngx_encode_base64 = ngx.encode_base64

local plugin_name = "auth-dew"

local schema = {
    type = "object",
    properties = {
        redis_host = { type = "string" },
        redis_port = { type = "integer", default = 6379 },
        redis_password = { type = "string" },
        redis_timeout = { type = "integer", default = 1000 },
        redis_database = { type = "integer", default = 0 },

        token_flag = { type = "string", default = "Dew-Token" },
        auth_flag = { type = "string", default = "Authorization" },
        date_flag = { type = "string", default = "Dew-Date" },
        host_flag = { type = "string", default = "Dew-Host" },
        request_date_offset_ms = { type = "integer", default = 5000 },

        ident_flag = { type = "string", default = "Dew-Ident" },

        cache_resources = { type = "string", default = "dew:iam:resources" },
        cache_change_resources = { type = "string", default = "dew:iam:change_resources" },
        cache_change_resources_timer_sec = { type = "integer", default = 30 },
        cache_token = { type = "string", default = "dew:iam:token:info:" },
        cache_token_exp_sec = { type = "integer", default = 60 },
        cache_aksk = { type = "string", default = "dew:iam:app:aksk:" },
        cache_aksk_exp_sec = { type = "integer", default = 60 },
    },
    required = { "redis_host" }
}

local _M = {
    version = 0.1,
    priority = 5001,
    type = 'auth',
    name = plugin_name,
    schema = schema,
}

function _M.check_schema(conf)
    local check_ok, check_err = core.schema.check(schema, conf)
    if not check_ok then
        core.log.error("Configuration parameter error")
        return false, check_err
    end
    local _, redis_err = m_redis.init(conf.redis_host, conf.redis_port, conf.redis_database, conf.redis_timeout, conf.redis_password)
    if redis_err then
        core.log.error("Connect redis error", redis_err)
        return false, redis_err
    end
    m_init.init(conf.cache_resources, conf.cache_change_resources, conf.cache_change_resources_timer_sec)
    return true
end

function _M.rewrite(conf, ctx)
    local ident_code, ident_message = m_ident.ident(conf, ctx)
    if ident_code ~= 200 then
        return ident_code, ident_message
    end
    local auth_code, auth_message = m_auth.auth(ctx.ident_info)
    if auth_code ~= 200 then
        return auth_code, auth_message
    end
    core.request.set_header(ctx, conf.ident_flag, ngx_encode_base64(json.decode({
        appCode = ctx.ident_info.app_code,
        tenantCode = ctx.ident_info.tenant_code,
        accountCode = ctx.ident_info.account_code,
        token = ctx.ident_info.token,
        tokenKind = ctx.ident_info.token_kind,
        ak = ctx.ident_info.ak,
        roles = ctx.ident_info.roles,
        groups = ctx.ident_info.groups,
    })))
end

return _M