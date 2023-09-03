use strum_macros::EnumIter;

// Define an enum called `Region` to represent geographical regions.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, poise::ChoiceParameter, EnumIter, Eq, Hash, PartialEq)]
pub enum Region {
    #[name = "North America & South America"]
    NASA,
    #[name = "Europe"]
    EU,
    #[name = "Asia & Oceania"]
    APAC,
}

impl Region {
    /// Custom function to find a variant by its associated name.
    ///
    /// # Arguments
    ///
    /// * `name` - A string containing the associated name of the variant to find.
    ///
    /// # Returns
    ///
    /// * `Some(Region)` - The enum variant if found.
    /// * `None` - If no variant is found for the given name.
    ///
    /// # Example
    ///
    /// ```
    /// use my_module::Region;
    ///
    /// let name_to_find = "Europe";
    ///
    /// if let Some(region) = Region::find_key(name_to_find) {
    ///     println!("Found variant: {:?}", region);
    /// } else {
    ///     println!("Variant not found for name: {}", name_to_find);
    /// }
    /// ```
    pub fn find_key(name: &str) -> Option<Region> {
        match name {
            "NASA" => Some(Region::NASA),
            "EU" => Some(Region::EU),
            "APAC" => Some(Region::APAC),
            _ => None,
        }
    }
}
