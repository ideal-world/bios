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
            local m_utils = require("apisix.plugins.auth-bios.utils")
            local m_resource = require("apisix.plugins.auth-bios.resource")
            local m_auth = require("apisix.plugins.auth-bios.auth")
            m_resource.add_res("FETCH","api://app1.tenant1/p1/**",{_start=ngx.time(),_end=ngx.time()+3600,tenant="#tenant1#",app="#app1#",account="#account1#"})
            m_resource.add_res("FETCH","api://app1.tenant1/p1/p2",{_start=ngx.time(),_end=ngx.time()+3600,app="#app2#",role="#role1#",account="",group_node="#group1#"})
            local matched_res = m_resource.match_res("FETCH","api://app1.tenant1/p1/p2")

            local result = m_auth.auth({
                res_action = "FETCH",
                res_uri = "api://app1.tenant1/p2",
                app_id = nil,
                tenant_id = nil,
                account_id = nil,
                roles = nil,
                groups = nil,
            })
            ngx.say(result)
            local result = m_auth.auth({
                res_action = "FETCH",
                res_uri = "api://app1.tenant1/p1/p2",
                app_id = nil,
                tenant_id = nil,
                account_id = nil,
                roles = nil,
                groups = nil,
            })
            ngx.say(result)
            local result = m_auth.auth({
                res_action = "FETCH",
                res_uri = "api://app1.tenant1/p1/p3",
                app_id = "app1",
                tenant_id = "tenant1",
                account_id = "account1",
                roles = nil,
                groups = nil,
            })
            ngx.say(result)
            local result = m_auth.auth({
                res_action = "FETCH",
                res_uri = "api://app1.tenant1/p1/p2",
                app_id = "app1",
                tenant_id = "tenant1",
                account_id = "account1",
                roles = nil,
                groups = nil,
            })
            ngx.say(result)
            local result = m_auth.auth({
                res_action = "FETCH",
                res_uri = "api://app1.tenant1/p1/p2",
                app_id = "app2",
                tenant_id = "tenant1",
                account_id = nil,
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

