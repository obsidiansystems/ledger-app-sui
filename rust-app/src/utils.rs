use ledger_prompts_ui::{PromptWrite, ScrollerError};

// A couple type ascription functions to help the compiler along.
pub const fn mkfn<A, B, C>(q: fn(&A, &mut B) -> C) -> fn(&A, &mut B) -> C {
    q
}
pub const fn mkmvfn<A, B, C>(q: fn(A, &mut B) -> Option<C>) -> fn(A, &mut B) -> Option<C> {
    q
}
/*
const fn mkvfn<A>(q: fn(&A,&mut Option<()>)->Option<()>) -> fn(&A,&mut Option<()>)->Option<()> {
q
}
*/

#[cfg(not(target_os = "nanos"))]
#[inline(never)]
pub fn scroller<F: for<'b> Fn(&mut PromptWrite<'b, 16>) -> Result<(), ScrollerError>>(
    title: &str,
    prompt_function: F,
) -> Option<()> {
    ledger_prompts_ui::write_scroller_three_rows(false, title, prompt_function)
}

#[cfg(target_os = "nanos")]
#[inline(never)]
pub fn scroller<F: for<'b> Fn(&mut PromptWrite<'b, 16>) -> Result<(), ScrollerError>>(
    title: &str,
    prompt_function: F,
) -> Option<()> {
    ledger_prompts_ui::write_scroller(false, title, prompt_function)
}

#[cfg(not(target_os = "nanos"))]
#[inline(never)]
pub fn scroller_paginated<F: for<'b> Fn(&mut PromptWrite<'b, 16>) -> Result<(), ScrollerError>>(
    title: &str,
    prompt_function: F,
) -> Option<()> {
    ledger_prompts_ui::write_scroller_three_rows(true, title, prompt_function)
}

#[cfg(target_os = "nanos")]
#[inline(never)]
pub fn scroller_paginated<F: for<'b> Fn(&mut PromptWrite<'b, 16>) -> Result<(), ScrollerError>>(
    title: &str,
    prompt_function: F,
) -> Option<()> {
    ledger_prompts_ui::write_scroller(true, title, prompt_function)
}
