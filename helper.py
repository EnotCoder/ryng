#!/usr/bin/env python3
"""A map of this project, for when reading it from the top down is too slow.

The room table is the thing worth pointing at: a room is a row, a hotspot is a
row inside it, and almost every change to the game is a change to that table.
Everything else here is plumbing around it.

    python3 helper.py                 # the same as `stats`
    python3 helper.py stats           # lines per file, per folder, totals
    python3 helper.py about rooms/data
    python3 helper.py doctor          # things worth knowing before you run the tests
    python3 helper.py help            # this list

`about` takes a path fragment, not a full path: `about rooms/data` is enough.
Descriptions live in DESCRIPTIONS below rather than being scraped from the source,
because only 22 of the 50 modules carry a `//!` header and half an answer is worse
than none. `python3 helper.py docs` is what keeps the table and the tree honest;
run it after adding or renaming a module.

Standard library only, so it cannot drift from Cargo.toml.
"""

from __future__ import annotations

import argparse
import re
import sys
import textwrap
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SRC = ROOT / "src"

# Which folders `stats` groups by. Read off the tree rather than listed, so a new
# folder shows up without editing this.
#
# Each entry owns its own modules and everything below it, so the numbers nest:
# `src/scenes` carries `scenes/game` instead of the two being counted separately
# and the parent also including the child.
FOLDERS = [
    path.relative_to(ROOT).as_posix()
    for path in sorted((ROOT / "src").iterdir())
    if path.is_dir()
] + ["src"]


# --------------------------------------------------------------------- scanning


@dataclass(frozen=True)
class Module:
    """One .rs file, and the few facts `about` and `stats` need about it."""

    path: Path
    rel: str
    lines: int
    module_doc: str | None
    public_items: int

    @property
    def is_test(self) -> bool:
        return self.path.name == "tests.rs" or self.path.name.endswith("_tests.rs")


def count_lines(path: Path) -> int:
    """Lines in a file, counted the way `wc -l` does."""
    with path.open(encoding="utf-8") as handle:
        return sum(1 for _ in handle)


def module_doc_of(path: Path) -> str | None:
    """The first `//!` paragraph, or None when the file has no module header.

    Only the first paragraph: that is the part written to say what the module is
    for, as opposed to the parts that explain how.
    """
    lines: list[str] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        stripped = raw.strip()
        if not stripped.startswith("//!"):
            break
        body = stripped[3:].strip()
        if not body:
            if lines:
                break
            continue
        lines.append(body)
    if not lines:
        return None
    return " ".join(lines)


def count_public_items(path: Path) -> int:
    """How many `pub` items a module exports.

    Approximate on purpose - it counts declarations rather than distinct names, and
    it says nothing about how useful they are. Enough to answer one question: is
    this module a leaf that a couple of files call, or a hub everything needs.
    """
    source = path.read_text(encoding="utf-8")
    return len(
        re.findall(
            r"^\s*pub(?:\(crate\))?\s+(?:fn|struct|enum|const|static|trait|type|mod|use)\b",
            source,
            re.MULTILINE,
        )
    )


def scan() -> list[Module]:
    """Every module under src/, sorted by path."""
    modules = []
    for path in sorted(SRC.rglob("*.rs")):
        modules.append(
            Module(
                path=path,
                rel=str(path.relative_to(ROOT)),
                lines=count_lines(path),
                module_doc=module_doc_of(path),
                public_items=count_public_items(path),
            )
        )
    return modules


# ---------------------------------------------------------------- dependencies

# One `use` statement. `[^;]+` runs past a line break so a grouped import written
# over several lines is picked up whole, and stops at `;` so the next statement is
# not swallowed with it.
USE_RE = re.compile(r"^\s*(?:pub(?:\(crate\))?\s+)?use\s+((?:crate|super)::[^;]+);", re.MULTILINE)

# `mod npc;` - a declaration is a dependency too. Without this the map calls every
# submodule a leaf, which is exactly backwards: `mod` is how a parent owns a child.
MOD_RE = re.compile(r"^\s*(?:pub(?:\(crate\))?\s+)?mod\s+([a-z_][a-z0-9_]*)\s*;", re.MULTILINE)

# A crate-internal path used inline rather than imported - `super::npc::Speech` in a
# signature, `crate::rooms::data::rooms` in an expression. These are real references
# and `use`-only parsing misses most of the sibling-module traffic in this codebase.
INLINE_RE = re.compile(r"\b((?:crate|super)::[A-Za-z_][A-Za-z0-9_:]*\b)")

# Comments, so a `//!` header or a ``/// See [`crate::...`]`` cross-reference does not
# get counted as code using that module. Doc comments start with `//` so the line
# rule covers them along with ordinary ones.
BLOCK_COMMENT_RE = re.compile(r"/\*.*?\*/", re.DOTALL)
LINE_COMMENT_RE = re.compile(r"//[^\n]*")


def module_key(rel: str) -> str:
    """Normalise a path for comparison: `src/a/b.rs` -> `src/a/b`, `src/a/mod.rs` -> `src/a`."""
    if rel.endswith("/mod.rs"):
        return rel[: -len("/mod.rs")]
    if rel.endswith(".rs"):
        return rel[:-3]
    return rel


def _existing_module(segments: list[str]) -> str | None:
    """The longest prefix of `segments` that names a real module, or None.

    `src` alone is never a module in this sense - it is the crate root, and every
    file lives under it - so a path that shortens that far has named nothing.
    """
    while len(segments) > 1:
        candidate = "/".join(segments)
        if (ROOT / f"{candidate}.rs").exists() or (ROOT / candidate / "mod.rs").exists():
            return candidate
        segments.pop()
    return None


def _segments_for(part: str, rel: str) -> list[str]:
    """Turn `crate::a::b` or `super::a` into path segments, or [] if it is neither.

    Split on `::` into plain names first: `Path` does not understand the separator,
    and joining before splitting would leave one segment containing all of it.

    `super` is resolved against the importing file's own module and not against its
    directory, because those differ for every non-`mod.rs` file. In
    `src/scenes/game/ui.rs`, `super::npc` is `src/scenes/game/npc`; resolving against
    the directory would give `src/scenes/npc`, which does not exist, and the path
    would shorten to the wrong module.
    """
    names = [n for n in part.split("::") if n]
    if not names:
        return []
    if names[0] == "crate":
        return ["src"] + names[1:]
    if names[0] == "super":
        base = module_key(rel).split("/")
        ups = names.count("super")
        if ups >= len(base):
            return []
        return base[: len(base) - ups] + names[ups:]
    return []


def dependencies_of(rel: str) -> set[str]:
    """Modules under src/ that this file refers to.

    Three ways a file points at another one, all counted: a `use`, a `mod` child,
    and an inline `crate::` / `super::` path in a signature or an expression.

    Each path is shortened outwards to the nearest real module, because that is the
    module that owns the name: `crate::scenes::game::rooms::data::room_def` belongs to
    `src/scenes/game/rooms/data`. Attribute it to the innermost path instead and
    every file would appear to depend only on itself.
    """
    found: set[str] = set()
    source = (ROOT / rel).read_text(encoding="utf-8")
    code = LINE_COMMENT_RE.sub("", BLOCK_COMMENT_RE.sub("", source))
    here = module_key(rel)

    def record(part: str) -> None:
        key = _existing_module(_segments_for(part, rel))
        # A file referencing itself is not a dependency.
        if key and key != here:
            found.add(key)

    for match in USE_RE.finditer(source):
        clause = match.group(1)

        # `use crate::a::{b, c};` - the head and each brace entry are separate
        # paths, so `a::b` and `a::c` do not collapse into one nonsense path.
        if "{" in clause:
            head, _, tail = clause.partition("{")
            prefix = head.strip().rstrip(":")
            record(prefix)
            for entry in tail.rstrip("}").split(","):
                if entry.strip():
                    record(f"{prefix}::{entry.strip()}")
        else:
            record(clause.strip().rstrip(":"))

    # `mod npc;` names a child of this file's own directory, or of `src` when this
    # file sits at the crate root - which is why `src` itself is accepted here.
    own = Path(rel).parent
    for match in MOD_RE.finditer(code):
        key = _existing_module(list(own.parts) + [match.group(1)])
        if key and key != here:
            found.add(key)

    for match in INLINE_RE.finditer(code):
        record(match.group(1))

    return found


def usage_index(modules: list[Module]) -> dict[str, list[str]]:
    """For each module, the modules that use it."""
    index: dict[str, list[str]] = defaultdict(list)
    for module in modules:
        for target in dependencies_of(module.rel):
            index[target].append(module.rel)
    return index


# ----------------------------------------------------------------- descriptions

# What each module is for. This table is the point of the tool, and it is the
# fallback rather than the source: the `//!` headers cover less than half the tree,
# and a half-answer for the rest is worse than a written-down one.
#
# Keys are `module_key` form - `src/a/b`, no `.rs`, because that is what the scan
# produces and the two are compared directly. A title, then the reason it exists:
# the title says what it is, the second half says what for.
#
# `python3 helper.py docs` fails when an entry here has no module to match.
DESCRIPTIONS: dict[str, tuple[str, str]] = {
    # top level
    "src/main": ("Entry point", "Thin wrapper; the app is built by `lib.rs` so the same crate can be an Android library."),
    "src/lib": ("App wiring", "Builds the App, owns the camera, `UiScale`, and the window-to-design-space scale that resizes the UI."),
    "src/state": ("Game states", "The four states the game moves through: Loading, Intro, Menu, Game."),
    "src/acts": ("Items and acts", "`Item`, the four-slot `Inventory`, and the three acts plus which room opens each one."),
    "src/cli": ("`--rooms`", "Starts the game in one room by number or name, skipping the intro and the menu. Prints the room list on a bare `--rooms`."),
    "src/ui_scale_tests": ("Rescale tests", "Checks that a resize actually changes button, slot, offset and font geometry - and that an idle frame does not."),
    "src/buttons": ("Button visuals", "Colours, sizes and the hover/press tints every button shares, plus the spawn helpers."),
    "src/buttons/click": ("Button clicks", "One `Query` and one `for_each_click` shared by all button kinds; applies the visual and reports a completed press."),
    "src/buttons/plain": ("Flat buttons", "Buttons drawn with a solid colour instead of a texture."),
    "src/buttons/textured": ("Textured buttons", "Buttons drawn with a background image and a tint, plus the red variant used for Back and Close."),
    "src/buttons/tests": ("Button tests", "Pins that hover is not a click and that the sizes come out at the scale given."),
    "src/cli/tests": ("`--rooms` tests", "Number and name resolution, the refusal on an ambiguous name, and the listing the flag prints."),
    "src/scenes": ("Scene plugins", "Declares the submodules of the scene tree."),
    "src/scenes/loading": ("Loading", "The splash, and the black overlay that waits for every texture before the game starts."),
    "src/scenes/fade": ("Transitions", "The fade-out/fade-in state machine behind every room change, and where the act changes."),
    "src/scenes/sound": ("Audio", "The music and one-shot tables. Which track a room wants is a column in the room table, not a string match on its path."),
    "src/scenes/intro": ("Intro plugin", "Registers the logo fall and its timer."),
    "src/scenes/intro/systems": ("Logo fall", "The eased curve the logo lands on and drops out of, and the hand-off to the menu."),
    "src/scenes/intro/ui": ("Logo node", "Spawns the logo sprite and its backdrop."),
    "src/scenes/intro/tests": ("Intro tests", "Checks the fall is continuous, bounded, and eased in and out."),
    "src/scenes/menu": ("Menu plugin", "Registers the menu buttons and their click handling."),
    "src/scenes/menu/systems": ("Menu clicks", "Turns a menu button press into a state change."),
    "src/scenes/menu/ui": ("Menu UI", "The logo, the backdrop and the Settings/Play/Quit row."),
    "src/scenes/settings": ("Settings plugin", "Registers the settings panel and its widgets."),
    "src/scenes/settings/systems": ("Settings logic", "Slider press/drag/release, the panel's open/closed state, and applying the result to the engine."),
    "src/scenes/settings/ui": ("Settings UI", "The panel, the volume slider, and the readout beside it."),
    "src/scenes/settings/tests": ("Settings tests", "That a slider press maps to the fraction the label shows."),
    "src/scenes/game": ("Game plugin", "Builds the game: resources, the room systems in their fixed order, and where `--rooms` is read."),
    "src/scenes/game/systems": ("Room systems", "Hotspot clicks, the carousel, the breathing room, the blinking chevron and the hover outline."),
    "src/scenes/game/ui": ("Game UI", "The room HUD, the carousel arrows, the caption, and the dialogue box."),
    "src/scenes/game/items": ("World items", "Which item is lying in which room. An item is either resting somewhere or in the bag, never both."),
    "src/scenes/game/items/tests": ("Item tests", "That an item cannot be in two places, and that a door's tool list is checked as a set."),
    "src/scenes/game/tests": ("Game tests", "Carousel arithmetic, the disjoint query check, and the hover outline's bars and layering."),
    "src/scenes/game/inventory": ("Inventory plugin", "Wires the slot textures and the two slot systems."),
    "src/scenes/game/inventory/ui": ("Inventory UI", "The four slots, their icons, and which slot is active."),
    "src/scenes/game/inventory/tests": ("Inventory tests", "That clicking a slot makes it the active one."),
    "src/scenes/game/npc": ("Characters", "Sprites, click targets and dialogue: click her, a line appears, any click puts it away."),
    "src/scenes/game/npc/data": ("NPC table", "One row per character. Adding one is a row here and a picture."),
    "src/scenes/game/npc/tests": ("NPC tests", "Dialogue order, that she cannot block the way out, and that her `.meta` is beside the picture."),
    "src/scenes/game/rooms": ("Rooms module", "Declares the room submodules."),
    "src/scenes/game/rooms/components": ("Room components", "`Room`, `Hotspot`, `HotspotDef`, and the outline's constants. The definition of what a hotspot is."),
    "src/scenes/game/rooms/spawn": ("Room spawning", "Turns a `RoomDef` into entities: the picture, the hotspot rectangles, the chevron, the outline bars."),
    "src/scenes/game/rooms/data": ("Room table lookup", "Looks a room up by path, derives the preload list, and says which act owns which row."),
    "src/scenes/game/rooms/data/p": ("Asset paths", "One constant per picture, so a rename is one edit."),
    "src/scenes/game/rooms/data/builders": ("Room builders", "The `room!` / `hop!` / `take!` macro family that writes the rows. Macros so every literal lands in the `static`."),
    "src/scenes/game/rooms/data/tests": ("Room table tests", "That every link resolves, every texture exists, every room is reachable, and the route is unchanged."),
    "src/scenes/game/rooms/data/table": ("Play order", "`ACTS`: the three act tables in the order the player walks them."),
    "src/scenes/game/rooms/data/table/act_one": ("Act 1 - The Curse", "The walk home, the concierge, the hall and the lift that stops."),
    "src/scenes/game/rooms/data/table/act_two": ("Act 2 - The Descent", "The basement and the long way back up, including the stairs."),
    "src/scenes/game/rooms/data/table/act_three": ("Act 3 - The Escape", "Floor 2, the three apartments, and the black door. The lobby at the end is parked."),
}

# Known problems, so they are visible without running the test suite.
KNOWN_ISSUES = [
    (
        "she_does_not_cover_the_way_out_of_the_room fails",
        "Commit 6e7f1b5 moved the concierge exit to (0, 50) at 400x400, and the granny's "
        "click target overlaps its left edge by about 25px. Either move her or shrink the "
        "exit. Until then `cargo test` is red and that is expected, not new.",
    ),
]


# --------------------------------------------------------------------- commands


def cmd_stats(args: argparse.Namespace, modules: list[Module]) -> int:
    total = sum(m.lines for m in modules)
    code = sum(m.lines for m in modules if not m.is_test)
    tests = total - code
    documented = sum(1 for m in modules if m.module_doc)

    print(f"{len(modules)} files, {total} lines in src/")
    print(f"  code        {code}")
    print(f"  tests       {tests}")
    print(f"  documented  {documented}/{len(modules)} have a //! header")
    print()

    print("by folder:")
    for folder in FOLDERS:
        # A folder owns its direct modules and everything under it, so the numbers
        # nest rather than double-count. Sorted by size so the big one is first.
        prefix = folder.rstrip("/") + "/"
        members = [m for m in modules if m.rel.startswith(prefix)]
        if not members:
            continue
        depth = prefix.count("/")
        print(f"{'  ' * depth}{folder:<24} {sum(m.lines for m in members):>6}")

    print()
    print("largest files:")
    for module in sorted(modules, key=lambda m: m.lines, reverse=True)[:10]:
        tag = "  (tests)" if module.is_test else ""
        print(f"  {module.rel:<46} {module.lines:>5}{tag}")
    return 0


def _folder_of(matches: list[Module]) -> str | None:
    """The single directory a group of matches belongs to, if there is one.

    `about rooms/data` matches `rooms/data.rs` and the seven modules inside
    `rooms/data/`. Those are one question about one place, so the deeper directory
    wins - `rooms/data` is what was typed, `rooms` is just where its file sits.

    `about tests` matches nine `tests.rs` in nine directories. A name that is simply
    common says nothing about a place, so that returns None and the caller asks for
    more of the path instead of guessing.
    """
    parents = {m.rel.rsplit("/", 1)[0] if "/" in m.rel else "" for m in matches}
    if len(parents) == 1:
        return parents.pop()
    if len(parents) == 2:
        outer, inner = sorted(parents)
        if outer == inner.rsplit("/", 1)[0]:
            return inner
    return None


def _resolve(query: str, modules: list[Module]) -> tuple[list[Module], str | None]:
    """Find the modules a query names, plus the directory if they name one.

    A fragment rather than a full path, because `about rooms/data` is what anyone
    actually types.

    Three passes, most specific first, because the order decides who wins: an exact
    path, then a module of exactly that name anywhere in the tree, then a substring.
    The substring pass goes last so a name that happens to sit inside a longer path
    cannot hijack an exact request: `about rooms/data` is the lookup module, not
    whichever file contains those letters somewhere.
    """
    needle = query.strip().removesuffix(".rs")
    passes = (
        [m for m in modules if needle in (m.rel, module_key(m.rel))],
        [m for m in modules if module_key(m.rel).endswith("/" + needle)],
        [m for m in modules if needle in m.rel],
    )
    for matches in passes:
        if matches:
            ordered = sorted(matches, key=lambda m: m.rel)
            return ordered, None if len(ordered) == 1 else _folder_of(ordered)
    return [], None


def cmd_about(args: argparse.Namespace, modules: list[Module]) -> int:
    matches, folder = _resolve(args.target, modules)

    if not matches:
        print(f"no module matching {args.target!r}. Try `helper.py stats` for the list.", file=sys.stderr)
        return 1

    if folder is not None:
        total = sum(m.lines for m in matches)
        print(f"{args.target!r} is the folder {folder}/  -  {len(matches)} modules, {total} lines")
        print()
        # Paths relative to the folder, so the listing reads like the directory.
        cut = len(folder) + 1
        for module in matches:
            title = DESCRIPTIONS.get(module_key(module.rel), ("", ""))[0]
            print(f"  {module.rel[cut:]:<18} {module.lines:>5}  {title}")
        print()
        print("ask about one of them with `about <name>`")
        return 0

    if len(matches) > 1:
        print(f"{args.target!r} matches {len(matches)} modules in different places:", file=sys.stderr)
        for module in matches:
            print(f"  {module.rel}", file=sys.stderr)
        print("give more of the path to pick one.", file=sys.stderr)
        return 1

    module = matches[0]
    title, why = DESCRIPTIONS.get(
        module_key(module.rel),
        ("(no description recorded)", "Add one to DESCRIPTIONS in helper.py."),
    )

    print(f"{module.rel}  -  {module.lines} lines")
    print(f"{title}: {why}")
    print()

    if module.module_doc:
        print("from the source:")
        print(f"  {module.module_doc}")
        print()

    plural = "item" if module.public_items == 1 else "items"
    print(f"exports {module.public_items} public {plural}")

    users = usage_index(modules).get(module_key(module.rel), [])
    real_users = [u for u in users if u != module.rel]
    if real_users:
        print("used by:")
        for user in sorted(real_users)[:8]:
            print(f"  {user}")
        if len(real_users) > 8:
            print(f"  ... and {len(real_users) - 8} more")
    else:
        print("used by: nothing else in src/")

    if not module.module_doc:
        print()
        print("(this module has no //! header, so the description above is only in helper.py)")
    return 0


def cmd_doctor(args: argparse.Namespace, modules: list[Module]) -> int:
    """Things worth knowing that do not show up as a compiler error.

    Known problems in the project, then anything left unfinished in the source.
    The DESCRIPTIONS table has its own command, `docs`.
    """
    print(f"known issues ({len(KNOWN_ISSUES)}):")
    for index, (title, detail) in enumerate(KNOWN_ISSUES, start=1):
        print(f"  {index}. {title}")
        for line in textwrap.wrap(detail, 72):
            print(f"     {line}")
    print()

    placeholders = []
    for module in modules:
        for number, line in enumerate(module.path.read_text(encoding="utf-8").splitlines(), 1):
            if re.search(r"\b(FIXME|XXX|TODO: fix)\b", line):
                placeholders.append(f"{module.rel}:{number}")
    if placeholders:
        print(f"unfinished markers in the source ({len(placeholders)}):")
        for spot in placeholders[:10]:
            print(f"  {spot}")
        if len(placeholders) > 10:
            print(f"  ... and {len(placeholders) - 10} more")
        print()
    else:
        print("no FIXME/XXX markers in the source")
        print()

    print("`docs` re-checks the description table against the tree")
    return 0


def cmd_docs(args: argparse.Namespace, modules: list[Module]) -> int:
    """Check the DESCRIPTIONS table against the tree it describes.

    This is the command that keeps the two from drifting apart, and it checks the
    thing that actually drifts: files. A new module with no entry, a renamed module
    with an entry left behind, a stub entry - all of those are silent failures
    everywhere else, because nothing in the Rust build reads this file.

    What it deliberately does NOT do is compare the wording against the `//!` headers.
    That was tried and it is not a check: any two English sentences about the same
    file share a common word, so the comparison passed even after the description for
    `rooms/data.rs` was replaced with "Audio mixer". Catching a reworded header needs
    a stored copy of the old one, which would just be a second copy of the source.

    Exits 1 when something is wrong, so it can be wired into a commit hook or CI.
    """
    keys = {module_key(m.rel) for m in modules}
    problems: list[str] = []

    undescribed = sorted(keys - set(DESCRIPTIONS))
    for key in undescribed:
        # The real path, not `key + ".rs"`: a key like `src/scenes` is `mod.rs`.
        where = next(m.rel for m in modules if module_key(m.rel) == key)
        problems.append(f"no description recorded for {where}  (add one to DESCRIPTIONS)")

    stale = sorted(set(DESCRIPTIONS) - keys)
    for key in stale:
        problems.append(f"DESCRIPTIONS names {key}, which is not a module  (rename or delete the entry)")

    for key, (title, why) in sorted(DESCRIPTIONS.items()):
        if not title.strip() or not why.strip():
            problems.append(f"{key} has an empty title or an empty description")

    undocumented = sorted(m.rel for m in modules if not m.module_doc)
    documented = len(modules) - len(undocumented)

    if not problems:
        print(f"descriptions: {len(DESCRIPTIONS)}/{len(modules)} modules, no stale or empty entries")
        print(f"//! headers:   {documented}/{len(modules)} modules document themselves in the source")
        if undocumented:
            print(f"              (the other {len(undocumented)} are described only here)")
        return 0

    print(f"{len(problems)} problems:")
    for problem in problems:
        print(f"  {problem}")
    return 1


def cmd_help(args: argparse.Namespace, modules: list[Module]) -> int:
    print(__doc__.strip())
    print()
    print("commands:")
    for name, summary in COMMANDS:
        print(f"  {name:<9} {summary}")
    print()
    print("options:")
    print("  a bare invocation is the same as `stats`")
    print("  `about` accepts a path fragment: rooms/data, cli.rs, inventory")
    return 0


COMMANDS = [
    ("stats", "lines per file and per folder, with the code/test split"),
    ("about", "what one module is for, what it exports, who uses it"),
    ("docs", "check the description table against the tree; exits 1 on drift"),
    ("doctor", "known problems in the project and unfinished markers"),
    ("help", "this list"),
]


# -------------------------------------------------------------------------- main


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        prog="helper.py",
        description="A map of the ryng codebase.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    sub = parser.add_subparsers(dest="command")
    sub.add_parser("stats", help="lines per file and per folder")
    about = sub.add_parser("about", help="what one module is for")
    about.add_argument("target", help="a path fragment, e.g. rooms/data")
    sub.add_parser("docs", help="check the description table against the tree")
    sub.add_parser("doctor", help="known problems and unfinished markers")
    sub.add_parser("help", help="list the commands")

    args = parser.parse_args(argv)
    modules = scan()

    # A bare invocation is `stats`: looking at the size of the thing is what you
    # want when you have not decided what to ask yet.
    handlers = {
        "about": cmd_about,
        "docs": cmd_docs,
        "doctor": cmd_doctor,
        "help": cmd_help,
        "stats": cmd_stats,
        None: cmd_stats,
    }
    return handlers[args.command](args, modules)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))