// {
//     "folders":
//     [
//         {
//             "path": "."
//         }
//     ],
//     "settings": {
//         "Scoggle" : {
//             "production_srcs" :
//             [
//                 "/app"
//             ],
//             "test_srcs" :
//             [
//                 "/test"
//             ],
//             "test_suffixes" :
//             [
//                 "Spec.scala"
//             ]
//         }
//     }
// }

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

