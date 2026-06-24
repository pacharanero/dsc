mod common;
use common::*;
use tempfile::TempDir;

#[test]
fn version_forum_reports_discourse_version_and_commit() {
    let Some(test) = test_discourse() else {
        return;
    };
    vprintln("e2e_version_forum: dsc version <forum> reads /about.json");
    let dir = TempDir::new().expect("tempdir");
    let config_path = write_temp_config(
        &dir,
        &format!(
            "[[discourse]]\nname = \"{}\"\nbaseurl = \"{}\"\napikey = \"{}\"\napi_username = \"{}\"\n",
            test.name, test.baseurl, test.apikey, test.api_username
        ),
    );
    let output = run_dsc(&["version", &test.name], &config_path);
    assert!(
        output.status.success(),
        "version <forum> failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&test.name) && stdout.contains("Discourse"),
        "expected '<forum>: Discourse <version> (<commit>)', got: {stdout}"
    );
}
