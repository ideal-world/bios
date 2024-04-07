use bios_auth::serv::auth_res_serv;
use tardis::{basic::result::TardisResult, serde_json::json, TardisFuns};

pub fn test_res() -> TardisResult<()> {
    auth_res_serv::init_res()?;

    let init_res_data = vec![
        ("GET", "iam-res://iam-serv", r##"{"apps":"#app1#app2#","tenants":"#tenant1#"}"##),
        ("GET", "iam-res://*/**", r##"{"apps":"#app1#app2#","tenants":"#tenant1#"}"##),
        ("GET", "iam-res://iam-serv/p1", r##"{"accounts":"#acc#"}"##),
        ("GET", "iam-res://iam-serv/p1/p2", r##"{"roles":"#role1#"}"##),
        ("GET", "iam-res://iam-serv/p1/p2/*", r##"{"roles":"#role1#"}"##),
        ("GET", "iam-res://iam-serv/p1/p2/**", r##"{"roles":"#role1#"}"##),
        ("GET", "iam-res://iam-serv/p4/p2/**", r##"{"roles":"#role1#"}"##),
        ("GET", "iam-res://iam-serv/*/p2", r##"{"roles":"#role1#"}"##),
        ("GET", "iam-res://iam-serv/**/p5", r##"{"roles":"#role1#"}"##),
        ("GET", "iam-res://iam-serv/p1?q1=aa", r##"{"roles":"#role1#"}"##),
        ("GET", "iam-res://iam-serv/p1?q1=aa&q2=bb", r##"{"roles":"#role1#"}"##),
        ("POST", "iam-res://iam-serv/p1", r##"{"roles":"#role1#"}"##),
    ];
    for (res_action, res_uri, auth_info) in init_res_data {
        auth_res_serv::add_res(res_action, res_uri, Some(TardisFuns::json.str_to_obj(auth_info)?), false, false, false, false)?;
    }

    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv")?.len(), 1);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv")?[0].uri, "iam-res://iam-serv");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam2")?.len(), 0);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/ss")?.len(), 1);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/ss")?[0].uri, "iam-res://*/**");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam2/ss")?.len(), 1);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam2/ss")?[0].uri, "iam-res://*/**");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p2")?.len(), 1);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p2")?[0].uri, "iam-res://*/**");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1")?.len(), 2);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1")?[0].uri, "iam-res://iam-serv/p1");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1")?[1].uri, "iam-res://*/**");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2")?.len(), 3);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2")?[0].uri, "iam-res://iam-serv/p1/p2");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2")?[1].uri, "iam-res://iam-serv/*/p2");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2")?[2].uri, "iam-res://*/**");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/x")?.len(), 3);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/x")?[0].uri, "iam-res://iam-serv/p1/p2/*");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/x")?[1].uri, "iam-res://iam-serv/p1/p2/**");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/x")?[2].uri, "iam-res://*/**");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/x/y")?.len(), 2);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/x/y")?[0].uri, "iam-res://iam-serv/p1/p2/**");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/x/y")?[1].uri, "iam-res://*/**");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/x/y/p5")?.len(), 3);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/x/y/p5")?[0].uri, "iam-res://iam-serv/p1/p2/**");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/x/y/p5")?[1].uri, "iam-res://iam-serv/**/p5");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/x/y/p5")?[2].uri, "iam-res://*/**");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1?q1=aa")?.len(), 2);
    // TODO iam-res://iam-serv/p1?q1=aa
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1?q1=aa")?[0].uri, "iam-res://iam-serv/p1");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1?q1=aa")?[1].uri, "iam-res://*/**");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1?Q2=bb&q1=aa")?.len(), 2);
    // TODO iam-res://iam-serv/p1?q1=aa&q2=bb
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1?Q2=bb&q1=aa")?[0].uri, "iam-res://iam-serv/p1");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1?Q2=bb&q1=aa")?[1].uri, "iam-res://*/**");
    assert_eq!(auth_res_serv::match_res("post", "iam-res://iam-serv/p1?Q2=bb&q1=aa")?.len(), 1);
    // TODO iam-res://iam-serv/p1?q1=aa&q2=bb
    assert_eq!(auth_res_serv::match_res("post", "iam-res://iam-serv/p1?Q2=bb&q1=aa")?[0].uri, "iam-res://iam-serv/p1");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://App1.Tenant1/p1?Q2=bb&q1=aa")?.len(), 1);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://App1.Tenant1/p1?Q2=bb&q1=aa")?[0].uri, "iam-res://*/**");

    auth_res_serv::remove_res("get", "iam-res://iam-serv/**")?;
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv")?.len(), 1);

    auth_res_serv::remove_res("get", "iam-res://iam-serv/")?;
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv")?.len(), 0);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p2")?.len(), 1);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p2")?[0].uri, "iam-res://*/**");

    auth_res_serv::remove_res("get", "iam-res://*/**")?;
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p2")?.len(), 0);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam2/ss")?.len(), 0);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1")?.len(), 1);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1")?[0].uri, "iam-res://iam-serv/p1");

    auth_res_serv::remove_res("get", "iam-res://iam-serv/p1")?;
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1")?.len(), 0);

    auth_res_serv::remove_res("get", "iam-res://iam-serv/p1/p2/*")?;
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/x")?.len(), 1);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/x")?[0].uri, "iam-res://iam-serv/p1/p2/**");
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/x/y")?.len(), 1);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/x/y")?[0].uri, "iam-res://iam-serv/p1/p2/**");

    auth_res_serv::remove_res("get", "iam-res://iam-serv/p1/p2/**")?;
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/x")?.len(), 0);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/x/y")?.len(), 0);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/p5")?.len(), 1);
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/p5")?[0].uri, "iam-res://iam-serv/**/p5");

    auth_res_serv::remove_res("get", "iam-res://iam-serv/**/p5")?;
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1/p2/p5")?.len(), 0);

    auth_res_serv::remove_res("get", "iam-res://iam-serv/p1?q1=aa")?;
    assert_eq!(auth_res_serv::match_res("get", "iam-res://iam-serv/p1?q1=aa")?.len(), 0);

    auth_res_serv::remove_res("get", "iam-res://iam-serv/p1?Q2=bb&q1=aa")?;
    auth_res_serv::remove_res("post", "iam-res://iam-serv/p1?Q2=bb&q1=aa")?;
    auth_res_serv::remove_res("get", "iam-res://iam-serv/p4/p2/**")?;
    auth_res_serv::remove_res("get", "iam-res://iam-serv/*/p2")?;
    auth_res_serv::remove_res("get", "iam-res://iam-serv/p1?q1=aa&q2=bb")?;
    auth_res_serv::remove_res("get", "iam-res://iam-serv/p1/p2")?;
    auth_res_serv::remove_res("post", "iam-res://iam-serv/p1")?;

    assert_eq!(auth_res_serv::get_res_json()?, json!({"children":{},"leaf_info":null}));

    Ok(())
}
