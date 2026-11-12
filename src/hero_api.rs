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
    icon_image_small: Option<String>,
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

// Hero keys from the client VMDL path are model folder names, which don't always match the API
// class_name. Try the key as-is, then without the hero_ prefix, then the static display name.
fn fetch(client: &ureq::Agent, hero_key: &str, portrait_style: HeroPortraitStyle) -> Result<HeroData, Box<dyn std::error::Error>> {
    let stripped = hero_key.trim_start_matches("hero_");
    let candidates = [Some(hero_key), Some(stripped), static_fallback(hero_key)];
    for name in candidates.into_iter().flatten() {
        match fetch_by_name(client, hero_key, name, portrait_style) {
            Ok(data) => return Ok(data),
            Err(err) => debug!("[api] Lookup \"{name}\" for {hero_key} failed: {err}"),
        }
    }
    Err(format!("unknown hero: {hero_key}").into())
}

// Display names for every active hero, keyed by the VMDL folder key and the API class_name where
// the two differ.
fn static_fallback(hero_key: &str) -> Option<&'static str> {
    let name = match hero_key {
        "hero_abrams" | "hero_atlas" => "Abrams",
        "hero_fencer" => "Apollo",
        "hero_baba" => "Baba",
        "hero_bebop" => "Bebop",
        "hero_punkgoat" => "Billy",
        "hero_nano" => "Calico",
        "hero_unicorn" => "Celeste",
        "hero_deadpack" => "Deadman Danny",
        "hero_drifter" => "Drifter",
        "hero_dynamo" => "Dynamo",
        "hero_necro" => "Graves",
        "hero_archer" | "hero_orion" => "Grey Talon",
        "hero_haze" => "Haze",
        "hero_astro" => "Holliday",
        "hero_inferno" => "Infernus",
        "hero_ivy" | "hero_tengu" => "Ivy",
        "hero_kelvin" => "Kelvin",
        "hero_geist" | "hero_ghost" => "Lady Geist",
        "hero_lash" => "Lash",
        "hero_mcginnis" | "hero_forge" => "McGinnis",
        "hero_vampirebat" => "Mina",
        "hero_mirage" => "Mirage",
        "hero_digger" | "hero_krill" => "Mo & Krill",
        "hero_nurse" => "Nurse Harrow",
        "hero_bookworm" => "Paige",
        "hero_chrono" => "Paradox",
        "hero_pocket" | "hero_synth" => "Pocket",
        "hero_ratking" => "Rat King",
        "hero_familiar" => "Rem",
        "hero_gigawatt_prisoner" | "hero_gigawatt" => "Seven",
        "hero_shiv" => "Shiv",
        "hero_werewolf" => "Silver",
        "hero_magician" => "Sinclair",
        "hero_chessmaster" => "Solomon",
        "hero_doorman" => "The Doorman",
        "hero_priest" => "Venator",
        "hero_frank" => "Victor",
        "hero_hornet" => "Vindicta",
        "hero_artist" => "Violet",
        "hero_viscous" => "Viscous",
        "hero_viper" => "Vyper",
        "hero_warden" => "Warden",
        "hero_wraith" => "Wraith",
        "hero_yamato" => "Yamato",
        _ => return None,
    };
    Some(name)
}

fn fetch_by_name(client: &ureq::Agent, hero_key: &str, name: &str, portrait_style: HeroPortraitStyle) -> Result<HeroData, Box<dyn std::error::Error>> {
    let encoded = name.replace(' ', "%20").replace('&', "%26");
    let url = format!("https://api.deadlock-api.com/v1/assets/heroes/by-name/{encoded}");
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
        HeroPortraitStyle::Icon => {
            let icon = images.icon_image_small.filter(|s| !s.is_empty());
            if icon.is_some() {
                debug!("[api] {hero_key}: using icon portrait");
            } else {
                debug!("[api] {hero_key}: icon portrait unavailable, falling back to icon_hero_card");
            }
            icon.or(images.icon_hero_card).unwrap_or_default()
        }
        HeroPortraitStyle::FullBody => {
            debug!("[api] {hero_key}: using full body portrait (Discord asset key)");
            hero_key.to_string()
        }
    };
    Ok(HeroData {
        name: hero.name.unwrap_or_else(|| hero_key.trim_start_matches("hero_").to_string()),
        hideout_text: hero.hideout_rich_presence.unwrap_or_default(),
        icon_url,
    })
}
