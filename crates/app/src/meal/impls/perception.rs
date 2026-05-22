use domain::FoodItem;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ParsedFoodItem {
    name: String,
    quantity: f32,
    unit: String,
}

pub fn parse_food_items_from_perception(raw: &str) -> Result<Vec<FoodItem>, String> {
    let json_like = match (raw.find('['), raw.rfind(']')) {
        (Some(start), Some(end)) if start <= end => &raw[start..=end],
        _ => raw,
    };

    let parsed: Vec<ParsedFoodItem> =
        serde_json::from_str(json_like).map_err(|err| err.to_string())?;
    Ok(parsed
        .into_iter()
        .filter(|item| !item.name.trim().is_empty())
        .map(|item| FoodItem::new(item.name, item.quantity.max(0.0), item.unit))
        .collect())
}
