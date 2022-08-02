#!/bin/bash

#
# Copyright 2022. the original author or authors.
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
    echo "[1/9] install-dependencies"
    echo ""
    ./utils/install-dependencies.sh
    echo "[2/9] install LuaRocks"
    echo ""
    ./utils/linux-install-luarocks.sh
    echo "[3/9] install etcd"
    echo ""
    ./utils/linux-install-etcd.sh
    echo "[4/9] start etcd"
    echo ""
    nohup etcd </dev/null >/dev/null 2>&1 &
    echo "[5/9] download apisix"
    echo ""
    wget https://mirrors.bfsu.edu.cn/apache/apisix/2.14.1/apache-apisix-2.14.1-src.tgz
    tar -cvf apisix.tar apisix
    tar -xf apache-apisix-2.14.1-src.tgz -C apisix
    tar -xf apisix.tar
    rm apisix.tar
    rm apache-apisix-2.14.1-src.tgz
    echo "[6/9] make deps"
    echo ""
    cd apisix
    make deps
    echo "[7/9] clone test-nginx"
    echo ""
    git clone --depth=1 https://github.com/iresty/test-nginx.git
    rm -rf test-nginx/.git
    echo "[8/9] install test-nginx deps"
    echo ""
    sudo apt-get install -y cpanminus
    sudo cpanm --notest Test::Nginx IPC::Run > build.log 2>&1 || (cat build.log && exit 1)
    export PERL5LIB=.:$PERL5LIB
    echo "[9/9] test"
    echo ""
    TEST_NGINX_BINARY=/usr/bin/openresty prove -Itest-nginx/lib -r t/plugin/auth-bios/utils.t
    echo "------------------"
    echo "Initialization completed"
    echo "------------------"
}

init_dev_env