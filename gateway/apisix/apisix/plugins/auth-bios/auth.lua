local m_utils = require("apisix.plugins.auth-bios.utils")
local m_resource = require("apisix.plugins.auth-bios.resource")

local _M = {}

function _M.auth(ident_info)
    local res_action = ident_info.res_action
    local res_uri = ident_info.res_uri
    local tenant_id = ident_info.tenant_id
    local app_id = ident_info.app_id
    local account_id = ident_info.account_id
    local roles = ident_info.roles
    local groups = ident_info.groups

    local matched_res = m_resource.match_res(res_action, res_uri)
    if m_utils.table_length(matched_res) == 0 then
        -- No authentication required
        return 200, { message = "" }
    end
    local auth_info = {}
    for _, res in pairs(matched_res) do
        for k, v in pairs(res.auth) do
            if auth_info[k] == nil then
                auth_info[k] = v
            end
        end
    end
    if auth_info["tenant"] ~= nil and string.len(auth_info["tenant"]) ~= 0
            and (tenant_id == nil or m_utils.contain(auth_info["tenant"], "#" .. tenant_id .. "#") == false) then
        return 401, { message = "Permission denied" }
    end
    if auth_info["app"] ~= nil and string.len(auth_info["app"]) ~= 0
            and (app_id == nil or m_utils.contain(auth_info["app"], "#" .. app_id .. "#") == false) then
        return 401, { message = "Permission denied" }
    end
    if auth_info["account"] ~= nil and string.len(auth_info["account"]) ~= 0
            and (account_id == nil or m_utils.contain(auth_info["account"], "#" .. account_id .. "#") == false) then
        return 401, { message = "Permission denied" }
    end
    if auth_info["role"] ~= nil and string.len(auth_info["role"]) ~= 0 then
        if roles == nil then
            return 401, { message = "Permission denied" }
        end
        for i = 1, m_utils.table_length(roles) do
            if m_utils.contain(auth_info["role"], "#" .. roles[i] .. "#") == false then
                return 401, { message = "Permission denied" }
            end
        end
    end
    if auth_info["group_node"] ~= nil and string.len(auth_info["group_node"]) ~= 0 then
        if groups == nil then
            return 401, { message = "Permission denied" }
        end
        for i = 1, m_utils.table_length(groups) do
            if m_utils.contain(auth_info["group_node"], "#" .. groups[i] .. "#") == false then
                return 401, { message = "Permission denied" }
            end
        end
    end
    return 200, { message = "" }
end

return _M