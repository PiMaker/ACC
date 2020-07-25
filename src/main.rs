#![allow(non_snake_case)]
#![allow(dead_code)]

use std::io;

use clap::{App, SubCommand, Arg};
use tokio::sync::mpsc;

mod signal;
use signal::{AcCtrl, Signal};
mod send;
mod recv;
mod consts;
use consts::*;

#[tokio::main]
async fn main() {
    let matches = App::new("AC Control")
        .version("0.1")
        .author("Stefan Reiter <stefan@pimaker.at>")
        .about("Tooling for AC remote control via IR.")
        .subcommand(SubCommand::with_name("receive")
            .about("Receive and print data in a loop. (mainly for debugging/reverse engineering)"))
        .subcommand(SubCommand::with_name("send")
            .about("Send a signal to the AC to configure the given settings.")
            .arg(Arg::with_name("temperature")
                .help("The temperature in the range 17°C to 30°C")
                .takes_value(true)
                .long("temperature")
                .short("t"))
            .arg(Arg::with_name("fan")
                .help("Fan speed. Defaults to 'auto'.")
                .possible_values(&["low", "medium", "high", "auto"])
                .takes_value(true)
                .long("fan")
                .short("f"))
            .arg(Arg::with_name("mode")
                .help("Operating mode. Defaults to 'cool'.")
                .possible_values(&["cool", "heat", "dry", "fan", "auto"])
                .takes_value(true)
                .long("mode")
                .short("m"))
            .arg(Arg::with_name("off")
                .long("off")
                .help("Specify to turn off the AC.")))
        .get_matches();

    if matches.subcommand_matches("receive").is_some() {
        let (mut cancel_tx, cancel_rx) = mpsc::channel(1);
        let handle = tokio::spawn(recv::receive_loop(cancel_rx, Box::new(|signals| {
            let signals = signal::verify_and_trim(signals);

            if let Some(ref signals) = signals {
                print!("Signal received: ");
                print_signal(signals);
            }
        })));

        println!("Press enter to stop receiving...");
        let stdin = io::stdin();
        let mut _input = String::new();
        stdin.read_line(&mut _input).unwrap();
        println!("Exiting!");

        cancel_tx.send(()).await.expect("Could not cancel, let's hope we exit anyway...");
        handle.await.expect("Could not wait for handle, let's hope we exit anyway...");

        return;
    }

    if let Some(matches) = matches.subcommand_matches("send") {

        let mut temp = 0;
        let mut fan_speed = FanSpeed::Off;
        let mut mode = Mode::Cool;
        let on = if matches.is_present("off") {
            false
        } else {
            temp = match matches.value_of("temperature") {
                Some(temp) => match temp.parse() {
                    Ok(itemp) => {
                        if itemp < 17 || itemp > 30 {
                            println!("Temperature has to be between 17 and 30");
                            std::process::exit(1);
                        }
                        itemp
                    },
                    Err(err) => {
                        println!("Temperature has to be an integer value: {}", err);
                        std::process::exit(1);
                    }
                },
                None => {
                    println!("Temperature argument is required when not requesting 'off' state!");
                    std::process::exit(1);
                }
            };

            fan_speed = match matches.value_of("fan") {
                Some(speed) => {
                    if speed == "low" {
                        FanSpeed::Low
                    } else if speed == "medium" {
                        FanSpeed::Medium
                    } else if speed == "high" {
                        FanSpeed::High
                    } else {
                        FanSpeed::Auto
                    }
                },
                None => FanSpeed::Auto,
            };

            mode = match matches.value_of("mode") {
                Some(mode) => {
                    if mode == "cool" {
                        Mode::Cool
                    } else if mode == "heat" {
                        Mode::Heat
                    } else if mode == "dry" {
                        Mode::Dry
                    } else if mode == "fan" {
                        Mode::Fan
                    } else {
                        Mode::Auto
                    }
                },
                None => Mode::Cool,
            };

            true
        };

        let ac_ctrl = AcCtrl::new(temp, fan_speed, mode, on);
        let signals = ac_ctrl.to_signals();

        print!("Signal constructed: ");
        print_signal(&signals.iter().collect::<Vec<&Signal>>());

        println!("Sending...");
        send::send(&signals).await;

        println!("Sent!");

        return;
    }

    println!("Please specify a subcommand: receive, send, web, help");
}

fn print_signal(signals: &[&Signal]) {
    let mut ctr = -1;
    for s in signals {
        ctr += 1;
        if ctr == 8 {
            ctr = 0;
            print!(" ");
        }
        print!("{}", s);
    }
    println!(" (chksum: {})", signal::verify_checksum(&signals[..]));
}

