local core = require("apisix.core")
local m_redis = require("apisix.plugins.auth-bios.redis")
local m_init = require("apisix.plugins.auth-bios.init")
local m_ident = require("apisix.plugins.auth-bios.ident")
local m_auth = require("apisix.plugins.auth-bios.auth")
local json = require("cjson")
local ngx_encode_base64 = ngx.encode_base64

local plugin_name = "auth-bios"

local schema = {
    type = "object",
    properties = {
        redis_host = { type = "string" },
        redis_port = { type = "integer", default = 6379 },
        redis_password = { type = "string" },
        redis_timeout = { type = "integer", default = 1000 },
        redis_database = { type = "integer", default = 0 },

        token_flag = { type = "string", default = "BIOS-Token" },
        auth_flag = { type = "string", default = "Authorization" },
        date_flag = { type = "string", default = "BIOS-Date" },
        host_flag = { type = "string", default = "BIOS-Host" },
        request_date_offset_ms = { type = "integer", default = 5000 },

        ident_flag = { type = "string", default = "BIOS-Ident" },

        cache_resources = { type = "string", default = "bios:iam:resources" },
        cache_change_resources = { type = "string", default = "bios:iam:change_resources:" },
        cache_change_resources_timer_sec = { type = "integer", default = 30 },
        cache_token = { type = "string", default = "bios:iam:token:info:" },
        cache_token_exp_sec = { type = "integer", default = 60 },
        cache_aksk = { type = "string", default = "bios:iam:app:aksk:" },
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
    local _, redis_err = m_redis.init(conf.redis_host, conf.redis_port, conf.redis_database, conf.redis_timeout, conf.redis_password, nil, nil)
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
        res_action = ctx.ident_info.resource_action,
        res_uri = ctx.ident_info.resource_uri,
        app_id = ctx.ident_info.app_id,
        tenant_id = ctx.ident_info.tenant_id,
        account_id = ctx.ident_info.account_id,
        token = ctx.ident_info.token,
        token_kind = ctx.ident_info.token_kind,
        ak = ctx.ident_info.ak,
        roles = ctx.ident_info.roles,
        groups = ctx.ident_info.groups,
    })))
end

return _M