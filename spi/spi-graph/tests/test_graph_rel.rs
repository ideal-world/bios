use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_graph::dto::graph_dto::{GraphNodeVersionResp, GraphRelAddReq, GraphRelDetailResp, GraphRelUpgardeDelRelReq, GraphRelUpgardeVersionReq};
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::serde_json::json;
use tardis::web::web_resp::Void;
use tardis::TardisFuns;

pub async fn test(client: &mut TestHttpClient) -> TardisResult<()> {
    client.set_auth(&TardisContext {
        own_paths: "t1/a1".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;

    let test_case_sqls = vec![
        ("iter-req", "iter1", "1", "req1", "1"),
        ("iter-req", "iter1", "1", "req2", "1"),
        ("req-task", "req1", "1", "task1", "1"),
        ("req-task", "req1", "1", "task2", "1"),
        ("req-task", "req2", "1", "task2", "1"),
        ("req-task", "req1", "2", "task1", "1"),
        ("req-task", "req1", "2", "task3", "1"),
        ("req-bug", "req1", "1", "bug1", "1"),
        ("req-bug-2", "req1", "1", "bug1", "1"),
        ("task-bug", "task1", "1", "bug1", "1"),
        ("task-bug", "task1", "1", "bug2", "1"),
        ("task-bug", "task2", "1", "bug2", "1"),
        ("task-bug", "task3", "1", "bug3", "1"),
    ];

    // Add Rel
    for (tag, from_key, from_version, to_key, to_version) in test_case_sqls {
        let _: Void = client
            .put(
                "/ci/rel",
                &GraphRelAddReq {
                    tag: tag.to_string(),
                    from_key: from_key.into(),
                    from_version: from_version.to_string(),
                    to_key: to_key.into(),
                    to_version: to_version.to_string(),
                },
            )
            .await;
    }

    //  Find Versions
    let result: Vec<GraphNodeVersionResp> = client.get("/ci/versions?tag=req-task&key=req1").await;
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].version, "2");
    assert_eq!(result[1].version, "1");

    //  Find Rels
    let result: GraphRelDetailResp = client.get("/ci/rels?from_key=req1&from_version=1").await;
    assert_eq!(
        TardisFuns::json.obj_to_json(&result)?,
        json!({
            "key": "req1",
            "version": "1",
            "form_rels": {
                "req-bug": [
                    {
                        "key": "bug1",
                        "version": "1",
                        "form_rels": {},
                        "to_rels": {}
                    }
                ],
                "req-task": [
                    {
                        "key": "task2",
                        "version": "1",
                        "form_rels": {
                            "task-bug": [
                                {
                                    "key": "bug2",
                                    "version": "1",
                                    "form_rels": {},
                                    "to_rels": {}
                                }
                            ]
                        },
                        "to_rels": {}
                    },
                    {
                        "key": "task1",
                        "version": "1",
                        "form_rels": {
                            "task-bug": [
                                {
                                    "key": "bug1",
                                    "version": "1",
                                    "form_rels": {},
                                    "to_rels": {}
                                },
                                {
                                    "key": "bug2",
                                    "version": "1",
                                    "form_rels": {},
                                    "to_rels": {}
                                }
                            ]
                        },
                        "to_rels": {}
                    }
                ],
                "req-bug-2": [
                    {
                        "key": "bug1",
                        "version": "1",
                        "form_rels": {},
                        "to_rels": {}
                    }
                ]
            },
            "to_rels": {
                "iter-req": [
                    {
                        "key": "iter1",
                        "version": "1",
                        "form_rels": {},
                        "to_rels": {}
                    }
                ]
            }
        })
    );
    let result: GraphRelDetailResp = client.get("/ci/rels?from_key=req1&from_version=2").await;
    assert_eq!(
        TardisFuns::json.obj_to_json(&result)?,
        json!({
            "key": "req1",
            "version": "2",
            "form_rels": {
                "req-task": [
                    {
                        "key": "task1",
                        "version": "1",
                        "form_rels": {
                            "task-bug": [
                                {
                                    "key": "bug1",
                                    "version": "1",
                                    "form_rels": {},
                                    "to_rels": {}
                                },
                                {
                                    "key": "bug2",
                                    "version": "1",
                                    "form_rels": {},
                                    "to_rels": {}
                                }
                            ]
                        },
                        "to_rels": {}
                    },
                    {
                        "key": "task3",
                        "version": "1",
                        "form_rels": {
                            "task-bug": [
                                {
                                    "key": "bug3",
                                    "version": "1",
                                    "form_rels": {},
                                    "to_rels": {}
                                }
                            ]
                        },
                        "to_rels": {}
                    }
                ]
            },
            "to_rels": {}
        })
    );

    // Delete Rel
    client.delete("/ci/rel?tag=req-task&to_key=task3").await;
    let result: GraphRelDetailResp = client.get("/ci/rels?from_key=req1&from_version=2").await;
    assert_eq!(
        TardisFuns::json.obj_to_json(&result)?,
        json!({
            "key": "req1",
            "version": "2",
            "form_rels": {
                "req-task": [
                    {
                        "key": "task1",
                        "version": "1",
                        "form_rels": {
                            "task-bug": [
                                {
                                    "key": "bug1",
                                    "version": "1",
                                    "form_rels": {},
                                    "to_rels": {}
                                },
                                {
                                    "key": "bug2",
                                    "version": "1",
                                    "form_rels": {},
                                    "to_rels": {}
                                }
                            ]
                        },
                        "to_rels": {}
                    }
                ]
            },
            "to_rels": {}
        })
    );

    // Upgrade Version
    let _: Void = client
        .put(
            "/ci/version",
            &GraphRelUpgardeVersionReq {
                key: "req1".into(),
                old_version: "1".to_string(),
                new_version: "3".to_string(),
                del_rels: Vec::new(),
            },
        )
        .await;
    let result: GraphRelDetailResp = client.get("/ci/rels?from_key=req1&from_version=1").await;
    // -- No changes
    assert_eq!(
        TardisFuns::json.obj_to_json(&result)?,
        json!({
            "key": "req1",
            "version": "1",
            "form_rels": {
                "req-bug": [
                    {
                        "key": "bug1",
                        "version": "1",
                        "form_rels": {},
                        "to_rels": {}
                    }
                ],
                "req-task": [
                    {
                        "key": "task2",
                        "version": "1",
                        "form_rels": {
                            "task-bug": [
                                {
                                    "key": "bug2",
                                    "version": "1",
                                    "form_rels": {},
                                    "to_rels": {}
                                }
                            ]
                        },
                        "to_rels": {}
                    },
                    {
                        "key": "task1",
                        "version": "1",
                        "form_rels": {
                            "task-bug": [
                                {
                                    "key": "bug1",
                                    "version": "1",
                                    "form_rels": {},
                                    "to_rels": {}
                                },
                                {
                                    "key": "bug2",
                                    "version": "1",
                                    "form_rels": {},
                                    "to_rels": {}
                                }
                            ]
                        },
                        "to_rels": {}
                    }
                ],
                "req-bug-2": [
                    {
                        "key": "bug1",
                        "version": "1",
                        "form_rels": {},
                        "to_rels": {}
                    }
                ]
            },
            "to_rels": {
                "iter-req": [
                    {
                        "key": "iter1",
                        "version": "1",
                        "form_rels": {},
                        "to_rels": {}
                    }
                ]
            }
        })
    );
    let result: GraphRelDetailResp = client.get("/ci/rels?from_key=req1&from_version=3").await;
    // -- No changes
    assert_eq!(
        TardisFuns::json.obj_to_json(&result)?,
        json!({
            "key": "req1",
            "version": "3",
            "form_rels": {
                "req-bug": [
                    {
                        "key": "bug1",
                        "version": "1",
                        "form_rels": {},
                        "to_rels": {}
                    }
                ],
                "req-task": [
                    {
                        "key": "task2",
                        "version": "1",
                        "form_rels": {
                            "task-bug": [
                                {
                                    "key": "bug2",
                                    "version": "1",
                                    "form_rels": {},
                                    "to_rels": {}
                                }
                            ]
                        },
                        "to_rels": {}
                    },
                    {
                        "key": "task1",
                        "version": "1",
                        "form_rels": {
                            "task-bug": [
                                {
                                    "key": "bug1",
                                    "version": "1",
                                    "form_rels": {},
                                    "to_rels": {}
                                },
                                {
                                    "key": "bug2",
                                    "version": "1",
                                    "form_rels": {},
                                    "to_rels": {}
                                }
                            ]
                        },
                        "to_rels": {}
                    }
                ],
                "req-bug-2": [
                    {
                        "key": "bug1",
                        "version": "1",
                        "form_rels": {},
                        "to_rels": {}
                    }
                ]
            },
            "to_rels": {
                "iter-req": [
                    {
                        "key": "iter1",
                        "version": "1",
                        "form_rels": {},
                        "to_rels": {}
                    }
                ]
            }
        })
    );
    let _: Void = client
        .put(
            "/ci/version",
            &GraphRelUpgardeVersionReq {
                key: "req1".into(),
                old_version: "3".to_string(),
                new_version: "4".to_string(),
                del_rels: vec![
                    GraphRelUpgardeDelRelReq {
                        tag: Some("req-task".to_string()),
                        rel_key: Some("task1".into()),
                        rel_version: None,
                    },
                    GraphRelUpgardeDelRelReq {
                        tag: Some("req-bug-2".to_string()),
                        rel_key: Some("bug1".into()),
                        rel_version: Some("1".to_string()),
                    },
                ],
            },
        )
        .await;
    let result: GraphRelDetailResp = client.get("/ci/rels?from_key=req1&from_version=4").await;
    assert_eq!(
        TardisFuns::json.obj_to_json(&result)?,
        json!({
            "key": "req1",
            "version": "4",
            "form_rels": {
                "req-bug": [
                    {
                        "key": "bug1",
                        "version": "1",
                        "form_rels": {},
                        "to_rels": {}
                    }
                ],
                "req-task": [
                    {
                        "key": "task2",
                        "version": "1",
                        "form_rels": {
                            "task-bug": [
                                {
                                    "key": "bug2",
                                    "version": "1",
                                    "form_rels": {},
                                    "to_rels": {}
                                }
                            ]
                        },
                        "to_rels": {}
                    }
                ]
            },
            "to_rels": {
                "iter-req": [
                    {
                        "key": "iter1",
                        "version": "1",
                        "form_rels": {},
                        "to_rels": {}
                    }
                ]
            }
        })
    );
    Ok(())
}
