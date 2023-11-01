use crate::{clash::TestCase, outputstyle::OutputStyle, formatter::show_whitespace};

#[derive(Debug, PartialEq)]
pub enum TestRunResult {
    Success,
    WrongOutput { stdout: String, stderr: String },
    RuntimeError { stdout: String, stderr: String },
}

pub struct TestRun {
    testcase: TestCase,
    result: TestRunResult,
}

impl TestRun {
    pub fn new(testcase: TestCase, result: TestRunResult) -> Self {
        Self { testcase, result }
    }

    pub fn expected(&self) -> &String {
        &self.testcase.test_out
    }

    pub fn actual(&self) -> &String {
        match &self.result {
            TestRunResult::Success => self.expected(),
            TestRunResult::RuntimeError { stdout, .. } => stdout,
            TestRunResult::WrongOutput { stdout, .. } => stdout,
        }
    }

    pub fn is_successful(&self) -> bool {
        self.result == TestRunResult::Success
    }

    pub fn print_mistakes(&self, style: &OutputStyle) {
        let title = style.title.paint(&self.testcase.title);
        match &self.result {
            TestRunResult::Success => {
                println!("{} {}", style.success.paint("PASS"), title);
            }

            TestRunResult::WrongOutput { stderr, stdout } => {
                println!("{} {}", style.failure.paint("FAIL"), title);
                print_testcase(&self.testcase, &stdout, &style);
                print_diff(&self.testcase, &stdout, &style);
                if !stderr.is_empty() {
                    println!("{}", style.stderr.paint(stderr.trim_end()));
                }
            }

            TestRunResult::RuntimeError { stdout, stderr } => {
                println!("{} {}", style.error.paint("ERROR"), title);
                if !stdout.is_empty() {
                    print_testcase(&self.testcase, &stdout, &style);
                    print_diff(&self.testcase, &stdout, &style);
                }
                if !stderr.is_empty() {
                    println!("{}", style.stderr.paint(stderr.trim_end()));
                }
            }
        }
    }
}


pub fn print_testcase(testcase: &TestCase, stdout: &str, ostyle: &OutputStyle) {
    println!(
        "{}\n{}",
        &ostyle.secondary_title.paint("===== INPUT ======"),
        testcase.styled_input(ostyle)
    );
    println!(
        "{}\n{}",
        &ostyle.secondary_title.paint("==== EXPECTED ===="),
        testcase.styled_output(ostyle)
    );
    println!(
        "{}\n{}",
        &ostyle.secondary_title.paint("==== RECEIVED ===="),
        if let Some(ws_style) = ostyle.output_whitespace {
            show_whitespace(stdout, &ostyle.output, &ws_style)
        } else {
            ostyle.output.paint(stdout).to_string()
        }
    );
    println!("{}", ostyle.secondary_title.paint("=================="));
}

// https://stackoverflow.com/a/40457615/5465108
pub struct LinesWithEndings<'a> {
    input: &'a str,
}

impl<'a> LinesWithEndings<'a> {
    pub fn from(input: &'a str) -> LinesWithEndings<'a> {
        LinesWithEndings { input }
    }
}

impl<'a> Iterator for LinesWithEndings<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        if self.input.is_empty() {
            return None;
        }
        let split = self.input.find('\n').map(|i| i + 1).unwrap_or(self.input.len());
        let (line, rest) = self.input.split_at(split);
        self.input = rest;
        Some(line)
    }
}

pub fn print_diff(testcase: &TestCase, stdout: &str, ostyle: &OutputStyle) {
    use dissimilar::Chunk::*;
    use itertools::Itertools;
    use itertools::EitherOrBoth::{Left, Right, Both};

    // (TODO) temporary styling, to be replaced with OutputStyle eventually
    let green = ansi_term::Style::new().fg(ansi_term::Color::RGB(0,185,0));
    let red = ansi_term::Style::new().fg(ansi_term::Color::Red);
    let error_red = ansi_term::Style::new().fg(ansi_term::Color::Red).on(ansi_term::Color::RGB(70,0,0));
    let dim_color = ansi_term::Style::new().fg(ansi_term::Color::RGB(50,50,50));
    let ws_style = &ostyle.output_whitespace.unwrap_or(ostyle.output);

    if stdout.is_empty() {
        println!("{}", dim_color.paint("(no output)"));
        return
    }

    let expected_lines = LinesWithEndings::from(&testcase.test_out);
    let actual_lines = LinesWithEndings::from(stdout);

    let mut missing_lines = 0;
    for either_or_both in expected_lines.zip_longest(actual_lines) {
        match either_or_both {
            Left(_) => missing_lines += 1,
            Right(s) => print!("{}", show_whitespace(s, &red, &error_red)),
            Both(a, b) => {
                let mut prev_deleted = false;

                for chunk in dissimilar::diff(a, b) {
                    match chunk {
                        Equal(text) if prev_deleted => {
                            let mut chars = text.chars();
                            let first_char = chars.next().expect("no chars???").to_string();
                            let rest = chars.as_str();
                            print!("{}", show_whitespace(&first_char, &red, &error_red));
                            if !rest.is_empty() {
                                print!("{}", show_whitespace(rest, &green, ws_style));
                            }
                        },
                        Equal(text) => print!("{}", show_whitespace(text, &green, ws_style)),
                        Insert(text) => print!("{}", show_whitespace(text, &red, &error_red)),
                        Delete(_) => {},
                    }

                    prev_deleted = matches!(chunk, Delete(_));
                }
            }
        }
    }

    if !stdout.ends_with('\n') {
        println!()
    }

    if missing_lines > 0 {
        let msg = format!("(expected {} more lines)", missing_lines);
        println!("{}", dim_color.paint(msg));
    }
}
