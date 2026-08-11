//! Concrete project detectors. Each lives in its own file, is independently
//! testable, and only inspects the marker files it needs. Adding a new detector
//! = implement `ProjectDetector` + register it in `DetectorRegistry`.

pub mod docker;
pub mod dotnet;
pub mod flutter;
pub mod git_detector;
pub mod go;
pub mod java;
pub mod node;
pub mod python;
pub mod rust;
pub mod unity;

pub use docker::DockerDetector;
pub use dotnet::DotNetDetector;
pub use flutter::FlutterDetector;
pub use git_detector::GitDetector;
pub use go::GoDetector;
pub use java::JavaDetector;
pub use node::NodeDetector;
pub use python::PythonDetector;
pub use rust::RustDetector;
pub use unity::UnityDetector;
