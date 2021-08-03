local m_utils = require("apisix.plugins.auth-dew.utils")
local m_resource = require("apisix.plugins.auth-dew.resource")

local _M = {}

function _M.auth(ident_info)
    local res_action = ident_info.res_action
    local res_uri = ident_info.res_uri
    local tenant_code = ident_info.tenant_code
    local app_code = ident_info.app_code
    local account_code = ident_info.account_code
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
    if auth_info["tenant_codes"] ~= nil and string.len(auth_info["tenant_codes"]) ~= 0
            and (tenant_code == nil or m_utils.contain(auth_info["tenant_codes"], "#" .. tenant_code .. "#") == false) then
        return 401, { message = "Permission denied" }
    end
    if auth_info["app_codes"] ~= nil and string.len(auth_info["app_codes"]) ~= 0
            and (app_code == nil or m_utils.contain(auth_info["app_codes"], "#" .. app_code .. "#") == false) then
        return 401, { message = "Permission denied" }
    end
    if auth_info["account_codes"] ~= nil and string.len(auth_info["account_codes"]) ~= 0
            and (account_code == nil or m_utils.contain(auth_info["account_codes"], "#" .. account_code .. "#") == false) then
        return 401, { message = "Permission denied" }
    end
    if auth_info["role_codes"] ~= nil and string.len(auth_info["role_codes"]) ~= 0 then
        if roles == nil then
            return 401, { message = "Permission denied" }
        end
        for i = 1, m_utils.table_length(roles) do
            if m_utils.contain(auth_info["role_codes"], "#" .. roles[i] .. "#") == false then
                return 401, { message = "Permission denied" }
            end
        end
    end
    if auth_info["group_codes"] ~= nil and string.len(auth_info["group_codes"]) ~= 0 then
        if groups == nil then
            return 401, { message = "Permission denied" }
        end
        for i = 1, m_utils.table_length(groups) do
            if m_utils.contain(auth_info["group_codes"], "#" .. groups[i] .. "#") == false then
                return 401, { message = "Permission denied" }
            end
        end
    end
    return 200, { message = "" }
end

return _M