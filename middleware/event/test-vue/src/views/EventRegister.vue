<template>
  <div>
    <div>
      <input v-model="url" placeholder="req url" />
    </div>
    <div>
      <textarea
        v-model="registerFrom"
        @input="parseJson"
        placeholder="Enter json here"
      ></textarea>
      <div v-if="parsedData">
        <pre>{{ parsedData }}</pre>
      </div>
      <p v-if="error" style="color: red">{{ error }}</p>
    </div>
    <button @click="register()">register</button>
    <div style="width: 100%">
      <table>
        <thead>
          <tr>
            <th>index</th>
            <!-- <th>registerFrom</th> -->
            <th>token</th>
            <th>event</th>
            <th>avatars</th>
            <th>listener_code</th>
            <th>ws_addr</th>
            <th>send msg</th>
            <th>to_avatars</th>
            <th>event</th>
            <th>state</th>
            <th>receive message</th>
            <th>operate</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(soket, index) in socketList" :key="index">
            <td>{{ index + 1 }}</td>
            <!-- <td>{{ soket.registerFrom }}</td> -->
            <td>{{ soket.token }}</td>
            <td>{{ JSON.stringify(soket.registerFrom.event) }}</td>
            <td>{{ JSON.stringify(soket.registerFrom.avatars) }}</td>
            <td>{{ soket.listener_code }}</td>
            <td>{{ soket.ws_addr }}</td>
            <td>
              <input
                v-model="soket.content.msg"
                placeholder="Enter what you want to send"
              />
            </td>
            <td>
              <select
                id="fruitSelection"
                v-model="soket.content.to_avatars"
                multiple
              >
                <option
                  v-for="to_avatar in to_avatars"
                  :key="to_avatar"
                  :value="to_avatar"
                >
                  {{ to_avatar }}
                </option>
              </select>
            </td>
            <td>
              <input
                v-model="soket.content.event"
                placeholder="Enter what you want to send"
              />
            </td>
            <td>{{ soket.state }}</td>
            <td>
              <table>
                <thead>
                  <tr>
                    <th>index</th>
                    <th>msg</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(msg, r_index) in soket.msgs" :key="r_index">
                    <td>{{ r_index + 1 }}</td>
                    <td>{{ msg.data }}</td>
                  </tr>
                </tbody>
              </table>
            </td>
            <td>
              <div>
                <button @click="ws_init(index)">connect</button>
                <button @click="send(index)">send</button>
                <button @click="remove(index)">remove</button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script>
import http from "../request/http";
export default {
  data() {
    return {
      url: "http://127.0.0.1:8080/event/listener",
      registerFrom:
        '{"topic_code":"","topic_sk":"","events":[],"avatars":[],"subscribe_mode":false}',
      socketList: [],
      parsedData: null,
      error: null,
      result: null,
      content: "",
    };
  },
  mounted() {},
  methods: {
    register: async function () {
      const registerFrom = JSON.parse(this.registerFrom);
      var from_avatar = "_";
      if (registerFrom.avatars.length > 0) {
        from_avatar = registerFrom.avatars[0];
      }
      const response = await http.post(this.url, registerFrom);
      if (response.data.code == "200") {
        let match = response.data.data.ws_addr.match(/[?&]token=([^&]*)/);
        let token = match && match[1];
        this.socketList.push({
          registerFrom: registerFrom,
          socket: {},
          state: "unconnected",
          content: {
            msg: "",
            from_avatar: from_avatar,
            to_avatars: [],
            event: null,
            ignore_self: true,
            spec_inst_id: null,
          },
          token: token,
          msgs: [],
          ...response.data.data,
        });
      } else {
        alert(response.data.msg);
      }
    },
    parseJson() {
      try {
        this.parsedData = JSON.parse(this.registerFrom);
        this.error = null;
      } catch (e) {
        this.parsedData = null;
        this.error = "Invalid JSON!";
      }
    },
    ws_init: function (index) {
      if (this.socketList[index].state != "unconnected") {
        alert("The current status is unable to connect");
        return;
      }
      if (typeof WebSocket === "undefined") {
        alert("Your browser does not support Sockets");
      } else {
        this.socketList[index].socket = new WebSocket(
          this.socketList[index].ws_addr
        );
        const i = index;
        this.socketList[index].socket.onopen = this.open;
        this.socketList[index].socket.onerror = this.close;
        this.socketList[index].socket.onclose = this.close;
        this.socketList[index].socket.onmessage = this.getMessage;
        console.log(this.socketList[index]);
      }
    },
    open: function (msg) {
      let match = msg.currentTarget.url.match(/[?&]token=([^&]*)/);
      let token = match && match[1];
      let index = this.socketList.findIndex((socket) => socket.token === token);
      this.socketList[index].state = "connected";
    },
    error: function (msg) {
      let match = msg.currentTarget.url.match(/[?&]token=([^&]*)/);
      let token = match && match[1];
      let index = this.socketList.findIndex((socket) => socket.token === token);
      this.socketList[index].state = "error";
    },
    getMessage: function (msg) {
      let match = msg.currentTarget.url.match(/[?&]token=([^&]*)/);
      let token = match && match[1];
      let index = this.socketList.findIndex((socket) => socket.token === token);
      this.socketList[index].msgs.push({
        data: msg.data,
      });
    },
    send: function (index) {
      if (this.socketList[index].content.event == "") {
        this.socketList[index].content.event = null;
      }
      if (this.socketList[index].content.to_avatars.length == 0) {
        this.socketList[index].content.to_avatars = null;
      }
      this.socketList[index].socket.send(
        JSON.stringify(this.socketList[index].content)
      );
      this.socketList[index].content.to_avatars = [];
    },
    close: function (that) {
      console.log(that, "The socket is closed.");
    },
    remove: async function (index) {
      const response = await http.delete(
        this.url +
          "/" +
          this.socketList[index].listener_code +
          "?token=" +
          this.socketList[index].token
      );
      if (this.socketList[index].state == "connected") {
        this.socketList[index].socket.close();
      }
      this.socketList.splice(index, 1);
    },
  },
  computed: {
    to_avatars: function () {
      var list = this.socketList
        .filter((socket) => socket.state == "connected")
        .flatMap((socket) => socket.registerFrom.avatars);
      list.push("spi-log/server");
      return list;
    },
  },
  destroyed() {},
};
</script>
<style scoped>
input {
  width: 100%;
  height: 18px;
}
textarea {
  width: 100%;
  height: 200px;
}
</style>
