use serde::{Deserialize, Deserializer,};
use serde_json;

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ExportType {
    Named,
    Default,
}

pub enum ExpandProps {
    Start,
    End,
    Boolean(bool),
}

impl<'de> Deserialize<'de> for ExpandProps {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
      D: Deserializer<'de>,
  {
      let value = serde_json::Value::deserialize(deserializer)?;
      match value {
          serde_json::Value::Bool(b) => Ok(ExpandProps::Boolean(b)),
          serde_json::Value::String(s) => match s.as_ref() {
              "start" => Ok(ExpandProps::Start),
              "end" => Ok(ExpandProps::End),
              _ => Err(serde::de::Error::custom("invalid expandProps value")),
          },
          _ => Err(serde::de::Error::custom("invalid expandProps value")),
      }
  }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JSXRuntime {
    Automatic,
    Classic,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JSXRuntimeImport {
    pub source: String,
    pub namespace: Option<String>,
    pub default_specifier: Option<String>,
    pub specifiers: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Caller {
    pub previous_export: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    pub component_name: String,
    pub caller: Caller,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "false_by_default")]
    pub typescript: bool,
    #[serde(default = "false_by_default")]
    pub title_prop: bool,
    #[serde(default = "false_by_default")]
    pub desc_prop: bool,
    #[serde(default = "default_expand_props")]
    pub expand_props: ExpandProps,
    #[serde(rename = "ref", default = "false_by_default")]
    pub forward_ref: bool,
    #[serde(default = "default_state")]
    pub state: State,
    #[serde(default = "false_by_default")]
    pub native: bool,
    #[serde(default = "false_by_default")]
    pub memo: bool,
    #[serde(default = "default_export_type")]
    pub export_type: ExportType,
    #[serde(default = "default_named_export")]
    pub named_export: String,
    pub jsx_runtime: Option<JSXRuntime>,
    #[serde(default = "default_jsx_runtime_import")]
    pub jsx_runtime_import: JSXRuntimeImport,
    #[serde(default = "default_import_source")]
    pub import_source: String,
}

// fn true_by_default() -> bool {
//     true
// }

pub fn false_by_default() -> bool {
  false
}

pub fn default_import_source() -> String {
  "react".to_string()
}

pub fn default_named_export() -> String {
  "ReactComponent".to_string()
}

pub fn default_export_type() -> ExportType {
  ExportType::Default
}

pub fn default_state() -> State {
  State {
      component_name: "SvgComponent".to_string(),
      caller: Caller {
          previous_export: "".to_string(),
      },
  }
}

pub fn default_expand_props() -> ExpandProps {
  ExpandProps::End
}

pub fn default_jsx_runtime_import() -> JSXRuntimeImport {
  JSXRuntimeImport {
      source: "react".to_string(),
      namespace: Some("React".to_string()),
      default_specifier: None,
      specifiers: None,
  }
}