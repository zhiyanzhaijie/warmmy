use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Nutrition {
    pub calories: f32,
    pub protein_g: f32,
    pub fat_g: f32,
    pub carbs_g: f32,
}

impl Nutrition {
    pub fn add(&self, other: &Nutrition) -> Nutrition {
        Nutrition {
            calories: self.calories + other.calories,
            protein_g: self.protein_g + other.protein_g,
            fat_g: self.fat_g + other.fat_g,
            carbs_g: self.carbs_g + other.carbs_g,
        }
    }
}
