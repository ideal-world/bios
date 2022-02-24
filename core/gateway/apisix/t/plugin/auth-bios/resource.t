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
            m_resource.add_res("GET","api://*.*/**",{_start=ngx.time(),_end=ngx.time()+3600,app="#app1#app2#",tenant="#tenant1#"})
            m_resource.add_res("GET","api://app1.tenant1/p1",{_start=ngx.time(),_end=ngx.time()+3600,account="#acc#"})
            m_resource.add_res("GET","api://app1.tenant1/p1/p2",{_start=ngx.time(),_end=ngx.time()+3600,role="#role1#"})
            m_resource.add_res("GET","api://app1.tenant1/p1/p2/*",{_start=ngx.time(),_end=ngx.time()+3600,role="#role1#"})
            m_resource.add_res("GET","api://app1.tenant1/p1/p2/**",{_start=ngx.time(),_end=ngx.time()+3600,role="#role1#"})
            m_resource.add_res("GET","api://app1.tenant1/p4/p2/**",{_start=ngx.time(),_end=ngx.time()+3600,role="#role1#"})
            m_resource.add_res("GET","api://app1.tenant1/*/p2",{_start=ngx.time(),_end=ngx.time()+3600,role="#role1#"})
            m_resource.add_res("GET","api://app1.tenant1/**/p5",{_start=ngx.time(),_end=ngx.time()+3600,role="#role1#"})
            m_resource.add_res("GET","api://app1.tenant1/p1?q1=aa",{_start=ngx.time(),_end=ngx.time()+3600,role="#role1#"})
            m_resource.add_res("GET","api://app1.tenant1/p1?q1=aa&q2=bb",{_start=ngx.time(),_end=ngx.time()+3600,role="#role1#"})
            m_resource.add_res("POST","api://app1.tenant1/p1?q1=aa&q2=bb",{_start=ngx.time(),_end=ngx.time()+3600,role="#role1#"})

            local matched_res = m_resource.match_res("get","api://app1.tenant1")
            ngx.say(json.encode(matched_res))
            matched_res = m_resource.match_res("get","api://app1.tenant1/p2")
            ngx.say(matched_res[1].uri)
            matched_res = m_resource.match_res("get","api://app1.tenant1/p1")
            ngx.say(m_utils.table_length(matched_res))
            ngx.say(matched_res[1].uri)
            ngx.say(matched_res[2].uri)
            local matched_res = m_resource.match_res("get","api://app1.tenant1/p1/p2")
            ngx.say(m_utils.table_length(matched_res))
            ngx.say(matched_res[1].uri)
            ngx.say(matched_res[2].uri)
            ngx.say(matched_res[3].uri)
            local matched_res = m_resource.match_res("get","api://app1.tenant1/p1/p2/x")
            ngx.say(m_utils.table_length(matched_res))
            ngx.say(matched_res[1].uri)
            ngx.say(matched_res[2].uri)
            ngx.say(matched_res[3].uri)
            local matched_res = m_resource.match_res("get","api://app1.tenant1/p1/p2/x/y")
            ngx.say(m_utils.table_length(matched_res))
            ngx.say(matched_res[1].uri)
            ngx.say(matched_res[2].uri)
            local matched_res = m_resource.match_res("get","api://app1.tenant1/p1/p2/x/y/p5")
            ngx.say(m_utils.table_length(matched_res))
            ngx.say(matched_res[1].uri)
            ngx.say(matched_res[2].uri)
            ngx.say(matched_res[3].uri)
            local matched_res = m_resource.match_res("get","api://app1.tenant1/p1?q1=aa")
            ngx.say(m_utils.table_length(matched_res))
            ngx.say(matched_res[1].uri)
            ngx.say(matched_res[2].uri)
            local matched_res = m_resource.match_res("get","api://App1.Tenant1/p1?Q2=bb&q1=aa")
            ngx.say(m_utils.table_length(matched_res))
            ngx.say(matched_res[1].uri)
            ngx.say(matched_res[2].uri)

            m_resource.remove_res("get","api://*.*/**")
            m_resource.remove_res("get","api://app1.tenant1/p1")
            m_resource.remove_res("get","api://app1.tenant1/p1/p2")
            m_resource.remove_res("get","api://app1.tenant1/p1/p2/*")
            m_resource.remove_res("get","api://app1.tenant1/p1/p2/**")
            m_resource.remove_res("get","api://app1.tenant1/p4/p2/**")
            m_resource.remove_res("get","api://app1.tenant1/*/p2")
            m_resource.remove_res("get","api://app1.tenant1/**/p5")
            m_resource.remove_res("get","api://app1.tenant1/p1?q1=aa")
            m_resource.remove_res("get","api://app1.tenant1/p1?q2=bb&q1=aa")
            m_resource.remove_res("post","api://app1.tenant1/p1?q2=bb&q1=aa")
            ngx.say(json.encode(m_resource.get_res()))
        }
    }
--- request
GET /t
--- response_body
{}
api://*.*/**
2
api://app1.tenant1/p1
api://*.*/**
3
api://app1.tenant1/p1/p2
api://app1.tenant1/*/p2
api://*.*/**
3
api://app1.tenant1/p1/p2/*
api://app1.tenant1/p1/p2/**
api://*.*/**
2
api://app1.tenant1/p1/p2/**
api://*.*/**
3
api://app1.tenant1/p1/p2/**
api://app1.tenant1/**/p5
api://*.*/**
2
api://app1.tenant1/p1?q1=aa
api://*.*/**
2
api://app1.tenant1/p1?q1=aa&q2=bb
api://*.*/**
{}
--- no_error_log
[error]

