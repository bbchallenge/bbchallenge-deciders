use super::*;

#[derive(Debug, Clone)]
pub struct BouncerCertificate {
    pub machine_std_format: String,
    pub formula_tape: FormulaTape,
    pub num_macro_steps_until_special_case: usize,
}

#[derive(Debug)]
pub enum CertificateError {
    InvalidCertificateError,
    FormulaTapeError(FormulaTapeError),
}

impl From<FormulaTapeError> for CertificateError {
    fn from(err: FormulaTapeError) -> Self {
        CertificateError::FormulaTapeError(err)
    }
}

impl BouncerCertificate {
    pub fn to_savask_format(&self) -> Result<String, CertificateError> {
        let mut to_return = String::new();
        let initial_formula_tape = self.formula_tape.clone();
        let mut working_formula_tape = self.formula_tape.clone();
        working_formula_tape.align()?;

        to_return += "\n";
        to_return += &working_formula_tape.to_savask_format();

        let mut formula_before_shift_rule = working_formula_tape.clone();

        let mut last_step = 0;

        for k in 0..self.num_macro_steps_until_special_case {
            let res = working_formula_tape.step()?;
            working_formula_tape.align()?;

            if working_formula_tape.head_is_pointing_at_repeater()? {
                let c = if last_step == 0 { 1 } else { 0 };
                to_return += &format!("\tSTEP {}", k - last_step + c);
                last_step = k + 1;
                to_return += "\n";
                to_return += &working_formula_tape.to_savask_format();
                formula_before_shift_rule = working_formula_tape.clone();
            }

            if let Some(shift_rule) = res {
                to_return += "\tRULE ";
                to_return += &shift_rule.to_savask_format();
                to_return += "\n";
                formula_before_shift_rule.apply_shift_rule(&shift_rule)?;
                to_return += &formula_before_shift_rule.to_savask_format();
            }

            if working_formula_tape.is_special_case_of(&initial_formula_tape)? {
                to_return += &format!("\tSTEP {}", k - last_step);
                to_return += "\n";
                to_return += &working_formula_tape.to_savask_format();
                to_return += "\tEND\n";
                return Ok(to_return);
            }
        }
        Err(CertificateError::InvalidCertificateError)
    }
}
