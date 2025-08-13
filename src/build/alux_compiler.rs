//! # Alux Compiler
//!
//! Alux script compiler integration for Alux projects.

use std::path::PathBuf;
use std::process::Stdio;

use async_trait::async_trait;
use tokio::process::Command;
use tracing::{debug, error, info};

use crate::build::Builder;
use crate::core::{Result, XyluxError};

/// Alux compiler for Alux script projects.
pub struct AluxCompiler {
    /// Alux compiler binary path.
    alux_path: String,
    /// Alux VM binary path.
    alux_vm_path: String,
}

impl AluxCompiler {
    /// Create a new Alux compiler.
    pub fn new() -> Self {
        Self { alux_path: "alux-compile".to_string(), alux_vm_path: "alux-vm".to_string() }
    }

    /// Create a new Alux compiler with custom paths.
    pub fn with_paths<S1: Into<String>, S2: Into<String>>(alux_path: S1, alux_vm_path: S2) -> Self {
        Self { alux_path: alux_path.into(), alux_vm_path: alux_vm_path.into() }
    }

    /// Execute an alux compiler command.
    async fn execute_alux_command(&self, project_root: &PathBuf, args: &[&str]) -> Result<String> {
        debug!("Executing {} {} in {}", self.alux_path, args.join(" "), project_root.display());

        let mut command = Command::new(&self.alux_path);
        command.args(args).current_dir(project_root).stdout(Stdio::piped()).stderr(Stdio::piped());

        let output = command.output().await.map_err(|e| {
            XyluxError::build_error(format!("Failed to execute alux compiler: {}", e))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            if !stdout.is_empty() {
                info!("Alux compiler output: {}", stdout.trim());
            }
            Ok(stdout.to_string())
        } else {
            let error_msg = if !stderr.is_empty() {
                stderr.to_string()
            } else {
                format!("Alux compiler failed with exit code: {:?}", output.status.code())
            };
            error!("Alux compiler error: {}", error_msg.trim());
            Err(XyluxError::build_error(error_msg))
        }
    }

    /// Execute an alux VM command.
    async fn execute_alux_vm_command(
        &self,
        project_root: &PathBuf,
        args: &[&str],
    ) -> Result<String> {
        debug!("Executing {} {} in {}", self.alux_vm_path, args.join(" "), project_root.display());

        let mut command = Command::new(&self.alux_vm_path);
        command.args(args).current_dir(project_root).stdout(Stdio::piped()).stderr(Stdio::piped());

        let output = command
            .output()
            .await
            .map_err(|e| XyluxError::build_error(format!("Failed to execute alux VM: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            if !stdout.is_empty() {
                info!("Alux VM output: {}", stdout.trim());
            }
            Ok(stdout.to_string())
        } else {
            let error_msg = if !stderr.is_empty() {
                stderr.to_string()
            } else {
                format!("Alux VM failed with exit code: {:?}", output.status.code())
            };
            error!("Alux VM error: {}", error_msg.trim());
            Err(XyluxError::build_error(error_msg))
        }
    }

    /// Check if the project has Alux scripts.
    pub fn is_alux_project(project_root: &PathBuf) -> bool {
        project_root.join("scripts").exists()
            || project_root.join("main.aux").exists()
            || project_root.join("src").join("main.aux").exists()
    }

    /// Find Alux script files in the project.
    pub async fn find_alux_scripts(&self, project_root: &PathBuf) -> Result<Vec<PathBuf>> {
        let mut scripts = Vec::new();

        // Check for main.aux in root
        let main_aux = project_root.join("main.aux");
        if main_aux.exists() {
            scripts.push(main_aux);
        }

        // Check for scripts in scripts directory
        let scripts_dir = project_root.join("scripts");
        if scripts_dir.exists() {
            let mut entries = tokio::fs::read_dir(&scripts_dir)
                .await
                .map_err(|e| XyluxError::io(e, "Failed to read scripts directory"))?;

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| XyluxError::io(e, "Failed to read directory entry"))?
            {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("aux") {
                    scripts.push(path);
                }
            }
        }

        // Check for scripts in src directory
        let src_dir = project_root.join("src");
        if src_dir.exists() {
            let mut entries = tokio::fs::read_dir(&src_dir)
                .await
                .map_err(|e| XyluxError::io(e, "Failed to read src directory"))?;

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| XyluxError::io(e, "Failed to read directory entry"))?
            {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("aux") {
                    scripts.push(path);
                }
            }
        }

        Ok(scripts)
    }

    /// Check if Alux compiler is available.
    pub async fn check_compiler_availability(&self) -> Result<String> {
        let output = Command::new(&self.alux_path)
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| XyluxError::build_error(format!("Alux compiler not found: {}", e)))?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            Ok(version.trim().to_string())
        } else {
            Err(XyluxError::build_error("Alux compiler is not available or not working"))
        }
    }

    /// Check if Alux VM is available.
    pub async fn check_vm_availability(&self) -> Result<String> {
        let output = Command::new(&self.alux_vm_path)
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| XyluxError::build_error(format!("Alux VM not found: {}", e)))?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            Ok(version.trim().to_string())
        } else {
            Err(XyluxError::build_error("Alux VM is not available or not working"))
        }
    }

    /// Compile a single Alux script.
    pub async fn compile_script(&self, script_path: &PathBuf) -> Result<PathBuf> {
        let output_path = script_path.with_extension("auxc");

        let script_str =
            script_path.to_str().ok_or_else(|| XyluxError::invalid_data("Invalid script path"))?;
        let output_str =
            output_path.to_str().ok_or_else(|| XyluxError::invalid_data("Invalid output path"))?;

        let args = vec![script_str, "-o", output_str];

        self.execute_alux_command(
            &script_path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf(),
            &args,
        )
        .await?;

        Ok(output_path)
    }

    /// Compile all Alux scripts in a project.
    pub async fn compile_project(&self, project_root: &PathBuf) -> Result<()> {
        info!("Compiling Alux project at: {}", project_root.display());

        if !Self::is_alux_project(project_root) {
            return Err(XyluxError::build_error("Not an Alux project (no .aux files found)"));
        }

        let scripts = self.find_alux_scripts(project_root).await?;

        if scripts.is_empty() {
            return Err(XyluxError::build_error("No Alux scripts found to compile"));
        }

        for script in &scripts {
            info!("Compiling script: {}", script.display());
            self.compile_script(script).await?;
        }

        info!("Successfully compiled {} Alux scripts", scripts.len());
        Ok(())
    }

    /// Run a compiled Alux script.
    pub async fn run_script(&self, bytecode_path: &PathBuf) -> Result<()> {
        let bytecode_str = bytecode_path
            .to_str()
            .ok_or_else(|| XyluxError::invalid_data("Invalid bytecode path"))?;

        let args = vec![bytecode_str];

        self.execute_alux_vm_command(
            &bytecode_path.parent().unwrap_or(&PathBuf::from(".")).to_path_buf(),
            &args,
        )
        .await?;

        Ok(())
    }

    /// Run the main script of an Alux project.
    pub async fn run_project(&self, project_root: &PathBuf) -> Result<()> {
        info!("Running Alux project at: {}", project_root.display());

        // Look for main.auxc (compiled bytecode)
        let main_bytecode = project_root.join("main.auxc");
        if main_bytecode.exists() {
            return self.run_script(&main_bytecode).await;
        }

        // Look for main.auxc in scripts directory
        let scripts_main_bytecode = project_root.join("scripts").join("main.auxc");
        if scripts_main_bytecode.exists() {
            return self.run_script(&scripts_main_bytecode).await;
        }

        // If no compiled bytecode found, try to compile first
        let main_script = project_root.join("main.aux");
        if main_script.exists() {
            let bytecode = self.compile_script(&main_script).await?;
            return self.run_script(&bytecode).await;
        }

        let scripts_main_script = project_root.join("scripts").join("main.aux");
        if scripts_main_script.exists() {
            let bytecode = self.compile_script(&scripts_main_script).await?;
            return self.run_script(&bytecode).await;
        }

        Err(XyluxError::build_error("No main script found to run"))
    }

    /// Clean compiled bytecode files.
    pub async fn clean_project(&self, project_root: &PathBuf) -> Result<()> {
        info!("Cleaning Alux project at: {}", project_root.display());

        let scripts = self.find_alux_scripts(project_root).await?;

        for script in &scripts {
            let bytecode_path = script.with_extension("auxc");
            if bytecode_path.exists() {
                tokio::fs::remove_file(&bytecode_path).await.map_err(|e| {
                    XyluxError::io(e, &format!("Failed to remove {}", bytecode_path.display()))
                })?;
                debug!("Removed: {}", bytecode_path.display());
            }
        }

        // Also check for any .auxc files in common directories
        for dir_name in &[".", "scripts", "src"] {
            let dir_path = project_root.join(dir_name);
            if dir_path.exists() {
                if let Ok(mut entries) = tokio::fs::read_dir(&dir_path).await {
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let path = entry.path();
                        if path.is_file()
                            && path.extension().and_then(|ext| ext.to_str()) == Some("auxc")
                        {
                            tokio::fs::remove_file(&path).await.map_err(|e| {
                                XyluxError::io(e, &format!("Failed to remove {}", path.display()))
                            })?;
                            debug!("Removed: {}", path.display());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Format Alux script files.
    pub async fn format_project(&self, project_root: &PathBuf) -> Result<()> {
        let scripts = self.find_alux_scripts(project_root).await?;

        for script in &scripts {
            let script_str =
                script.to_str().ok_or_else(|| XyluxError::invalid_data("Invalid script path"))?;

            let args = vec!["fmt", script_str];
            self.execute_alux_command(project_root, &args).await?;
        }

        Ok(())
    }

    /// Check Alux scripts for syntax errors.
    pub async fn check_project(&self, project_root: &PathBuf) -> Result<()> {
        let scripts = self.find_alux_scripts(project_root).await?;

        for script in &scripts {
            let script_str =
                script.to_str().ok_or_else(|| XyluxError::invalid_data("Invalid script path"))?;

            let args = vec!["check", script_str];
            self.execute_alux_command(project_root, &args).await?;
        }

        Ok(())
    }

    /// Get Alux compiler information.
    pub async fn get_compiler_info(&self) -> Result<serde_json::Value> {
        let mut info = serde_json::Map::new();

        if let Ok(compiler_version) = self.check_compiler_availability().await {
            info.insert(
                "compiler_version".to_string(),
                serde_json::Value::String(compiler_version),
            );
        }

        if let Ok(vm_version) = self.check_vm_availability().await {
            info.insert("vm_version".to_string(), serde_json::Value::String(vm_version));
        }

        info.insert("compiler_path".to_string(), serde_json::Value::String(self.alux_path.clone()));
        info.insert("vm_path".to_string(), serde_json::Value::String(self.alux_vm_path.clone()));

        Ok(serde_json::Value::Object(info))
    }
}

impl Default for AluxCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Builder for AluxCompiler {
    async fn build(&self, project_root: &PathBuf) -> Result<()> {
        self.compile_project(project_root).await
    }

    async fn run(&self, project_root: &PathBuf) -> Result<()> {
        self.run_project(project_root).await
    }

    async fn test(&self, project_root: &PathBuf) -> Result<()> {
        // Alux doesn't have a standard test framework yet
        // For now, we just check the scripts for syntax errors
        self.check_project(project_root).await
    }

    async fn clean(&self, project_root: &PathBuf) -> Result<()> {
        self.clean_project(project_root).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    async fn create_test_alux_project(temp_dir: &TempDir) -> PathBuf {
        let project_path = temp_dir.path().to_path_buf();

        // Create scripts directory and main script
        fs::create_dir_all(project_path.join("scripts")).await.unwrap();
        fs::write(
            project_path.join("scripts").join("main.aux"),
            r#"// Test Alux script
fn main() {
    print("Hello from Alux!");
}

fn fibonacci(n: i32) -> i32 {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn test_fibonacci() {
    let result = fibonacci(10);
    print("Fibonacci(10) = " + result.to_string());
}
"#,
        )
        .await
        .unwrap();

        // Create another script
        fs::write(
            project_path.join("scripts").join("utils.aux"),
            r#"// Utility functions
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn multiply(a: i32, b: i32) -> i32 {
    return a * b;
}
"#,
        )
        .await
        .unwrap();

        project_path
    }

    #[test]
    fn test_alux_compiler_creation() {
        let compiler = AluxCompiler::new();
        assert_eq!(compiler.alux_path, "alux-compile");
        assert_eq!(compiler.alux_vm_path, "alux-vm");

        let custom_compiler = AluxCompiler::with_paths("/usr/bin/alux-compile", "/usr/bin/alux-vm");
        assert_eq!(custom_compiler.alux_path, "/usr/bin/alux-compile");
        assert_eq!(custom_compiler.alux_vm_path, "/usr/bin/alux-vm");
    }

    #[test]
    fn test_is_alux_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Should not be an Alux project initially
        assert!(!AluxCompiler::is_alux_project(&project_path));

        // Create scripts directory
        std::fs::create_dir_all(project_path.join("scripts")).unwrap();
        assert!(AluxCompiler::is_alux_project(&project_path));

        // Test with main.aux in root
        let project_path2 = temp_dir.path().join("project2");
        std::fs::create_dir_all(&project_path2).unwrap();
        std::fs::write(project_path2.join("main.aux"), "// test").unwrap();
        assert!(AluxCompiler::is_alux_project(&project_path2));
    }

    #[tokio::test]
    async fn test_find_alux_scripts() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = create_test_alux_project(&temp_dir).await;

        let compiler = AluxCompiler::new();
        let scripts = compiler.find_alux_scripts(&project_path).await.unwrap();

        assert_eq!(scripts.len(), 2);
        assert!(scripts.iter().any(|p| p.file_name().unwrap() == "main.aux"));
        assert!(scripts.iter().any(|p| p.file_name().unwrap() == "utils.aux"));
    }

    #[tokio::test]
    async fn test_alux_availability() {
        let compiler = AluxCompiler::new();

        // These tests may fail in environments without Alux tools
        // We just test that the methods work, regardless of result
        let _unused = compiler.check_compiler_availability().await;
        let _unused = compiler.check_vm_availability().await;
    }

    #[tokio::test]
    async fn test_build_non_alux_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        let compiler = AluxCompiler::new();
        let result = compiler.build(&project_path).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not an Alux project"));
    }

    #[tokio::test]
    async fn test_clean_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = create_test_alux_project(&temp_dir).await;

        // Create some fake bytecode files
        fs::write(project_path.join("scripts").join("main.auxc"), "fake bytecode").await.unwrap();
        fs::write(project_path.join("scripts").join("utils.auxc"), "fake bytecode").await.unwrap();

        let compiler = AluxCompiler::new();
        let result = compiler.clean_project(&project_path).await;

        assert!(result.is_ok());

        // Check that bytecode files were removed
        assert!(!project_path.join("scripts").join("main.auxc").exists());
        assert!(!project_path.join("scripts").join("utils.auxc").exists());
    }

    #[tokio::test]
    async fn test_compiler_info() {
        let compiler = AluxCompiler::new();
        let info_result = compiler.get_compiler_info().await;

        assert!(info_result.is_ok());

        if let Ok(info) = info_result {
            assert!(info.is_object());
            assert!(info.get("compiler_path").is_some());
            assert!(info.get("vm_path").is_some());
        }
    }

    #[tokio::test]
    async fn test_empty_scripts_directory() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create empty scripts directory
        fs::create_dir_all(project_path.join("scripts")).await.unwrap();

        let compiler = AluxCompiler::new();
        let scripts = compiler.find_alux_scripts(&project_path).await.unwrap();

        assert!(scripts.is_empty());

        // Should fail to compile
        let result = compiler.compile_project(&project_path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No Alux scripts found"));
    }
}
