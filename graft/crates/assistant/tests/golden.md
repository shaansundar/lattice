# crates/assistant/tests/golden.rs

- fixture · function · L10-L13 — fn fixture(name: &str) -> Value
- rows · function · L15-L17 — fn rows(v: &Value) -> Vec<Vec<Value>>
- observations_match_training · function · L20-L42 — fn observations_match_training()
- system_prompt_and_snapshot_match_training · function · L45-L54 — fn system_prompt_and_snapshot_match_training()
- tool_call_parsing_matches_training · function · L57-L65 — fn tool_call_parsing_matches_training()
- docs_search_matches_training · function · L68-L85 — fn docs_search_matches_training()
- harness_runs_real_tools_end_to_end · function · L88-L105 — fn harness_runs_real_tools_end_to_end()
