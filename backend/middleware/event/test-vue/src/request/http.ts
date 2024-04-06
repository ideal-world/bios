import axios from "axios";
// import * as bios from "bios-enhance-wasm";

const apiClient = axios.create({
  headers: {
    "Content-type": "application/json",
  },
});

apiClient.interceptors.request.use(
  function (config) {
    return config;
  },
  function (error) {
    return Promise.reject(error);
  }
);

apiClient.interceptors.response.use(
  function (response) {
    console.log("======" + JSON.stringify(response));
    const url = new URL(response.config.url!);
    const path = url.pathname + (url.search != "" ? url.search : "");
    if (typeof response.data !== "undefined" && response.data !== "") {
    }
    return response;
  },
  function (error) {
    return Promise.reject(error);
  }
);

export default apiClient;
