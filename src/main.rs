/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::env;
use std::io::{stderr, Write};
use std::process;
use std::process::Command;

extern crate termsize;

fn main() {
    let mut args = env::args();
    args.next(); // Ignore args[0]; it's the program name.

    let columns = match termsize::get() {
        Some(size) => u32::from(size.cols),
        None => 80,
    };
    let mut reserve_for_args = 80 - 6; // 6 is the width of the PID column
    let mut select_pids = vec![];

    let mut can_flag = true;
    let mut used_query = false;
    for arg in args {
        match arg.as_str() {
            "-d" if can_flag =>
                reserve_for_args = 44,
            "-l" if can_flag =>
                reserve_for_args = columns - 6, // PID column, as above
            "--" if can_flag =>
                can_flag = false,
            s if can_flag && s.starts_with("-") => {
                writeln!(stderr(), "Unrecognized option: {}", s).unwrap();
                process::exit(1);
            },
            query => {
                select_pids.append(&mut find_pids(query));
                used_query = true;
            },
        }
    }

    print_ps(columns, reserve_for_args, &select_pids);
    if used_query {
        match select_pids.len() {
            0 => println!("No matching processes."),
            1 => println!("One matching process: {}", select_pids[0]),
            _ => println!("{} matching processes: {}",
                          select_pids.len(),
                          select_pids.join(" ")),
        }
    }
}

fn find_pids(needle: &str) -> Vec<String> {
    // Return a list of the PIDs that match the query.
    let ps = Command::new("ps")
                     .arg("-e")
                     .arg("-o")
                     .arg("pid,args")
                     .arg("--no-headers")
                     .output()
                     .unwrap() // panics if the command failed
                     ;
    let output = String::from_utf8(ps.stdout).unwrap();
    let mut rv = Vec::new();
    for line in output.lines() {
        if line.contains(needle) {
            let pid = line.split_whitespace().next().unwrap();
            rv.push(pid.to_string());
        }
    }
    rv
}

fn print_ps(width: u32, reserve_for_args: u32, select_pids: &Vec<String>) {
    let ps = Command::new("ps")
                     .arg("-e")
                     .arg("-o")
                     .arg(format_string(width, reserve_for_args))
                     .arg("-H")
                     .output()
                     .unwrap() // panics if the command failed
                     ;

    // I shouldn't need to assume that it's UTF-8, but it's a lot easier.
    let output = String::from_utf8(ps.stdout).unwrap();
    let mut lines = output.lines();
    let first_line = lines.next().unwrap();
    println!("{}", first_line);
    for line in lines {
        // If the last column is args, it might be wider than specified.
        // Normally, that would require knowing how many cells the string takes
        // up, but ps seems to mangle anything that would be complicated into a
        // "?", so assuming one char per cell actually works.
        let end = match line.char_indices().nth(width as usize) {
            Some((i, _)) => i,
            None => line.len(),
        };
        let line = &line[..end];

        if line_matches(line, select_pids) {
            println!("\x1b[01;31m{}\x1b[0m", line);
        } else {
            println!("{}", line);
        }
    }
    println!("{}", first_line);
}

fn line_matches(line: &str, pids: &Vec<String>) -> bool {
    // Assumes that PID is the first column.
    pids.iter().any(|pid| line.split_whitespace().next() == Some(pid))
}

fn format_string(total_width: u32, reserve_for_args: u32) -> String {
    struct Col<'a> {
        name: &'a str,
        width: u32,
        order: i32,
    }
    impl<'a> Col<'a> {
        fn new(name: &'a str, width: u32, order: i32) -> Col<'a> {
            Col { name: name, width: width, order: order }
        }
    }

    // We have to iterate over the columns twice in different orders: first in
    // order of preference to choose which ones will fit, and then in display
    // order to build the format string.  This list is in order of preference.
    let unset = -1;
    let mut cols = [Col::new("args", 0, unset),
                    Col::new("pid", 5, unset),
                    Col::new("stat", 4, unset),
                    Col::new("nice", 3, unset),
                    Col::new("%mem", 4, unset),
                    Col::new("euser", 8, unset),
                    Col::new("tname", 6, unset),
                    Col::new("start_time", 5, unset),
                    Col::new("psr", 3, unset),
                    Col::new("cputime", 8, unset),
                    Col::new("egroup", 8, unset),
                    Col::new("pgid", 5, unset)];

    let display_order = ["pid", "pgid", "args", "psr", "stat", "nice", "%mem",
                        "cputime", "start_time", "tname", "euser", "egroup"];
    for (i, name) in display_order.iter().enumerate() {
        let col = cols.iter_mut().find(|c| c.name == *name).unwrap();
        col.order = i as i32;
    }

    // The +1's account for the 1-space gap between columns.
    let mut space = total_width - reserve_for_args + 1;
    let mut cols_chosen = Vec::new();
    for col in cols.iter_mut() {
        if space > col.width {
            space -= col.width + 1;
            cols_chosen.push(col);
        }
    }
    assert_eq!(cols_chosen[0].name, "args");
    cols_chosen.first_mut().unwrap().width = reserve_for_args + space;

    cols_chosen.sort_by_key(|col| col.order);
    let parts: Vec<String> = cols_chosen.iter()
                           .map(|col| format!("{}:{}", col.name, col.width))
                           .collect();
    parts.join(",")
}
