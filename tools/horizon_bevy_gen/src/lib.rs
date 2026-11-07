pub mod ast;
pub mod generator;

use std::fs;
use std::path::Path;

pub use ast::parser::parse_rust_source;
pub use ast::AstModule;
pub use generator::{generate_crate, write_generated_crate, GeneratedCrate};

/// High-level function: parses Rust files and generates the complete Bevy Horizon crate
pub fn generate_from_paths(sources: &[&Path], output_dir: &Path) -> Result<GeneratedCrate, String> {
    let mut modules = Vec::new();

    for path in sources {
        if path.is_file() {
            let content = fs::read_to_string(path)
                .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
            let module = parse_rust_source(&content, &path.to_string_lossy())?;
            modules.push(module);
        } else if path.is_dir() {
            for entry in fs::read_dir(path)
                .map_err(|e| format!("Failed to read dir {}: {e}", path.display()))?
            {
                let entry = entry.map_err(|e| e.to_string())?;
                let entry_path = entry.path();
                if entry_path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    let content = fs::read_to_string(&entry_path)
                        .map_err(|e| format!("Failed to read {}: {e}", entry_path.display()))?;
                    let module = parse_rust_source(&content, &entry_path.to_string_lossy())?;
                    modules.push(module);
                }
            }
        }
    }

    let generated = generate_crate(&modules);
    write_generated_crate(output_dir, &generated)?;
    Ok(generated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_parsing_and_codegen() {
        let rust_code = r#"
            pub struct Mob {
                pub id: u32,
                pub kind: String,
                pub pos: [f64; 3],
                pub health: f32,
                pub max_health: f32,
            }

            pub fn entity_control_interact(game: &mut Game, target: u32) {
                // hook
            }
        "#;

        let module = parse_rust_source(rust_code, "test.rs").expect("failed to parse");
        assert_eq!(module.structs.len(), 1);
        assert_eq!(module.structs[0].name, "Mob");
        assert_eq!(module.functions.len(), 1);
        assert!(module.functions[0].is_native_hook);

        let generated = generate_crate(&[module]);
        assert!(generated.components_rs.contains("pub struct Position"));
        assert!(generated.components_rs.contains("pub struct Zombie"));
        assert!(generated.events_rs.contains("pub struct EntityInteractEvent"));
        assert!(generated.bridge_rs.contains("pub struct HorizonBridge"));
    }
}
