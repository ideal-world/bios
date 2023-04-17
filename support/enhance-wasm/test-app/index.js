import * as bios from "bios-enhance-wasm";

async function doInit(strictSecurityMode) {
    await bios.init('', {
        "strict_security_mode": strictSecurityMode,
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
                "uri": "iam/cs/**",
                "need_crypto_req": true,
                "need_crypto_resp": true,
                "need_double_auth": false
            },
            {
                "action": "get",
                "uri": "iam/ct/need_double_auth/**",
                "need_crypto_req": false,
                "need_crypto_resp": false,
                "need_double_auth": true
            }],
        "login_req_method": "put",
        "login_req_paths": ["iam/cp/login/userpwd"],
        "logout_req_method": "delete",
        "logout_req_path": "iam/cp/logout/",
        "double_auth_req_method": "put",
        "double_auth_req_path": "iam/cp/login/check",
    });
}

// 1. Init
document.getElementById("InitNonSecurityMode").addEventListener("click", async () => {
    await doInit(false)
    document.getElementById("mode").innerText = "-------------------- Non Strict Security Mode --------------------"
});

document.getElementById("InitSecurityMode").addEventListener("click", async () => {
    await doInit(true)
    document.getElementById("mode").innerText = "-------------------- Strict Security Mode --------------------"
});

// 2. Browser uri crypto example
document.getElementById("uriEncrypt").addEventListener("click", async () => {
    document.getElementById("uriExample").value = bios.encrypt(document.getElementById("uriExample").value);
});

document.getElementById("uriDecrypt").addEventListener("click", async () => {
    document.getElementById("uriExample").value = bios.decrypt(document.getElementById("uriExample").value);
});

// 3. API transmission crypto example
document.getElementById("apiNonCrypto").addEventListener("click", () => {
    const body = document.getElementById("bodyExample").value;
    const headers = { "X-Key": "k001" };
    const encryptRequest = bios.on_before_request("post", "iam/cc/xxxx", body, headers)
    document.getElementById("output").innerHTML = "Request:<br/>POST:iam/cc/xxxx<br/>Headers:" + JSON.stringify(headers) + "<br/>Body:" + body + "<br/>------------------------<br/>Encrypt:<br/>" + JSON.stringify(encryptRequest);
});

document.getElementById("apiReqCrypto").addEventListener("click", () => {
    const body = document.getElementById("bodyExample").value;
    const headers = { "X-Key": "k001" };
    const encryptRequest = bios.on_before_request("get", "iam/ct/need_crypto_req/xxxx", body, headers)
    document.getElementById("output").innerHTML = "Request:<br/>GET:iam/ct/need_crypto_req/xxxx<br/>Headers:" + JSON.stringify(headers) + "<br/>Body:" + body + "<br/>------------------------<br/>Encrypt:<br/>" + JSON.stringify(encryptRequest);
});

document.getElementById("apiRespCrypto").addEventListener("click", () => {
    const body = document.getElementById("bodyExample").value;
    const headers = { "X-Key": "k001" };
    const encryptRequest = bios.on_before_request("get", "iam/ct/need_crypto_resp/xxxx", body, headers)
    document.getElementById("output").innerHTML = "Request:<br/>GET:iam/ct/need_crypto_resp/xxxx<br/>Headers:" + JSON.stringify(headers) + "<br/>Body:" + body + "<br/>------------------------<br/>Encrypt:<br/>" + JSON.stringify(encryptRequest);
});

document.getElementById("apiReqRespCrypto").addEventListener("click", () => {
    const body = document.getElementById("bodyExample").value;
    const headers = { "X-Key": "k001" };
    const encryptRequest = bios.on_before_request("get", "iam/cs/xxxx", body, headers)
    document.getElementById("output").innerHTML = "Request:<br/>GET:iam/cs/xxxx<br/>Headers:" + JSON.stringify(headers) + "<br/>Body:" + body + "<br/>------------------------<br/>Encrypt:<br/>" + JSON.stringify(encryptRequest);
});

// 4. Login example
document.getElementById("login").addEventListener("click", async () => {
    bios.on_response_success("put", "iam/cp/login/userpwd", { "token": "t0001", "account": "a0000", "roles": ["admin"] });
    console.log("mock login success");
});

document.getElementById("logout").addEventListener("click", async () => {
    bios.on_response_success("delete", "iam/cp/logout/", {});
    console.log("mock logout success");
});

// 5. Double Auth example
document.getElementById("needDoubleAuth").addEventListener("click", async () => {
    const body = document.getElementById("bodyExample").value;
    const headers = { "X-Key": "k001" };
    try {
        let encryptRequest = bios.on_before_request("get", "iam/ct/need_double_auth/111", body, headers)
        document.getElementById("output").innerHTML = "Request:<br/>GET:iam/ct/need_double_auth/11<br/>Headers:" + JSON.stringify(headers) + "<br/>Body:" + body + "<br/>------------------------<br/>Encrypt:<br/>" + JSON.stringify(encryptRequest);
    } catch (e) {
        document.getElementById("output").innerHTML = "Request:<br/>GET:iam/ct/need_double_auth/11<br/>Headers:" + JSON.stringify(headers) + "<br/>Body:" + body + "<br/>------------------------<br/>Encrypt:<br/>" + e;
    }
});
document.getElementById("doubleAuth").addEventListener("click", async () => {
    bios.on_response_success("put", "iam/cp/login/check", {});
    console.log("mock double auth success");
});


// 6. Crypto performance test
document.getElementById("performanceTest").addEventListener("click", async () => {
    const body = document.getElementById("bodyExample").value;
    const headers = { "X-Key": "k001" };
    const start = new Date().getTime();
    Array.from(Array(100)).forEach((x) => {
        bios.on_before_request("get", "iam/cs/xxxx", body + x, headers)
    });
    const averageTime = (new Date().getTime() - start) / 100;
    document.getElementById("output").innerHTML = "Request:<br/>GET:iam/ct/need_double_auth/11<br/>Headers:" + JSON.stringify(headers) + "<br/>Body:" + body + "<br/>------------------------<br/>Average Time(ms):<br/>" + averageTime;
});