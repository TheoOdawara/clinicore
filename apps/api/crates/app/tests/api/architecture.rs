use std::fs;
use std::path::{Path, PathBuf};

const SQL_MARKERS: [&str; 4] = ["sqlx::query", "query!(", "query_as!(", "query_scalar!("];

fn violations(relative: &str, source: &str) -> Vec<String> {
    let mut found = Vec::new();
    let path = Path::new(relative);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let is_top_level_feature =
        path.parent() == Some(Path::new("")) && !matches!(file_name, "lib.rs" | "main.rs");
    let sql_allowed = file_name == "queries.rs" || is_top_level_feature;

    if !sql_allowed && SQL_MARKERS.iter().any(|marker| source.contains(marker)) {
        found.push(format!("{relative}: SQL outside queries.rs"));
    }
    if relative != "http/error.rs" && source.contains("IntoResponse for") {
        found.push(format!("{relative}: IntoResponse outside http/error.rs"));
    }
    if source.contains(".route(") {
        found.push(format!("{relative}: .route( instead of routes!"));
    }
    found
}

fn sources(folder: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(folder).expect("the source folder is readable") {
        let path = entry.expect("the entry is readable").path();
        if path.is_dir() {
            sources(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

#[test]
fn the_app_sources_respect_the_boundaries() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&root, &mut files);
    assert!(!files.is_empty());

    let found: Vec<String> = files
        .iter()
        .flat_map(|file| {
            let relative = file
                .strip_prefix(&root)
                .expect("the file is under src")
                .to_string_lossy()
                .replace('\\', "/");
            let source = fs::read_to_string(file).expect("the file is readable");
            violations(&relative, &source)
        })
        .collect();

    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn sql_is_refused_outside_queries_and_single_file_features() {
    let sql = "let row = sqlx::query_as!(Row, \"select 1\");";

    assert_eq!(violations("patients/handlers.rs", sql).len(), 1);
    assert_eq!(violations("http/error.rs", "query!(\"select 1\")").len(), 1);
    assert_eq!(violations("lib.rs", sql).len(), 1);
    assert!(violations("patients/queries.rs", sql).is_empty());
    assert!(violations("health.rs", sql).is_empty());
}

#[test]
fn into_response_is_refused_outside_the_error_module() {
    let source = "impl IntoResponse for PatientError {}";

    assert_eq!(violations("patients/error.rs", source).len(), 1);
    assert!(violations("http/error.rs", source).is_empty());
}

#[test]
fn a_bare_route_is_refused() {
    assert_eq!(
        violations("lib.rs", "Router::new().route(\"/x\", get(show))").len(),
        1
    );
    assert!(violations("lib.rs", "router.route_layer(layer)").is_empty());
}
