local core   = require("apisix.core")
local http   = require("resty.http")
local type   = type

local schema = {
    type = "object",
    properties = {
        host = { type = "string" },
        ssl_verify = {
            type = "boolean",
            default = true,
        },
        timeout = {
            type = "integer",
            minimum = 1,
            maximum = 60000,
            default = 3000,
            description = "timeout in milliseconds",
        },
        keepalive = { type = "boolean", default = true },
        keepalive_timeout = { type = "integer", minimum = 1000, default = 60000 },
        keepalive_pool = { type = "integer", minimum = 1, default = 5 },
        with_route = { type = "boolean", default = false },
        with_service = { type = "boolean", default = false },
        with_consumer = { type = "boolean", default = false },
        head_key_context = { type = "string", default = "Tardis-Context" },
        head_key_crypto = { type = "string", default = "Tardis-Crypto" },
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
    required = { "host" }
}


local _M = {
    version = 0.1,
    priority = 5001,
    name = "auth",
    schema = schema,
}


function _M.check_schema(conf)
    return core.schema.check(schema, conf)
end

local function cors(conf)
    core.response.set_header("Access-Control-Allow-Origin", conf.cors_allow_origin)
    core.response.set_header("Access-Control-Allow-Methods", conf.cors_allow_methods)
    core.response.set_header("Access-Control-Allow-Headers", conf.cors_allow_headers)
    core.response.set_header("Access-Control-Max-Age", "3600000")
    core.response.set_header("Access-Control-Allow-Credentials", "true")
    core.response.set_header("Content-Type", "application/json")
end

local function request_uri(method, endpoint, body, conf, ctx)
    -- TODO Test
    if body.headers[conf.head_key_crypto] ~= nil then
        local req_body = core.request.get_body(ctx)
        body.body = req_body
    end
    core.log.trace("auth-bios forward_body:", core.json.encode(body));
    local params = {
        method = method,
        body = core.json.encode(body),
        headers = {
            ["Content-Type"] = "application/json",
        },
        keepalive = conf.keepalive,
        ssl_verify = conf.ssl_verify
    }

    if conf.keepalive then
        params.keepalive_timeout = conf.keepalive_timeout
        params.keepalive_pool = conf.keepalive_pool
    end

    local httpc = http.new()
    httpc:set_timeout(conf.timeout)

    local res, req_err = httpc:request_uri(endpoint, params)
    if not res then
        core.log.error("failed auth service, err: ", req_err)
        cors(conf)
        return nil, 403
    end

    core.log.trace("auth service response body:", res.body);
    local forward_resp, err = core.json.decode(res.body)

    if not forward_resp then
        core.log.error("invalid response body: ", res.body, " err: ", err)
        cors(conf)
        return nil, 503
    end

    if forward_resp.code ~= '200' then
        core.log.error("invalid auth service return code: ", forward_resp.code,
            " err:", forward_resp.msg)
        cors(conf)
        return nil, 502
    end

    local result = forward_resp.data
    return result
end

function _M.access(conf, ctx)
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

    local uri = ctx.var.uri
    if uri == nil or uri == '' then
        uri = "/"
    end

    local forward_body = {
        scheme  = core.request.get_scheme(ctx),
        method  = core.request.get_method(),
        host    = core.request.get_host(ctx),
        port    = core.request.get_port(ctx),
        path    = uri,
        headers = core.request.headers(ctx),
        query   = core.request.get_uri_args(ctx),
    }
    -- TODO Test
    if forward_body.headers[conf.head_key_crypto] ~= nil then
        req_body = core.request.get_body(ctx)
        forward_body.body = req_body
    end
    core.log.trace("auth-bios forward_body:", core.json.encode(forward_body));
    local params = {
        method = "PUT",
        body = core.json.encode(forward_body),
        headers = {
            ["Content-Type"] = "application/json",
        },
        keepalive = conf.keepalive,
        ssl_verify = conf.ssl_verify
    }

    if conf.keepalive then
        params.keepalive_timeout = conf.keepalive_timeout
        params.keepalive_pool = conf.keepalive_pool
    end

    local host_end_idx = string.find(string.sub(conf.host, -2), "/")
    local endpoint = conf.host .. "/auth/auth"
    if host_end_idx then
        endpoint = conf.host .. "auth/auth"
    end


    local httpc = http.new()
    httpc:set_timeout(conf.timeout)

    local res, req_err = httpc:request_uri(endpoint, params)

    if not res then
        core.log.error("failed auth service, err: ", req_err)
        cors(conf)
        return 403
    end

    core.log.trace("auth service response body:", res.body);
    local forward_resp, err = core.json.decode(res.body)

    if not forward_resp then
        core.log.error("invalid response body: ", res.body, " err: ", err)
        cors(conf)
        return 503
    end

    if forward_resp.code ~= '200' then
        core.log.error("invalid auth service return code: ", forward_resp.code,
            " err:", forward_resp.msg)
        cors(conf)
        return 502
    end

    local result = forward_resp.data

    if not result.allow then
        local status_code = 403
        if result.status_code then
            status_code = result.status_code
        end

        local reason = nil
        if result.reason then
            reason = type(result.reason) == "table"
                and core.json.encode(result.reason)
                or result.reason
        end
        if result.body ~= nil then
            -- TODO
            core.request.set_body_data(result.body)
        end

        cors(conf)
        return status_code, { code = status_code .. '-gateway-cert-error', message = reason }
    else
        if result.headers then
            core.log.trace("request.headers: ", core.json.encode(result.headers[conf.head_key_context]))
            core.request.set_header(ctx, conf.head_key_context, result.headers[conf.head_key_context])
        end
    end
end

function _M.body_filter(_, ctx)
    local conf = ctx.body_transformer_conf
    if conf.response then
        local body = core.response.hold_body_chunk(ctx)
        if ngx.arg[2] == false and not body then
            return
        end

        local result = request_uri('PUT', '/auth/crypto', body, conf, ctx)

        if not result then
            core.log.error("failed to  response body: ", body)
            return
        end

        ngx.arg[1] = result
    end
end

return _M
