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
            local m_init = require("apisix.plugins.auth-bios.init")
            local m_resource = require("apisix.plugins.auth-bios.resource")
            local m_redis = require("apisix.plugins.auth-bios.redis")
            local m_utils = require("apisix.plugins.auth-bios.utils")

            m_redis.init("127.0.0.1", 6379, 1, 1000, "123456")
            m_redis.hset("iam:res:info","iam-res://iam-serv/p1?a=1##get","{\"st\":"..ngx.time()..",\"et\":"..(ngx.time()+3600)..",\"accounts\":\"#acc1#\"}")
            m_redis.hset("iam:res:info","iam-res://iam-serv/p1?a=2##get","{\"st\":"..ngx.time()..",\"et\":"..(ngx.time()+3600)..",\"accounts\":\"#acc2#\"}")
            m_redis.hset("iam:res:info","iam-res://iam-serv/p1?a=3##get","{\"st\":"..ngx.time()..",\"et\":"..(ngx.time()+3600)..",\"accounts\":\"#acc3#\"}")
            m_redis.hset("iam:res:info","iam-res://iam-serv/p1?a=4##get","{\"st\":"..ngx.time()..",\"et\":"..(ngx.time()+3600)..",\"accounts\":\"#acc4#\"}")
            m_redis.hset("iam:res:info","iam-res://iam-serv/p1?a=5##get","{\"st\":"..ngx.time()..",\"et\":"..(ngx.time()+3600)..",\"accounts\":\"#acc5#\"}")

            m_init.init("iam:res:info","iam:res:changed:info:",1)

            local resources = m_resource.get_res()
            ngx.say(resources["iam-res"]["iam-serv"]["p1"]["?"]["a=1"]["$"]["get"]["uri"])
            ngx.say(resources["iam-res"]["iam-serv"]["p1"]["?"]["a=5"]["$"]["get"]["uri"])

            m_redis.set("iam:res:changed:info:xx","iam-res://iam-serv/p1?a=1##get")
            m_redis.set("iam:res:changed:info:yy","iam-res://iam-serv/p1?a=6##get")
            m_redis.set("iam:res:changed:info:zz","iam-res://iam-serv/p1?a=7##get")
            m_redis.hdel("iam:res:info","iam-res://iam-serv/p1?a=1##get")
            m_redis.hset("iam:res:info","iam-res://iam-serv/p1?a=6##get","{\"st\":"..ngx.time()..",\"et\":"..(ngx.time()+3600)..",\"accounts\":\"#acc6#\"}")
            m_redis.hset("iam:res:info","iam-res://iam-serv/p1?a=7##get","{\"st\":"..ngx.time()..",\"et\":"..(ngx.time()+3600)..",\"accounts\":\"#acc7#\"}")

            ngx.sleep(2)

            ngx.say(resources["iam-res"]["iam-serv"]["p1"]["?"]["a=1"])
            ngx.say(resources["iam-res"]["iam-serv"]["p1"]["?"]["a=6"]["$"]["get"]["uri"])
            ngx.say(resources["iam-res"]["iam-serv"]["p1"]["?"]["a=7"]["$"]["get"]["uri"])
        }
    }
--- request
GET /t
--- response_body
iam-res://iam-serv/p1?a=1
iam-res://iam-serv/p1?a=5
nil
iam-res://iam-serv/p1?a=6
iam-res://iam-serv/p1?a=7
--- no_error_log
[error]

