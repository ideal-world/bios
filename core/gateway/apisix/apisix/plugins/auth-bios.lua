local core = require("apisix.core")
local m_redis = require("apisix.plugins.auth-bios.redis")
local m_init = require("apisix.plugins.auth-bios.init")
local m_ident = require("apisix.plugins.auth-bios.ident")
local m_auth = require("apisix.plugins.auth-bios.auth")
local json = require("cjson")
local ngx = ngx
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

        head_key_token = { type = "string", default = "Bios-Token" },
        head_key_app = { type = "string", default = "Bios-App" },
        head_key_protocol = { type = "string", default = "Bios-Proto" },
        head_key_context = { type = "string", default = "Tardis-Context" },

        cache_key_token_info = { type = "string", default = "iam:cache:token:info:" },
        cache_key_account_info = { type = "string", default = "iam:cache:account:info:" },
        cache_key_token_local_expire_sec = { type = "integer", default = 0 },

        cache_key_res_info = { type = "string", default = "iam:res:info" },
        cache_key_res_changed_info = { type = "string", default = "iam:res:changed:info:" },
        cache_key_res_changed_timer_sec = { type = "integer", default = 30 },

        cors_allow_origin = { type = "string", default = "*" },
        cors_allow_methods = { type = "string", default = "*" },
        cors_allow_headers = { type = "string", default = "*" },

        exclude_prefix_paths = {
            type = "array",
            items = {
                type = "string",
            },
            default = {}
        },
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
    json.encode_empty_table_as_object(false)
    m_init.init(conf.cache_key_res_info, conf.cache_key_res_changed_info, conf.cache_key_res_changed_timer_sec)
    return true
end

local function cors(conf)
    core.response.set_header("Access-Control-Allow-Origin", conf.cors_allow_origin)
    core.response.set_header("Access-Control-Allow-Methods", conf.cors_allow_methods)
    core.response.set_header("Access-Control-Allow-Headers", conf.cors_allow_headers)
    core.response.set_header("Access-Control-Max-Age", "3600000")
    core.response.set_header("Access-Control-Allow-Credentials", "true")
    core.response.set_header("Content-Type", "application/json")
end

function _M.rewrite(conf, ctx)
    local path = ngx.var.request_uri
    for _, prefix_path in pairs(conf.exclude_prefix_paths) do
        if string.sub(path, 1, string.len(prefix_path)) == prefix_path then
            return 200
        end
    end
    if ctx.var.request_method == "OPTIONS" then
        cors(conf)
        return 200
    end
    local ident_code, ident_message = m_ident.ident(conf, ctx)
    if ident_code ~= 200 then
        cors(conf)
        return ident_code, { message = ident_message }
    end
    local auth_code, auth_message = m_auth.auth(ctx.ident_info)
    if auth_code ~= 200 then
        cors(conf)
        return auth_code, { message = auth_message }
    end
    local context = json.encode({
        own_paths = ctx.ident_info.own_paths,
        owner = ctx.ident_info.iam_account_id,
        ak = ctx.ident_info.ak,
        roles = ctx.ident_info.iam_roles,
        groups = ctx.ident_info.iam_groups,
    })
    core.request.set_header(ctx, conf.head_key_context, ngx_encode_base64(context))
end

return _M