//! Seeded fictional projects and tasks. Due dates are offsets from the
//! supplied date, so the list always shows overdue, due-today and upcoming work.

use crate::store::{Priority, ProjectId, Store, TaskDraft};
use crate::support::data::{Rng, SEED};
use dutchy_egui::calendar::Date;

struct Seed {
    title: &'static str,
    notes: &'static str,
    due: Option<i64>,
    priority: Priority,
    done: bool,
}

const fn seed(
    title: &'static str,
    notes: &'static str,
    due: Option<i64>,
    priority: Priority,
    done: bool,
) -> Seed {
    Seed {
        title,
        notes,
        due,
        priority,
        done,
    }
}

const GARDEN: [Seed; 6] = [
    seed(
        "Order seed trays",
        "Twelve modules each; the greenhouse shelf fits four trays.",
        Some(-3),
        Priority::High,
        false,
    ),
    seed(
        "Repair the fence post by the gate",
        "The concrete footing cracked over winter.",
        Some(-1),
        Priority::Medium,
        false,
    ),
    seed(
        "Plant garlic in the raised bed",
        "Cloves are in the shed, pointed end up, ten centimetres apart.",
        Some(0),
        Priority::High,
        false,
    ),
    seed(
        "Prune the apple trees",
        "Only dead and crossing branches before the sap rises.",
        Some(6),
        Priority::Low,
        false,
    ),
    seed(
        "Compare rainwater tanks",
        "Two hundred litres minimum, with a tap near the base.",
        None,
        Priority::Low,
        false,
    ),
    seed("Clear the gutters", "", Some(-7), Priority::Medium, true),
];

const KITCHEN: [Seed; 5] = [
    seed(
        "Measure the alcove for the fridge",
        "Allow fifteen millimetres each side for ventilation.",
        Some(1),
        Priority::High,
        false,
    ),
    seed(
        "Choose worktop samples",
        "Oak or the pale laminate; bring both home for the light.",
        Some(4),
        Priority::Medium,
        false,
    ),
    seed(
        "Book the electrician",
        "Two extra sockets and the cooker circuit.",
        Some(9),
        Priority::Medium,
        false,
    ),
    seed(
        "Empty the pantry shelves",
        "",
        Some(-2),
        Priority::Low,
        true,
    ),
    seed(
        "Return the broken tap",
        "Receipt is in the drawer under the kettle.",
        Some(-4),
        Priority::High,
        true,
    ),
];

const READING_ADJECTIVES: [&str; 6] = ["Quiet", "Copper", "Northern", "Late", "Paper", "Blue"];
const READING_NOUNS: [&str; 6] = [
    "Harbours", "Orchards", "Lanterns", "Rivers", "Letters", "Maps",
];

/// Build the demo store and return it with the first project selected.
pub fn seeded(today: Date) -> (Store, ProjectId) {
    let mut store = Store::new();
    let garden = store.add_project("Garden renovation");
    fill(&mut store, garden, &GARDEN, today);
    let kitchen = store.add_project("Kitchen refit");
    fill(&mut store, kitchen, &KITCHEN, today);
    let reading = store.add_project("Reading list");
    reading_list(&mut store, reading, today);
    (store, garden)
}

fn fill(store: &mut Store, project: ProjectId, seeds: &[Seed], today: Date) {
    for seed in seeds {
        let id = store.add_task(
            project,
            TaskDraft {
                title: seed.title.into(),
                notes: seed.notes.into(),
                due: seed.due.and_then(|days| today.add_days(days)),
                priority: seed.priority,
            },
        );
        if seed.done {
            store.set_done(id, true);
        }
    }
}

/// Generated titles from two word lists, with seeded dates and priorities.
fn reading_list(store: &mut Store, project: ProjectId, today: Date) {
    let mut rng = Rng::seeded(SEED);
    for adjective in READING_ADJECTIVES {
        let noun = rng.pick(&READING_NOUNS);
        let due = rng
            .chance(70)
            .then(|| today.add_days(i64::from(rng.below(21)) - 5))
            .flatten();
        let priority = *rng.pick(&Priority::ALL);
        let id = store.add_task(
            project,
            TaskDraft {
                title: format!("Read \"{adjective} {noun}\""),
                notes: format!(
                    "Recommended by {}.",
                    rng.pick(&crate::support::data::PEOPLE)
                ),
                due,
                priority,
            },
        );
        if rng.chance(30) {
            store.set_done(id, true);
        }
    }
}
