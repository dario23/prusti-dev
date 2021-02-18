// compile-flags: -Pprint_desugared_specs=true -Pprint_typeckd_specs=true -Pno_verify=true -Phide_uuids=true
// normalize-stdout-test: "[a-z0-9]{32}" -> "$(NUM_UUID)"
// normalize-stdout-test: "[a-z0-9]{8}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{12}" -> "$(UUID)"

/// Failure case for predicate parsing: can only be used on function definitions

use prusti_contracts::*;

// doesn't work on just function decl
#[predicate]
fn result_is_one() -> bool;

// doesn't work on non-function-y items
#[predicate]
static FOO: usize = 0;

fn main() {}
