//! Model and frame budget checks for the baker.

use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub struct FrameBudget {
    pub triangles: u64,
    pub shadow_triangles: u64,
    pub draw_calls: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Budgets {
    pub frame: FrameBudget,
    pub models: HashMap<String, u32>,
}

impl Budgets {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read budgets file at {}: {e}", path.display()))?;
        parse_budgets_json(&content)
    }
}

pub fn parse_budgets_json(text: &str) -> Result<Budgets, String> {
    let mut frame = FrameBudget {
        triangles: 0,
        shadow_triangles: 0,
        draw_calls: 0,
    };
    let mut models = HashMap::new();
    let mut in_models = false;
    let mut in_frame = false;

    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("\"frame\"") {
            in_frame = true;
            in_models = false;
        } else if line.starts_with("\"models\"") {
            in_models = true;
            in_frame = false;
        } else if line.starts_with('}') {
            in_frame = false;
            in_models = false;
        } else if in_frame
            && let Some((k, v)) = line.split_once(':')
        {
            let k = k.trim().trim_matches('"');
            let v = v.trim().trim_end_matches(',').parse::<u64>().unwrap_or(0);
            match k {
                "triangles" => frame.triangles = v,
                "shadowTriangles" | "shadow_triangles" => frame.shadow_triangles = v,
                "drawCalls" | "draw_calls" => frame.draw_calls = v,
                _ => {}
            }
        } else if in_models
            && let Some((k, v)) = line.split_once(':')
        {
            let k = k.trim().trim_matches('"');
            let v_str = v.trim().trim_end_matches(',');
            if let Ok(cap) = v_str.parse::<u32>() {
                models.insert(k.to_string(), cap);
            }
        }
    }
    Ok(Budgets { frame, models })
}

pub fn calculate_folder_cap(current_biggest: usize) -> u32 {
    let double = current_biggest * 2;
    let plus_2000 = current_biggest + 2000;
    let val = double.max(plus_2000);
    let rem = val % 1000;
    if rem == 0 {
        val as u32
    } else {
        (val + (1000 - rem)) as u32
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModelBudgetReport {
    pub folder: String,
    pub model: String,
    pub tris: usize,
    pub cap: u32,
    pub over_budget: bool,
}

pub fn check_model_budget(folder: &str, model: &str, tris: usize, cap: Option<u32>) -> ModelBudgetReport {
    let cap = cap.unwrap_or(u32::MAX);
    ModelBudgetReport {
        folder: folder.to_string(),
        model: model.to_string(),
        tris,
        cap,
        over_budget: tris as u32 > cap,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bench_calculate_folder_cap() {
        assert_eq!(calculate_folder_cap(1800), 4000);
        assert_eq!(calculate_folder_cap(5614), 12000);
        assert_eq!(calculate_folder_cap(600), 3000);
        assert_eq!(calculate_folder_cap(2000), 4000);
    }

    #[test]
    fn bench_budget_check_within_cap() {
        let rep = check_model_budget("animals", "deer", 1632, Some(4000));
        assert!(!rep.over_budget);
        assert_eq!(rep.cap, 4000);
    }

    #[test]
    fn bench_budget_check_over_cap() {
        let rep = check_model_budget("buildings", "super_tower", 15000, Some(12000));
        assert!(rep.over_budget);
    }

    #[test]
    fn bench_parse_budgets_json() {
        let json = r#"{
            "frame": {
                "triangles": 120000,
                "shadowTriangles": 80000,
                "drawCalls": 50
            },
            "models": {
                "animals": 4000,
                "buildings": 12000
            }
        }"#;
        let b = parse_budgets_json(json).unwrap();
        assert_eq!(b.frame.triangles, 120000);
        assert_eq!(b.frame.shadow_triangles, 80000);
        assert_eq!(b.frame.draw_calls, 50);
        assert_eq!(b.models.get("animals"), Some(&4000));
        assert_eq!(b.models.get("buildings"), Some(&12000));
    }
}
