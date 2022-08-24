local m_utils = require("apisix.plugins.auth-bios.utils")
local m_resource = require("apisix.plugins.auth-bios.resource")

local _M = {}

function _M.auth(ident_info)

    local rbum_uri = ident_info.rbum_uri
    local rbum_action = ident_info.rbum_action
    local iam_app_id = ident_info.iam_app_id
    local iam_tenant_id = ident_info.iam_tenant_id
    local iam_account_id = ident_info.iam_account_id
    local iam_roles = ident_info.iam_roles
    local iam_groups = ident_info.iam_groups

    local matched_res = m_resource.match_res(rbum_action, rbum_uri)
    if m_utils.table_length(matched_res) == 0 then
        -- No authentication required
        return 200
    end
    local auth_info = {}
    for _, res in pairs(matched_res) do
        for k, v in pairs(res.auth) do
            if auth_info[k] == nil then
                auth_info[k] = v
            else
                 auth_info[k] =  auth_info[k] .. v
            end
        end
    end
    if auth_info.accounts ~= nil and auth_info.accounts ~= '' and iam_account_id ~= nil and iam_account_id ~= '' and m_utils.contain(auth_info.accounts, "#" .. iam_account_id .. "#") then
        return 200
    end
    if auth_info.roles ~= nil and auth_info.roles ~= '' and iam_roles ~= nil then
        for _, iam_role in pairs(iam_roles) do
            if iam_role ~= nil and iam_role ~= '' and m_utils.contain(auth_info.roles, "#" .. iam_role .. "#") then
                return 200
            end
        end
    end
    if auth_info.groups ~= nil and auth_info.groups ~= '' and iam_groups ~= nil then
        for _, iam_group in pairs(iam_groups) do
            if iam_group ~= nil and iam_group ~= '' and m_utils.contain_with_regex(auth_info.groups, "#" .. iam_group .. "%w-#") then
                return 200
            end
        end
    end
    if auth_info.apps ~= nil and auth_info.apps ~= '' and iam_app_id ~= nil and iam_app_id ~= '' and m_utils.contain(auth_info.apps, "#" .. iam_app_id .. "#") then
        return 200
    end
    if auth_info.tenants ~= nil and auth_info.tenants ~= '' and iam_tenant_id ~= nil and iam_tenant_id ~= '' and m_utils.contain(auth_info.tenants, "#" .. iam_tenant_id .. "#") then
        return 200
    end

    return 401, "Permission denied"
end

return _M