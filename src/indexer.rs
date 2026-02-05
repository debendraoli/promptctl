//! Project indexing to analyze codebase structure and detect technologies.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Detected project information
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ProjectIndex {
    /// Root directory of the project
    pub root: PathBuf,
    /// Detected programming languages with file counts
    pub languages: HashMap<String, LanguageInfo>,
    /// Detected frameworks and libraries
    pub frameworks: Vec<Framework>,
    /// Project configuration files found
    pub config_files: Vec<PathBuf>,
    /// Directory structure summary
    pub structure: ProjectStructure,
}

#[derive(Debug, Clone, Default)]
pub struct LanguageInfo {
    pub name: String,
    pub file_count: usize,
    pub extensions: HashSet<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Framework {
    pub name: String,
    pub category: FrameworkCategory,
    pub config_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FrameworkCategory {
    Web,
    Cli,
    Library,
    Testing,
    Build,
    Database,
    Other,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectStructure {
    pub has_src: bool,
    pub has_tests: bool,
    pub has_docs: bool,
    pub has_ci: bool,
    pub top_level_dirs: Vec<String>,
}

impl ProjectIndex {
    /// Index a project starting from the given root directory
    pub fn scan(root: &Path) -> Self {
        let mut index = Self {
            root: root.to_path_buf(),
            ..Default::default()
        };

        index.scan_directory(root, 0);
        index.detect_frameworks(root);
        index.scan_structure(root);

        index
    }

    fn scan_directory(&mut self, dir: &Path, depth: usize) {
        // Limit depth to avoid huge repos
        if depth > 10 {
            return;
        }

        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden and common ignored directories
            if name.starts_with('.')
                || matches!(
                    name.as_str(),
                    "node_modules"
                        | "target"
                        | "vendor"
                        | "dist"
                        | "build"
                        | "__pycache__"
                        | ".git"
                )
            {
                continue;
            }

            if path.is_dir() {
                self.scan_directory(&path, depth + 1);
            } else if path.is_file() {
                self.process_file(&path);
            }
        }
    }

    fn process_file(&mut self, path: &Path) {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let (lang, ext_str) = match ext {
                "rs" => ("rust", "rs"),
                "go" => ("go", "go"),
                "leo" => ("leo", "leo"),
                "aleo" => ("leo", "aleo"),
                "py" | "pyi" => ("python", ext),
                "ts" | "tsx" => ("typescript", ext),
                "js" | "jsx" | "mjs" | "cjs" => ("javascript", ext),
                "rb" => ("ruby", "rb"),
                "java" => ("java", "java"),
                "kt" | "kts" => ("kotlin", ext),
                "swift" => ("swift", "swift"),
                "c" | "h" => ("c", ext),
                "cpp" | "cc" | "cxx" | "hpp" => ("cpp", ext),
                "zig" => ("zig", "zig"),
                "ex" | "exs" => ("elixir", ext),
                "erl" | "hrl" => ("erlang", ext),
                "hs" => ("haskell", "hs"),
                "ml" | "mli" => ("ocaml", ext),
                "scala" | "sc" => ("scala", ext),
                "clj" | "cljs" | "cljc" => ("clojure", ext),
                "lua" => ("lua", "lua"),
                "sh" | "bash" | "zsh" => ("shell", ext),
                "sql" => ("sql", "sql"),
                "proto" => ("protobuf", "proto"),
                "graphql" | "gql" => ("graphql", ext),
                _ => return,
            };

            let info = self
                .languages
                .entry(lang.to_string())
                .or_insert_with(|| LanguageInfo {
                    name: lang.to_string(),
                    ..Default::default()
                });

            info.file_count += 1;
            info.extensions.insert(ext_str.to_string());
        }

        // Track config files
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if is_config_file(filename) {
            self.config_files.push(path.to_path_buf());
        }
    }

    fn detect_frameworks(&mut self, root: &Path) {
        // Rust frameworks
        if root.join("Cargo.toml").exists() {
            if let Ok(content) = fs::read_to_string(root.join("Cargo.toml")) {
                self.detect_rust_frameworks(&content, root);
            }
        }

        // Go frameworks
        if root.join("go.mod").exists() {
            if let Ok(content) = fs::read_to_string(root.join("go.mod")) {
                self.detect_go_frameworks(&content, root);
            }
        }

        // Leo/Aleo project
        if root.join("program.json").exists() {
            self.detect_leo_project(root);
        }

        // Node.js/JavaScript frameworks
        if root.join("package.json").exists() {
            if let Ok(content) = fs::read_to_string(root.join("package.json")) {
                self.detect_node_frameworks(&content, root);
            }
        }

        // Python frameworks
        if root.join("pyproject.toml").exists() || root.join("requirements.txt").exists() {
            self.detect_python_frameworks(root);
        }
    }

    fn detect_leo_project(&mut self, root: &Path) {
        // Add Leo language if not already detected
        let lang = self
            .languages
            .entry("leo".to_string())
            .or_insert_with(|| LanguageInfo {
                name: "leo".to_string(),
                ..Default::default()
            });

        // Try to detect version from program.json
        if let Ok(content) = fs::read_to_string(root.join("program.json")) {
            // Look for "version" field in program.json
            if let Some(line) = content.lines().find(|l| l.contains("\"version\"")) {
                if let Some(ver) = line.split('"').nth(3) {
                    lang.version = Some(ver.to_string());
                }
            }
        }

        self.frameworks.push(Framework {
            name: "Aleo".to_string(),
            category: FrameworkCategory::Other,
            config_file: Some(root.join("program.json")),
        });
    }

    fn detect_rust_frameworks(&mut self, content: &str, root: &Path) {
        // Detect version
        if let Some(lang) = self.languages.get_mut("rust") {
            if let Some(line) = content.lines().find(|l| l.contains("rust-version")) {
                if let Some(ver) = line.split('"').nth(1) {
                    lang.version = Some(ver.to_string());
                }
            }
        }

        let frameworks_to_detect = [
            ("tokio", "Tokio", FrameworkCategory::Web),
            ("axum", "Axum", FrameworkCategory::Web),
            ("actix-web", "Actix Web", FrameworkCategory::Web),
            ("rocket", "Rocket", FrameworkCategory::Web),
            ("warp", "Warp", FrameworkCategory::Web),
            ("hyper", "Hyper", FrameworkCategory::Web),
            ("clap", "Clap", FrameworkCategory::Cli),
            ("serde", "Serde", FrameworkCategory::Library),
            ("sqlx", "SQLx", FrameworkCategory::Database),
            ("diesel", "Diesel", FrameworkCategory::Database),
            ("sea-orm", "SeaORM", FrameworkCategory::Database),
            ("tracing", "Tracing", FrameworkCategory::Library),
            ("anyhow", "Anyhow", FrameworkCategory::Library),
            ("thiserror", "Thiserror", FrameworkCategory::Library),
        ];

        for (key, name, category) in frameworks_to_detect {
            if content.contains(key) {
                self.frameworks.push(Framework {
                    name: name.to_string(),
                    category,
                    config_file: Some(root.join("Cargo.toml")),
                });
            }
        }
    }

    fn detect_go_frameworks(&mut self, content: &str, root: &Path) {
        // Detect version
        if let Some(lang) = self.languages.get_mut("go") {
            if let Some(line) = content.lines().find(|l| l.starts_with("go ")) {
                lang.version = Some(line.trim_start_matches("go ").trim().to_string());
            }
        }

        let frameworks_to_detect = [
            ("github.com/gin-gonic/gin", "Gin", FrameworkCategory::Web),
            ("github.com/labstack/echo", "Echo", FrameworkCategory::Web),
            ("github.com/gofiber/fiber", "Fiber", FrameworkCategory::Web),
            ("github.com/gorilla/mux", "Gorilla Mux", FrameworkCategory::Web),
            ("github.com/go-chi/chi", "Chi", FrameworkCategory::Web),
            ("github.com/spf13/cobra", "Cobra", FrameworkCategory::Cli),
            ("github.com/urfave/cli", "urfave/cli", FrameworkCategory::Cli),
            ("gorm.io/gorm", "GORM", FrameworkCategory::Database),
            ("github.com/jmoiron/sqlx", "sqlx", FrameworkCategory::Database),
            ("entgo.io/ent", "Ent", FrameworkCategory::Database),
            ("github.com/stretchr/testify", "Testify", FrameworkCategory::Testing),
        ];

        for (key, name, category) in frameworks_to_detect {
            if content.contains(key) {
                self.frameworks.push(Framework {
                    name: name.to_string(),
                    category,
                    config_file: Some(root.join("go.mod")),
                });
            }
        }
    }

    fn detect_node_frameworks(&mut self, content: &str, root: &Path) {
        let frameworks_to_detect = [
            ("react", "React", FrameworkCategory::Web),
            ("next", "Next.js", FrameworkCategory::Web),
            ("vue", "Vue", FrameworkCategory::Web),
            ("nuxt", "Nuxt", FrameworkCategory::Web),
            ("svelte", "Svelte", FrameworkCategory::Web),
            ("express", "Express", FrameworkCategory::Web),
            ("fastify", "Fastify", FrameworkCategory::Web),
            ("nestjs", "NestJS", FrameworkCategory::Web),
            ("hono", "Hono", FrameworkCategory::Web),
            ("prisma", "Prisma", FrameworkCategory::Database),
            ("drizzle", "Drizzle", FrameworkCategory::Database),
            ("jest", "Jest", FrameworkCategory::Testing),
            ("vitest", "Vitest", FrameworkCategory::Testing),
            ("mocha", "Mocha", FrameworkCategory::Testing),
            ("commander", "Commander", FrameworkCategory::Cli),
            ("yargs", "Yargs", FrameworkCategory::Cli),
        ];

        for (key, name, category) in frameworks_to_detect {
            if content.contains(&format!("\"{key}\"")) || content.contains(&format!("'{key}'")) {
                self.frameworks.push(Framework {
                    name: name.to_string(),
                    category,
                    config_file: Some(root.join("package.json")),
                });
            }
        }
    }

    fn detect_python_frameworks(&mut self, root: &Path) {
        let files_to_check = ["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"];
        let mut content = String::new();

        for file in files_to_check {
            if let Ok(c) = fs::read_to_string(root.join(file)) {
                content.push_str(&c);
                content.push('\n');
            }
        }

        let frameworks_to_detect = [
            ("django", "Django", FrameworkCategory::Web),
            ("flask", "Flask", FrameworkCategory::Web),
            ("fastapi", "FastAPI", FrameworkCategory::Web),
            ("starlette", "Starlette", FrameworkCategory::Web),
            ("pytest", "Pytest", FrameworkCategory::Testing),
            ("sqlalchemy", "SQLAlchemy", FrameworkCategory::Database),
            ("pydantic", "Pydantic", FrameworkCategory::Library),
            ("click", "Click", FrameworkCategory::Cli),
            ("typer", "Typer", FrameworkCategory::Cli),
        ];

        for (key, name, category) in frameworks_to_detect {
            if content.to_lowercase().contains(key) {
                self.frameworks.push(Framework {
                    name: name.to_string(),
                    category,
                    config_file: None,
                });
            }
        }
    }

    fn scan_structure(&mut self, root: &Path) {
        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.filter_map(Result::ok) {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if !name.starts_with('.') {
                            self.structure.top_level_dirs.push(name.clone());

                            match name.as_str() {
                                "src" | "lib" => self.structure.has_src = true,
                                "test" | "tests" | "spec" | "__tests__" => {
                                    self.structure.has_tests = true
                                }
                                "docs" | "doc" | "documentation" => self.structure.has_docs = true,
                                ".github" | ".gitlab-ci" | ".circleci" => {
                                    self.structure.has_ci = true
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // Check for CI files
        let ci_files = [
            ".github/workflows",
            ".gitlab-ci.yml",
            ".circleci/config.yml",
            "Jenkinsfile",
            ".travis.yml",
        ];
        for ci in ci_files {
            if root.join(ci).exists() {
                self.structure.has_ci = true;
                break;
            }
        }
    }

    /// Get the primary language (most files)
    pub fn primary_language(&self) -> Option<&LanguageInfo> {
        self.languages.values().max_by_key(|l| l.file_count)
    }

    /// Generate a project context summary for prompts
    pub fn to_context_string(&self) -> String {
        let mut parts = vec![];

        // Languages
        if !self.languages.is_empty() {
            let mut langs: Vec<_> = self.languages.values().collect();
            langs.sort_by(|a, b| b.file_count.cmp(&a.file_count));

            let lang_strs: Vec<String> = langs
                .iter()
                .take(3)
                .map(|l| {
                    if let Some(ref v) = l.version {
                        format!("{} {v}", l.name)
                    } else {
                        l.name.clone()
                    }
                })
                .collect();
            parts.push(format!("Languages: {}", lang_strs.join(", ")));
        }

        // Frameworks
        if !self.frameworks.is_empty() {
            let fw_names: Vec<_> = self.frameworks.iter().map(|f| f.name.as_str()).collect();
            parts.push(format!("Frameworks: {}", fw_names.join(", ")));
        }

        // Structure
        let mut structure_parts = vec![];
        if self.structure.has_tests {
            structure_parts.push("tests");
        }
        if self.structure.has_docs {
            structure_parts.push("docs");
        }
        if self.structure.has_ci {
            structure_parts.push("CI");
        }
        if !structure_parts.is_empty() {
            parts.push(format!("Has: {}", structure_parts.join(", ")));
        }

        parts.join("\n")
    }
}

fn is_config_file(name: &str) -> bool {
    matches!(
        name,
        "Cargo.toml"
            | "go.mod"
            | "package.json"
            | "program.json"
            | "pyproject.toml"
            | "requirements.txt"
            | "tsconfig.json"
            | "vite.config.ts"
            | "webpack.config.js"
            | "Makefile"
            | "Dockerfile"
            | "docker-compose.yml"
            | ".env"
            | ".env.example"
    )
}
