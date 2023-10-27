import axios from 'axios';
import * as bios from "bios-enhance-wasm";

const apiClient = axios.create({
    headers: {
        'Content-type': 'application/json',
    },
})

apiClient.interceptors.request.use(function (config) {
    const url = new URL(config.url!)
    const path = url.pathname + (url.search != "" ? url.search : "")
    const body = typeof config.data === 'undefined' ? "" : config.data
    config.raw_method = config.method
    config.raw_path = path
    let mix_request
    try {
        mix_request = bios.on_before_request(typeof config.method === 'undefined' ? "get" : config.method, path, body, config.headers,false)
    } catch (e) {
        if (e.message == "Need double auth.") {
            return Promise.reject(e);
        }
    }
    config.method = mix_request.method
    config.url = url.protocol + "//" + url.host + mix_request.uri
    config.data = mix_request.body
    config.headers = mix_request.headers
    return config;
}, function (error) {
    return Promise.reject(error);
});

apiClient.interceptors.response.use(function (response) {
    console.log("======" + JSON.stringify(response))
    const url = new URL(response.config.url!)
    const path = url.pathname + (url.search != "" ? url.search : "")
    if (typeof response.data !== 'undefined' && response.data !== "") {
        const body = bios.on_before_response(response.data, response.headers)
        response.data = JSON.parse(body)
    }
    // TODO Check for success
    bios.on_response_success(typeof response.config.raw_method, response.config.raw_path, response.data)
    return response;
}, function (error) {
    return Promise.reject(error);
});

export default apiClient