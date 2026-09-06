//! Doctor feature module providing fast environment, Git, and toolchain diagnostics.

pub mod checks;
pub mod engine;
pub mod models;

pub use checks::*;
pub use engine::run_doctor;
pub use models::{
    DoctorCategory, DoctorCheckItem, DoctorCheckStatus, DoctorError, DoctorOptions, DoctorReport,
    DoctorVerdict,
};
