//! The in-memory mailbox the application owns: folders, messages and the
//! moves that archive, delete and restore them.

use dutchy_egui::{FuzzyMatch, calendar::Date, fuzzy_match};
use std::{cmp::Reverse, collections::HashMap, ops::Range};

/// Stable folder identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FolderId(pub u32);

/// Stable message identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MessageId(pub u64);

/// Well-known folders. Custom folders get identities from `FIRST_CUSTOM`.
pub const INBOX: FolderId = FolderId(1);
pub const DRAFTS: FolderId = FolderId(2);
pub const SENT: FolderId = FolderId(3);
pub const ARCHIVE: FolderId = FolderId(4);
pub const TRASH: FolderId = FolderId(5);
pub const FIRST_CUSTOM: u32 = 100;

#[derive(Clone, Debug)]
pub struct Folder {
    pub id: FolderId,
    pub name: String,
    /// Parent folder for nesting; `None` for top-level folders.
    pub parent: Option<FolderId>,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub id: MessageId,
    pub folder: FolderId,
    pub from: String,
    pub address: String,
    pub subject: String,
    /// Markdown body.
    pub body: String,
    pub date: Date,
    /// Minutes after midnight, for ordering within a day.
    pub minute: u16,
    pub unread: bool,
}

/// A message in a search result, with the byte ranges of its sender and
/// subject that the query matched.
pub struct Found<'a> {
    pub message: &'a Message,
    pub from: Vec<Range<usize>>,
    pub subject: Vec<Range<usize>>,
    /// One score per query word.
    scores: Vec<i32>,
}

/// Match each word against the sender and the subject and keep the better
/// field, so "mara build" finds Mara's mail about the build.
fn find<'a>(message: &'a Message, words: &[&str]) -> Option<Found<'a>> {
    let (mut from, mut subject) = (FuzzyMatch::default(), FuzzyMatch::default());
    let mut scores = Vec::with_capacity(words.len());
    for word in words {
        let in_from = fuzzy_match(word, &message.from);
        let in_subject = fuzzy_match(word, &message.subject);
        let (field, found) = match (in_from, in_subject) {
            (Some(f), Some(s)) if s.score > f.score => (&mut subject, s),
            (Some(f), _) => (&mut from, f),
            (None, Some(s)) => (&mut subject, s),
            (None, None) => return None,
        };
        scores.push(found.score);
        field.merge(found);
    }
    Some(Found {
        message,
        from: from.ranges,
        subject: subject.ranges,
        scores,
    })
}

/// A move and the per-message state Undo can restore.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Moved {
    sequence: u64,
    origins: Vec<MoveOrigin>,
}

impl Moved {
    pub fn count(&self) -> usize {
        self.origins.len()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MoveOrigin {
    id: MessageId,
    folder: FolderId,
    previous_move: Option<u64>,
}

#[derive(Default)]
pub struct Mailbox {
    folders: Vec<Folder>,
    messages: Vec<Message>,
    next_id: u64,
    move_sequence: u64,
    last_move: HashMap<MessageId, u64>,
}

impl Mailbox {
    pub fn new(folders: Vec<Folder>, messages: Vec<Message>) -> Self {
        let next_id = messages.iter().map(|m| m.id.0 + 1).max().unwrap_or(1);
        Self {
            folders,
            messages,
            next_id,
            ..Self::default()
        }
    }

    /// Folders in display order.
    pub fn folders(&self) -> &[Folder] {
        &self.folders
    }

    pub fn folder(&self, id: FolderId) -> Option<&Folder> {
        self.folders.iter().find(|folder| folder.id == id)
    }

    pub fn message(&self, id: MessageId) -> Option<&Message> {
        self.messages.iter().find(|message| message.id == id)
    }

    pub fn unread(&self, folder: FolderId) -> usize {
        self.messages
            .iter()
            .filter(|m| m.folder == folder && m.unread)
            .count()
    }

    #[cfg(test)]
    pub fn total(&self, folder: FolderId) -> usize {
        self.messages.iter().filter(|m| m.folder == folder).count()
    }

    /// Messages of one folder whose sender or subject fuzzy-matches every
    /// word of `query`. A blank query lists the folder newest first. Otherwise
    /// the best matches lead, newest first among equals, and a message is
    /// dropped when one of its words scores under three quarters of that
    /// word's best score in the folder: loose matches are noise in a long list.
    pub fn search(&self, folder: FolderId, query: &str) -> Vec<Found<'_>> {
        let words: Vec<&str> = query.split_whitespace().collect();
        let mut found: Vec<Found<'_>> = self
            .messages
            .iter()
            .filter(|m| m.folder == folder)
            .filter_map(|m| find(m, &words))
            .collect();
        let best: Vec<i32> = (0..words.len())
            .map(|word| found.iter().map(|f| f.scores[word]).max().unwrap_or(0))
            .collect();
        found.retain(|f| {
            let mut scores = f.scores.iter().zip(&best);
            scores.all(|(score, best)| score * 4 >= best * 3)
        });
        found.sort_by_key(|f| {
            let m = f.message;
            let score: i32 = f.scores.iter().sum();
            (Reverse(score), Reverse((m.date, m.minute, m.id)))
        });
        found
    }

    /// The messages of [`search`](Self::search), without their matches.
    pub fn view(&self, folder: FolderId, query: &str) -> Vec<&Message> {
        let found = self.search(folder, query);
        found.into_iter().map(|f| f.message).collect()
    }

    pub fn set_unread(&mut self, id: MessageId, unread: bool) {
        if let Some(message) = self.messages.iter_mut().find(|m| m.id == id) {
            message.unread = unread;
        }
    }

    /// Move messages and return what Undo needs; unknown ids are skipped.
    pub fn move_to(&mut self, ids: &[MessageId], to: FolderId) -> Moved {
        self.move_sequence += 1;
        let sequence = self.move_sequence;
        let mut origins = Vec::new();
        for message in &mut self.messages {
            if ids.contains(&message.id) && message.folder != to {
                origins.push(MoveOrigin {
                    id: message.id,
                    folder: message.folder,
                    previous_move: self.last_move.insert(message.id, sequence),
                });
                message.folder = to;
            }
        }
        Moved { sequence, origins }
    }

    /// Undo only the messages for which this move is still current.
    pub fn restore(&mut self, moved: &Moved) {
        for origin in &moved.origins {
            if self.last_move.get(&origin.id) != Some(&moved.sequence) {
                continue;
            }
            if let Some(message) = self.messages.iter_mut().find(|m| m.id == origin.id) {
                message.folder = origin.folder;
                if let Some(sequence) = origin.previous_move {
                    self.last_move.insert(origin.id, sequence);
                } else {
                    self.last_move.remove(&origin.id);
                }
            }
        }
    }

    /// Add a message and return its new identity.
    pub fn add(&mut self, mut message: Message) -> MessageId {
        message.id = MessageId(self.next_id);
        self.next_id += 1;
        let id = message.id;
        self.messages.push(message);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::data::FIXED_TODAY;

    fn message(id: u64, folder: FolderId, subject: &str, minute: u16) -> Message {
        Message {
            id: MessageId(id),
            folder,
            from: "Mara Lindqvist".into(),
            address: "mara.lindqvist@example.com".into(),
            subject: subject.into(),
            body: String::new(),
            date: FIXED_TODAY,
            minute,
            unread: true,
        }
    }

    #[test]
    fn views_are_newest_first_and_match_every_query_word() {
        let mailbox = Mailbox::new(
            vec![],
            vec![
                message(1, INBOX, "Garden club minutes", 60),
                message(2, INBOX, "Studio build schedule", 120),
                message(3, ARCHIVE, "Garden tools", 30),
            ],
        );
        let ids =
            |query| -> Vec<u64> { mailbox.view(INBOX, query).iter().map(|m| m.id.0).collect() };
        assert_eq!(ids(""), [2, 1]);
        assert_eq!(ids("garden"), [1]);
        assert_eq!(ids("MARA build"), [2]);
        assert!(ids("garden build").is_empty());
        assert_eq!(ids("grdn"), [1]);
    }

    #[test]
    fn search_ranks_the_best_match_and_drops_loose_ones() {
        let mailbox = Mailbox::new(
            vec![],
            vec![
                message(1, INBOX, "Invoice for the shelving", 10),
                message(2, INBOX, "Reading group: invoices", 20),
                message(3, INBOX, "Bike repair estimate, new valve", 30),
                message(4, INBOX, "Garden club minutes", 40),
            ],
        );
        let ids = |query| -> Vec<u64> {
            let found = mailbox.search(INBOX, query);
            found.iter().map(|f| f.message.id.0).collect()
        };
        // Both start a word with the query; the subject that opens with it
        // leads although it is older. "Bike ... new valve" only scatters it.
        assert_eq!(ids("inv"), [1, 2]);
        // A loose word is not rescued by a strong one.
        assert_eq!(ids("mara inv"), [1, 2]);
        let found = mailbox.search(INBOX, "inv mara");
        assert_eq!(found[0].subject, std::slice::from_ref(&(0..3)));
        assert_eq!(found[0].from, std::slice::from_ref(&(0..4)));
        assert_eq!(mailbox.search(INBOX, "").len(), 4);
    }

    #[test]
    fn moves_report_their_origin_and_restore_it() {
        let mut mailbox = Mailbox::new(
            vec![],
            vec![message(1, INBOX, "A", 1), message(2, ARCHIVE, "B", 2)],
        );
        let moved = mailbox.move_to(&[MessageId(1), MessageId(2), MessageId(9)], ARCHIVE);
        assert_eq!(moved.count(), 1);
        assert_eq!(mailbox.total(ARCHIVE), 2);
        mailbox.restore(&moved);
        assert_eq!(mailbox.total(INBOX), 1);
        assert_eq!(mailbox.unread(INBOX), 1);
        let id = mailbox.add(message(0, SENT, "C", 3));
        assert_eq!(id, MessageId(3));
    }

    #[test]
    fn undo_a_batch_skips_only_messages_that_moved_again() {
        let mut mailbox = Mailbox::new(
            vec![],
            vec![message(1, INBOX, "A", 1), message(2, INBOX, "B", 2)],
        );
        let a = MessageId(1);
        let b = MessageId(2);
        let archived = mailbox.move_to(&[a, b], ARCHIVE);
        mailbox.move_to(&[a], TRASH);
        mailbox.restore(&archived);
        assert_eq!(mailbox.message(a).unwrap().folder, TRASH);
        assert_eq!(mailbox.message(b).unwrap().folder, INBOX);
    }

    #[test]
    fn undo_tracks_moves_even_when_the_message_returns_to_the_same_folder() {
        let mut mailbox = Mailbox::new(vec![], vec![message(1, INBOX, "A", 1)]);
        let id = MessageId(1);
        let first_archive = mailbox.move_to(&[id], ARCHIVE);
        let trashed = mailbox.move_to(&[id], TRASH);
        let second_archive = mailbox.move_to(&[id], ARCHIVE);
        mailbox.restore(&first_archive);
        assert_eq!(mailbox.message(id).unwrap().folder, ARCHIVE);

        // Undo in reverse order can still unwind the valid moves.
        mailbox.restore(&second_archive);
        assert_eq!(mailbox.message(id).unwrap().folder, TRASH);
        mailbox.restore(&trashed);
        assert_eq!(mailbox.message(id).unwrap().folder, ARCHIVE);
        mailbox.restore(&first_archive);
        assert_eq!(mailbox.message(id).unwrap().folder, INBOX);
    }
}
