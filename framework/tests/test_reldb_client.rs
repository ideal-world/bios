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

// https://github.com/rbatis/rbatis
// https://github.com/rbatis/rbatis/blob/master/example/src/crud_test.rs

#[macro_use]
extern crate rbatis;

use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use rbatis::core::value::DateTimeNow;
use rbatis::crud::{CRUDMut, CRUD};
use rbatis::executor::Executor;
use rbatis::plugin::page::{Page, PageRequest};

use bios_framework::basic::error::BIOSResult;
use bios_framework::basic::logger::BIOSLogger;
use bios_framework::db::reldb_client::BIOSRelDBClient;
use bios_framework::db::reldb_client::BIOSDB;
use bios_framework::test::test_container::BIOSTestContainer;

#[crud_table(table_name:biz_activity)]
#[derive(Clone, Debug)]
struct BizActivity {
    pub id: Option<String>,
    pub name: Option<String>,
    pub pc_link: Option<String>,
    pub h5_link: Option<String>,
    pub pc_banner_img: Option<String>,
    pub h5_banner_img: Option<String>,
    pub sort: Option<String>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub create_time: Option<NaiveDateTime>,
    pub version: Option<BigDecimal>,
    pub delete_flag: Option<i32>,
}

#[tokio::test]
async fn test_reldb_client() -> BIOSResult<()> {
    BIOSLogger::init("").unwrap();
    BIOSTestContainer::mysql(|url| async move {
        BIOSRelDBClient::init(&url, 10).await?;

        BIOSDB
            .exec(
                r#"
CREATE TABLE `biz_activity` (
  `id` varchar(50) NOT NULL DEFAULT '' COMMENT '唯一活动码',
  `name` varchar(255) NOT NULL,
  `pc_link` varchar(255) DEFAULT NULL,
  `h5_link` varchar(255) DEFAULT NULL,
  `sort` varchar(255) NOT NULL COMMENT '排序',
  `status` int(11) NOT NULL COMMENT '状态（0：已下线，1：已上线）',
  `version` int(11) NOT NULL,
  `remark` varchar(255) DEFAULT NULL,
  `create_time` datetime NOT NULL,
  `delete_flag` int(1) NOT NULL,
  `pc_banner_img` varchar(255) DEFAULT NULL,
  `h5_banner_img` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`id`) USING BTREE
) ENGINE=InnoDB DEFAULT CHARSET=utf8  COMMENT='运营管理-活动管理';
    "#,
                &vec![],
            )
            .await?;

        BIOSDB
            .save(&BizActivity {
                id: Some("1".to_string()),
                name: Some("测试".to_string()),
                pc_link: None,
                h5_link: None,
                pc_banner_img: None,
                h5_banner_img: None,
                sort: Some("1".to_string()),
                status: Some(1),
                remark: None,
                create_time: Some(NaiveDateTime::now()),
                version: Some(BigDecimal::from(1)),
                delete_flag: Some(1),
            })
            .await?;

        BIOSDB
            .save_batch(&vec![BizActivity {
                id: Some("2".to_string()),
                name: Some("测试".to_string()),
                pc_link: Some("http://xxxx".to_string()),
                h5_link: None,
                pc_banner_img: None,
                h5_banner_img: None,
                sort: Some("1".to_string()),
                status: Some(1),
                remark: None,
                create_time: Some(NaiveDateTime::now()),
                version: Some(BigDecimal::from(1)),
                delete_flag: Some(1),
            }])
            .await?;

        let result_opt: Option<BizActivity> =
            BIOSDB.fetch_by_column("id", &"0".to_string()).await?;
        assert!(result_opt.is_none());

        let result_opt: Option<BizActivity> =
            BIOSDB.fetch_by_column("id", &"1".to_string()).await?;
        assert_eq!(result_opt.unwrap().name.unwrap(), "测试");

        let result_list: Vec<BizActivity> = BIOSDB.fetch_list().await?;
        assert_eq!(result_list.len(), 2);

        let result_list: Vec<BizActivity> = BIOSDB
            .fetch_list_by_column("id", &["1".to_string()])
            .await?;
        assert_eq!(result_list.len(), 1);

        let result_opt: Option<BizActivity> = BIOSDB
            .fetch_by_wrapper(&BIOSDB.new_wrapper().eq("id", "1").and().eq("name", "测试"))
            .await?;
        assert_eq!(result_opt.unwrap().name.unwrap(), "测试");

        BIOSDB
            .remove_by_column::<BizActivity, _>("id", &"1".to_string())
            .await?;

        BIOSDB
            .update_by_wrapper(
                &mut BizActivity {
                    id: Some("2".to_string()),
                    name: Some("测试2".to_string()),
                    pc_link: None,
                    h5_link: None,
                    pc_banner_img: None,
                    h5_banner_img: None,
                    sort: Some("1".to_string()),
                    status: Some(1),
                    remark: None,
                    create_time: Some(NaiveDateTime::now()),
                    version: Some(BigDecimal::from(1)),
                    delete_flag: Some(1),
                },
                &BIOSDB.new_wrapper().eq("id", "2"),
                true,
            )
            .await?;
        let result_opt: Option<BizActivity> = BIOSDB
            .fetch_by_wrapper(&BIOSDB.new_wrapper().eq("id", "2"))
            .await?;
        assert_eq!(result_opt.as_ref().unwrap().name.as_ref().unwrap(), "测试2");
        assert_eq!(
            result_opt.as_ref().unwrap().pc_link.as_ref().unwrap(),
            "http://xxxx"
        );

        BIOSDB
            .update_by_wrapper(
                &mut BizActivity {
                    id: Some("2".to_string()),
                    name: Some("测试2".to_string()),
                    pc_link: None,
                    h5_link: None,
                    pc_banner_img: None,
                    h5_banner_img: None,
                    sort: Some("1".to_string()),
                    status: Some(1),
                    remark: None,
                    create_time: Some(NaiveDateTime::now()),
                    version: Some(BigDecimal::from(1)),
                    delete_flag: Some(1),
                },
                &BIOSDB.new_wrapper().eq("id", "2"),
                false,
            )
            .await?;
        let result_opt: Option<BizActivity> = BIOSDB
            .fetch_by_wrapper(&BIOSDB.new_wrapper().eq("id", "2"))
            .await?;
        assert_eq!(result_opt.as_ref().unwrap().name.as_ref().unwrap(), "测试2");
        assert!(result_opt.as_ref().unwrap().pc_link.as_ref().is_none());

        // paging

        let result_page: Page<BizActivity> = BIOSDB
            .fetch_page_by_wrapper(&BIOSDB.new_wrapper(), &PageRequest::new(1, 20))
            .await?;
        assert_eq!(result_page.page_no, 1);
        assert_eq!(result_page.page_size, 20);
        assert_eq!(result_page.pages, 1);
        assert_eq!(result_page.total, 1);
        assert_eq!(result_page.records.len(), 1);
        assert_eq!(
            result_page.records.get(0).unwrap().name.as_ref().unwrap(),
            "测试2"
        );

        // TX

        let mut tx = BIOSDB.acquire_begin().await?;
        tx.update_by_wrapper(
            &mut BizActivity {
                id: Some("2".to_string()),
                name: Some("测试3".to_string()),
                pc_link: None,
                h5_link: None,
                pc_banner_img: None,
                h5_banner_img: None,
                sort: Some("1".to_string()),
                status: Some(1),
                remark: None,
                create_time: Some(NaiveDateTime::now()),
                version: Some(BigDecimal::from(1)),
                delete_flag: Some(1),
            },
            &BIOSDB.new_wrapper().eq("id", "2"),
            vec![""],
        )
        .await?;
        tx.commit().await?;
        let result_opt: Option<BizActivity> = BIOSDB
            .fetch_by_wrapper(&BIOSDB.new_wrapper().eq("id", "2"))
            .await?;
        assert_eq!(result_opt.unwrap().name.unwrap(), "测试3");
        Ok(())
    })
    .await
}
