#!/bin/bash

#
# Copyright 2022. gudaoxuri
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#

set -o pipefail
set -u


init_dev_env(){
    echo "------------------"
    echo "Initializing the development environment"
    echo "------------------"
    echo "[1/10] add openresty source"
    echo ""
    wget -qO - https://openresty.org/package/pubkey.gpg | sudo apt-key add -
    sudo apt-get update
    sudo apt-get -y install software-properties-common
    sudo add-apt-repository -y "deb http://openresty.org/package/ubuntu $(lsb_release -sc) main"
    sudo apt-get update
    echo "[2/10] install lua openresty and dev tools"
    echo ""
    # Add libpcre3-dev to repair https://github.com/Kong/kong/issues/23
    sudo apt-get -y install git curl liblua5.1-0-dev openresty openresty-openssl111-dev cpanminus libpcre3-dev
    echo "[3/10] install etcd"
    echo ""
    wget https://github.com/etcd-io/etcd/releases/download/v3.4.13/etcd-v3.4.13-linux-amd64.tar.gz
    tar -xvf etcd-v3.4.13-linux-amd64.tar.gz
    rm etcd-v3.4.13-linux-amd64.tar.gz
    mv etcd-v3.4.13-linux-amd64 etcd
    cd etcd
    sudo cp -a etcd etcdctl /usr/bin/
    cd ..
    echo "[4/10] start etcd"
    echo ""
    nohup etcd </dev/null >/dev/null 2>&1 &
    echo "[5/10] install LuaRocks"
    echo ""
    cd utils
    ./linux-install-luarocks.sh
    cd ..
    echo "[6/10] download apisix"
    echo ""
    wget https://mirrors.bfsu.edu.cn/apache/apisix/2.10.0/apache-apisix-2.10.0-src.tgz
    tar -cvf apisix.tar apisix
    tar -xf apache-apisix-2.10.0-src.tgz -C apisix
    tar -xf apisix.tar
    rm apisix.tar
    rm apache-apisix-2.10.0-src.tgz
    echo "[7/10] make deps"
    echo ""
    cd apisix
    make deps
    echo "[8/10] clone test-nginx"
    echo ""
    git clone --depth=1 https://github.com/iresty/test-nginx.git
    rm -rf test-nginx/.git
    echo "[9/10] install test-nginx deps"
    echo ""
    sudo cpanm --notest Test::Nginx IPC::Run > build.log 2>&1 || (cat build.log && exit 1)
    export PERL5LIB=.:$PERL5LIB
    echo "[10/10] test"
    echo ""
    TEST_NGINX_BINARY=/usr/bin/openresty prove -Itest-nginx/lib -r t/plugin/auth-bios/utils.t
    echo "------------------"
    echo "Initialization completed"
    echo "------------------"
}

init_dev_env