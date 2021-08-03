local ngx = ngx
local core = require("apisix.core")
local m_utils = require("apisix.plugins.auth-dew.utils")
local m_redis = require("apisix.plugins.auth-dew.redis")
local json = require("cjson")
local hmac_sha1 = ngx.hmac_sha1
local ngx_encode_base64 = ngx.encode_base64

local _M = {}

function _M.ident(conf, ctx)
    -- Fetch args
    local token_flag = conf.token_flag
    local auth_flag = conf.auth_flag
    local date_flag = conf.date_flag
    local host_flag = conf.host_flag
    local request_date_offset_ms = conf.request_date_offset_ms

    local cache_token = conf.cache_token
    local cache_token_exp_sec = conf.cache_token_exp_sec
    local cache_aksk = conf.cache_aksk
    local cache_aksk_exp_sec = conf.cache_aksk_exp_sec

    local host = core.request.header(ctx, host_flag)
    local req_method = ngx.var.request_method

    -- Check
    if host == nil or host == "" then
        return 400, { message = "Request is not legal, missing [" .. host_flag .. "] in Header" }
    end

    local resource_uri = ngx.var.request_uri
    if resource_uri == nil or resource_uri == "" then
        return 400, { message = "Request is not legal, missing [path]" }
    end
    local i = string.find(string.sub(resource_uri, 2), "/")
    if i == nil then
        return 400, { message = "Request is not legal, missing [service flag] in path" }
    end
    local resource_action = string.lower(req_method)
    resource_uri = string.sub(resource_uri, 2, i) .. "://" .. host .. string.sub(resource_uri, i + 1)

    -- Ident
    local token = core.request.header(ctx, token_flag)
    local authorization = core.request.header(ctx, auth_flag)

    -- public
    if token == nil and authorization == nil then
        ctx.ident_info = {
            res_action = resource_action,
            res_uri = resource_uri,
            app_code = nil,
            tenant_code = nil,
            account_code = nil,
            account_name = nil,
            token_kind = nil,
            roles = nil,
            groups = nil,
        }
        return 200, { message = "" }
    end
    -- token
    if token ~= nil then
        local opt_info, redis_err = m_redis.get(cache_token .. token, cache_token_exp_sec)
        if redis_err then
            error("Redis get error: " .. redis_err)
        end
        if opt_info == nil or opt_info == "" then
            return 401, { message = "Token [" .. token .. "] is not legal" }
        end
        opt_info = json.decode(opt_info)
        ctx.ident_info = {
            res_action = resource_action,
            res_uri = resource_uri,
            app_code = opt_info.app_code,
            tenant_code = opt_info.tenant_code,
            account_code = opt_info.account_code,
            account_name = opt_info.account_name,
            token_kind = opt_info.token_kind,
            roles = opt_info.roles,
            groups = opt_info.groups,
        }
        return 200, { message = "" }
    end

    -- authorization
    local req_date = core.request.header(ctx, date_flag)
    if req_date == nil or req_date == "" then
        return 400, { message = "Request is not legal, missing [" .. date_flag .. "]" }
    end
    local req_time = ngx.parse_http_time(req_date)
    if req_time == nil then
        return 400, { message = "Request Date [" .. req_date .. "] is not legal" }
    end
    if req_time * 1000 + request_date_offset_ms < ngx.time() * 1000 then
        return 400, { message = "Request has expired" }
    end
    if m_utils.contain(authorization, ":") == false then
        return 400, { message = "Authorization [" .. authorization .. "] is not legal" }
    end
    local auth_info = m_utils.split(authorization, ":")
    local req_ak = auth_info[1]
    local req_signature = auth_info[2]
    local req_path = ngx.var.uri
    local sorted_req_query = m_utils.sort_query(ngx.req.get_uri_args())

    local aksk_info, redis_err = m_redis.get(cache_aksk .. req_ak, cache_aksk_exp_sec)
    if redis_err then
        error("Redis get error: " .. redis_err)
    end
    if aksk_info == nil or aksk_info == "" then
        return 401, { message = "Authorization [" .. authorization .. "] is not legal" }
    end
    aksk_info = m_utils.split(aksk_info, ":")
    local sk = aksk_info[1]
    local tenant_code = aksk_info[2]
    local app_code = aksk_info[3]
    local calc_signature = ngx_encode_base64(hmac_sha1(sk, string.lower(req_method .. "\n" .. req_date .. "\n" .. req_path .. "\n" .. sorted_req_query)))
    if calc_signature ~= req_signature then
        return 401, { message = "Authorization [" .. authorization .. "] is not legal" }
    end
    ctx.ident_info = {
        res_action = resource_action,
        res_uri = resource_uri,
        app_code = app_code,
        tenant_code = tenant_code,
        account_code = nil,
        account_name = nil,
        token_kind = nil,
        roles = nil,
        groups = nil,
    }
    return 200, { message = "" }
end

return _M