use crate::clash::TestCase;

#[derive(Debug, Clone)]
pub enum TestResult {
    Success,
    UnableToRun { error_msg: String },
    WrongOutput { stdout: String, stderr: String },
    RuntimeError { stdout: String, stderr: String },
    Timeout { stdout: String, stderr: String },
}

#[derive(Debug, Clone)]
pub struct TestRun<'a> {
    testcase: &'a TestCase,
    result: TestResult,
}

impl<'a> TestRun<'a> {
    pub fn new(testcase: &'a TestCase, result: TestResult) -> Self {
        Self { testcase, result }
    }

    pub fn expected(&self) -> &str {
        &self.testcase.test_out
    }

    pub fn actual(&self) -> &str {
        match &self.result {
            TestResult::Success => self.expected(),
            TestResult::UnableToRun { .. } => "",
            TestResult::RuntimeError { stdout, .. } => stdout,
            TestResult::WrongOutput { stdout, .. } => stdout,
            TestResult::Timeout { stdout, .. } => stdout,
        }
    }

    pub fn is_successful(&self) -> bool {
        matches!(self.result, TestResult::Success)
    }

    pub fn testcase(&self) -> &'a TestCase {
        self.testcase
    }

    pub fn result(&'a self) -> &'a TestResult {
        &self.result
    }
}
