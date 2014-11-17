#[phase(plugin)]
extern crate regex_macros;
extern crate regex;
extern crate serialize;
extern crate git2;
extern crate docopt;
#[phase(plugin)] extern crate docopt_macros;
#[phase(plugin, link)] extern crate log;
extern crate term;

pub use errors;

use git2::Repository;
use std::os;
use std::io::process::Command;
use utils::say::sayln;
use errors::{DeliveryError};

pub fn get_repository() -> Result<git2::Repository, DeliveryError> {
    let repo = try!(git2::Repository::discover(&os::getcwd()));
    Ok(repo)
}

pub fn get_head(repo: git2::Repository) -> Result<String, DeliveryError> {
    let head = try!(repo.head());
    let shorthand = head.shorthand();
    let result = match shorthand {
        Some(result) => Ok(String::from_str(result)),
        None => Err(DeliveryError{ kind: errors::NotOnABranch, detail: None })
    };
    result
}

pub fn git_push(branch: &str, target: &str) -> Result<String, DeliveryError> {
    let mut command = Command::new("git");
    command.arg("push");
    command.arg("--porcelain");
    command.arg("--progress");
    command.arg("--verbose");
    command.arg("origin");
    command.arg(format!("{}:_for/{}/{}", branch, target, branch));
    debug!("Running: {}", command);
    let output = match command.output() {
        Ok(o) => o,
        Err(e) => { return Err(DeliveryError{ kind: errors::FailedToExecute, detail: Some(format!("failed to execute git: {}", e.desc))}) },
    };
    if !output.status.success() {
        return Err(DeliveryError{ kind: errors::PushFailed, detail: Some(format!("STDOUT: {}\nSTDERR: {}\n", String::from_utf8_lossy(output.output.as_slice()), String::from_utf8_lossy(output.error.as_slice())))});
    }
    let stdout = String::from_utf8_lossy(output.output.as_slice()).into_string();
    let stderr = String::from_utf8_lossy(output.error.as_slice()).into_string();
    debug!("Git exited: {}", output.status);
    let output = try!(parse_git_push_output(stdout.as_slice(), stderr.as_slice()));
    for result in output.iter() {
        match result.flag {
            SuccessfulFastForward => sayln("green", format!("Updated change: {}", result.reason).as_slice()),
            SuccessfulForcedUpdate => sayln("green", format!("Force updated change: {}", result.reason).as_slice()),
            SuccessfulDeletedRef => sayln("red", format!("Deleted change: {}", result.reason).as_slice()),
            SuccessfulPushedNewRef => sayln("green", format!("Created change: {}", result.reason).as_slice()),
            Rejected => sayln("red", format!("Rejected change: {}", result.reason).as_slice()),
            UpToDate => sayln("yellow", format!("Nothing added to the existing change").as_slice()),
        }
    }
    Ok(stdout.into_string())
}

pub enum PushResultFlags {
    SuccessfulFastForward,
    SuccessfulForcedUpdate,
    SuccessfulDeletedRef,
    SuccessfulPushedNewRef,
    Rejected,
    UpToDate,
}

pub struct PushResult {
    flag: PushResultFlags,
    from: String,
    to: String,
    reason: String
}

pub fn parse_git_push_output(push_output: &str, push_error: &str) -> Result<Vec<PushResult>, DeliveryError> {
    let mut push_results: Vec<PushResult> = Vec::new();
    for line in push_error.lines_any() {
        debug!("error: {}", line);
        if line.starts_with("remote") {
            let r = regex!(r"remote: (.+)");
            let caps_result = r.captures(line);
            match caps_result {
                Some(caps) => sayln("white", format!("{}", caps.at(1)).as_slice()),
                None => {}
            }
        }
    }
    for line in push_output.lines_any() {
        debug!("output: {}", line);
        if line.starts_with("To") {
            continue;
        } else if line.starts_with("Done") {
            continue;
        }
        let r = regex!(r"(.)\t(.+):(.+)\t\[(.+)\]");
        let caps_result = r.captures(line);
        let caps = match caps_result {
            Some(caps) => caps,
            None => { return Err(DeliveryError{ kind: errors::BadGitOutputMatch, detail: Some(format!("Failed to match: {}", line)) }) }
        };
        let result_flag = match caps.at(1) {
            " " => SuccessfulFastForward,
            "+" => SuccessfulForcedUpdate,
            "-" => SuccessfulDeletedRef,
            "*" => SuccessfulPushedNewRef,
            "!" => Rejected,
            "=" => UpToDate,
            _ => { return Err(DeliveryError{ kind: errors::BadGitOutputMatch, detail: Some(format!("Unknown result flag")) }) }
        };
        push_results.push(
            PushResult{
                flag: result_flag,
                from: String::from_str(caps.at(2)),
                to: String::from_str(caps.at(3)),
                reason: String::from_str(caps.at(4))
            }
        )
    }
    Ok(push_results)
}

#[test]
fn test_parse_git_push_output_success() {
    let stdout = "To ssh://adam@127.0.0.1/Users/adam/src/opscode/delivery/opscode/delivery-cli2
=	refs/heads/foo:refs/heads/_for/master/foo	[up to date]
Done";
    let stderr = "Pushing to ssh://adam@Chef@172.31.6.130:8989/Chef/adam_universe/delivery-cli
Total 0 (delta 0), reused 0 (delta 0)
remote: Patchset already up to date, nothing to do 
remote: https://172.31.6.130/e/Chef/#/organizations/adam_universe/projects/delivery-cli/changes/146a9573-1bd0-4a27-a106-528347761811
updating local tracking ref 'refs/remotes/origin/_for/master/adam/test6'";
    let result = parse_git_push_output(stdout, stderr);
    match result {
        Ok(pr_vec) => {
            // assert!(r_vec[0].flag, UpToDate);
            assert_eq!(pr_vec[0].from.as_slice(), "refs/heads/foo");
            assert_eq!(pr_vec[0].to.as_slice(), "refs/heads/_for/master/foo");
            assert_eq!(pr_vec[0].reason.as_slice(), "up to date");
        },
        Err(e) => panic!("No result")
    };
}

