# Avatar

`Avatar` is a square identity mark: initials derived from a display name, or
an image once it has loaded, with an optional presence marker. The host owns
the name, the image source and presence.

```rust
use dutchy_egui::{egui, Avatar, AvatarSize, Presence};

fn sender(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add(Avatar::new("Mara Velde").size(AvatarSize::Large).presence(Presence::Online));
        ui.label("Mara Velde");
    });
}
```

## Initials and images

Initials are the first letter or digit of the first and last words of the
name, uppercased: "Grace Brewster Murray Hopper" gives "GH", "ada" gives "A".
Words without letters or digits, such as "-", are skipped. Characters are
Unicode scalar values, so a letter written with a separate combining mark
keeps only its base letter; pass `.initials(...)` to show other text as
written. A blank name paints the empty square.

`.image(source)` takes any egui `ImageSource` and uses the context's installed
loaders, such as the `url-images` feature or a host loader. A ready image is
cropped to the square, centered; initials show while it loads and when it
fails. Images are only polled while the avatar is visible.

## Sizes, variants and presence

| Size | Side | Initials | Marker |
| --- | --- | --- | --- |
| `Small` | 24 | Xs | 4 |
| `Medium` (default) | 32 | Sm | 8 |
| `Large` | 48 | Lg | 12 |
| `XLarge` | 64 | Xl2 | 16 |

Variants are `Muted` (default), `Default` (foreground block), `Primary` and
`Outline`. Each marker size keeps two centered initials clear. `presence`
paints a square in the bottom-right corner with a background-colored ring:
success for Online, warning for Away, destructive for Busy and the muted
foreground for Offline.

## Accessibility

An avatar is an image named after the display name, or "Avatar" when the name
is blank; `accessible_name` overrides it. Presence is the description, such
as "Online", so it is never conveyed by color alone. A loading image marks the
avatar busy. Avatars are passive: they take neither focus nor clicks, so they
compose inside List rows and other clickable containers.

The **Avatars** showcase covers sizes, variants with presence, a people
directory in List rows and a generated image source. Not included: avatar
groups or stacks, overflow counts, editable photos, status text, or colors
derived from the name.
