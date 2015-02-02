#![allow(unstable)]
pub use errors;

use std::old_io::process::Command;
use utils::say::{say, sayln, Spinner};
use errors::{DeliveryError, Kind};
use std::os;

fn cwd() -> Path {
    os::getcwd().unwrap()
}

pub fn get_head() -> Result<String, DeliveryError> {
    let gitr = try!(git_command(&["branch"], &cwd()));
    let result = try!(parse_get_head(gitr.stdout.as_slice()));
    Ok(result)
}

fn parse_get_head(stdout: &str) -> Result<String, DeliveryError> {
    for line in stdout.lines_any() {
        let r = regex!(r"(.) (.+)");
        let caps_result = r.captures(line);
        let caps = match caps_result {
            Some(caps) => caps,
            None => { return Err(DeliveryError{ kind: Kind::BadGitOutputMatch, detail: Some(format!("Failed to match: {}", line)) }) }
        };
        let token = caps.at(1).unwrap();
        if token == "*" {
            let branch = caps.at(2).unwrap();
            return Ok(String::from_str(branch));
        }
    }
    return Err(DeliveryError{ kind: Kind::NotOnABranch, detail: None });
}

#[test]
fn test_parse_get_head() {
    let stdout = "  adam/review
  adam/test
  adam/test6
  builder
  first
  foo
  foo2
* master
  snazzy
  testerton";
    let result = parse_get_head(stdout);
    match result {
        Ok(branch) => {
            assert_eq!(branch.as_slice(), "master");
        },
        Err(_) => panic!("No result")
    };
}

pub struct GitResult {
    pub stdout: String,
    pub stderr: String
}

pub fn git_command(args: &[&str], cwd: &Path) -> Result<GitResult, DeliveryError> {
    let spinner = Spinner::start();
    let mut command = Command::new("git");
    command.args(args);
    command.cwd(cwd);
    debug!("Git command: {:?}", command);
    let output = match command.output() {
        Ok(o) => o,
        Err(e) => { spinner.stop(); return Err(DeliveryError{ kind: Kind::FailedToExecute, detail: Some(format!("failed to execute git: {}", e.desc))}) },
    };
    debug!("Git exited: {}", output.status);
    spinner.stop();
    if !output.status.success() {
        return Err(DeliveryError{ kind: Kind::GitFailed, detail: Some(format!("STDOUT: {}\nSTDERR: {}\n", String::from_utf8_lossy(output.output.as_slice()), String::from_utf8_lossy(output.error.as_slice())))});
    }
    let stdout = String::from_utf8_lossy(output.output.as_slice()).to_string();
    debug!("Git stdout: {}", stdout);
    let stderr = String::from_utf8_lossy(output.error.as_slice()).to_string();
    debug!("Git stderr: {}", stderr);
    Ok(GitResult{ stdout: stdout, stderr: stderr })
}

pub fn git_push(branch: &str, target: &str) -> Result<String, DeliveryError> {
    let gitr = try!(git_command(&[
                     "push", "--porcelain", "--progress", "--verbose", "delivery", format!("{}:_for/{}/{}", branch, target, branch).as_slice()
                     ],
                     &cwd()));
    let output = try!(parse_git_push_output(gitr.stdout.as_slice(), gitr.stderr.as_slice()));
    for result in output.iter() {
        match result.flag {
            PushResultFlags::SuccessfulFastForward => sayln("green", format!("Updated change: {}", result.reason).as_slice()),
            PushResultFlags::SuccessfulForcedUpdate => sayln("green", format!("Force updated change: {}", result.reason).as_slice()),
            PushResultFlags::SuccessfulDeletedRef => sayln("red", format!("Deleted change: {}", result.reason).as_slice()),
            PushResultFlags::SuccessfulPushedNewRef => sayln("green", format!("Created change: {}", result.reason).as_slice()),
            PushResultFlags::Rejected => sayln("red", format!("Rejected change: {}", result.reason).as_slice()),
            PushResultFlags::UpToDate => sayln("yellow", format!("Nothing added to the existing change").as_slice()),
        }
    }
    Ok(gitr.stdout.to_string())
}

pub enum PushResultFlags {
    SuccessfulFastForward,
    SuccessfulForcedUpdate,
    SuccessfulDeletedRef,
    SuccessfulPushedNewRef,
    Rejected,
    UpToDate,
}

impl Copy for PushResultFlags { }

pub struct PushResult {
    flag: PushResultFlags,
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
                Some(caps) => sayln("white", format!("{}", caps.at(1).unwrap()).as_slice()),
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
            None => { return Err(DeliveryError{ kind: Kind::BadGitOutputMatch, detail: Some(format!("Failed to match: {}", line)) }) }
        };
        let result_flag = match caps.at(1).unwrap() {
            " " => PushResultFlags::SuccessfulFastForward,
            "+" => PushResultFlags::SuccessfulForcedUpdate,
            "-" => PushResultFlags::SuccessfulDeletedRef,
            "*" => PushResultFlags::SuccessfulPushedNewRef,
            "!" => PushResultFlags::Rejected,
            "=" => PushResultFlags::UpToDate,
            _ => { return Err(DeliveryError{ kind: Kind::BadGitOutputMatch, detail: Some(format!("Unknown result flag")) }) }
        };
        push_results.push(
            PushResult{
                flag: result_flag,
                reason: String::from_str(caps.at(4).unwrap())
            }
        )
    }
    Ok(push_results)
}

pub fn delivery_ssh_url(user: &str, server: &str, ent: &str, org: &str, proj: &str) -> String {
    format!("ssh://{}@{}@{}:8989/{}/{}/{}", user, ent, server, ent, org, proj)
}

pub fn config_repo(user: &str, server: &str, ent: &str, org: &str, proj: &str, path: &Path) -> Result<(), DeliveryError> {
    let result = git_command(&["remote", "add", "delivery", delivery_ssh_url(user, server, ent, org, proj).as_slice()], path);
    match result {
        Ok(_) => return Ok(()),
        Err(e) => {
            match e.detail {
                Some(msg) => {
                    if msg.contains("remote delivery already exists") {
                        return Err(DeliveryError{ kind: Kind::GitSetupFailed, detail: None });
                    }
                },
                None => {
                    return Err(e)
                }
            }
        },
    }
    Ok(())
}

pub fn checkout_branch_name(change: &str, patchset: &str) -> String {
    if patchset == "latest" {
        return String::from_str(change);
    } else {
        return format!("{}/{}", change, patchset);
    }
}

pub fn diff(change: &str, patchset: &str, pipeline: &str, local: &bool) -> Result<(), DeliveryError> {
    try!(git_command(&["fetch", "delivery"], &cwd()));
    let mut first_branch = format!("delivery/{}", pipeline);
    if *local {
        first_branch = String::from_str("HEAD");
    }
    let diff = try!(git_command(&["diff", "--color=always", first_branch.as_slice(), format!("delivery/_reviews/{}/{}/{}", pipeline, change, patchset).as_slice()], &cwd()));
    say("white", "\n");
    sayln("white", diff.stdout.as_slice());
    Ok(())
}

pub fn clone(project: &str, git_url: &str) -> Result<(), DeliveryError> {
    try!(git_command(&["clone", git_url, project], &cwd()));
    Ok(())
}

pub fn checkout_review(change: &str, patchset: &str, pipeline: &str) -> Result<(), DeliveryError> {
    try!(git_command(&["fetch", "delivery"], &cwd()));
    let branchname = checkout_branch_name(change, patchset);
    let result = git_command(&["branch", "--track", branchname.as_slice(), format!("delivery/_reviews/{}/{}/{}", pipeline, change, patchset).as_slice()], &cwd());
    match result {
        Ok(_) => {
            try!(git_command(&["checkout", branchname.as_slice()], &cwd()));
            return Ok(())
        },
        Err(e) => {
            match e.detail {
                Some(msg) => {
                    if msg.contains("already exists.") {
                        try!(git_command(&["checkout", branchname.as_slice()], &cwd()));
                        sayln("white", "Branch already exists, checking it out.");
                        let r = try!(git_command(&["status"], &cwd()));
                        sayln("white", r.stdout.as_slice());
                        return Ok(())
                    } else {
                        return Err(DeliveryError{kind: Kind::GitFailed, detail: Some(msg)});
                    }
                },
                None => {
                    return Err(e)
                }
            }
        },
    }
}