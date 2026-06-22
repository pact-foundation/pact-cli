#[test]
fn cli_tests() {
    let cases = trycmd::TestCases::new();
    cases.case("tests/cmd/*.toml");

    // On Windows, trycmd renders the binary name as `pact[EXE]` in clap's
    // `Usage:` lines, which would not match the plain `pact` shown in the
    // README. Run the README help snapshots only where they match the
    // rendered documentation.
    #[cfg(not(windows))]
    cases.case("README.md");
}
