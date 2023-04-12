import * as bios from "bios-enhance-wasm";


// pri key : 4a0d660b70a8ee0a46c6ebf8482b158d321e59fab2d15c3fdd89ddaea24144aa
// pub key : 02fbba662032fd384079b7824c07ec8eeaac615187e27ce6a58fcd1597105c1065
await bios.init('', {
    "strict_security_mode": true,
    "pub_key": "02fbba662032fd384079b7824c07ec8eeaac615187e27ce6a58fcd1597105c1065",
    "double_auth_exp_sec": 60,
    "apis": [
        {
            "action": "get",
            "uri": "iam/ct/need_crypto_req/**",
            "need_crypto_req": true,
            "need_crypto_resp": false,
            "need_double_auth": false
        },
        {
            "action": "get",
            "uri": "iam/ct/need_crypto_resp/**",
            "need_crypto_req": false,
            "need_crypto_resp": true,
            "need_double_auth": false
        },
        {
            "action": "get",
            "uri": "iam/ct/need_double_auth/**",
            "need_crypto_req": false,
            "need_crypto_resp": false,
            "need_double_auth": true
        },
        {
            "action": "get",
            "uri": "iam/cs/**",
            "need_crypto_req": true,
            "need_crypto_resp": true,
            "need_double_auth": false
        }],
    "login_req_method": "put",
    "login_req_paths": ["iam/cp/login/userpwd"],
    "logout_req_method": "delete",
    "logout_req_path": "iam/cp/logout/",
    "double_auth_req_method": "put",
    "double_auth_req_path": "iam/cp/login/check",
});

// simple crypto
const raw_uri = "/iam/cs/xxxx?id=123";
const encrypt_uri = bios.encrypt(raw_uri);
console.log("simple encrypt: " + encrypt_uri);
console.log("simple decrypt: " + bios.decrypt(encrypt_uri));

// request process
console.log("no need encrypt = " + bios.on_before_request("post", "iam/cc/xxxx", "hello world", {}));
console.log("request & response need encrypt = " + JSON.stringify(bios.on_before_request("get", "iam/cs/xxxx", "hello world", {})));
console.log("request need encrypt = " + JSON.stringify(bios.on_before_request("get", "iam/ct/need_crypto_req/xxxx", "hello world", {})));
console.log("response need encrypt = " + JSON.stringify(bios.on_before_request("get", "iam/ct/need_crypto_resp/xxxx", "hello world", {})));

// ------------------------

const mock_req_body = "中台经过几年“滚雪球”的发展或是资本地运作，已是个“庞然大物”，是到了“减肥”，“减负”的时候。一言以避之：解构中台，让他融合到更大的IT能力共享架构中，把共享交给开放平台，把技术还给技术平台，让中台专注于领域服务及事件 。";

const start = new Date().getTime();
Array.from(Array(100)).forEach((x, i) => {
    JSON.stringify(bios.on_before_request("post", "iam/cs/xxxx", mock_req_body, {}))
});
console.log("average time (ms) :" + (new Date().getTime() - start) / 100);

document.getElementById("mockLogin").addEventListener("click", () => {
    bios.on_response_success("put", "iam/cp/login/userpwd", { "token": "t0001" });
    console.log("mock login success");
});

document.getElementById("mockRequest").addEventListener("click", () => {
    console.log("mock request = " + JSON.stringify(bios.on_before_request("get", "iam/cs/xxxx", "hello world", {})));
});