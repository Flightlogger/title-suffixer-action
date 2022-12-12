use std::collections::HashSet;

use octocrab::Octocrab;
use regex::{Regex, RegexBuilder};

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN envvar is required");
    let octocrab = Octocrab::builder().personal_token(token).build().expect("Octocrab creation has to succeed");

    let reference = std::env::var("GITHUB_REF").expect("GITHUB_REF envvar is required");
    let pr_num = pr_num_from_gh_ref(&reference).expect("Failed to get PR number from GITHUB_REF");
    let pr = octocrab.pulls("Flightlogger", "flightlogger").get(pr_num).await?;
    let issue_nums = pr_issues_from_body(&pr.body.unwrap()).expect("Failed to get issue number(s) from PR body");
    let suffix = issue_nums.iter().map(|num| format!("[#{}]", num)).collect::<Vec<String>>().join(" ");
    let title_re = Regex::new(r"\s?\[#\d+\]\s?").unwrap();
    let old_title_sanitized = title_re.replace_all(&pr.title.unwrap(), "").to_string();
    let new_title = format!("{} {}", old_title_sanitized, suffix);
    let updated = octocrab.issues("Flightlogger", "flightlogger").update(pr_num).title(&new_title).send().await?;
    println!("Updated PR title to: {}", updated.title);
    Ok(())
}

fn pr_num_from_gh_ref(gh_ref: &str) -> Result<u64, &'static str> {
    let re = Regex::new(r"\d+").map_err(|_| "Failed to compile regex")?;
    let re_match = re.find(gh_ref).ok_or("Failed to match GitHub reference")?;
    let parsed: u64 = re_match.as_str().parse().map_err(|_| "Failed to parse regex match as number")?;
    Ok(parsed)
}

fn pr_issues_from_body(body: &str) -> Result<Vec<u64>, &'static str> {
    let mut hash = HashSet::new();
    let re = RegexBuilder::new(r"(close|closes|closed|fix|fixes|fixed|resolve|resolves|resolved) #(\d+)")
        .case_insensitive(true)
        .multi_line(true)
        .build()
        .map_err(|_| "Failed to compile regex")?;

    for capture in re.captures_iter(body) {
        let parsed = capture[2].parse::<u64>().map_err(|_| "Failed to parse issue number from PR body")?;
        hash.insert(parsed);
    }

    let mut result: Vec<u64> = hash.into_iter().collect();
    result.sort_unstable();
    Ok(result)
}
