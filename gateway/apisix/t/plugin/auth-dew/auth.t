use t::APISIX 'no_plan';

no_long_string();
no_root_location();
no_shuffle();
run_tests;
log_level('debug');

__DATA__

=== TEST 1: test resource
--- config
    location /t {
        content_by_lua_block {
            local json = require("cjson")
            local m_utils = require("apisix.plugins.auth-dew.utils")
            local m_resource = require("apisix.plugins.auth-dew.resource")
            local m_auth = require("apisix.plugins.auth-dew.auth")
            m_resource.add_res("FETCH","api://app1.tenant1/p1/**",{tenant_codes="#tenant1#",app_codes="#app1#",account_codes="#account1#"})
            m_resource.add_res("FETCH","api://app1.tenant1/p1/p2",{app_codes="#app2#",role_code="#role1#",account_codes="",role_code="#group1#"})
            local matched_res = m_resource.match_res("FETCH","api://app1.tenant1/p1/p2")

            local result = m_auth.auth({
                res_action = "FETCH",
                res_uri = "api://app1.tenant1/p2",
                app_code = nil,
                tenant_code = nil,
                account_code = nil,
                roles = nil,
                groups = nil,
            })
            ngx.say(result)
            local result = m_auth.auth({
                res_action = "FETCH",
                res_uri = "api://app1.tenant1/p1/p2",
                app_code = nil,
                tenant_code = nil,
                account_code = nil,
                roles = nil,
                groups = nil,
            })
            ngx.say(result)
            local result = m_auth.auth({
                res_action = "FETCH",
                res_uri = "api://app1.tenant1/p1/p3",
                app_code = "app1",
                tenant_code = "tenant1",
                account_code = "account1",
                roles = nil,
                groups = nil,
            })
            ngx.say(result)
            local result = m_auth.auth({
                res_action = "FETCH",
                res_uri = "api://app1.tenant1/p1/p2",
                app_code = "app1",
                tenant_code = "tenant1",
                account_code = "account1",
                roles = nil,
                groups = nil,
            })
            ngx.say(result)
            local result = m_auth.auth({
                res_action = "FETCH",
                res_uri = "api://app1.tenant1/p1/p2",
                app_code = "app2",
                tenant_code = "tenant1",
                account_code = nil,
                roles = {"role1"},
                groups = {"group1"},
            })
            ngx.say(result)
        }
    }
--- request
GET /t
--- response_body
200
401
200
401
200
--- no_error_log
[error]

