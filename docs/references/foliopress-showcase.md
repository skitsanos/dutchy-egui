# Foliopress showcase — native component references

Reviewed [try.foliopress.app](https://try.foliopress.app/) in the user's open
browser on 2026-11-08. These are live observations, not a claim of a full UX or
accessibility audit. The reference supplies reusable patterns; routing, saved
styles, authentication and generation remain host application behavior.

## Controls and trailing spacing

The search and sort wrappers are 48 px tall with 16 px horizontal padding and
12 px between content slots. The sort control combines a leading icon, native
select and decorative chevron. The search combines a magnifier, editable query,
conditional Clear search button and compact key hint. Its empty query uses a
specific placeholder; typing "continuous line" reduced the visible result count
to one, and Clear restored the full collection. The original empty search and
curated ordering were retained after review.

Use the same spacing rhythm in native framed fields: the color picker now
reserves 16 logical points between its right edge and chevron endpoint, plus a
12-point gap before the glyph for text. This moves the arrow 8 points inward.
Stroke thickness occupies a further half-stroke at the endpoint. Existing
Select already has comparable breathing room through egui's padded icon slot.
Larger accordion headers keep their own 24-point content padding; independent
buttons such as close, calendar and number-step controls keep full hit targets.

## Picture cards

Cards combine a sharp 2 px border, image, category, title, short description and
directional affordance. The preview and metadata are action surfaces; Save is a
separate named toggle over the image. The DOM includes a "Try this style" image
overlay. Its pointer hover animation was not exercised. This suggests a
MediaCard composition with independent primary/save actions, rather than a
whole-card click that also captures nested buttons. Keep saved/selected state,
image source and returned action IDs in the host.

## Native opportunities

- SearchInput: leading search icon, optional clear action and key hint, composing
  the existing Input and Kbd without owning filtering or global shortcuts.
- MediaCard: existing image loaders, Card, Badge and future IconButton, with
  keyboard-visible image actions and loading/error states.
- Select adornments: an optional leading icon and a reserved trailing chevron
  slot can support sort controls without another selection widget. Dropdown
  already accepts a host-supplied trigger; keep its menu behavior separate.
- A gallery-browser showcase combining those controls with Select, category
  filters and an explicit result count would make these APIs easy to discover.

See [the component roadmap](../component-roadmap.md). Native interaction and
accessibility require their own tests; browser appearance alone does not prove
equivalence. No production Foliopress source or user data was changed.
