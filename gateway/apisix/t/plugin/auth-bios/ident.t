use t::APISIX 'no_plan';

no_long_string();
no_root_location();
no_shuffle();
run_tests;
log_level('debug');

__DATA__

=== TEST 1: request missing BIOS-Host
--- config
    location /api/ {
        content_by_lua_block {
            local m_ident = require("apisix.plugins.auth-bios.ident")
            local ctx ={
                headers={
                }
            }
            local result,err = m_ident.ident({
                token_flag="BIOS-Token",
                auth_flag="Authorization",
                date_flag="BIOS-Date",
                host_flag="BIOS-Host",
                request_date_offset_ms=5000,
                cache_token="bios:iam:token:info:",
                cache_token_exp_sec=60,
                cache_aksk="bios:iam:app:aksk:",
                cache_aksk_exp_sec=60
           },ctx)
           ngx.say(result)
           ngx.say(err.message)
        }
    }
--- request
GET /api/p1
--- response_body
400
Request is not legal, missing [BIOS-Host] in Header
--- no_error_log
[error]

=== TEST 2: request public
--- config
    location /api/ {
        content_by_lua_block {
            local m_ident = require("apisix.plugins.auth-bios.ident")
            local ctx ={
                headers={
                  ["BIOS-Host"]="app1.tenant1"
                }
            }
            local result,err = m_ident.ident({
                token_flag="BIOS-Token",
                auth_flag="Authorization",
                date_flag="BIOS-Date",
                host_flag="BIOS-Host",
                request_date_offset_ms=5000,
                cache_token="bios:iam:token:info:",
                cache_token_exp_sec=60,
                cache_aksk="bios:iam:app:aksk:",
                cache_aksk_exp_sec=60
           },ctx)
           ngx.say(result)
        }
    }
--- request
GET /api/p1?a=x
--- response_body
200
--- no_error_log
[error]

=== TEST 3: request token error
--- config
    location /api/ {
        content_by_lua_block {
            local m_ident = require("apisix.plugins.auth-bios.ident")
            local m_redis = require("apisix.plugins.auth-bios.redis")
            m_redis.init("127.0.0.1", 6379, 1, 1000, "123456")
            local ctx ={
                headers={
                    ["BIOS-Host"]="app1.tenant1",
                    ["BIOS-Token"]="token123",
                }
            }
            local result,err = m_ident.ident({
                token_flag="BIOS-Token",
                auth_flag="Authorization",
                date_flag="BIOS-Date",
                host_flag="BIOS-Host",
                request_date_offset_ms=5000,
                cache_token="bios:iam:token:info:",
                cache_token_exp_sec=60,
                cache_aksk="bios:iam:app:aksk:",
                cache_aksk_exp_sec=60
           },ctx)
           ngx.say(result)
           ngx.say(err.message)
        }
    }
--- request
GET /api/p1
--- response_body
401
Token [token123] is not legal
--- no_error_log
[error]

=== TEST 4: request token success
--- config
    location /api/ {
        content_by_lua_block {
            local m_ident = require("apisix.plugins.auth-bios.ident")
            local m_redis = require("apisix.plugins.auth-bios.redis")
            m_redis.init("127.0.0.1", 6379, 1, 1000, "123456")
            m_redis.set("bios:iam:token:info:tokenxxx",
               "{\"app_id\":\"app1\",\"tenant_id\":\"tenant1\",\"account_id\":\"account1\",\"token_kind\":\"default\",\"roles\":[\"r001\"],\"groups\":[\"g001\"]}",0)
            local ctx ={
                headers={
                    ["BIOS-Host"]="app1.tenant1",
                    ["BIOS-Token"]="tokenxxx",
                }
            }
            local result,err = m_ident.ident({
                token_flag="BIOS-Token",
                auth_flag="Authorization",
                date_flag="BIOS-Date",
                host_flag="BIOS-Host",
                request_date_offset_ms=5000,
                cache_token="bios:iam:token:info:",
                cache_token_exp_sec=60,
                cache_aksk="bios:iam:app:aksk:",
                cache_aksk_exp_sec=60
           },ctx)
           ngx.say(result)
           ngx.say(ctx.ident_info.tenant_id)
           ngx.say(ctx.ident_info.account_id)
           ngx.say(ctx.ident_info.roles[1])
           ngx.say(ctx.ident_info.groups[1])
        }
    }
--- request
GET /api/p1?bb=y&aa=x
--- response_body
200
tenant1
account1
r001
g001
--- no_error_log
[error]

=== TEST 5: request auth missing
--- config
    location /api/ {
        content_by_lua_block {
            local m_ident = require("apisix.plugins.auth-bios.ident")
            local ctx ={
                headers={
                    ["BIOS-Host"]="app1.tenant1",
                    ["Authorization"]="xxxx:xxxx",
                }
            }
            local result,err = m_ident.ident({
                token_flag="BIOS-Token",
                auth_flag="Authorization",
                date_flag="BIOS-Date",
                host_flag="BIOS-Host",
                request_date_offset_ms=5000,
                cache_token="bios:iam:token:info:",
                cache_token_exp_sec=60,
                cache_aksk="bios:iam:app:aksk:",
                cache_aksk_exp_sec=60
           },ctx)
           ngx.say(result)
           ngx.say(err.message)
        }
    }
--- request
GET /api/p1?bb=y&aa=x
--- response_body
400
Request is not legal, missing [BIOS-Date]
--- no_error_log
[error]

=== TEST 6: request auth date error
--- config
    location /api/ {
        content_by_lua_block {
            local m_ident = require("apisix.plugins.auth-bios.ident")
            local ctx ={
                headers={
                    ["BIOS-Host"]="app1.tenant1",
                    ["Authorization"]="xxxx:xxxx",
                    ["BIOS-Date"]="Thu, 18 Nov 2021 11:27:3GMT",
                }
            }
            local result,err = m_ident.ident({
                token_flag="BIOS-Token",
                auth_flag="Authorization",
                date_flag="BIOS-Date",
                host_flag="BIOS-Host",
                request_date_offset_ms=5000,
                cache_token="bios:iam:token:info:",
                cache_token_exp_sec=60,
                cache_aksk="bios:iam:app:aksk:",
                cache_aksk_exp_sec=60
           },ctx)
           ngx.say(result)
           ngx.say(err.message)
        }
    }
--- request
GET /api/p1?bb=y&aa=x
--- response_body
400
Request Date [Thu, 18 Nov 2021 11:27:3GMT] is not legal
--- no_error_log
[error]

=== TEST 7: request auth date expire
--- config
    location /api/ {
        content_by_lua_block {
            local m_ident = require("apisix.plugins.auth-bios.ident")
            local test_date = ngx.http_time(ngx.time() - 10)
            local ctx ={
                headers={
                    ["BIOS-Host"]="app1.tenant1",
                    ["Authorization"]="xxxx:xxxx",
                    ["BIOS-Date"]=test_date,
                }
            }
            local result,err = m_ident.ident({
                token_flag="BIOS-Token",
                auth_flag="Authorization",
                date_flag="BIOS-Date",
                host_flag="BIOS-Host",
                request_date_offset_ms=5000,
                cache_token="bios:iam:token:info:",
                cache_token_exp_sec=60,
                cache_aksk="bios:iam:app:aksk:",
                cache_aksk_exp_sec=60
           },ctx)
           ngx.say(result)
           ngx.say(err.message)
        }
    }
--- request
GET /api/p1?bb=y&aa=x
--- response_body
400
Request has expired
--- no_error_log
[error]

=== TEST 8: request auth is not legal
--- config
    location /api/ {
        content_by_lua_block {
            local m_ident = require("apisix.plugins.auth-bios.ident")
            local m_redis = require("apisix.plugins.auth-bios.redis")
            m_redis.init("127.0.0.1", 6379, 1, 1000, "123456")
            local test_date = ngx.http_time(ngx.time())
            local ctx ={
                headers={
                    ["BIOS-Host"]="app1.tenant1",
                    ["Authorization"]="xxxx:xxxx",
                    ["BIOS-Date"]=test_date,
                }
            }
            local result,err = m_ident.ident({
                token_flag="BIOS-Token",
                auth_flag="Authorization",
                date_flag="BIOS-Date",
                host_flag="BIOS-Host",
                request_date_offset_ms=5000,
                cache_token="bios:iam:token:info:",
                cache_token_exp_sec=60,
                cache_aksk="bios:iam:app:aksk:",
                cache_aksk_exp_sec=60
           },ctx)
           ngx.say(result)
           ngx.say(err.message)
        }
    }
--- request
GET /api/p1?bb=y&aa=x
--- response_body
401
Authorization [xxxx:xxxx] is not legal
--- no_error_log
[error]

=== TEST 9: request auth is not legal
--- config
    location /api/ {
        content_by_lua_block {
            local m_ident = require("apisix.plugins.auth-bios.ident")
            local m_redis = require("apisix.plugins.auth-bios.redis")
            m_redis.init("127.0.0.1", 6379, 1, 1000, "123456")
            m_redis.set("bios:iam:app:aksk:ak01","sk01:tenant1:app1",0)
            local test_date = ngx.http_time(ngx.time())
            local ctx ={
                headers={
                    ["BIOS-Host"]="app1.tenant1",
                    ["Authorization"]="ak01:xxxx",
                    ["BIOS-Date"]=test_date,
                }
            }
            local result,err = m_ident.ident({
                token_flag="BIOS-Token",
                auth_flag="Authorization",
                date_flag="BIOS-Date",
                host_flag="BIOS-Host",
                request_date_offset_ms=5000,
                cache_token="bios:iam:token:info:",
                cache_token_exp_sec=60,
                cache_aksk="bios:iam:app:aksk:",
                cache_aksk_exp_sec=60
           },ctx)
           ngx.say(result)
           ngx.say(err.message)
        }
    }
--- request
GET /api/p1?bb=y&aa=x
--- response_body
401
Authorization [ak01:xxxx] is not legal
--- no_error_log
[error]

=== TEST 10: request auth success
--- config
    location /api/ {
        content_by_lua_block {
            local m_ident = require("apisix.plugins.auth-bios.ident")
            local m_redis = require("apisix.plugins.auth-bios.redis")
            local hmac_sha1 = ngx.hmac_sha1
            local ngx_encode_base64 = ngx.encode_base64
            m_redis.init("127.0.0.1", 6379, 1, 1000, "123456")
            m_redis.set("bios:iam:app:aksk:ak01","sk01:tenant1:app1",0)
            local test_date = ngx.http_time(ngx.time())
            local test_signature = ngx_encode_base64(hmac_sha1("sk01", string.lower("GET\n" .. test_date .. "\n/api/p1\naa=x&bb=y")))
            local ctx ={
                headers={
                    ["BIOS-Host"]="app1.tenant1",
                    ["Authorization"]="ak01:"..test_signature,
                    ["BIOS-Date"]=test_date,
                }
            }
            local result,err = m_ident.ident({
                token_flag="BIOS-Token",
                auth_flag="Authorization",
                date_flag="BIOS-Date",
                host_flag="BIOS-Host",
                request_date_offset_ms=5000,
                cache_token="bios:iam:token:info:",
                cache_token_exp_sec=60,
                cache_aksk="bios:iam:app:aksk:",
                cache_aksk_exp_sec=60
           },ctx)
           ngx.say(result)
           ngx.say(ctx.ident_info.tenant_id)
           ngx.say(ctx.ident_info.account_name)
           ngx.say(ctx.ident_info.roles)
           ngx.say(ctx.ident_info.groups)
        }
    }
--- request
GET /api/p1?bb=y&aa=x
--- response_body
200
tenant1
nil
nil
nil
--- no_error_log
[error]

