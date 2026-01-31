use std::collections::HashMap;

use bios_reach::{
    api::reach_api_ci::deduplicate_send_requests,
    dto::{ReachMsgReceive, ReachMsgSendReq},
    reach_constants::ReachReceiveKind,
};

/// 测试基本去重功能：rel_item_id和replace相同，receive_ids有重复
#[test]
fn test_basic_deduplication() {
    let mut body = vec![
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![ReachMsgReceive {
                receive_group_code: "GROUP1".to_string(),
                receive_kind: ReachReceiveKind::Account,
                receive_ids: vec!["user1".to_string(), "user2".to_string()],
            }],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), Some("value1".to_string()));
                map
            },
        },
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![ReachMsgReceive {
                receive_group_code: "GROUP1".to_string(),
                receive_kind: ReachReceiveKind::Account,
                receive_ids: vec!["user2".to_string(), "user3".to_string()],
            }],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), Some("value1".to_string()));
                map
            },
        },
    ];

    deduplicate_send_requests(&mut body);

    // 第一个send_req应该保持不变
    assert_eq!(body.len(), 2);
    assert_eq!(body[0].receives[0].receive_ids.len(), 2);
    assert_eq!(body[0].receives[0].receive_ids, vec!["user1", "user2"]);

    // 第二个send_req中的"user2"应该被移除
    assert_eq!(body[1].receives[0].receive_ids.len(), 1);
    assert_eq!(body[1].receives[0].receive_ids, vec!["user3"]);
}

/// 测试：去重后receive_ids为空，应该移除该receive
#[test]
fn test_remove_empty_receive() {
    let mut body = vec![
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![
                ReachMsgReceive {
                    receive_group_code: "GROUP1".to_string(),
                    receive_kind: ReachReceiveKind::Account,
                    receive_ids: vec!["user1".to_string()],
                },
                ReachMsgReceive {
                    receive_group_code: "GROUP2".to_string(),
                    receive_kind: ReachReceiveKind::Account,
                    receive_ids: vec!["user2".to_string()],
                },
            ],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), Some("value1".to_string()));
                map
            },
        },
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![
                ReachMsgReceive {
                    receive_group_code: "GROUP1".to_string(),
                    receive_kind: ReachReceiveKind::Account,
                    receive_ids: vec!["user1".to_string()], // 这个会被完全移除
                },
                ReachMsgReceive {
                    receive_group_code: "GROUP2".to_string(),
                    receive_kind: ReachReceiveKind::Account,
                    receive_ids: vec!["user3".to_string()], // 这个保留
                },
            ],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), Some("value1".to_string()));
                map
            },
        },
    ];

    deduplicate_send_requests(&mut body);

    // 第一个send_req应该保持不变
    assert_eq!(body[0].receives.len(), 2);

    // 第二个send_req中，GROUP1的receive应该被移除（因为user1被去重后为空）
    assert_eq!(body[1].receives.len(), 1);
    assert_eq!(body[1].receives[0].receive_group_code, "GROUP2");
    assert_eq!(body[1].receives[0].receive_ids, vec!["user3"]);
}

/// 测试：去重后receives为空，应该删除整个send_req
#[test]
fn test_remove_empty_send_req() {
    let mut body = vec![
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![ReachMsgReceive {
                receive_group_code: "GROUP1".to_string(),
                receive_kind: ReachReceiveKind::Account,
                receive_ids: vec!["user1".to_string(), "user2".to_string()],
            }],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), Some("value1".to_string()));
                map
            },
        },
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![ReachMsgReceive {
                receive_group_code: "GROUP1".to_string(),
                receive_kind: ReachReceiveKind::Account,
                receive_ids: vec!["user1".to_string(), "user2".to_string()], // 全部重复，会被移除
            }],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), Some("value1".to_string()));
                map
            },
        },
    ];

    deduplicate_send_requests(&mut body);

    // 第二个send_req应该被完全移除
    assert_eq!(body.len(), 1);
    assert_eq!(body[0].receives[0].receive_ids.len(), 2);
}

/// 测试：rel_item_id不同，不应该去重
#[test]
fn test_different_rel_item_id() {
    let mut body = vec![
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![ReachMsgReceive {
                receive_group_code: "GROUP1".to_string(),
                receive_kind: ReachReceiveKind::Account,
                receive_ids: vec!["user1".to_string()],
            }],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), Some("value1".to_string()));
                map
            },
        },
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![ReachMsgReceive {
                receive_group_code: "GROUP1".to_string(),
                receive_kind: ReachReceiveKind::Account,
                receive_ids: vec!["user1".to_string()],
            }],
            rel_item_id: "item2".to_string(), // 不同的rel_item_id
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), Some("value1".to_string()));
                map
            },
        },
    ];

    deduplicate_send_requests(&mut body);

    // 两个send_req都应该保留，因为rel_item_id不同
    assert_eq!(body.len(), 2);
    assert_eq!(body[0].receives[0].receive_ids.len(), 1);
    assert_eq!(body[1].receives[0].receive_ids.len(), 1);
}

/// 测试：replace不同，不应该去重
#[test]
fn test_different_replace() {
    let mut body = vec![
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![ReachMsgReceive {
                receive_group_code: "GROUP1".to_string(),
                receive_kind: ReachReceiveKind::Account,
                receive_ids: vec!["user1".to_string()],
            }],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), Some("value1".to_string()));
                map
            },
        },
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![ReachMsgReceive {
                receive_group_code: "GROUP1".to_string(),
                receive_kind: ReachReceiveKind::Account,
                receive_ids: vec!["user1".to_string()],
            }],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), Some("value2".to_string())); // 不同的replace值
                map
            },
        },
    ];

    deduplicate_send_requests(&mut body);

    // 两个send_req都应该保留，因为replace不同
    assert_eq!(body.len(), 2);
    assert_eq!(body[0].receives[0].receive_ids.len(), 1);
    assert_eq!(body[1].receives[0].receive_ids.len(), 1);
}

/// 测试：多个send_req的复杂场景
#[test]
fn test_multiple_send_reqs() {
    let mut body = vec![
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![ReachMsgReceive {
                receive_group_code: "GROUP1".to_string(),
                receive_kind: ReachReceiveKind::Account,
                receive_ids: vec!["user1".to_string(), "user2".to_string()],
            }],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), Some("value1".to_string()));
                map
            },
        },
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![ReachMsgReceive {
                receive_group_code: "GROUP1".to_string(),
                receive_kind: ReachReceiveKind::Account,
                receive_ids: vec!["user2".to_string(), "user3".to_string()],
            }],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), Some("value1".to_string()));
                map
            },
        },
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![ReachMsgReceive {
                receive_group_code: "GROUP1".to_string(),
                receive_kind: ReachReceiveKind::Account,
                receive_ids: vec!["user3".to_string(), "user4".to_string()],
            }],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), Some("value1".to_string()));
                map
            },
        },
    ];

    deduplicate_send_requests(&mut body);

    // 第一个send_req应该保持不变
    assert_eq!(body[0].receives[0].receive_ids, vec!["user1", "user2"]);

    // 第二个send_req中，user2应该被移除（因为第一个中有user2）
    assert_eq!(body[1].receives[0].receive_ids, vec!["user3"]);

    // 第三个send_req中，user3应该被移除（因为第二个中有user3，虽然第二个已经被去重了）
    // 注意：这里user3在第二个send_req中，而第二个send_req相对于第一个已经被去重了
    // 但是第三个send_req需要检查相对于前面所有send_req的重复
    // 实际上，第三个send_req应该检查相对于第一个和第二个的重复
    // user3在第二个send_req中，所以应该被移除
    assert_eq!(body[2].receives[0].receive_ids, vec!["user4"]);
}

/// 测试：replace为空的情况
#[test]
fn test_empty_replace() {
    let mut body = vec![
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![ReachMsgReceive {
                receive_group_code: "GROUP1".to_string(),
                receive_kind: ReachReceiveKind::Account,
                receive_ids: vec!["user1".to_string()],
            }],
            rel_item_id: "item1".to_string(),
            replace: HashMap::new(), // 空的replace
        },
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![ReachMsgReceive {
                receive_group_code: "GROUP1".to_string(),
                receive_kind: ReachReceiveKind::Account,
                receive_ids: vec!["user1".to_string(), "user2".to_string()],
            }],
            rel_item_id: "item1".to_string(),
            replace: HashMap::new(), // 空的replace
        },
    ];

    deduplicate_send_requests(&mut body);

    // 应该去重
    assert_eq!(body.len(), 2);
    assert_eq!(body[0].receives[0].receive_ids, vec!["user1"]);
    assert_eq!(body[1].receives[0].receive_ids, vec!["user2"]);
}

/// 测试：replace中有None值的情况
#[test]
fn test_replace_with_none() {
    let mut body = vec![
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![ReachMsgReceive {
                receive_group_code: "GROUP1".to_string(),
                receive_kind: ReachReceiveKind::Account,
                receive_ids: vec!["user1".to_string()],
            }],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), None);
                map
            },
        },
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![ReachMsgReceive {
                receive_group_code: "GROUP1".to_string(),
                receive_kind: ReachReceiveKind::Account,
                receive_ids: vec!["user1".to_string(), "user2".to_string()],
            }],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), None);
                map
            },
        },
    ];

    deduplicate_send_requests(&mut body);

    // 应该去重
    assert_eq!(body.len(), 2);
    assert_eq!(body[0].receives[0].receive_ids, vec!["user1"]);
    assert_eq!(body[1].receives[0].receive_ids, vec!["user2"]);
}

/// 测试：多个receives，部分去重
#[test]
fn test_multiple_receives_partial_deduplication() {
    let mut body = vec![
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![
                ReachMsgReceive {
                    receive_group_code: "GROUP1".to_string(),
                    receive_kind: ReachReceiveKind::Account,
                    receive_ids: vec!["user1".to_string()],
                },
                ReachMsgReceive {
                    receive_group_code: "GROUP2".to_string(),
                    receive_kind: ReachReceiveKind::Role,
                    receive_ids: vec!["role1".to_string()],
                },
            ],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), Some("value1".to_string()));
                map
            },
        },
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![
                ReachMsgReceive {
                    receive_group_code: "GROUP1".to_string(),
                    receive_kind: ReachReceiveKind::Account,
                    receive_ids: vec!["user1".to_string(), "user2".to_string()],
                },
                ReachMsgReceive {
                    receive_group_code: "GROUP2".to_string(),
                    receive_kind: ReachReceiveKind::Role,
                    receive_ids: vec!["role1".to_string(), "role2".to_string()],
                },
            ],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), Some("value1".to_string()));
                map
            },
        },
    ];

    deduplicate_send_requests(&mut body);

    // 第一个send_req应该保持不变
    assert_eq!(body[0].receives.len(), 2);
    assert_eq!(body[0].receives[0].receive_ids, vec!["user1"]);
    assert_eq!(body[0].receives[1].receive_ids, vec!["role1"]);

    // 第二个send_req中，user1和role1应该被移除
    assert_eq!(body[1].receives.len(), 2);
    assert_eq!(body[1].receives[0].receive_ids, vec!["user2"]);
    assert_eq!(body[1].receives[1].receive_ids, vec!["role2"]);
}

/// 测试：没有重复的情况
#[test]
fn test_no_duplication() {
    let mut body = vec![
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![ReachMsgReceive {
                receive_group_code: "GROUP1".to_string(),
                receive_kind: ReachReceiveKind::Account,
                receive_ids: vec!["user1".to_string()],
            }],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), Some("value1".to_string()));
                map
            },
        },
        ReachMsgSendReq {
            scene_code: "SCENE1".to_string(),
            receives: vec![ReachMsgReceive {
                receive_group_code: "GROUP1".to_string(),
                receive_kind: ReachReceiveKind::Account,
                receive_ids: vec!["user2".to_string()],
            }],
            rel_item_id: "item1".to_string(),
            replace: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), Some("value1".to_string()));
                map
            },
        },
    ];

    deduplicate_send_requests(&mut body);

    // 两个send_req都应该保留，因为没有重复
    assert_eq!(body.len(), 2);
    assert_eq!(body[0].receives[0].receive_ids, vec!["user1"]);
    assert_eq!(body[1].receives[0].receive_ids, vec!["user2"]);
}

/// 测试：空body
#[test]
fn test_empty_body() {
    let mut body = vec![];

    deduplicate_send_requests(&mut body);

    assert_eq!(body.len(), 0);
}

/// 测试：单个send_req
#[test]
fn test_single_send_req() {
    let mut body = vec![ReachMsgSendReq {
        scene_code: "SCENE1".to_string(),
        receives: vec![ReachMsgReceive {
            receive_group_code: "GROUP1".to_string(),
            receive_kind: ReachReceiveKind::Account,
            receive_ids: vec!["user1".to_string()],
        }],
        rel_item_id: "item1".to_string(),
        replace: {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), Some("value1".to_string()));
            map
        },
    }];

    deduplicate_send_requests(&mut body);

    assert_eq!(body.len(), 1);
    assert_eq!(body[0].receives[0].receive_ids, vec!["user1"]);
}
