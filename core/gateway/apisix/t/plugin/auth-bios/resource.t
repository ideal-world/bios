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
            m_resource.add_res("GET","iam-res://iam-serv",{st=ngx.time(),et=ngx.time()+3600,app="#app1#app2#",tenant="#tenant1#"})
            m_resource.add_res("GET","iam-res://*/**",{st=ngx.time(),et=ngx.time()+3600,app="#app1#app2#",tenant="#tenant1#"})
            m_resource.add_res("GET","iam-res://iam-serv/p1",{st=ngx.time(),et=ngx.time()+3600,account="#acc#"})
            m_resource.add_res("GET","iam-res://iam-serv/p1/p2",{st=ngx.time(),et=ngx.time()+3600,role="#role1#"})
            m_resource.add_res("GET","iam-res://iam-serv/p1/p2/*",{st=ngx.time(),et=ngx.time()+3600,role="#role1#"})
            m_resource.add_res("GET","iam-res://iam-serv/p1/p2/**",{st=ngx.time(),et=ngx.time()+3600,role="#role1#"})
            m_resource.add_res("GET","iam-res://iam-serv/p4/p2/**",{st=ngx.time(),et=ngx.time()+3600,role="#role1#"})
            m_resource.add_res("GET","iam-res://iam-serv/*/p2",{st=ngx.time(),et=ngx.time()+3600,role="#role1#"})
            m_resource.add_res("GET","iam-res://iam-serv/**/p5",{st=ngx.time(),et=ngx.time()+3600,role="#role1#"})
            m_resource.add_res("GET","iam-res://iam-serv/p1?q1=aa",{st=ngx.time(),et=ngx.time()+3600,role="#role1#"})
            m_resource.add_res("GET","iam-res://iam-serv/p1?q1=aa&q2=bb",{st=ngx.time(),et=ngx.time()+3600,role="#role1#"})
            m_resource.add_res("POST","iam-res://iam-serv/p1?q1=aa&q2=bb",{st=ngx.time(),et=ngx.time()+3600,role="#role1#"})

            ngx.say("get|" .. "iam-res://iam-serv -> " .. m_resource.match_res("get","iam-res://iam-serv").uri)
            ngx.say("get|" .. "iam-res://iam-serv/iam2 ->")
            ngx.say(m_resource.match_res("get","iam-res://iam2"))
            ngx.say("get|" .. "iam-res://iam-serv/ss -> " .. m_resource.match_res("get","iam-res://iam-serv/ss").uri)
            ngx.say("get|" .. "iam-res://iam2/ss -> " .. m_resource.match_res("get","iam-res://iam2/ss").uri)
            ngx.say("get|" .. "iam-res://iam-serv/p2 -> " .. m_resource.match_res("get","iam-res://iam-serv/p2").uri)
            ngx.say("get|" .. "iam-res://iam-serv/p1 -> " .. m_resource.match_res("get","iam-res://iam-serv/p1").uri)
            ngx.say("get|" .. "iam-res://iam-serv/p1/p2 -> " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2").uri)
            ngx.say("get|" .. "iam-res://iam-serv/p1/p2/x -> " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2/x").uri)
            ngx.say("get|" .. "iam-res://iam-serv/p1/p2/x/y -> " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2/x/y").uri)
            ngx.say("get|" .. "iam-res://iam-serv/p1/p2/x/y/p5 -> " .. m_resource.match_res("get","iam-res://iam-serv/p1/p2/x/y/p5").uri)
            ngx.say("get|" .. "iam-res://iam-serv/p1?q1=aa -> " .. m_resource.match_res("get","iam-res://iam-serv/p1?q1=aa").uri)
            ngx.say("get|" .. "iam-res://App1.Tenant1/p1?Q2=bb&q1=aa -> " .. m_resource.match_res("get","iam-res://App1.Tenant1/p1?Q2=bb&q1=aa").uri)
            ngx.say("get|" .. "iam-res://iam-serv/p1?Q2=bb&q1=aa -> " .. m_resource.match_res("get","iam-res://iam-serv/p1?Q2=bb&q1=aa").uri)

            ngx.say("----")

            m_resource.remove_res("get","iam-res://iam-serv/**")
            ngx.say("1")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv").uri)

            m_resource.remove_res("get","iam-res://iam-serv")
            ngx.say("2")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv"))
            ngx.say("3")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv/p2").uri)

            m_resource.remove_res("get","iam-res://*/**")
            ngx.say("4")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv/p2"))

            m_resource.remove_res("get","iam-res://iam-serv/p1")
            ngx.say("5")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv/p1"))

            m_resource.remove_res("get","iam-res://iam-serv/p1/p2")
            ngx.say("6")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv/p1/p2").uri)
            ngx.say("7")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv/p1/p2/x").uri)

            m_resource.remove_res("get","iam-res://iam-serv/p1/p2/*")
            ngx.say("8")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv/p1/p2/x").uri)
            ngx.say("9")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv/p1/p2/x/y").uri)

            m_resource.remove_res("get","iam-res://iam-serv/p1/p2/**")
            ngx.say("10")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv/p1/p2/x"))
            ngx.say("11")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv/p1/p2/x/y"))
            ngx.say("12")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv/p1/p2/p5").uri)

            m_resource.remove_res("get","iam-res://iam-serv/p4/p2/**")
            m_resource.remove_res("get","iam-res://iam-serv/*/p2")
            ngx.say("13")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv/p1/p2"))

            m_resource.remove_res("get","iam-res://iam-serv/**/p5")
            ngx.say("14")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv/p1/p2/p5"))
            ngx.say("15")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv/p1?q1=aa").uri)

            m_resource.remove_res("get","iam-res://iam-serv/p1?q1=aa")
            ngx.say("16")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv/p1?q1=aa"))
            ngx.say("17")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv/p1?Q2=bb&q1=aa").uri)

            m_resource.remove_res("get","iam-res://iam-serv/p1?q2=bb&q1=aa")
            m_resource.remove_res("post","iam-res://iam-serv/p1?q2=bb&q1=aa")
            ngx.say("18")
            ngx.say(m_resource.match_res("get","iam-res://iam-serv/p1?Q2=bb&q1=aa"))
            ngx.say(json.encode(m_resource.get_res()))
        }
    }
--- request
GET /t
--- response_body
get|iam-res://iam-serv -> iam-res://iam-serv
get|iam-res://iam-serv/iam2 ->
nil
get|iam-res://iam-serv/ss -> iam-res://*/**
get|iam-res://iam2/ss -> iam-res://*/**
get|iam-res://iam-serv/p2 -> iam-res://*/**
get|iam-res://iam-serv/p1 -> iam-res://iam-serv/p1
get|iam-res://iam-serv/p1/p2 -> iam-res://iam-serv/p1/p2
get|iam-res://iam-serv/p1/p2/x -> iam-res://iam-serv/p1/p2/*
get|iam-res://iam-serv/p1/p2/x/y -> iam-res://iam-serv/p1/p2/**
get|iam-res://iam-serv/p1/p2/x/y/p5 -> iam-res://iam-serv/p1/p2/**
get|iam-res://iam-serv/p1?q1=aa -> iam-res://iam-serv/p1?q1=aa
get|iam-res://App1.Tenant1/p1?Q2=bb&q1=aa -> iam-res://*/**
get|iam-res://iam-serv/p1?Q2=bb&q1=aa -> iam-res://iam-serv/p1?q1=aa&q2=bb
----
1
iam-res://iam-serv
2
nil
3
iam-res://*/**
4
nil
5
nil
6
iam-res://iam-serv/*/p2
7
iam-res://iam-serv/p1/p2/*
8
iam-res://iam-serv/p1/p2/**
9
iam-res://iam-serv/p1/p2/**
10
nil
11
nil
12
iam-res://iam-serv/**/p5
13
nil
14
nil
15
iam-res://iam-serv/p1?q1=aa
16
nil
17
iam-res://iam-serv/p1?q1=aa&q2=bb
18
nil
{}
--- no_error_log
[error]

