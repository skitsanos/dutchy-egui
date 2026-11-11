//! A deterministic, fictional mailbox: invented people, reserved-domain
//! addresses and generated subjects, the same on every run.

use crate::mailbox::{
    ARCHIVE, DRAFTS, FIRST_CUSTOM, Folder, FolderId, INBOX, Mailbox, Message, MessageId, SENT,
    TRASH,
};
use crate::support::data::{PEOPLE, Rng, SEED, email};
use dutchy_egui::calendar::Date;

/// Messages generated for the seeded mailbox.
pub const MESSAGES: usize = 2_400;

pub const PROJECTS: FolderId = FolderId(FIRST_CUSTOM);
pub const GARDEN: FolderId = FolderId(FIRST_CUSTOM + 1);
pub const STUDIO: FolderId = FolderId(FIRST_CUSTOM + 2);
pub const TRAVEL: FolderId = FolderId(FIRST_CUSTOM + 3);
pub const NEWSLETTERS: FolderId = FolderId(FIRST_CUSTOM + 4);

const TOPICS: [&str; 12] = [
    "Garden club minutes",
    "Studio build schedule",
    "Ferry times for Saturday",
    "Seed swap list",
    "Invoice for the shelving",
    "Photos from the canal walk",
    "Reading group: next title",
    "Allotment water rota",
    "Workshop booking confirmed",
    "Draft agenda",
    "Bike repair estimate",
    "Weekend market stall",
];
const PREFIXES: [&str; 4] = ["", "Re: ", "Fwd: ", "Update: "];
const OPENINGS: [&str; 5] = [
    "Thanks for the quick reply.",
    "A short update before the weekend.",
    "Here is what we agreed on Tuesday.",
    "Could you take a look when you have a moment?",
    "Good news on this one.",
];

fn folders() -> Vec<Folder> {
    let top = |id, name: &str| Folder {
        id,
        name: name.into(),
        parent: None,
    };
    let child = |id, name: &str| Folder {
        id,
        name: name.into(),
        parent: Some(PROJECTS),
    };
    vec![
        top(INBOX, "Inbox"),
        top(DRAFTS, "Drafts"),
        top(SENT, "Sent"),
        top(ARCHIVE, "Archive"),
        top(TRASH, "Trash"),
        top(PROJECTS, "Projects"),
        child(GARDEN, "Garden club"),
        child(STUDIO, "Studio build"),
        child(TRAVEL, "Travel"),
        top(NEWSLETTERS, "Newsletters"),
    ]
}

/// Inbox-heavy folder spread; Drafts and Trash stay small.
fn folder_for(rng: &mut Rng) -> FolderId {
    match rng.below(100) {
        0..=44 => INBOX,
        45..=46 => DRAFTS,
        47..=54 => SENT,
        55..=79 => ARCHIVE,
        80..=81 => TRASH,
        82..=86 => GARDEN,
        87..=91 => STUDIO,
        92..=94 => TRAVEL,
        _ => NEWSLETTERS,
    }
}

fn body(rng: &mut Rng, from: &str, topic: &str) -> String {
    let first = from.split_whitespace().next().unwrap_or(from);
    format!(
        "Hi,\n\n{} About **{}**:\n\n- {}\n- {}\n\n> Reply when it suits you; nothing here is urgent.\n\nBest,\n{first}",
        rng.pick(&OPENINGS),
        topic.to_lowercase(),
        rng.pick(&[
            "The list is attached in the shared folder.",
            "Times are in the calendar invite.",
            "Numbers are still rough."
        ]),
        rng.pick(&[
            "Bring a spare key.",
            "Let me know if Thursday works.",
            "No need to print anything."
        ]),
    )
}

/// The seeded mailbox, anchored on `today`.
pub fn seeded(today: Date) -> Mailbox {
    let mut rng = Rng::seeded(SEED ^ 0x4d41_494c);
    let messages = (0..MESSAGES)
        .map(|index| {
            let from = (*rng.pick(&PEOPLE)).to_owned();
            let topic = *rng.pick(&TOPICS);
            let folder = folder_for(&mut rng);
            // Spread over about four months; the newest arrive today.
            let age = i64::from(rng.below(120)) * i64::from(rng.below(100)) / 100;
            Message {
                id: MessageId(index as u64 + 1),
                folder,
                address: email(&from),
                subject: format!("{}{topic}", rng.pick(&PREFIXES)),
                body: body(&mut rng, &from, topic),
                from,
                date: today.add_days(-age).unwrap_or(today),
                minute: rng.below(24 * 60) as u16,
                unread: folder != SENT
                    && folder != DRAFTS
                    && rng.chance(if age < 7 { 60 } else { 8 }),
            }
        })
        .collect();
    Mailbox::new(folders(), messages)
}

/// The message a simulated sync delivers.
pub fn arrival(today: Date, count: usize) -> Message {
    let mut rng = Rng::seeded(SEED ^ count as u64);
    let from = (*rng.pick(&PEOPLE)).to_owned();
    let topic = *rng.pick(&TOPICS);
    Message {
        id: MessageId(0),
        folder: INBOX,
        address: email(&from),
        subject: topic.to_owned(),
        body: body(&mut rng, &from, topic),
        from,
        date: today,
        minute: 23 * 60 + 59,
        unread: true,
    }
}
