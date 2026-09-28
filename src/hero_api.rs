use crate::config::HeroPortraitStyle;
use log::{debug, info, warn};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HeroData {
    pub name: String,
    pub hideout_text: String,
    pub icon_url: String,
}

#[derive(Deserialize)]
struct ApiHero {
    name: Option<String>,
    hideout_rich_presence: Option<String>,
    images: Option<ApiImages>,
}

#[derive(Deserialize)]
struct ApiImages {
    icon_hero_card: Option<String>,
    hero_card_gloat: Option<String>,
    hero_card_critical: Option<String>,
}

pub struct HeroCache {
    map: HashMap<String, HeroData>,
    client: ureq::Agent,
    portrait_style: HeroPortraitStyle,
}

impl HeroCache {
    pub fn new(portrait_style: HeroPortraitStyle) -> Self {
        let client = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(5))
            .build();
        Self { map: HashMap::new(), client, portrait_style }
    }

    pub fn set_portrait_style(&mut self, style: HeroPortraitStyle) {
        if self.portrait_style != style {
            self.portrait_style = style;
            self.map.clear();
        }
    }

    // Returns cached data if available, otherwise fetches from the API using the hero class_name.
    pub fn get_or_fetch(&mut self, hero_key: &str) -> Option<&HeroData> {
        use std::collections::hash_map::Entry;
        match self.map.entry(hero_key.to_owned()) {
            Entry::Occupied(e) => Some(e.into_mut()),
            Entry::Vacant(e) => match fetch(&self.client, hero_key, self.portrait_style) {
                Ok(data) => {
                    info!("[api] Cached {}: \"{}\"", hero_key, data.name);
                    Some(e.insert(data))
                }
                Err(err) => {
                    warn!("[api] Failed to fetch {hero_key}: {err}");
                    None
                }
            },
        }
    }
}

// The endpoint resolves a hero by class_name, display name, or the bare form of either, so the
// normalised class_name from the log watcher can be passed straight through.
fn fetch(client: &ureq::Agent, hero_key: &str, portrait_style: HeroPortraitStyle) -> Result<HeroData, Box<dyn std::error::Error>> {
    let url = format!("https://api.deadlock-api.com/v1/assets/heroes/by-name/{hero_key}");
    debug!("[api] GET {url}");
    let hero: ApiHero = client.get(&url).call()?.into_json()?;
    let images = hero.images.ok_or("hero has no images")?;
    let icon_url = match portrait_style {
        HeroPortraitStyle::Normal => {
            debug!("[api] {hero_key}: using normal portrait");
            images.icon_hero_card.unwrap_or_default()
        }
        HeroPortraitStyle::Gloat => {
            let gloat = images.hero_card_gloat.filter(|s| !s.is_empty());
            if gloat.is_some() {
                debug!("[api] {hero_key}: using gloat portrait");
            } else {
                debug!("[api] {hero_key}: gloat portrait unavailable, falling back to icon_hero_card");
            }
            gloat.or(images.icon_hero_card).unwrap_or_default()
        }
        HeroPortraitStyle::Critical => {
            let critical = images.hero_card_critical.filter(|s| !s.is_empty());
            if critical.is_some() {
                debug!("[api] {hero_key}: using critical portrait");
            } else {
                debug!("[api] {hero_key}: critical portrait unavailable, falling back to icon_hero_card");
            }
            critical.or(images.icon_hero_card).unwrap_or_default()
        }
    };
    Ok(HeroData {
        name: hero.name.unwrap_or_else(|| hero_key.trim_start_matches("hero_").to_string()),
        hideout_text: hero.hideout_rich_presence.unwrap_or_default(),
        icon_url,
    })
}
