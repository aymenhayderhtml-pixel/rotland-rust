/// Formats the driving gauge text (port of scripts/Vehicles/CarGauge.cs).
pub fn describe_gauge(
    speed_kmh: f32,
    fuel_litres: f32,
    fuel_fraction: f32,
    health: f32,
    max_health: f32,
    is_wrecked: bool,
    is_empty: bool,
) -> String {
    let bar_cells = 10;
    let filled = (fuel_fraction * bar_cells as f32).round() as usize;
    let filled = filled.min(bar_cells);
    let bar = "#".repeat(filled) + &"-".repeat(bar_cells - filled);
    let state = if is_wrecked {
        "  WRECKED"
    } else if is_empty {
        "  NO FUEL"
    } else {
        ""
    };
    format!(
        "{:.0} km/h\nFuel [{bar}] {fuel_litres:.1} L\nCar {health:.0}/{max_health:.0}{state}",
        speed_kmh.abs()
    )
}
