use continuum::ScholarOutput;

#[test]
fn builds_scholar_output_with_summary_and_task_scope() {
    let output = ScholarOutput::new(
        "summarized mission intent",
        "proposed atomic task scope",
    );

    assert_eq!(output.mission_summary, "summarized mission intent");
    assert_eq!(output.selected_task_scope, "proposed atomic task scope");
}
