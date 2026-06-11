use axum::Json;

pub async fn security_check() -> Json<serde_json::Value> {
    let mut vulns: Vec<serde_json::Value> = Vec::new();

    match std::process::Command::new("firefox").arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                vulns.push(serde_json::json!({"type": "info", "component": "firefox", "detail": version}));
            } else {
                vulns.push(serde_json::json!({"type": "info", "component": "firefox", "detail": "not found"}));
            }
        }
        Err(_) => {
            vulns.push(serde_json::json!({"type": "info", "component": "firefox", "detail": "not found"}));
        }
    }

    match std::process::Command::new("uname").arg("-r").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            vulns.push(serde_json::json!({"type": "info", "component": "kernel", "detail": version}));
        }
        Err(_) => {}
    }

    match std::process::Command::new("ss").args(["-tlnp"]).output() {
        Ok(output) => {
            let listening = String::from_utf8_lossy(&output.stdout);
            let exposed: Vec<&str> = listening.lines()
                .filter(|l| l.contains("LISTEN") && (l.contains("0.0.0.0") || l.contains(":::")))
                .filter_map(|l| l.split_whitespace().nth(3))
                .filter_map(|addr| addr.rsplit(':').next())
                .collect();
            if !exposed.is_empty() {
                vulns.push(serde_json::json!({
                    "type": "warning",
                    "component": "network",
                    "detail": format!("Exposed ports: {}", exposed.join(", ")),
                }));
            }
        }
        Err(_) => {}
    }

    Json(serde_json::json!({"vulnerabilities": vulns}))
}
