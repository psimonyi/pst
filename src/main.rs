use std::collections::HashMap;
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
            query => select_pids.append(&mut find_pids(query)),
        }
    }

    print_ps(columns, reserve_for_args, select_pids);
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

fn print_ps(width: u32, reserve_for_args: u32, select_pids: Vec<String>) {
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
        if line_matches(line, &select_pids) {
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
    // Columns, in order of preference for being allocated space
    let cols_preferred = ["args", "pid", "stat", "nice", "%mem", "euser",
                          "tname", "start_time", "psr", "cputime", "egroup",
                          "pgid"];
    // Columns, in the order to display them
    let cols_ordered = ["pid", "pgid", "args", "psr", "stat", "nice", "%mem",
                        "cputime", "start_time", "tname", "euser", "egroup"];

    // Column widths we'll use.  The width for args is set specially to include
    // whatever space is left over.
    let cols_widths_p = [ ("pid", 5), ("%cpu", 5), ("%mem", 4), ("args", 0),
                          ("start_time", 5), ("cputime", 8), ("nice", 3),
                          ("psr", 3), ("sgi_p", 1), ("session", 5),
                          ("stat", 4), ("s", 1), ("tname", 6), ("rssize", 6),
                          ("size", 6), ("vsz", 6), ("pgid", 5), ("euser", 8),
                          ("egroup", 8) ];
    // Make that a mapping.  This clearly isn't the most efficient, but it's a
    // straightforward translation from the Python.
    let mut cols_widths = HashMap::new();
    for &(name, width) in cols_widths_p.into_iter() {
        cols_widths.insert(name, width);
    }

    let mut space = total_width - reserve_for_args + 1;
    let mut cols_chosen : HashMap<&str, u32> = HashMap::new();
    for col in cols_preferred.into_iter() {
        let width = *cols_widths.get(col).unwrap();
        if space > width {
            space -= width + 1;
            cols_chosen.insert(col, width);
        }
    }

    cols_chosen.insert("args", reserve_for_args + space);

    let mut cols = Vec::new();
    for col in cols_ordered.into_iter() {
        match cols_chosen.get(col) {
            Some(width) => cols.push(format!("{}:{}", col, width)),
            None => (),
        }
    }
    return cols.join(",");
}
