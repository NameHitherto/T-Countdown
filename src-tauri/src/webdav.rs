use base64::{engine::general_purpose, Engine as _};

use crate::config::{load_webdav_credentials, load_webdav_proxy_config};

fn make_auth_header(username: &str, password: &str) -> String {
    let creds = format!("{}:{}", username, password);
    format!("Basic {}", general_purpose::STANDARD.encode(creds.as_bytes()))
}

fn create_webdav_agent() -> Result<ureq::Agent, String> {
    let mut builder = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(15));

    let proxy_config = load_webdav_proxy_config()?;
    if proxy_config.enabled {
        if let Some(port) = proxy_config.port {
            let proxy = ureq::Proxy::new(&format!("http://127.0.0.1:{}", port))
                .map_err(|e| format!("代理配置无效: {}", e))?;
            builder = builder.proxy(proxy);
        }
    }

    Ok(builder.build())
}

pub async fn test_webdav(server: String, username: String, password: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let url = if server.ends_with('/') {
            server
        } else {
            format!("{}/", server)
        };
        let auth = make_auth_header(&username, &password);
        let agent = create_webdav_agent()?;

        match agent
            .request("PROPFIND", &url)
            .set("Authorization", &auth)
            .set("Depth", "0")
            .call()
        {
            Ok(_) => Ok(()),
            Err(ureq::Error::Status(code, _)) => {
                if code == 401 || code == 403 {
                    Err("认证失败，请检查账号和应用密码".to_string())
                } else {
                    Err(format!("连接失败，状态码: {}", code))
                }
            }
            Err(e) => Err(format!("网络错误: {}", e)),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn upload(json: String) -> Result<(), String> {
    let credentials = load_webdav_credentials()?;

    tokio::task::spawn_blocking(move || {
        let auth = make_auth_header(&credentials.username, &credentials.password);
        let base_url = if credentials.server.ends_with('/') {
            credentials.server.clone()
        } else {
            format!("{}/", credentials.server)
        };
        let folder_url = format!("{}T-Countdown/", base_url);
        let file_url = format!("{}data.json", folder_url);
        let agent = create_webdav_agent()?;

        let _ = agent
            .request("MKCOL", &folder_url)
            .set("Authorization", &auth)
            .call();

        agent
            .put(&file_url)
            .set("Authorization", &auth)
            .set("Content-Type", "application/json; charset=utf-8")
            .send_string(&json)
            .map_err(|e| format!("上传失败: {}", e))?;

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn download() -> Result<String, String> {
    let credentials = load_webdav_credentials()?;

    tokio::task::spawn_blocking(move || {
        let auth = make_auth_header(&credentials.username, &credentials.password);
        let base_url = if credentials.server.ends_with('/') {
            credentials.server.clone()
        } else {
            format!("{}/", credentials.server)
        };
        let file_url = format!("{}T-Countdown/data.json", base_url);
        let agent = create_webdav_agent()?;

        match agent.get(&file_url).set("Authorization", &auth).call() {
            Ok(resp) => {
                let body = resp.into_string().map_err(|e| format!("读取失败: {}", e))?;
                if serde_json::from_str::<serde_json::Value>(&body).is_err() {
                    return Err("下载的数据格式无效，可能是网络代理干扰，请检查网络连接".to_string());
                }
                Ok(body)
            }
            Err(ureq::Error::Status(404, _)) => Ok("[]".to_string()),
            Err(e) => Err(format!("下载失败: {}", e)),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
