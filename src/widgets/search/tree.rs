use crate::MenuItem;
use egui::IdSalt;

/// Preserve flat-list identity. Nested leaves use their full sibling-local key
/// path, so equal values/keys in different branches remain independent.
pub(crate) fn leaves<'a, T: Clone>(items: &[MenuItem<'a, T>]) -> Vec<MenuItem<'a, T>> {
    let mut result = Vec::new();
    visit(items, &mut Vec::new(), &mut Vec::new(), true, &mut result);
    result
}
fn visit<'a, T: Clone>(
    items: &[MenuItem<'a, T>],
    keys: &mut Vec<IdSalt>,
    names: &mut Vec<String>,
    enabled: bool,
    result: &mut Vec<MenuItem<'a, T>>,
) {
    for item in items {
        keys.push(item.id);
        if let Some(children) = item.children {
            names.push(item.label.to_string());
            visit(children, keys, names, enabled && item.enabled, result);
            names.pop();
        } else {
            let mut leaf = item.clone();
            leaf.enabled &= enabled;
            if !names.is_empty() {
                leaf.id = IdSalt::new(&*keys);
                let mut path = names.join(" › ");
                if let Some(group) = &leaf.group {
                    path.push_str(" › ");
                    path.push_str(group);
                }
                leaf.group = Some(path.into());
            }
            result.push(leaf);
        }
        keys.pop();
    }
}
