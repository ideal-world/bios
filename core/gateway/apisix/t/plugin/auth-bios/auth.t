use t::APISIX 'no_plan';

no_long_string();
no_root_location();
no_shuffle();
run_tests;
log_level('debug');

__DATA__

=== TEST Auth
--- config
    location /t {
        content_by_lua_block {
            local m_utils = require("apisix.plugins.auth-bios.utils")
            local m_resource = require("apisix.plugins.auth-bios.resource")
            local m_auth = require("apisix.plugins.auth-bios.auth")

            local result = m_auth.auth({
                rbum_uri = "iam-res://iam-serv",
                rbum_action = "get",
                iam_app_id = nil,
                iam_tenant_id = nil,
                iam_account_id = nil,
                iam_roles = nil,
                iam_groups = nil
            })
            ngx.say('public -> ',result)

            m_resource.add_res("GET","iam-res://iam-serv",{apps="#app1#app2#",tenants="#tenant1#"})
            local result = m_auth.auth({
                rbum_uri = "iam-res://iam-serv",
                rbum_action = "get",
                iam_app_id = nil,
                iam_tenant_id = nil,
                iam_account_id = nil,
                iam_roles = nil,
                iam_groups = nil
            })
            ngx.say('private -> ',result)

            m_resource.add_res("GET","iam-res://iam-serv",{accounts="#acc1#acc2#"})
            local result = m_auth.auth({
                rbum_uri = "iam-res://iam-serv",
                rbum_action = "get",
                iam_app_id = nil,
                iam_tenant_id = nil,
                iam_account_id = "acc3",
                iam_roles = nil,
                iam_groups = nil
            })
            ngx.say('acc0 -> ',result)
            local result = m_auth.auth({
                rbum_uri = "iam-res://iam-serv",
                rbum_action = "get",
                iam_app_id = nil,
                iam_tenant_id = nil,
                iam_account_id = "acc1",
                iam_roles = nil,
                iam_groups = nil
            })
            ngx.say('acc1 -> ',result)

            m_resource.add_res("GET","iam-res://iam-serv",{roles="#role1#role2#"})
            local result = m_auth.auth({
                rbum_uri = "iam-res://iam-serv",
                rbum_action = "get",
                iam_app_id = nil,
                iam_tenant_id = nil,
                iam_account_id = "",
                iam_roles = {"role0"},
                iam_groups = nil
            })
            ngx.say('role0 -> ',result)
            local result = m_auth.auth({
                rbum_uri = "iam-res://iam-serv",
                rbum_action = "get",
                iam_app_id = nil,
                iam_tenant_id = nil,
                iam_account_id = "",
                iam_roles = {"role1"},
                iam_groups = nil
            })
            ngx.say('role1 -> ',result)

            m_resource.add_res("GET","iam-res://iam-serv",{groups="#g2.aaaa#g1.aaab##g1.aaaaaaaa##g1.aaaaaaab#"})
            local result = m_auth.auth({
                rbum_uri = "iam-res://iam-serv",
                rbum_action = "get",
                iam_app_id = nil,
                iam_tenant_id = nil,
                iam_account_id = "",
                iam_roles = "",
                iam_groups = {"g2.bbbb"}
            })
            ngx.say('g2.bbbb -> ',result)
            local result = m_auth.auth({
                rbum_uri = "iam-res://iam-serv",
                rbum_action = "get",
                iam_app_id = nil,
                iam_tenant_id = nil,
                iam_account_id = "",
                iam_roles = "",
                iam_groups = {"g1.aaab"}
            })
            ngx.say('g1.aaab -> ',result)
            local result = m_auth.auth({
                rbum_uri = "iam-res://iam-serv",
                rbum_action = "get",
                iam_app_id = nil,
                iam_tenant_id = nil,
                iam_account_id = "",
                iam_roles = "",
                iam_groups = {"g1.aaaa"}
            })
            ngx.say('g1.aaaa -> ',result)

            m_resource.add_res("GET","iam-res://iam-serv",{apps="#app1#app2#"})
            local result = m_auth.auth({
                rbum_uri = "iam-res://iam-serv",
                rbum_action = "get",
                iam_app_id = "app0",
                iam_tenant_id = nil,
                iam_account_id = "",
                iam_roles = {},
                iam_groups = {}
            })
            ngx.say('app0 -> ',result)
            local result = m_auth.auth({
                rbum_uri = "iam-res://iam-serv",
                rbum_action = "get",
                iam_app_id = "app1",
                iam_tenant_id = nil,
                iam_account_id = "",
                iam_roles = {},
                iam_groups = {}
            })
            ngx.say('app1 -> ',result)

            m_resource.add_res("GET","iam-res://iam-serv",{tenants="#tenant1#tenant2#"})
            local result = m_auth.auth({
                rbum_uri = "iam-res://iam-serv",
                rbum_action = "get",
                iam_app_id = "",
                iam_tenant_id = "tenant0",
                iam_account_id = "",
                iam_roles = {},
                iam_groups = {}
            })
            ngx.say('tenant0 -> ',result)
            local result = m_auth.auth({
                rbum_uri = "iam-res://iam-serv",
                rbum_action = "get",
                iam_app_id = "",
                iam_tenant_id = "tenant1",
                iam_account_id = "",
                iam_roles = {},
                iam_groups = {}
            })
            ngx.say('tenant1 -> ',result)


            m_resource.add_res("GET","iam-res://iam-serv",{tenants="#tenant1#tenant2#"})
            local result = m_auth.auth({
                rbum_uri = "iam-res://iam-serv",
                rbum_action = "get",
                iam_app_id = "app1",
                iam_tenant_id = "tenant1",
                iam_account_id = "acc1",
                iam_roles = {},
                iam_groups = {}
            })
            ngx.say('all -> ',result)

            m_resource.add_res("GET","iam-res://app1/ct/account/001",{accounts="#acc1#acc2#"})
            local result = m_auth.auth({
                rbum_uri = "iam-res://app1/ct/account/001",
                rbum_action = "get",
                iam_app_id = nil,
                iam_tenant_id = nil,
                iam_account_id = "acc3",
                iam_roles = nil,
                iam_groups = nil
            })
            ngx.say('app[1] -> ',result)
            local result = m_auth.auth({
                rbum_uri = "iam-res://app1/ct/account/001",
                rbum_action = "get",
                iam_app_id = nil,
                iam_tenant_id = nil,
                iam_account_id = "acc1",
                iam_roles = nil,
                iam_groups = nil
            })
            ngx.say('app[2] -> ',result)
            m_resource.add_res("GET","iam-res://app1/ct/account/**",{accounts="#acc3#"})
            local result = m_auth.auth({
                rbum_uri = "iam-res://app1/ct/account/001",
                rbum_action = "get",
                iam_app_id = nil,
                iam_tenant_id = nil,
                iam_account_id = "acc3",
                iam_roles = nil,
                iam_groups = nil
            })
            ngx.say('app[3] -> ',result)
            m_resource.add_res("GET","iam-res://app1/ct/**",{roles="#tenant_admin#"})
            local result = m_auth.auth({
                 rbum_uri = "iam-res://app1/ct/account/001",
                rbum_action = "get",
                iam_app_id = nil,
                iam_tenant_id = nil,
                iam_account_id = "",
                iam_roles = {"tenant_admin"},
                iam_groups = nil
            })
             ngx.say('app[4] -> ',result)
        }
    }
--- request
GET /t
--- response_body
public -> 200
private -> 401
acc0 -> 401
acc1 -> 200
role0 -> 401
role1 -> 200
g2.bbbb -> 401
g1.aaab -> 200
g1.aaaa -> 200
app0 -> 401
app1 -> 200
tenant0 -> 401
tenant1 -> 200
all -> 200
app[1] -> 401
app[2] -> 200
app[3] -> 200
app[4] -> 200
--- no_error_log
[error]

