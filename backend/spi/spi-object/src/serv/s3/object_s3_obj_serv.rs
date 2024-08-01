use tardis::{
    basic::{error::TardisError, result::TardisResult},
    os::{
        os_client::TardisOSClient,
        serde_types::{BucketLifecycleConfiguration, Expiration, LifecycleFilter, LifecycleRule},
    },
};

pub(crate) struct S3Service;
impl super::S3 for S3Service {
    async fn rebuild_path(bucket_name: Option<&str>, origin_path: &str, obj_exp: Option<u32>, client: &TardisOSClient) -> TardisResult<String> {
        if let Some(obj_exp) = obj_exp {
            let resp = client.get_lifecycle(bucket_name).await;
            match resp {
                Ok(config) => {
                    let mut rules = config.rules;
                    let prefix = if let Some(is_have_prefix) = rules
                        .iter()
                        .filter(|r| r.status == *"Enabled" && r.expiration.clone().is_some_and(|exp| exp.days.is_some_and(|days| days == obj_exp)))
                        .filter_map(|r| r.filter.clone())
                        .find_map(|f| f.prefix)
                    {
                        is_have_prefix
                    } else {
                        let rand_id = tardis::rand::random::<usize>().to_string();
                        let prefix = format!("{}/", rand_id);
                        //add rule
                        let add_rule = LifecycleRule::builder("Enabled")
                            .id(&rand_id)
                            .expiration(Expiration::new(None, Some(obj_exp), None))
                            .filter(LifecycleFilter::new(None, None, None, Some(prefix.clone()), None))
                            .build();
                        rules.push(add_rule);
                        client.put_lifecycle(bucket_name, BucketLifecycleConfiguration::new(rules)).await?;
                        prefix
                    };
                    Ok(format!("{}{}", prefix, origin_path))
                }
                Err(e) => {
                    if e.code != "404" {
                        return Err(TardisError::internal_error(&format!("Bucket {:?} get lifecycle failed", bucket_name), &format!("{:?}", e)));
                    }
                    let mut rules = vec![];
                    let rand_id = tardis::rand::random::<usize>().to_string();
                    let prefix = format!("{}/", rand_id);
                    //add rule
                    let add_rule = LifecycleRule::builder("Enabled")
                        .id(&rand_id)
                        .expiration(Expiration::new(None, Some(obj_exp), None))
                        .filter(LifecycleFilter::new(None, None, None, Some(prefix.clone()), None))
                        .build();
                    rules.push(add_rule);
                    client.put_lifecycle(bucket_name, BucketLifecycleConfiguration::new(rules)).await?;
                    Ok(format!("{}{}", prefix, origin_path))
                }
            }
        } else {
            Ok(origin_path.to_string())
        }
    }
}
