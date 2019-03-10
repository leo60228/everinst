use reqwest::Client;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct EverestVersion {
    pub branch: String,
    pub url: String,
    pub ver: usize,
}

pub fn get_versions() -> Vec<EverestVersion> {
    let mut entries = vec![];

    let client = Client::new();
    let req =
        client.get("https://dev.azure.com/EverestAPI/Everest/_apis/build/builds?api-version=5.0");
    let azure: Value = match req.send() {
        Ok(mut resp) => match resp.json() {
            Ok(json) => json,
            _ => return entries,
        },
        _ => return entries,
    };

    let list = match azure["value"] {
        Value::Array(ref vec) => vec,
        _ => return entries,
    };

    for build in list {
        if build["status"].as_str() != Some("completed")
            || build["result"].as_str() != Some("succeeded")
        {
            continue;
        }

        if build["reason"].as_str() != Some("manual")
            && build["reason"].as_str() != Some("individualCI")
        {
            continue;
        }

        let id = match build["id"].as_u64() {
            Some(id) => id,
            _ => continue,
        };

        let branch = match build["sourceBranch"] {
            Value::String(ref branch) => branch.replace("refs/heads/", ""),
            _ => continue,
        };

        let url = format!("https://dev.azure.com/EverestAPI/Everest/_apis/build/builds/{}/artifacts?artifactName=main&api-version=5.0&%24format=zip", id);

        entries.push(EverestVersion {
            branch,
            url,
            ver: (id + 700) as usize,
        });
    }

    entries
}
