use std::collections::HashSet;
use octocrab::Octocrab;
use regex::{Regex, RegexBuilder};

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN envvar is required");
    let octocrab = Octocrab::builder().personal_token(token).build().expect("Octocrab client creation to succeed");
    let reference = std::env::var("GITHUB_REF").expect("GITHUB_REF envvar is required");
    let pr_num = pr_num_from_gh_ref(&reference);
    let pr = octocrab.pulls("Flightlogger", "flightlogger").get(pr_num).await?;
    let issue_nums = pr_issues_from_body(&pr.body.expect("PR to have a body"));
    let suffix = issue_nums.iter().map(|num| format!("[#{}]", num)).collect::<Vec<String>>().join(" ");
    let title_re = Regex::new(r"\s?\[#\d+\]\s?").expect("PR title regex to compile");
    let old_title_sanitized = title_re.replace_all(&pr.title.expect("PR to have a title"), "").to_string();
    let new_title = format!("{} {}", old_title_sanitized, suffix);
    let updated = octocrab.issues("Flightlogger", "flightlogger").update(pr_num).title(&new_title).send().await?;

    println!("Updated PR title to: {}", updated.title);
    Ok(())
}

fn pr_num_from_gh_ref(gh_ref: &str) -> u64 {
    let re = Regex::new(r"\d+").expect("GH ref regex to compile");
    let re_match = re.find(gh_ref).expect("GH ref to contain PR number");
    re_match.as_str().parse().expect("GH ref PR number to be parsed as number")
}

fn pr_issues_from_body(body: &str) -> Vec<u64> {
    let mut hash = HashSet::new();
    let re = RegexBuilder::new(r"(close|closes|closed|fix|fixes|fixed|resolve|resolves|resolved) #(\d+)")
        .case_insensitive(true)
        .multi_line(true)
        .build()
        .expect("PR body regex to compile");

    for capture in re.captures_iter(body) {
        let parsed = capture[2].parse::<u64>().expect("Regex digits capture to parse to u64");
        hash.insert(parsed);
    }

    let mut result: Vec<u64> = hash.into_iter().collect();
    result.sort_unstable();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_num_from_gh_ref() {
        assert_eq!(pr_num_from_gh_ref("refs/pull/16901/merge"), 16901);
    }

    #[test]
    fn test_pr_issues_from_body_empty() {
        assert_eq!(pr_issues_from_body(&""), Vec::<u64>::new());
    }

    #[test]
    fn test_pr_issues_from_body_no_issue() {
        assert_eq!(pr_issues_from_body(&"this has no issue number identifier"), Vec::<u64>::new());
    }

    #[test]
    fn test_pr_issues_from_body_single() {
        assert_eq!(pr_issues_from_body(&"resolves #123"), vec![123]);
    }

    #[test]
    fn test_pr_issues_from_body_multiple() {
        assert_eq!(pr_issues_from_body(&"resolves #123\nfix #456"), vec![123, 456]);
    }

    #[test]
    fn test_pr_issues_from_body_no_duplicates() {
        assert_eq!(pr_issues_from_body(&"resolves #123\nfix #456\nsolved #123"), vec![123, 456]);
    }

    #[test]
    fn test_pr_issues_from_body_invalid_prefix() {
        assert_eq!(pr_issues_from_body(&"resolv #123"), Vec::<u64>::new());
    }

    #[test]
    fn test_pr_issues_from_body_invalid_no_space() {
        assert_eq!(pr_issues_from_body(&"resolves#123"), Vec::<u64>::new());
    }

    #[test]
    fn test_pr_issues_from_body_invalid_no_number() {
        assert_eq!(pr_issues_from_body(&"resolves #"), Vec::<u64>::new());
    }

    #[test]
    fn test_pr_issues_from_body_invalid_no_prefix() {
        assert_eq!(pr_issues_from_body(&"#123"), Vec::<u64>::new());
    }
}
