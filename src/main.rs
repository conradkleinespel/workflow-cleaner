use chrono::{DateTime, Duration, Utc};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use std::env;

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
    let token = env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set");

    let client = Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("token {}", token)).unwrap(),
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("rust-script"));

    let repos = fetch_repos(&client, &headers);

    let cutoff_date = Utc::now() - Duration::days(7);

    for repo in repos {
        println!("Looking at {}", repo);
        delete_workflow_runs(client.clone(), headers.clone(), cutoff_date, repo.as_str());
    }
}

fn fetch_repos(client: &Client, headers: &HeaderMap) -> Vec<String> {
    let url = "https://api.github.com/user/repos";
    let response = client
        .get(url)
        .headers(headers.clone())
        .send()
        .expect("Failed to fetch repos");
    let repos: Vec<Repo> = response.json().expect("Failed to parse repos");
    repos.into_iter().map(|repo| repo.full_name).collect()
}

fn delete_workflow_runs(
    client: Client,
    headers: HeaderMap,
    cutoff_date: DateTime<Utc>,
    repo: &str,
) {
    let url = format!("https://api.github.com/repos/{}/actions/runs", repo);

    let mut deleted_workflow_runs_num = 1;

    while deleted_workflow_runs_num > 0 {
        let response = client.get(&url).headers(headers.clone()).send();
        if response.is_err() {
            return;
        }

        let json_object = response.unwrap().json::<WorkflowRunsResponse>();
        if json_object.is_err() {
            return;
        }

        deleted_workflow_runs_num = 0;
        for run in json_object.unwrap().workflow_runs {
            let run_date = run.created_at.parse::<DateTime<Utc>>().unwrap();
            if run_date < cutoff_date {
                let delete_url = format!(
                    "https://api.github.com/repos/{}/actions/runs/{}",
                    repo, run.id
                );
                let response = client.delete(&delete_url).headers(headers.clone()).send();
                if response.is_ok() {
                    deleted_workflow_runs_num += 1;
                    println!(
                        "Deleted run {} from repo {}, response text = \"{}\"",
                        run.id,
                        repo,
                        response.unwrap().text().unwrap()
                    );
                } else {
                    println!("Deletion failed: {:?}", response.err().unwrap())
                }
            }
        }
    }
}
