use std::fs::File;
use std::io::Write;
use std::process::Command;

// use tokio::time::{Duration, delay_for};

use crate::signal::*;

pub async fn send(signals: &[Signal]) {
    let mut file = File::create("/tmp/irrp.rec").expect("create failed");
    file.write_all(signals_to_json(signals).as_bytes()).expect("write failed");

    send_cmd();
    // delay_for(Duration::from_millis(100)).await;
    // send_cmd();
}

fn send_cmd() {
    let result = Command::new("/usr/bin/python3")
        .arg("./irrp.py")
        .arg("-g18") // hardcoded for now
        .arg("-f/tmp/irrp.rec")
        .arg("-p")
        .arg("generated")
        .output()
        .expect("ERROR: failed to execute irrp.py");

    if !result.status.success() {
        println!("---");
        println!("An error seems to have occured during sending.");
        println!("STDOUT:");
        println!("{}", String::from_utf8_lossy(&result.stdout));
        println!("STDERR:");
        println!("{}", String::from_utf8_lossy(&result.stderr));
        println!("---");
    }
}
