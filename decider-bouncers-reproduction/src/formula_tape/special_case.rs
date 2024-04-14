use super::*;

impl FormulaTape {
    /// Detects if the formula tape is a *special* case of an other, given formula tape.
    ///
    /// f' is a special case of f if f' can be obtained from f by replacying repeaters words of the form (r) by r^n(r)r^m for some n,m>=0.
    fn is_special_case(self, formule_tape: FormulaTape) -> Result<bool, FormulaTapeError> {
        Ok(true)
    }
}
