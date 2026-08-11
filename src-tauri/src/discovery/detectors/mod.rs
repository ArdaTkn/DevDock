//! Concrete project detectors. Each lives in its own file, is independently
//! testable, and only inspects the marker files it needs. Adding a new detector
//! = implement `ProjectDetector` + register it in `DetectorRegistry`.

pub mod git_detector;
pub mod node;
pub mod python;
pub mod rust;
pub mod go;
pub mod flutter;
pub mod dotnet;
pub mod docker;
pub mod unity;
pub mod java;

pub use git_detector::GitDetector;
pub use node::NodeDetector;
pub use python::PythonDetector;
pub use rust::RustDetector;
pub use go::GoDetector;
pub use flutter::FlutterDetector;
pub use dotnet::DotNetDetector;
pub use docker::DockerDetector;
pub use unity::UnityDetector;
pub use java::JavaDetector;
