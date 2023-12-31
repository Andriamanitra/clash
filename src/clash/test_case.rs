use serde::{Deserialize, Deserializer, Serialize};

use crate::formatter::show_whitespace;
use crate::outputstyle::OutputStyle;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestCase {
    #[serde(skip_serializing, skip_deserializing)]
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
    let mut testcases = Vec::<TestCase>::deserialize(de)?;

    for (i, testcase) in testcases.iter_mut().enumerate() {
        testcase.index = i + 1;
    }

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
