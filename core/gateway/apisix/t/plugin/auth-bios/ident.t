use t::APISIX 'no_plan';

no_long_string();
no_root_location();
no_shuffle();
run_tests;
log_level('debug');

__DATA__

=== TEST Request is not legal, missing [domain] in path
--- config
    location /iam {
        content_by_lua_block {
            local m_ident = require("apisix.plugins.auth-bios.ident")
            local result,err = m_ident.ident({
                head_key_token="Bios-Token",
                head_key_app="Bios-App",
                head_key_protocol="Bios-Proto",
                cache_key_token_info="iam:cache:token:info:",
                cache_key_account_info="iam:cache:account:info:",
                cache_key_token_local_expire_sec=0
           },{})
           ngx.say(result)
           ngx.say(err.message)
        }
    }
--- request
GET /iam
--- response_body
400
Request is not legal, missing [domain] in path
--- no_error_log
[error]

=== TEST Token is not legal
--- config
    location /iam/ {
        content_by_lua_block {
            local m_ident = require("apisix.plugins.auth-bios.ident")
            local m_redis = require("apisix.plugins.auth-bios.redis")
            m_redis.init("127.0.0.1", 6379, 1, 1000, "123456")
            ngx.req.set_header("Bios-Token", "aaaa")
            local result,err = m_ident.ident({
                head_key_token="Bios-Token",
                head_key_app="Bios-App",
                head_key_protocol="Bios-Proto",
                cache_key_token_info="iam:cache:token:info:",
                cache_key_account_info="iam:cache:account:info:",
                cache_key_token_local_expire_sec=0
           },{})
           ngx.say(result)
           ngx.say(err.message)
        }
    }
--- request
GET /iam/cp/account
--- response_body
401
Token [aaaa] is not legal
--- no_error_log
[error]

=== TEST Request Public
--- config
    location /iam/ {
        content_by_lua_block {
            local m_ident = require("apisix.plugins.auth-bios.ident")
            local ctx ={}
            local result,err = m_ident.ident({
                head_key_token="Bios-Token",
                head_key_app="Bios-App",
                head_key_protocol="Bios-Proto",
                cache_key_token_info="iam:cache:token:info:",
                cache_key_account_info="iam:cache:account:info:",
                cache_key_token_local_expire_sec=0
           },ctx)
           ngx.say(result)
           ngx.say(ctx.ident_info.rbum_uri)
           ngx.say(ctx.ident_info.rbum_action)
           ngx.say(ctx.ident_info.iam_app_id)
           ngx.say(ctx.ident_info.iam_tenant_id)
           ngx.say(ctx.ident_info.iam_account_id)
           ngx.say(ctx.ident_info.iam_roles)
           ngx.say(ctx.ident_info.iam_groups)
        }
    }
--- request
POST /iam/cp/login?p=xx
--- response_body
200
iam-res://iam/cp/login?p=xx
post
nil
nil
nil
nil
nil
--- no_error_log
[error]

=== TEST Request Token By System Account
--- config
    location /iam/ {
        content_by_lua_block {
            local m_ident = require("apisix.plugins.auth-bios.ident")
            local m_redis = require("apisix.plugins.auth-bios.redis")
            m_redis.init("127.0.0.1", 6379, 1, 1000, "123456")
            m_redis.set("iam:cache:token:info:tokenxxx", "default,accountxxx",0)
            m_redis.hset("iam:cache:account:info:accountxxx","",
                    "{\"own_paths\":\"\",\"owner\":\"account1\",\"roles\":[\"r001\"],\"groups\":[\"g001\"]}",0)
            local ctx ={}
            ngx.req.set_header("Bios-Token", "tokenxxx")
            local result,err = m_ident.ident({
                head_key_token="Bios-Token",
                head_key_app="Bios-App",
                head_key_protocol="Bios-Proto",
                cache_key_token_info="iam:cache:token:info:",
                cache_key_account_info="iam:cache:account:info:",
                cache_key_token_local_expire_sec=0
           },ctx)
           ngx.say(result)
           ngx.say(ctx.ident_info.rbum_uri)
           ngx.say(ctx.ident_info.rbum_action)
           ngx.say(ctx.ident_info.iam_app_id)
           ngx.say(ctx.ident_info.iam_tenant_id)
           ngx.say(ctx.ident_info.iam_account_id)
           ngx.say(ctx.ident_info.iam_roles[1])
           ngx.say(ctx.ident_info.iam_groups[1])
        }
    }
--- request
GET /iam/api/p1?bb=y&aa=x
--- response_body
200
iam-res://iam/api/p1?bb=y&aa=x
get


account1
r001
g001
--- no_error_log
[error]

=== TEST Request Token By Tenant Account
--- config
    location /iam/ {
        content_by_lua_block {
            local m_ident = require("apisix.plugins.auth-bios.ident")
            local m_redis = require("apisix.plugins.auth-bios.redis")
            m_redis.init("127.0.0.1", 6379, 1, 1000, "123456")
            m_redis.set("iam:cache:token:info:tokenxxx", "default,accountxxx",0)
            m_redis.hset("iam:cache:account:info:accountxxx","",
                    "{\"own_paths\":\"tenant1\",\"owner\":\"account1\",\"roles\":[\"r001\"],\"groups\":[\"g001\"]}",0)
            local ctx ={}
            ngx.req.set_header("Bios-Token", "tokenxxx")
            local result,err = m_ident.ident({
                head_key_token="Bios-Token",
                head_key_app="Bios-App",
                head_key_protocol="Bios-Proto",
                cache_key_token_info="iam:cache:token:info:",
                cache_key_account_info="iam:cache:account:info:",
                cache_key_token_local_expire_sec=0
           },ctx)
           ngx.say(result)
           ngx.say(ctx.ident_info.rbum_uri)
           ngx.say(ctx.ident_info.rbum_action)
           ngx.say(ctx.ident_info.iam_app_id)
           ngx.say(ctx.ident_info.iam_tenant_id)
           ngx.say(ctx.ident_info.iam_account_id)
           ngx.say(ctx.ident_info.iam_roles[1])
           ngx.say(ctx.ident_info.iam_groups[1])
        }
    }
--- request
GET /iam/api/p1?bb=y&aa=x
--- response_body
200
iam-res://iam/api/p1?bb=y&aa=x
get

tenant1
account1
r001
g001
--- no_error_log
[error]

=== TEST Request Token By App Account With Error
--- config
    location /iam/ {
        content_by_lua_block {
            local m_ident = require("apisix.plugins.auth-bios.ident")
            local m_redis = require("apisix.plugins.auth-bios.redis")
            m_redis.init("127.0.0.1", 6379, 1, 1000, "123456")
            m_redis.set("iam:cache:token:info:tokenxxx", "default,accountxxx",0)
            m_redis.hset("iam:cache:account:info:accountxxx","",
                    "{\"own_paths\":\"tenant1\",\"owner\":\"account1\",\"roles\":[\"r001\"],\"groups\":[\"g001\"]}",0)
            m_redis.hset("iam:cache:account:info:accountxxx","app1",
                    "{\"own_paths\":\"tenant1/app1\",\"owner\":\"account1\",\"roles\":[\"r001\"],\"groups\":[\"g001\"]}",0)
            local ctx ={}
            ngx.req.set_header("Bios-Token", "tokenxxx")
            ngx.req.set_header("Bios-App", "app2")
            local result,err = m_ident.ident({
                head_key_token="Bios-Token",
                head_key_app="Bios-App",
                head_key_protocol="Bios-Proto",
                cache_key_token_info="iam:cache:token:info:",
                cache_key_account_info="iam:cache:account:info:",
                cache_key_token_local_expire_sec=0
           },ctx)
           ngx.say(result)
           ngx.say(err.message)
        }
    }
--- request
GET /iam/api/p1?bb=y&aa=x
--- response_body
401
Token [tokenxxx] with App [app2] is not legal
--- no_error_log
[error]

=== TEST Request Token By App Account
--- config
    location /iam/ {
        content_by_lua_block {
            local m_ident = require("apisix.plugins.auth-bios.ident")
            local m_redis = require("apisix.plugins.auth-bios.redis")
            m_redis.init("127.0.0.1", 6379, 1, 1000, "123456")
            m_redis.set("iam:cache:token:info:tokenxxx", "default,accountxxx",0)
            m_redis.hset("iam:cache:account:info:accountxxx","",
                   "{\"own_paths\":\"tenant1\",\"owner\":\"account1\",\"roles\":[\"r001\"],\"groups\":[\"g001\"]}",0)
            m_redis.hset("iam:cache:account:info::accountxxx","app1",
                    "{\"own_paths\":\"tenant1/app1\",\"owner\":\"account1\",\"roles\":[\"r001\"],\"groups\":[\"g001\"]}",0)
            local ctx ={}
            ngx.req.set_header("Bios-Token", "tokenxxx")
            ngx.req.set_header("Bios-App", "app1")
            local result,err = m_ident.ident({
                head_key_token="Bios-Token",
                head_key_app="Bios-App",
                head_key_protocol="Bios-Proto",
                cache_key_token_info="iam:cache:token:info:",
                cache_key_account_info="iam:cache:account:info:",
                cache_key_token_local_expire_sec=0
           },ctx)
           ngx.say(result)
           ngx.say(ctx.ident_info.rbum_uri)
           ngx.say(ctx.ident_info.rbum_action)
           ngx.say(ctx.ident_info.iam_app_id)
           ngx.say(ctx.ident_info.iam_tenant_id)
           ngx.say(ctx.ident_info.iam_account_id)
           ngx.say(ctx.ident_info.iam_roles[1])
           ngx.say(ctx.ident_info.iam_groups[1])
        }
    }
--- request
GET /iam/api/p1
--- response_body
200
iam-res://iam/api/p1
get
app1
tenant1
account1
r001
g001
--- no_error_log
[error]
