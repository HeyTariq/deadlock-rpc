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

#[derive(Deserialize)]
struct HeroIndexEntry {
    class_name: String,
    name: String,
}

pub struct HeroCache {
    map: HashMap<String, HeroData>,
    // class_name → display name, loaded at startup from the heroes index endpoint
    hero_index: HashMap<String, String>,
    client: ureq::Agent,
    portrait_style: HeroPortraitStyle,
}

impl HeroCache {
    pub fn new(portrait_style: HeroPortraitStyle) -> Self {
        let client = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(5))
            .build();
        let hero_index = fetch_hero_index(&client);
        Self { map: HashMap::new(), hero_index, client, portrait_style }
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
        let hero_index = &self.hero_index;
        match self.map.entry(hero_key.to_owned()) {
            Entry::Occupied(e) => Some(e.into_mut()),
            Entry::Vacant(e) => match fetch(&self.client, hero_key, self.portrait_style, hero_index) {
                Ok(data) => {
                    info!("[api] Cached: {} → \"{}\"", hero_key, data.name);
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

fn fetch_hero_index(client: &ureq::Agent) -> HashMap<String, String> {
    const URL: &str = "https://api.deadlock-api.com/v1/assets/heroes?only_active=true";
    debug!("[api] Loading hero index from {URL}");
    let result: Result<Vec<HeroIndexEntry>, Box<dyn std::error::Error>> =
        (|| Ok(client.get(URL).call()?.into_json()?))();
    match result {
        Ok(entries) => {
            let count = entries.len();
            let map = entries.into_iter().map(|e| (e.class_name, e.name)).collect();
            info!("[api] Loaded hero index: {count} heroes");
            map
        }
        Err(err) => {
            warn!("[api] Failed to load hero index, falling back to static dict: {err}");
            HashMap::new()
        }
    }
}

fn fetch(client: &ureq::Agent, hero_key: &str, portrait_style: HeroPortraitStyle, hero_index: &HashMap<String, String>) -> Result<HeroData, Box<dyn std::error::Error>> {
    debug!("[api] Fetching: {hero_key}");

    if let Some(display_name) = hero_index.get(hero_key) {
        debug!("[api] Index lookup: {hero_key} → \"{display_name}\"");
        if let Ok(data) = fetch_by_name(client, display_name, portrait_style) {
            return Ok(data);
        }
    }

    let stripped = hero_key.trim_start_matches("hero_");
    if let Ok(data) = fetch_by_name(client, stripped, portrait_style) {
        debug!("[api] Resolved via stripped key: {stripped}");
        return Ok(data);
    }

    if let Some(display_name) = static_fallback(hero_key) {
        debug!("[api] Static fallback: {hero_key} → \"{display_name}\"");
        if let Ok(data) = fetch_by_name(client, display_name, portrait_style) {
            return Ok(data);
        }
    }

    Err(format!("unknown hero: {hero_key}").into())
}

// Backup mapping for when the hero index endpoint is unreachable at startup.
fn static_fallback(asset_key: &str) -> Option<&'static str> {
    match asset_key {
        "hero_inferno"  => Some("Infernus"),
        "hero_gigawatt_prisoner" => Some("Seven"),
        "hero_hornet"   => Some("Vindicta"),
        "hero_geist"    => Some("Lady Geist"),
        "hero_atlas"    => Some("Abrams"),
        "hero_wraith"   => Some("Wraith"),
        "hero_forge"    => Some("McGinnis"),
        "hero_dynamo"   => Some("Dynamo"),
        "hero_haze"     => Some("Haze"),
        "hero_kelvin"   => Some("Kelvin"),
        "hero_lash"     => Some("Lash"),
        "hero_bebop"    => Some("Bebop"),
        "hero_shiv"     => Some("Shiv"),
        "hero_viscous"  => Some("Viscous"),
        "hero_warden"   => Some("Warden"),
        "hero_yamato"   => Some("Yamato"),
        "hero_archer"    => Some("Grey Talon"),
        "hero_digger"    => Some("Mo & Krill"),
        "hero_synth"    => Some("Pocket"),
        "hero_chrono"   => Some("Paradox"),
        "hero_astro"    => Some("Holliday"),
        "hero_cadence"  => Some("Calico"),
        "hero_werewolf" => Some("Silver"),
        "hero_magician" => Some("Sinclair"),
        "hero_tengu"    => Some("Ivy"),
        _ => None,
    }
}

fn fetch_by_name(client: &ureq::Agent, name: &str, portrait_style: HeroPortraitStyle) -> Result<HeroData, Box<dyn std::error::Error>> {
    let url = format!("https://assets.deadlock-api.com/v2/heroes/by-name/{name}");
    debug!("[api] GET {url}");
    let hero: ApiHero = client.get(&url).call()?.into_json()?;
    let images = hero.images.ok_or("hero not found")?;
    let icon_url = match portrait_style {
        HeroPortraitStyle::Normal => {
            debug!("[api] {name}: using normal portrait");
            images.icon_hero_card.unwrap_or_default()
        }
        HeroPortraitStyle::Gloat => {
            let gloat = images.hero_card_gloat.filter(|s| !s.is_empty());
            if gloat.is_some() {
                debug!("[api] {name}: using gloat portrait");
            } else {
                debug!("[api] {name}: gloat portrait unavailable, falling back to icon_hero_card");
            }
            gloat.or(images.icon_hero_card).unwrap_or_default()
        }
        HeroPortraitStyle::Critical => {
            let critical = images.hero_card_critical.filter(|s| !s.is_empty());
            if critical.is_some() {
                debug!("[api] {name}: using critical portrait");
            } else {
                debug!("[api] {name}: critical portrait unavailable, falling back to icon_hero_card");
            }
            critical.or(images.icon_hero_card).unwrap_or_default()
        }
    };
    Ok(HeroData {
        name: hero.name.unwrap_or_else(|| name.trim_start_matches("hero_").to_string()),
        hideout_text: hero.hideout_rich_presence.unwrap_or_default(),
        icon_url,
    })
}
