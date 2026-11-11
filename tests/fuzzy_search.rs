//! Fuzzy ranking and matched-glyph emphasis in command and option search.
mod common;
use dutchy_egui::{
    CommandPalette, MenuItem, SearchState, SearchableSelect, SelectOption, Text, Theme,
};
use egui::{Key, accesskit::Role};
use egui_kittest::{Harness, kittest::Queryable};

struct Commands {
    search: SearchState,
    actions: Vec<u8>,
}

fn commands(query: &str) -> Harness<'static, Commands> {
    let mut search = SearchState::default();
    search.open = true;
    search.query = query.into();
    let mut h = Harness::builder()
        .with_size([800.0, 600.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut Commands| {
                let items = [
                    MenuItem::new(1, "Sandbox data").group("Data"),
                    MenuItem::new(2, "Dashboard").group("Pages"),
                    MenuItem::new(3, "Create project")
                        .group("Actions")
                        .separator_before(true),
                    MenuItem::new(4, "Open data folder").group("Data"),
                ];
                if let Some(result) =
                    CommandPalette::new("commands", &mut state.search, &items).show(ui.ctx())
                    && let Some(action) = result.action
                {
                    state.actions.push(action);
                }
            },
            Commands {
                search,
                actions: Vec::new(),
            },
        );
    h.run_steps(4);
    h
}

fn top(h: &Harness<'_, impl Sized>, label: &str) -> f32 {
    h.get_by_role_and_label(Role::ListBoxOption, label)
        .rect()
        .top()
}

#[test]
fn abbreviations_find_commands_and_enter_runs_the_match() {
    let mut h = commands("crp");
    assert!(
        h.query_by_role_and_label(Role::ListBoxOption, "Dashboard")
            .is_none()
    );
    assert!(
        h.query_by_role_and_label(Role::ListBoxOption, "Create project")
            .is_some()
    );
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert_eq!(h.state().actions, [3]);
}

#[test]
fn the_best_match_leads_and_groups_follow_their_best_command() {
    let mut h = commands("da");
    // "Dashboard" starts with the query; the Data commands only contain it.
    assert!(top(&h, "Dashboard") < top(&h, "Open data folder"));
    assert!(top(&h, "Open data folder") < top(&h, "Sandbox data"));
    assert_eq!(
        h.query_all_by_label("Data").count(),
        1,
        "one heading per group"
    );
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert_eq!(h.state().actions, [2]);
}

#[test]
fn a_blank_query_keeps_the_host_order() {
    let h = commands("  ");
    let order = ["Sandbox data", "Dashboard", "Create project"];
    for pair in order.windows(2) {
        assert!(top(&h, pair[0]) < top(&h, pair[1]), "{pair:?}");
    }
    assert!(top(&h, "Create project") < top(&h, "Open data folder"));
}

#[test]
fn matched_glyphs_are_emphasized_without_changing_the_name() {
    let h = commands("dash");
    let galleys = common::galleys(&h.output().shapes);
    let label = galleys
        .iter()
        .find(|galley| galley.job.text == "Dashboard")
        .expect("the option label is painted");
    let sections: Vec<_> = label
        .job
        .sections
        .iter()
        .map(|section| section.byte_range.start.0..section.byte_range.end.0)
        .collect();
    // Without bundled faces every weight is the host font, so egui merges
    // the pieces back into one section.
    #[cfg(not(feature = "bundled-fonts"))]
    assert_eq!(sections.len(), 1);
    #[cfg(feature = "bundled-fonts")]
    assert_eq!(sections, [0..4, 4..9]);
    #[cfg(feature = "bundled-fonts")]
    assert_ne!(
        label.job.sections[0].format.font_id.family, label.job.sections[1].format.font_id.family,
        "matched glyphs use another face"
    );
    assert!(
        h.query_by_role_and_label(Role::ListBoxOption, "Dashboard")
            .is_some()
    );
}

#[derive(Default)]
struct Countries {
    value: Option<u8>,
    search: SearchState,
    commits: usize,
}

#[test]
fn option_search_ranks_fuzzy_matches_and_enter_commits_the_best() {
    let mut h = Harness::builder()
        .with_size([500.0, 500.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut Countries| {
                let options = [
                    SelectOption::new(1, "Algeria"),
                    SelectOption::new(2, "Germany"),
                    SelectOption::new(3, "Niger"),
                    SelectOption::new(4, "Georgia"),
                    SelectOption::new(5, "Peru"),
                ];
                let result =
                    SearchableSelect::new("country", &mut state.value, &mut state.search, &options)
                        .accessible_name("Country")
                        .show(ui);
                state.commits += usize::from(result.committed);
            },
            Countries {
                value: Some(4),
                ..Countries::default()
            },
        );
    h.get_by_label("Country").focus();
    h.run();
    h.get_by_label("Country").type_text("ger");
    h.run_steps(4);
    assert!(
        h.query_by_role_and_label(Role::ListBoxOption, "Peru")
            .is_none()
    );
    let order = ["Germany", "Georgia", "Algeria", "Niger"];
    for pair in order.windows(2) {
        assert!(top(&h, pair[0]) < top(&h, pair[1]), "{pair:?}");
    }
    // The selected option keeps its fill but not its bold label, so bold
    // glyphs always mean a match while a query is active.
    #[cfg(feature = "bundled-fonts")]
    {
        let galleys = common::galleys(&h.output().shapes);
        let selected = galleys.iter().find(|g| g.job.text == "Georgia").unwrap();
        let pieces: Vec<_> = selected
            .job
            .sections
            .iter()
            .map(|section| section.byte_range.start.0..section.byte_range.end.0)
            .collect();
        assert_eq!(pieces, [0..2, 2..3, 3..4, 4..7]);
    }
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert_eq!((h.state().value, h.state().commits), (Some(2), 1));
    assert_eq!(h.state().search.query, "Germany");
}

fn pieces(text: Text<'_>) -> Vec<(String, bool)> {
    let job = text.layout_job(&Theme::default(), 400.0);
    let base = job.sections.last().unwrap().format.font_id.clone();
    job.sections
        .iter()
        .map(|section| {
            (
                job.text[section.byte_range.start.0..section.byte_range.end.0].to_owned(),
                section.format.font_id != base,
            )
        })
        .collect()
}

fn text(pieces: &[(String, bool)]) -> String {
    pieces.iter().map(|(piece, _)| piece.as_str()).collect()
}

#[test]
fn text_highlight_keeps_the_text_and_skips_invalid_ranges() {
    assert_eq!(
        text(&pieces(Text::new("Open File").highlight(&[0..1, 5..6]))),
        "Open File"
    );
    // Ranges inside a character, out of order or past the end are skipped.
    let skipped = [2..3, 0..1, 0..1, 9..12];
    assert_eq!(text(&pieces(Text::new("Año").highlight(&skipped))), "Año");
    // Uppercasing happens per piece, so ranges still address the source text.
    let first = std::slice::from_ref(&(0..3));
    let upper = Text::new("straße").uppercase(true).highlight(first);
    assert_eq!(text(&pieces(upper)), "STRASSE");
    assert_eq!(pieces(Text::new("").highlight(first)).len(), 1);
}

#[cfg(feature = "bundled-fonts")]
#[test]
fn text_highlight_marks_exactly_the_valid_ranges() {
    let piece = |s: &str, marked: bool| (s.to_owned(), marked);
    assert_eq!(
        pieces(Text::new("Open File").highlight(&[0..1, 5..6])),
        [
            piece("O", true),
            piece("pen ", false),
            piece("F", true),
            piece("ile", false)
        ]
    );
    let skipped = [2..3, 0..1, 0..1, 9..12];
    assert_eq!(
        pieces(Text::new("Año").highlight(&skipped)),
        [piece("A", true), piece("ño", false)]
    );
    let first = std::slice::from_ref(&(0..3));
    assert_eq!(
        pieces(Text::new("straße").uppercase(true).highlight(first)),
        [piece("STR", true), piece("ASSE", false)]
    );
    assert_eq!(
        pieces(Text::new("Plain").highlight(&[])),
        [piece("Plain", false)]
    );
}
