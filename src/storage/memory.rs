impl Memory {
    fn initial_budget() -> BudgetValue {
        BudgetValue::from_floats(
            DEFAULT_PARAMETERS.concept_initial_priority,
            DEFAULT_PARAMETERS.concept_initial_durability,
            DEFAULT_PARAMETERS.concept_initial_quality,
        )
    }
}
