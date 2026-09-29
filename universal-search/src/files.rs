use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    File,
    Directory,
}

#[derive(Debug, Clone)]
pub struct FileSearchResult {
    pub name: String,
    pub path: String,
    pub file_type: FileType,
}

impl FileSearchResult {
    pub fn new(name: String, path: String, file_type: FileType) -> Self {
        Self {
            name,
            path,
            file_type,
        }
    }
}

#[derive(Clone)]
pub struct FileSearcher {
    exclude_dirs: Vec<String>,
}

impl FileSearcher {
    pub fn new() -> Self {
        Self {
            exclude_dirs: vec![
                "/Code".to_string(),
                "/node_modules".to_string(),
                "/go".to_string(),
                "/bin".to_string(),
                "Amazfit".to_string(),
            ],
        }
    }

    pub fn search(&self, query: &str) -> Result<Vec<FileSearchResult>, Box<dyn std::error::Error>> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(vec![]);
        }

        let home_dir = std::env::var("HOME")?;

        let mut cmd = Command::new("fd");
        cmd.arg("--ignore-case")
            .arg("--color")
            .arg("never")
            .arg("--absolute-path")
            .arg("--type")
            .arg("d")
            .arg("--type")
            .arg("f");

        for exclude in &self.exclude_dirs {
            cmd.arg("--exclude").arg(exclude);
        }

        cmd.arg(query).arg(&home_dir);

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("fd command failed: {}", stderr).into());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut results = self.parse_fd_output(&stdout);

        self.sort_results(&mut results, query);

        Ok(results)
    }

    fn parse_fd_output(&self, output: &str) -> Vec<FileSearchResult> {
        if output.is_empty() {
            return vec![];
        }

        output
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let path_str = line.trim_end_matches('/');
                let path = Path::new(path_str);
                let file_type = if line.ends_with('/') || path.is_dir() {
                    FileType::Directory
                } else {
                    FileType::File
                };

                let name = path
                    .file_name()
                    .and_then(OsStr::to_str)
                    .unwrap_or(path_str)
                    .to_string();

                FileSearchResult::new(name, line.to_string(), file_type)
            })
            .collect()
    }

    fn sort_results(&self, results: &mut [FileSearchResult], query: &str) {
        let query_lower = query.to_lowercase();

        results.sort_by(|a, b| {
            let a_exact = a.name.to_lowercase() == query_lower;
            let b_exact = b.name.to_lowercase() == query_lower;

            if a_exact && !b_exact {
                return std::cmp::Ordering::Less;
            }
            if !a_exact && b_exact {
                return std::cmp::Ordering::Greater;
            }

            let a_starts = a.name.to_lowercase().starts_with(&query_lower);
            let b_starts = b.name.to_lowercase().starts_with(&query_lower);

            if a_starts && !b_starts {
                return std::cmp::Ordering::Less;
            }
            if !a_starts && b_starts {
                return std::cmp::Ordering::Greater;
            }

            if a.file_type != b.file_type {
                match (&a.file_type, &b.file_type) {
                    (FileType::Directory, FileType::File) => return std::cmp::Ordering::Less,
                    (FileType::File, FileType::Directory) => return std::cmp::Ordering::Greater,
                    _ => {}
                }
            }

            if a.name.len() != b.name.len() {
                return a.name.len().cmp(&b.name.len());
            }

            a.name.cmp(&b.name)
        });
    }
}

impl Default for FileSearcher {
    fn default() -> Self {
        Self::new()
    }
}
