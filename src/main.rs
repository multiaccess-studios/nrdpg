use clap::Parser;
use rand::seq::{IteratorRandom, SliceRandom};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

#[derive(Parser, Debug)]
struct Opt {
    cards: PathBuf,
    runner: usize,
    corp: usize,
}

#[derive(Debug, Copy, Clone, Ord, PartialEq, PartialOrd, Eq)]
pub enum Rarity {
    Rare,
    Uncommon,
    Common,
    Agenda,
}

#[derive(Debug, Copy, Clone, Ord, PartialEq, PartialOrd, Eq)]
pub enum Side {
    Runner,
    Corp,
}

#[derive(Debug, Copy, Clone, Ord, PartialEq, PartialOrd, Eq)]
pub enum CardType {
    Event,
    Resource,
    Program,
    Hardware,
    Agenda,
    Asset,
    Ice,
    Operation,
    Upgrade,
}

#[derive(Debug, Copy, Clone, Ord, PartialEq, PartialOrd, Eq)]
pub enum Faction {
    Neutral,
    Anarch,
    Criminal,
    Shaper,
    Jinteki,
    HaasBioroid,
    NBN,
    WeylandConsortium,
}

pub struct Card {
    name: String,
    card_type: CardType,
    rarity: Rarity,
    side: Side,
    faction: Faction,
}

fn main() {
    let opts = Opt::parse();
    let mut cards = Vec::new();
    let mut rarity_cache = BTreeMap::new();
    let mut side_cache = BTreeMap::new();
    let mut card_type_cache = BTreeMap::new();
    let mut faction_cache = BTreeMap::new();
    for file in std::fs::read_dir(&opts.cards).unwrap() {
        let file = file.unwrap();
        let json: serde_json::Value =
            serde_json::from_reader(std::fs::File::open(file.path()).unwrap()).unwrap();
        if json["designed_by"].as_str().unwrap() != "null_signal_games" {
            continue;
        }
        let name = json["stripped_title"].as_str().unwrap();
        match name {
            "Direct Access" => continue,
            "Jeitinho" => continue,
            _ => {}
        }
        let card_type = match json["card_type_id"].as_str().unwrap() {
            "runner_identity" | "corp_identity" => continue,
            "event" => CardType::Event,
            "resource" => CardType::Resource,
            "program" => CardType::Program,
            "hardware" => CardType::Hardware,
            "agenda" => CardType::Agenda,
            "asset" => CardType::Asset,
            "ice" => CardType::Ice,
            "operation" => CardType::Operation,
            "upgrade" => CardType::Upgrade,
            m => panic!("card_type_id={:?} ({name})", m),
        };
        let rarity = match json["influence_cost"].as_number().and_then(|r| r.as_i64()) {
            _ if name == "Tribuatry" => Rarity::Rare,
            _ if name == "Gold Farmer" => Rarity::Rare,
            _ if name == "Rezeki" => Rarity::Uncommon,
            _ if name == "Nanisivik Grid" => Rarity::Rare,
            _ if name == "Engram Flush" => Rarity::Rare,
            Some(0..=2) => Rarity::Common,
            Some(3) => Rarity::Uncommon,
            Some(4..=5) => Rarity::Rare,
            _ if matches!(card_type, CardType::Agenda) => Rarity::Agenda,
            n => panic!("rarity={:?} ({name})", n),
        };
        let side = match json["side_id"].as_str().unwrap() {
            "runner" => Side::Runner,
            "corp" => Side::Corp,
            m => panic!("side_id={:?} ({name})", m),
        };
        let faction = match json["faction_id"].as_str().unwrap() {
            "neutral_runner" | "neutral_corp" => Faction::Neutral,
            "anarch" => Faction::Anarch,
            "criminal" => Faction::Criminal,
            "shaper" => Faction::Shaper,
            "jinteki" => Faction::Jinteki,
            "haas_bioroid" => Faction::HaasBioroid,
            "nbn" => Faction::NBN,
            "weyland_consortium" => Faction::WeylandConsortium,
            m => panic!("faction_id={:?} ({name})", m),
        };
        let i = cards.len();
        cards.push(Card {
            name: name.to_string(),
            card_type,
            rarity,
            side,
            faction,
        });
        rarity_cache
            .entry(rarity)
            .or_insert_with(|| BTreeSet::new())
            .insert(i);
        side_cache
            .entry(side)
            .or_insert_with(|| BTreeSet::new())
            .insert(i);
        card_type_cache
            .entry(card_type)
            .or_insert_with(|| BTreeSet::new())
            .insert(i);
        faction_cache
            .entry(faction)
            .or_insert_with(|| BTreeSet::new())
            .insert(i);
    }

    println!("{}", side_cache.get(&Side::Runner).unwrap().len());

    let mut rng = rand::rng();
    for _ in 0..opts.runner {
        let mut pack = BTreeSet::new();
        let mut exclude = BTreeSet::new();
        let mut type_count = BTreeMap::<_, i32>::new();
        let constraints = [
            (Side::Runner, Rarity::Rare, None),
            (Side::Runner, Rarity::Uncommon, Some(Faction::Anarch)),
            (Side::Runner, Rarity::Uncommon, Some(Faction::Criminal)),
            (Side::Runner, Rarity::Uncommon, Some(Faction::Shaper)),
            (Side::Runner, Rarity::Common, Some(Faction::Anarch)),
            (Side::Runner, Rarity::Common, Some(Faction::Criminal)),
            (Side::Runner, Rarity::Common, Some(Faction::Shaper)),
            (Side::Runner, Rarity::Common, None),
            (Side::Runner, Rarity::Common, None),
            (Side::Runner, Rarity::Common, None),
        ];
        for (side, rarity, faction) in constraints {
            add_guarded(
                &side_cache,
                &rarity_cache,
                &faction_cache,
                &card_type_cache,
                &mut exclude,
                &mut pack,
                &cards,
                &mut type_count,
                &mut rng,
                side,
                rarity,
                faction,
                &[
                    (CardType::Event, 2),
                    (CardType::Resource, 2),
                    (CardType::Program, 2),
                    (CardType::Hardware, 1),
                ],
                3,
            );
        }

        for card in pack {
            let card = &cards[card];
            println!(
                "1 {} ({})",
                card.name,
                match card.rarity {
                    Rarity::Agenda => "⭐",
                    Rarity::Rare => "🌕",
                    Rarity::Uncommon => "🌖",
                    Rarity::Common => "🌘",
                }
            );
        }
        println!("");
        println!("");
        println!("");
    }

    for _ in 0..opts.corp {
        let mut pack = BTreeSet::new();
        let mut exclude = BTreeSet::new();
        let mut type_count = BTreeMap::<_, i32>::new();

        let mut agenda_faction_list = vec![
            Faction::HaasBioroid,
            Faction::WeylandConsortium,
            Faction::Jinteki,
            Faction::NBN,
        ];
        agenda_faction_list.shuffle(&mut rng);

        let mut uncommon_faction_list = vec![
            Faction::HaasBioroid,
            Faction::WeylandConsortium,
            Faction::Jinteki,
            Faction::NBN,
        ];
        uncommon_faction_list.shuffle(&mut rng);

        let constraints = [
            (Side::Corp, Rarity::Rare, None),
            (Side::Corp, Rarity::Agenda, Some(agenda_faction_list[0])),
            (Side::Corp, Rarity::Agenda, Some(agenda_faction_list[1])),
            (Side::Corp, Rarity::Uncommon, Some(agenda_faction_list[0])),
            (Side::Corp, Rarity::Uncommon, Some(agenda_faction_list[1])),
            (Side::Corp, Rarity::Common, Some(Faction::HaasBioroid)),
            (Side::Corp, Rarity::Common, Some(Faction::WeylandConsortium)),
            (Side::Corp, Rarity::Common, Some(Faction::Jinteki)),
            (Side::Corp, Rarity::Common, Some(Faction::NBN)),
            (Side::Corp, Rarity::Common, None),
        ];

        for (side, rarity, faction) in constraints {
            add_guarded(
                &side_cache,
                &rarity_cache,
                &faction_cache,
                &card_type_cache,
                &mut exclude,
                &mut pack,
                &cards,
                &mut type_count,
                &mut rng,
                side,
                rarity,
                faction,
                &[
                    (CardType::Asset, 2),
                    (CardType::Ice, 2),
                    (CardType::Operation, 2),
                    (CardType::Agenda, 2),
                ],
                2,
            );
        }

        for card in pack {
            let card = &cards[card];
            println!(
                "1 {} ({})",
                card.name,
                match card.rarity {
                    Rarity::Agenda => "⭐",
                    Rarity::Rare => "🌕",
                    Rarity::Uncommon => "🌖",
                    Rarity::Common => "🌘",
                }
            );
        }
        println!("");
        println!("");
        println!("");
    }
}

fn add_guarded(
    side_cache: &BTreeMap<Side, BTreeSet<usize>>,
    rarity_cache: &BTreeMap<Rarity, BTreeSet<usize>>,
    faction_cache: &BTreeMap<Faction, BTreeSet<usize>>,
    card_type_cache: &BTreeMap<CardType, BTreeSet<usize>>,
    exclude: &mut BTreeSet<usize>,
    pack: &mut BTreeSet<usize>,
    cards: &Vec<Card>,
    type_count: &mut BTreeMap<CardType, i32>,
    rng: &mut rand::prelude::ThreadRng,
    side: Side,
    rarity: Rarity,
    faction: Option<Faction>,
    type_soft_limits: &[(CardType, i32)],
    type_hard_limit: i32,
) {
    let side_cards = side_cache.get(&side).unwrap();
    let rarity_cards = rarity_cache.get(&rarity).unwrap();
    let mut pool = side_cards
        .intersection(rarity_cards)
        .copied()
        .collect::<BTreeSet<_>>();
    if let Some(faction) = faction {
        let faction_cards = faction_cache.get(&faction).unwrap();
        pool = pool.intersection(faction_cards).copied().collect();
    }
    let Some(chosen) = pool
        .into_iter()
        .filter(|f| !exclude.contains(f))
        .choose(rng)
    else {
        for card in pack.iter() {
            println!("{}", cards[*card].name);
            println!("Could not select {:?}{:?}{:?}", side, rarity, faction);
        }
        panic!();
    };
    exclude.insert(chosen);
    pack.insert(chosen);
    let chosen_card = &cards[chosen];
    let chosen_type = chosen_card.card_type;
    *type_count.entry(chosen_type).or_default() += 1;
    let mut should_exclude = false;
    if type_soft_limits
        .into_iter()
        .map(|&(entry_type, soft_limit)| {
            let val = type_count.get(&entry_type).copied().unwrap_or(0);
            std::cmp::max(val - soft_limit, 0)
        })
        .sum::<i32>()
        == type_hard_limit
    {
        should_exclude = true;
    }
    if should_exclude {
        exclude.extend(card_type_cache.get(&chosen_type).unwrap().iter().copied());
    }
}
