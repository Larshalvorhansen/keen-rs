use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let Some(path) = env::args_os().nth(1) else {
        eprintln!("Usage: minsky-cli <model.mky>");
        return ExitCode::FAILURE;
    };
    // A deliberate opt-in probe; default schema-3 CLI behavior is unchanged.
    if path == "--probe-schema" {
        let Some(file) = env::args_os().nth(2) else {
            eprintln!("Usage: minsky-cli --probe-schema <model.mky>");
            return ExitCode::FAILURE;
        };
        let xml = match std::fs::read_to_string(&file) {
            Ok(xml) => xml,
            Err(e) => { eprintln!("Cannot read model: {e}"); return ExitCode::FAILURE; }
        };
        return match minsky_format::schema_probe::probe_schema(&xml) {
            Ok(generation) => {
                println!("Detected schema: {generation:?}");
                println!("Schema-3 inspection supported: {}", generation.can_inspect_with_current_parser());
                ExitCode::SUCCESS
            }
            Err(e) => { eprintln!("Schema probe failed: {e}"); ExitCode::FAILURE }
        };
    }
    match minsky_format::inspect_file(&path) {
        Ok(model) => {
            println!("Schema: {}", model.schema_version);
            println!("Minsky version: {}", model.minsky_version.as_deref().unwrap_or("unknown"));
            println!("Title: {}", model.title);
            println!("Items: {}", model.items.len());
            println!("Wires: {}", model.wires.len());
            println!("Groups: {}", model.groups.len());
            for item in &model.items {
                println!("  item {:?}: {} ({} ports)", item.id, item.kind.as_deref().unwrap_or("unknown"), item.ports.len());
            }
            let diagnostics = minsky_format::validate_structure(&model);
            for diagnostic in &diagnostics {
                eprintln!("[{}] {}", diagnostic.code, diagnostic.message);
            }
            if diagnostics.is_empty() { println!("Structural validation: OK"); ExitCode::SUCCESS }
            else { eprintln!("Structural validation: {} issue(s)", diagnostics.len()); ExitCode::FAILURE }
        }
        Err(error) => {
            eprintln!("Cannot inspect model: {error}");
            ExitCode::FAILURE
        }
    }
}
