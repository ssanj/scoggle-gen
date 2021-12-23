use serde::Serialize;

#[derive(Serialize)]
pub struct SublimeProject {
  pub folders: Vec<PathObject>,
  pub settings: SettingsObject
}

#[derive(Serialize)]
pub struct PathObject {
  pub path: String
}

#[derive(Serialize)]
pub struct SettingsObject {
  #[serde(rename = "Scoggle")]
  pub scoggle: ScoggleObject
}

#[derive(Serialize)]
pub struct ScoggleObject {
  pub production_srcs: Vec<String>,
  pub test_srcs: Vec<String>,
  pub test_suffixes: Vec<String>,
}

pub struct ProdSource(pub String);

pub struct TestSource(pub String);


#[derive(Clone)]
pub struct ProjectType(pub Vec<String>);

pub enum SBTExecution {
  CouldNotRun(String),
  CouldNotDecodeOutput(String),
  UnrecognisedOutputStructure(String),
  SuccessfulExecution(ProjectType)
}

pub enum SBTVersion {
  Valid,
  UnsupportedVersion(String),
  UnknownVersionString(String),
  NotFound
}

pub enum ProjectName {
  ProjectDir(String),
  Random()
}
