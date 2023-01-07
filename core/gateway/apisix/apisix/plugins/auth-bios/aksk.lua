local ngx = ngx
local core = require("apisix.core")
local m_utils = require("apisix.plugins.auth-bios.utils")
local m_redis = require("apisix.plugins.auth-bios.redis")

local _M = {}

function _M.aksk(conf, ctx)
    -- Fetch args
    local head_key_ak = conf.head_key_ak
    local head_key_sk = conf.head_key_sk
    local app_flag = conf.head_key_app
    local protocol_flag = conf.head_key_protocol

    local cache_key_aksk= conf.cache_key_aksk_info
    local cache_aksk_exp_sec = conf.cache_key_aksk_local_expire_sec

    local resource_uri = ngx.var.request_uri
    local req_method = ngx.var.request_method

    -- package rbum info
    local domain_end_idx = string.find(string.sub(resource_uri, 2), "/")
    if domain_end_idx == nil then
        return 400, "Request is not legal, missing [domain] in path"
    end
    local rbum_kind = core.request.header(ctx, protocol_flag)
    if rbum_kind == nil or rbum_kind == "" then
        rbum_kind = "iam-res"
    end
    local rbum_domain = string.sub(resource_uri, 2, domain_end_idx)
    local rbum_item = string.sub(resource_uri, domain_end_idx + 1)
    local rbum_uri = rbum_kind .. "://" .. rbum_domain .. rbum_item
    local rbum_action = string.lower(req_method)

    -- from header
    local ak = core.request.header(ctx, head_key_ak)
    local sk = core.request.header(ctx, head_key_sk)

    local app_id = core.request.header(ctx, app_flag)
    if app_id == nil or app_id == "" then
        app_id = ""
    end

    -- public
    if ak == nil then
        ctx.ident_info = {
            rbum_uri = rbum_uri,
            rbum_action = rbum_action,
            iam_app_id = '',
            iam_tenant_id = '',
            iam_account_id = '',
            iam_roles = {},
            iam_groups = {},
            own_paths = '',
            ak = '',
        }
        return 200
    end

    -- ak
    if ak ~= nil then
        -- cache_sk = (sk,tenant_id,[appid]), see IamConfig:cache_key_aksk_info_
        local cache_sk_info, redis_err = m_redis.get(cache_key_aksk .. ak, cache_aksk_exp_sec)
        if redis_err then
            error("Redis get error: " .. redis_err)
        end
        local cache_sk = m_utils.split(cache_sk_info, ',')[1]
        if cache_sk_info == nil or cache_sk_info == "" or cache_sk ~= sk then
            return 401, "Ak [" .. ak .. "] is not legal"
        end

        local tenant_id = m_utils.split(cache_sk_info, ',')[2]
        local appid = m_utils.split(cache_sk_info, ',')[3]

        if redis_err then
            error("Redis get error: " .. redis_err)
        end

        local own_paths = tenant_id

        if app_id ~= ""  then
            if app_id~=appid then
                return 401, "Ak [" .. ak .. "] with App [" .. app_id .. "] is not legal"
            end
            own_paths=tenant_id.."/"..app_id
        end

        ctx.ident_info = {
            rbum_uri = rbum_uri,
            rbum_action = rbum_action,
            iam_app_id = appid or '',
            iam_tenant_id = tenant_id or '',
            iam_account_id = '',
            iam_roles = {},
            iam_groups = {},
            own_paths = own_paths,
            ak = ak,
        }
        return 200
    end

    return 200
end

return _M