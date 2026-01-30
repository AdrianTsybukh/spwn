#[cfg(target_os = "windows")]
pub async fn run_command(input: String) -> Result<String, String> {
    let output = tokio::process::Command::new("cmd")
        .args(["/C", &input])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        if err.trim().is_empty() {
            Err(format!("Error: Command failed with status {:?}", output.status.code()))
        } else {
            Err(err)
        }
    }
}

#[cfg(target_os = "linux")]
pub async fn run_command(input: String) -> Result<String, String> {
    let args = shell_words::split(&input)
        .map_err(|e| format!("Parse error: {}", e))?;

    if args.is_empty() {
        return Ok(String::new());
    }

    let command = &args[0];
    let arguments = &args[1..];

    let output = tokio::process::Command::new(command)
        .args(arguments)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|e| e.to_string())
    } else {
        let err = String::from_utf8(output.stderr).unwrap_or("Unknown error".to_string());
        Err(err)
    }
}
