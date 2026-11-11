//! The in-memory mailbox the application owns: folders, messages and the
//! moves that archive, delete and restore them.

use dutchy_egui::calendar::Date;

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

/// Where a moved message came from, so Undo can put it back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Moved {
    pub ids: Vec<(MessageId, FolderId)>,
    pub to: FolderId,
}

#[derive(Default)]
pub struct Mailbox {
    folders: Vec<Folder>,
    messages: Vec<Message>,
    next_id: u64,
}

impl Mailbox {
    pub fn new(folders: Vec<Folder>, messages: Vec<Message>) -> Self {
        let next_id = messages.iter().map(|m| m.id.0 + 1).max().unwrap_or(1);
        Self {
            folders,
            messages,
            next_id,
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

    /// Messages of one folder, newest first, whose sender or subject contains
    /// every word of `query`, ignoring case. Plain substring matching keeps
    /// the sketch honest until a fuzzy scorer exists.
    pub fn view(&self, folder: FolderId, query: &str) -> Vec<&Message> {
        let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
        let mut view: Vec<&Message> = self
            .messages
            .iter()
            .filter(|m| m.folder == folder)
            .filter(|m| {
                let haystack = format!("{} {}", m.from, m.subject).to_lowercase();
                words.iter().all(|word| haystack.contains(word))
            })
            .collect();
        view.sort_by_key(|m| std::cmp::Reverse((m.date, m.minute, m.id)));
        view
    }

    pub fn set_unread(&mut self, id: MessageId, unread: bool) {
        if let Some(message) = self.messages.iter_mut().find(|m| m.id == id) {
            message.unread = unread;
        }
    }

    /// Move messages and return what Undo needs; unknown ids are skipped.
    pub fn move_to(&mut self, ids: &[MessageId], to: FolderId) -> Moved {
        let mut moved = Vec::new();
        for message in &mut self.messages {
            if ids.contains(&message.id) && message.folder != to {
                moved.push((message.id, message.folder));
                message.folder = to;
            }
        }
        Moved { ids: moved, to }
    }

    /// Put moved messages back where they were.
    pub fn restore(&mut self, moved: &Moved) {
        for (id, from) in &moved.ids {
            if let Some(message) = self.messages.iter_mut().find(|m| m.id == *id) {
                message.folder = *from;
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
    }

    #[test]
    fn moves_report_their_origin_and_restore_it() {
        let mut mailbox = Mailbox::new(
            vec![],
            vec![message(1, INBOX, "A", 1), message(2, ARCHIVE, "B", 2)],
        );
        let moved = mailbox.move_to(&[MessageId(1), MessageId(2), MessageId(9)], ARCHIVE);
        assert_eq!(moved.ids, [(MessageId(1), INBOX)]);
        assert_eq!(mailbox.total(ARCHIVE), 2);
        mailbox.restore(&moved);
        assert_eq!(mailbox.total(INBOX), 1);
        assert_eq!(mailbox.unread(INBOX), 1);
        let id = mailbox.add(message(0, SENT, "C", 3));
        assert_eq!(id, MessageId(3));
    }
}
