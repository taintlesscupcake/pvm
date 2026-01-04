//! Activation script generation

use std::path::Path;

/// Generate activation script path for an environment
pub fn generate_activation_script(env_path: &Path) -> String {
    let activate_path = env_path.join("bin").join("activate");
    format!(
        r#"# pvm activation script
source "{}"
"#,
        activate_path.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_generate_activation_script() {
        let env_path = PathBuf::from("/home/user/.pvm/envs/myenv");
        let script = generate_activation_script(&env_path);

        assert!(script.contains("source"));
        assert!(script.contains("/home/user/.pvm/envs/myenv/bin/activate"));
    }
}
