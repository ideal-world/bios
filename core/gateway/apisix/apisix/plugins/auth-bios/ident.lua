local ngx = ngx
local core = require("apisix.core")
local m_utils = require("apisix.plugins.auth-bios.utils")
local m_redis = require("apisix.plugins.auth-bios.redis")
local json = require("cjson")

local _M = {}

function _M.ident(conf, ctx)
    -- Fetch args
    local token_flag = conf.head_key_token
    local app_flag = conf.head_key_app
    local protocol_flag = conf.head_key_protocol

    local cache_token = conf.cache_key_token_info
    local cache_account = conf.cache_key_account_info
    local cache_token_exp_sec = conf.cache_key_token_local_expire_sec

    local resource_uri = ngx.var.request_uri
    local req_method = ngx.var.request_method

    -- check
    if resource_uri == nil or resource_uri == "" then
        return 400, { message = "Request is not legal, missing [path]" }
    end
    local domain_end_idx = string.find(string.sub(resource_uri, 2), "/")
    if domain_end_idx == nil then
        return 400, { message = "Request is not legal, missing [domain] in path" }
    end
    local rbum_kind = core.request.header(ctx, protocol_flag)
    if rbum_kind == nil or rbum_kind == "" then
        rbum_kind = "iam-res"
    end
    local app_id = core.request.header(ctx, app_flag)
    if app_id == nil or app_id == "" then
        app_id = ""
    end

    -- package rbum info
    local rbum_domain = string.sub(resource_uri, 2, domain_end_idx)
    local rbum_item = string.sub(resource_uri, domain_end_idx + 1)
    local rbum_uri = rbum_kind .. "://" .. rbum_domain .. rbum_item
    local rbum_action = string.lower(req_method)

    -- ident
    local token = core.request.header(ctx, token_flag)

    -- public
    if token == nil then
        ctx.ident_info = {
            rbum_uri = rbum_uri,
            rbum_action = rbum_action,
            iam_app_id = nil,
            iam_tenant_id = nil,
            iam_account_id = nil,
            iam_roles = nil,
            iam_groups = nil,
        }
        return 200, { message = "" }
    end

    -- token
    if token ~= nil then
        local account_info, redis_err = m_redis.get(cache_token .. token, cache_token_exp_sec)
        if redis_err then
            error("Redis get error: " .. redis_err)
        end
        if account_info == nil or account_info == "" then
            return 401, { message = "Token [" .. token .. "] is not legal" }
        end
        local account_id = m_utils.split(account_info, ',')[2]
        local context, redis_err = m_redis.hget(cache_account .. account_id, app_id)
        if redis_err then
            error("Redis get error: " .. redis_err)
        end
        if context == nil or context == "" then
            return 401, { message = "Token [" .. token .. "] with App [" .. app_id .. "] is not legal" }
        end
        context = json.decode(context)
        local own_paths = m_utils.split(context.own_paths, '/')
        ctx.ident_info = {
            rbum_uri = rbum_uri,
            rbum_action = rbum_action,
            iam_app_id = own_paths[2] or '',
            iam_tenant_id = own_paths[1] or '',
            iam_account_id = context.owner,
            iam_roles = context.roles,
            iam_groups = context.groups,
            own_paths = context.own_paths,
            ak = context.ak,
        }
        return 200, { message = "" }
    end

    return 200, { message = "" }
end

return _M