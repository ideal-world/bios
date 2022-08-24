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

            m_resource.add_res("GET","iam-res://iam-serv",{apps="#app1#app2#",tenants="#tenant1#"})
            m_resource.add_res("GET","iam-res://*/**",{apps="#app1#app2#",tenants="#tenant1#"})
            m_resource.add_res("GET","iam-res://iam-serv/p1",{accounts="#acc#"})
            m_resource.add_res("GET","iam-res://iam-serv/p1/p2",{roles="#role1#"})
            m_resource.add_res("GET","iam-res://iam-serv/p1/p2/*",{roles="#role1#"})
            m_resource.add_res("GET","iam-res://iam-serv/p1/p2/**",{roles="#role1#"})
            m_resource.add_res("GET","iam-res://iam-serv/p4/p2/**",{roles="#role1#"})
            m_resource.add_res("GET","iam-res://iam-serv/*/p2",{roles="#role1#"})
            m_resource.add_res("GET","iam-res://iam-serv/**/p5",{roles="#role1#"})
            m_resource.add_res("GET","iam-res://iam-serv/p1?q1=aa",{roles="#role1#"})
            m_resource.add_res("GET","iam-res://iam-serv/p1?q1=aa&q2=bb",{roles="#role1#"})
            m_resource.add_res("POST","iam-res://iam-serv/p1?q1=aa&q2=bb",{roles="#role1#"})

            ngx.say("get|iam-res://iam-serv -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv")) 
                .. " " .. m_resource.match_res("get","iam-res://iam-serv")[1].uri)
            ngx.say("get|iam-res://iam-serv/iam2 -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam2")))
            ngx.say("get|iam-res://iam-serv/ss -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/ss")) 
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/ss")[1].uri)
            ngx.say("get|iam-res://iam2/ss -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam2/ss")) 
                .. " " .. m_resource.match_res("get","iam-res://iam2/ss")[1].uri)
            ngx.say("get|iam-res://iam-serv/p2 -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p2")) 
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p2")[1].uri)
            ngx.say("get|iam-res://iam-serv/p1 -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1")) 
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1")[1].uri 
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1")[2].uri)
            ngx.say("get|iam-res://iam-serv/p1/p2 -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1/p2")) 
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2")[1].uri
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2")[2].uri
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2")[3].uri)
            ngx.say("get|iam-res://iam-serv/p1/p2/x -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1/p2/x")) 
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2/x")[1].uri
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2/x")[2].uri
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2/x")[3].uri)
            ngx.say("get|iam-res://iam-serv/p1/p2/x/y -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1/p2/x/y")) 
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2/x/y")[1].uri
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2/x/y")[2].uri)
            ngx.say("get|iam-res://iam-serv/p1/p2/x/y/p5 -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1/p2/x/y/p5")) 
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2/x/y/p5")[1].uri
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2/x/y/p5")[2].uri
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2/x/y/p5")[3].uri)
            ngx.say("get|iam-res://iam-serv/p1?q1=aa -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1?q1=aa")) 
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1?q1=aa")[1].uri
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1?q1=aa")[2].uri)
            ngx.say("get|iam-res://iam-serv/p1?Q2=bb&q1=aa -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1?Q2=bb&q1=aa")) 
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1?Q2=bb&q1=aa")[1].uri
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1?Q2=bb&q1=aa")[2].uri)
            ngx.say("post|iam-res://iam-serv/p1?Q2=bb&q1=aa -> " .. m_utils.table_length(m_resource.match_res("post","iam-res://iam-serv/p1?Q2=bb&q1=aa")) 
                .. " " .. m_resource.match_res("post","iam-res://iam-serv/p1?Q2=bb&q1=aa")[1].uri)
            ngx.say("get|iam-res://App1.Tenant1/p1?Q2=bb&q1=aa -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://App1.Tenant1/p1?Q2=bb&q1=aa")) 
                .. " " .. m_resource.match_res("get","iam-res://App1.Tenant1/p1?Q2=bb&q1=aa")[1].uri)

            ngx.say("----")

            ngx.say("DEL:get|iam-res://iam-serv/**")
            m_resource.remove_res("get","iam-res://iam-serv/**")
            ngx.say("get|iam-res://iam-serv -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv")))

            ngx.say("DEL:get|iam-res://iam-serv/")
            m_resource.remove_res("get","iam-res://iam-serv/")
            ngx.say("get|iam-res://iam-serv -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv")))
            ngx.say("get|iam-res://iam-serv/p2 -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p2")) 
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p2")[1].uri)

            ngx.say("DEL:get|iam-res://*/**")
            m_resource.remove_res("get","iam-res://*/**")
            ngx.say("get|iam-res://iam-serv/p2 -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p2")))
            ngx.say("get|iam-res://iam2/ss -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam2/ss")))
            ngx.say("get|iam-res://iam-serv/p1 -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1")) 
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1")[1].uri)

            ngx.say("DEL:get|iam-res://iam-serv/p1")
            m_resource.remove_res("get","iam-res://iam-serv/p1")
            ngx.say("get|iam-res://iam-serv/p1 -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1")))

            ngx.say("DEL:get|iam-res://iam-serv/p1/p2")
            m_resource.remove_res("get","iam-res://iam-serv/p1/p2")
            ngx.say("get|iam-res://iam-serv/p1/p2 -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1/p2"))
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2")[1].uri)
            ngx.say("get|iam-res://iam-serv/p1/p2/x -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1/p2/x")) 
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2/x")[1].uri
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2/x")[2].uri)

            ngx.say("DEL:get|iam-res://iam-serv/p1/p2/*")
            m_resource.remove_res("get","iam-res://iam-serv/p1/p2/*")
            ngx.say("get|iam-res://iam-serv/p1/p2/x -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1/p2/x"))
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2/x")[1].uri)
            ngx.say("get|iam-res://iam-serv/p1/p2/x/y -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1/p2/x/y"))
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2/x/y")[1].uri)

            ngx.say("DEL:get|iam-res://iam-serv/p1/p2/**")
            m_resource.remove_res("get","iam-res://iam-serv/p1/p2/**")
            ngx.say("get|iam-res://iam-serv/p1/p2/x -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1/p2/x")))
            ngx.say("get|iam-res://iam-serv/p1/p2/x/y -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1/p2/x/y")))
            ngx.say("get|iam-res://iam-serv/p1/p2/p5 -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1/p2/p5"))
                .. " " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2/p5")[1].uri)

            ngx.say("DEL:get|iam-res://iam-serv/**/p5")
            m_resource.remove_res("get","iam-res://iam-serv/**/p5")
            ngx.say("get|iam-res://iam-serv/p1/p2/p5 -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1/p2/p5")))

            ngx.say("DEL:get|iam-res://iam-serv/p1?q1=aa")
            m_resource.remove_res("get","iam-res://iam-serv/p1?q1=aa")
            ngx.say("get|iam-res://iam-serv/p1?q1=aa -> " .. m_utils.table_length(m_resource.match_res("get","iam-res://iam-serv/p1?q1=aa")))


            m_resource.remove_res("get","iam-res://iam-serv/p1?Q2=bb&q1=aa")
            m_resource.remove_res("post","iam-res://iam-serv/p1?Q2=bb&q1=aa")
            m_resource.remove_res("get","iam-res://iam-serv/p4/p2/**")
            m_resource.remove_res("get","iam-res://iam-serv/*/p2")

            ngx.say(json.encode(m_resource.get_res()))
        }
    }
--- request
GET /t
--- response_body
get|iam-res://iam-serv -> 1 iam-res://iam-serv
get|iam-res://iam-serv/iam2 -> 0
get|iam-res://iam-serv/ss -> 1 iam-res://*/**
get|iam-res://iam2/ss -> 1 iam-res://*/**
get|iam-res://iam-serv/p2 -> 1 iam-res://*/**
get|iam-res://iam-serv/p1 -> 2 iam-res://iam-serv/p1 iam-res://*/**
get|iam-res://iam-serv/p1/p2 -> 3 iam-res://iam-serv/p1/p2 iam-res://iam-serv/*/p2 iam-res://*/**
get|iam-res://iam-serv/p1/p2/x -> 3 iam-res://iam-serv/p1/p2/* iam-res://iam-serv/p1/p2/** iam-res://*/**
get|iam-res://iam-serv/p1/p2/x/y -> 2 iam-res://iam-serv/p1/p2/** iam-res://*/**
get|iam-res://iam-serv/p1/p2/x/y/p5 -> 3 iam-res://iam-serv/p1/p2/** iam-res://iam-serv/**/p5 iam-res://*/**
get|iam-res://iam-serv/p1?q1=aa -> 2 iam-res://iam-serv/p1?q1=aa iam-res://*/**
get|iam-res://iam-serv/p1?Q2=bb&q1=aa -> 2 iam-res://iam-serv/p1?q1=aa&q2=bb iam-res://*/**
post|iam-res://iam-serv/p1?Q2=bb&q1=aa -> 1 iam-res://iam-serv/p1?q1=aa&q2=bb
get|iam-res://App1.Tenant1/p1?Q2=bb&q1=aa -> 1 iam-res://*/**
----
DEL:get|iam-res://iam-serv/**
get|iam-res://iam-serv -> 1
DEL:get|iam-res://iam-serv/
get|iam-res://iam-serv -> 0
get|iam-res://iam-serv/p2 -> 1 iam-res://*/**
DEL:get|iam-res://*/**
get|iam-res://iam-serv/p2 -> 0
get|iam-res://iam2/ss -> 0
get|iam-res://iam-serv/p1 -> 1 iam-res://iam-serv/p1
DEL:get|iam-res://iam-serv/p1
get|iam-res://iam-serv/p1 -> 0
DEL:get|iam-res://iam-serv/p1/p2
get|iam-res://iam-serv/p1/p2 -> 1 iam-res://iam-serv/*/p2
get|iam-res://iam-serv/p1/p2/x -> 2 iam-res://iam-serv/p1/p2/* iam-res://iam-serv/p1/p2/**
DEL:get|iam-res://iam-serv/p1/p2/*
get|iam-res://iam-serv/p1/p2/x -> 1 iam-res://iam-serv/p1/p2/**
get|iam-res://iam-serv/p1/p2/x/y -> 1 iam-res://iam-serv/p1/p2/**
DEL:get|iam-res://iam-serv/p1/p2/**
get|iam-res://iam-serv/p1/p2/x -> 0
get|iam-res://iam-serv/p1/p2/x/y -> 0
get|iam-res://iam-serv/p1/p2/p5 -> 1 iam-res://iam-serv/**/p5
DEL:get|iam-res://iam-serv/**/p5
get|iam-res://iam-serv/p1/p2/p5 -> 0
DEL:get|iam-res://iam-serv/p1?q1=aa
get|iam-res://iam-serv/p1?q1=aa -> 0
{}
--- no_error_log
[error]

