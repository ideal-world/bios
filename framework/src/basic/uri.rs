/*
 * Copyright 2021. gudaoxuri
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::basic::error::{BIOSError, BIOSResult};

pub fn format(uri_str: &str) -> BIOSResult<String> {
    let uri_result = url::Url::parse(uri_str);
    if uri_result.is_err() {
        return Err(BIOSError::FormatError(
            uri_result.err().unwrap().to_string(),
        ));
    }
    let uri = uri_result.unwrap();
    if uri.host().is_none() {
        // E.g. jdbc:h2:men:iam 不用解析
        return Ok(uri.to_string());
    }
    let query = sort_query(uri.query());
    let path = if uri.path().is_empty() {
        ""
    } else if uri.path().ends_with("/") {
        &uri.path()[..uri.path().len() - 1]
    } else {
        uri.path()
    };
    let port = if uri.port().is_none() {
        "".to_string()
    } else {
        format!(":{}", uri.port().unwrap())
    };
    let query = if uri.query().is_none() {
        "".to_string()
    } else {
        format!("?{}", query)
    };
    let formatted_uri = format!(
        "{}://{}{}{}{}",
        uri.scheme(),
        uri.host().unwrap(),
        port,
        path,
        query
    );
    Ok(formatted_uri)
}

fn sort_query(query: Option<&str>) -> String {
    if query.is_none() {
        return "".to_owned();
    }
    let mut query = query.unwrap().split("&").collect::<Vec<&str>>();
    query.sort_by(|a, b| Ord::cmp(a.split("=").next().unwrap(), b.split("=").next().unwrap()));
    query.join("&")
}
