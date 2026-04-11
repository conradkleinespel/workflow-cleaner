use chrono::{DateTime, Duration as ChronoDuration, Utc};
use clap::Parser;
use humantime::Duration as HumanDuration;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use reqwest::Error;
use serde::Deserialize;
use std::env;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Delete workflow runs older than this duration (e.g., 30d, 5m, 4w)
    #[arg(long, value_name = "DURATION", default_value = "30d")]
    delete_older_than: HumanDuration,
}

#[derive(Deserialize)]
struct WorkflowRun {
    id: u64,
    created_at: String,
}

#[derive(Deserialize)]
struct WorkflowRunsResponse {
    workflow_runs: Vec<WorkflowRun>,
}

#[derive(Deserialize)]
struct Repo {
    full_name: String,
}

fn main() {
    let cli = Cli::parse();
    let token = env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set");

    let client = Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("token {}", token)).unwrap(),
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("rust-script"));

    let repos = fetch_repos(&client, &headers);
    let duration = ChronoDuration::from_std(*cli.delete_older_than).expect("Invalid duration");
    let cutoff_date = Utc::now() - duration;

    for repo in repos {
        println!("Looking at {}", repo);
        match delete_workflow_runs(client.clone(), headers.clone(), cutoff_date, repo.as_str()) {
            Ok(num) => println!("Deleted {} runs from {}", num, repo),
            Err(e) => eprintln!("Error deleting runs from {}: {}", repo, e),
        }
    }
}

fn fetch_repos(client: &Client, headers: &HeaderMap) -> Vec<String> {
    let mut all_repos = Vec::new();
    let mut page = 1;
    let per_page = 100;

    loop {
        let url = format!(
            "https://api.github.com/user/repos?per_page={}&page={}",
            per_page, page
        );
        let response = client
            .get(&url)
            .headers(headers.clone())
            .send()
            .expect("Failed to fetch repos");

        if [401u16, 403u16].contains(&response.status().as_u16()) {
            println!("Error: Invalid GITHUB_TOKEN: has it expired? does it have the correct permissions?");
            return vec![];
        }

        let repos: Vec<Repo> = response.json().expect("Failed to parse repos");

        if repos.is_empty() {
            break;
        }

        all_repos.extend(repos.into_iter().map(|repo| repo.full_name));
        page += 1;
    }

    all_repos
}

fn delete_workflow_runs(
    client: Client,
    headers: HeaderMap,
    cutoff_date: DateTime<Utc>,
    repo: &str,
) -> Result<i32, Error> {
    let mut all_runs_to_delete = Vec::new();
    let mut page = 1;
    let per_page = 100;

    loop {
        let url = format!(
            "https://api.github.com/repos/{}/actions/runs?per_page={}&page={}",
            repo, per_page, page
        );

        let response = client.get(&url).headers(headers.clone()).send()?;
        let json_object = response.json::<WorkflowRunsResponse>()?;

        if json_object.workflow_runs.is_empty() {
            break;
        }

        let mut has_old_runs = false;
        for run in &json_object.workflow_runs {
            let run_date = run.created_at.parse::<DateTime<Utc>>().unwrap();
            if run_date < cutoff_date {
                all_runs_to_delete.push(run.id);
                has_old_runs = true;
            }
        }

        // If we found NO old runs on this page, and since they are sorted descending
        // (newest first), all runs on subsequent pages will also be newer than cutoff.
        if !has_old_runs {
            break;
        }

        page += 1;
    }

    let mut deleted_workflow_runs_num = 0;
    for run_id in all_runs_to_delete {
        let delete_url = format!(
            "https://api.github.com/repos/{}/actions/runs/{}",
            repo, run_id
        );
        let response = client.delete(&delete_url).headers(headers.clone()).send()?;
        deleted_workflow_runs_num += 1;
        println!(
            "Deleted run {} from repo {}, response text = \"{}\"",
            run_id,
            repo,
            response.text()?
        );
    }

    Ok(deleted_workflow_runs_num)
}
