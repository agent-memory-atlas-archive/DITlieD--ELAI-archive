//! Quality-arm selection for the QDELTA fixture runner.

use super::{BenchQdeltaError, QdeltaConfig};

/// Select recorded maintainability/SOLID checker verdicts for the quality arm.
/// A cell is true ("defective") when the frozen maintainability / SOLID proxy
/// checkers flag the diff produced for that task. The fixture cells are recorded
/// verdicts of those checkers; producing them from a live gate-ON / gate-OFF run
/// is a later leg this slice does not build. This function claims no coverage
/// beyond those recorded fixture verdicts.
pub fn quality_cells(config: &QdeltaConfig) -> Result<(Vec<bool>, Vec<bool>), BenchQdeltaError> {
    let mut on = Vec::with_capacity(config.tasks.len());
    let mut off = Vec::with_capacity(config.tasks.len());
    for (row_index, row) in config.tasks.iter().enumerate() {
        let on_value = row
            .on_quality_violation
            .ok_or(BenchQdeltaError::QualityCellMissing { row_index })?;
        let off_value = row
            .off_quality_violation
            .ok_or(BenchQdeltaError::QualityCellMissing { row_index })?;
        on.push(on_value);
        off.push(off_value);
    }
    Ok((on, off))
}

/// Return the stable report label for the selected QDELTA arm.
pub fn arm_label(quality: bool) -> &'static str {
    if quality { "quality" } else { "defect" }
}
