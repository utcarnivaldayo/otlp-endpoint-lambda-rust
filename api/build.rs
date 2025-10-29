use std::process::{Command, Output};

fn vcs_ref_head_name() {
    const VCS_REF_HEAD_NAME: &str = "VCS_REF_HEAD_NAME";
    let output: Output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("failed to execute git");
    let vcs_ref_head_name: String = String::from_utf8(output.stdout).unwrap();
    println!(
        "cargo:rustc-env={}={}",
        VCS_REF_HEAD_NAME,
        vcs_ref_head_name.trim()
    );
}

fn vcs_ref_head_revision() {
    const VCS_REF_HEAD_REVISION: &str = "VCS_REF_HEAD_REVISION";
    let output: Output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("failed to execute git");
    let vcs_ref_head_revision: String = String::from_utf8(output.stdout).unwrap();
    println!(
        "cargo:rustc-env={}={}",
        VCS_REF_HEAD_REVISION,
        vcs_ref_head_revision.trim()
    );
}

fn vcs_repository_url_full() {
    use git_url_parse::GitUrl;
    use git_url_parse::types::provider::GenericProvider;
    const VCS_REPOSITORY_URL_FULL: &str = "VCS_REPOSITORY_URL_FULL";
    const VCS_REPOSITORY_NAME: &str = "VCS_REPOSITORY_NAME";
    let output: Output = Command::new("git")
        .args(&["config", "--get", "remote.origin.url"])
        .output()
        .expect("failed to execute git");

    if output.stdout.is_empty() {
        println!(
            "cargo:rustc-env={}={}",
            VCS_REPOSITORY_URL_FULL,
            ""
        );
        println!(
            "cargo:rustc-env={}={}",
            VCS_REPOSITORY_NAME,
            ""
        );
        return;
    }

    let git_remote_url: GitUrl = GitUrl::parse(&String::from_utf8(output.stdout).unwrap().trim()).unwrap();
    let generic_provider: GenericProvider = git_remote_url.provider_info().unwrap();
    println!(
        "cargo:rustc-env={}={}",
        VCS_REPOSITORY_URL_FULL,
        format!("https://github.com/{}", generic_provider.fullname())
    );
    println!(
        "cargo:rustc-env={}={}",
        VCS_REPOSITORY_NAME,
        generic_provider.repo()
    );
}

fn project_name() {
    const PROJECT_NAME: &str = "PROJECT_NAME";
    let project_name: String =
        std::env::var(PROJECT_NAME).unwrap_or(String::new());
    println!("cargo:rustc-env={}={}", PROJECT_NAME, project_name.trim());
}

fn pulumi_stack() {
    const PULUMI_STACK: &str = "PULUMI_STACK";
    let pulumi_stack: String = std::env::var(PULUMI_STACK).unwrap_or("dev".to_string());
    println!("cargo:rustc-env={}={}", PULUMI_STACK, pulumi_stack.trim());
}

fn telemetry_sdk_version() {
    const TELEMETRY_SDK_VERSION: &str = "TELEMETRY_SDK_VERSION";
    let telemetry_sdk_version: &str = "0.31.0";
    println!(
        "cargo:rustc-env={}={}",
        TELEMETRY_SDK_VERSION, telemetry_sdk_version.trim()
    );
}

fn api_base_path() {
    const API_BASE_PATH: &str = "API_BASE_PATH";
    let api_base_path: String = std::env::var(API_BASE_PATH).unwrap_or("/api".to_string());
    println!("cargo:rustc-env={}={}", API_BASE_PATH, api_base_path.trim());
}

fn api_lambda_arn() {
    const API_LAMBDA_ARN: &str = "API_LAMBDA_ARN";
    let api_lambda_arn: String = std::env::var(API_LAMBDA_ARN).unwrap_or(String::new());
    println!("cargo:rustc-env={}={}", API_LAMBDA_ARN, api_lambda_arn.trim());
}

fn remote_endpoint() {
    const REMOTE_ENDPOINT: &str = "REMOTE_ENDPOINT";
    let remote_endpoint: String =
        std::env::var(REMOTE_ENDPOINT).unwrap_or("http://localhost:3030".to_string());
    println!("cargo:rustc-env={}={}", REMOTE_ENDPOINT, remote_endpoint.trim());
}

fn main() {
    vcs_ref_head_name();
    vcs_ref_head_revision();
    vcs_repository_url_full();
    pulumi_stack();
    project_name();
    telemetry_sdk_version();
    api_base_path();
    api_lambda_arn();
    remote_endpoint();
}
