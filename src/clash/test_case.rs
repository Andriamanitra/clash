use serde::{Deserialize, Deserializer, Serialize};

use crate::formatter::show_whitespace;
use crate::outputstyle::OutputStyle;

#[derive(Debug, Serialize, Clone)]
pub struct TestCase {
    #[serde(skip_serializing)]
    index: usize,
    #[serde(deserialize_with = "deserialize_testcase_title")]
    pub title: String,
    #[serde(rename = "testIn")]
    pub test_in: String,
    #[serde(rename = "testOut")]
    pub test_out: String,
    #[serde(rename = "isValidator")]
    pub is_validator: bool,
}

pub fn deserialize_testcases<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<TestCase>, D::Error> {
    type TempTestCaseVec = Vec<TempTestCase>;

    #[derive(Deserialize)]
    struct TempTestCase {
        #[serde(deserialize_with = "deserialize_testcase_title")]
        pub title: String,
        #[serde(rename = "testIn")]
        pub test_in: String,
        #[serde(rename = "testOut")]
        pub test_out: String,
        #[serde(rename = "isValidator")]
        pub is_validator: bool,
    }

    let testcases = TempTestCaseVec::deserialize(de)?
        .into_iter()
        .enumerate()
        .map(|(i, t)| TestCase {
            index: i + 1,
            title: t.title,
            test_in: t.test_in,
            test_out: t.test_out,
            is_validator: t.is_validator,
        })
        .collect();
    Ok(testcases)
}

// Workaround for some old clashes which have testcase title as
// { "title": { "2": "The Actual Title" } } for whatever reason
fn deserialize_testcase_title<'de, D: Deserializer<'de>>(de: D) -> Result<String, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TempTitle {
        Normal(String),
        Weird {
            #[serde(rename = "2")]
            title: String,
        },
    }
    let title = match TempTitle::deserialize(de)? {
        TempTitle::Normal(title) => title,
        TempTitle::Weird { title } => title,
    };
    Ok(title)
}

impl TestCase {
    pub fn styled_title(&self, ostyle: &OutputStyle) -> String {
        ostyle.title.paint(format!("#{} {}", self.index, self.title)).to_string()
    }

    pub fn styled_input(&self, ostyle: &OutputStyle) -> String {
        match ostyle.input_whitespace {
            Some(ws_style) => show_whitespace(&self.test_in, &ostyle.input, &ws_style),
            None => ostyle.input.paint(&self.test_in).to_string(),
        }
    }

    pub fn styled_output(&self, ostyle: &OutputStyle) -> String {
        match ostyle.output_whitespace {
            Some(ws_style) => show_whitespace(&self.test_out, &ostyle.output, &ws_style),
            None => ostyle.output.paint(&self.test_out).to_string(),
        }
    }
}
