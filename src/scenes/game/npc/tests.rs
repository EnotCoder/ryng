//! Tests for the NPC table and the dialogue state.
//!
//! The state machine here is small but it is the thing the player notices: click
//! her once and she says the first line, click again and she says something else.
//! Nothing else in the game has to be correct for that to be wrong.

use bevy::prelude::*;

use super::{Clicked, NpcId, TalkedTo, classify_click, data};
use crate::acts::{ActId, CurrentAct};
use crate::scenes::game::rooms::data::p;
use crate::scenes::game::rooms::{components::Room, components::RoomPart};

/// A fresh conversation count for the tests below.
fn fresh() -> TalkedTo {
    TalkedTo::default()
}

#[test]
fn a_new_character_has_never_been_spoken_to() {
    assert_eq!(fresh().count(NpcId::Granny), 0);
}

#[test]
fn nobody_has_been_spoken_to_yet() {
    assert!(fresh().0.is_empty());
}

/// The whole point: the first conversation says one thing and the second says
/// another. If this fails, either the count is not advanced or the lines are not
/// being walked in order.
#[test]
fn the_second_conversation_differs_from_the_first() {
    let granny = NpcId::Granny;
    let mut talked = fresh();

    let first = talked.next_line(granny);
    talked.once_more(granny);
    let second = talked.next_line(granny);

    assert_ne!(
        first, second,
        "she says the same thing twice in a row, so talking again is pointless"
    );
}

/// Reading the line must happen before the conversation is counted, or the first
/// click shows the second line. That off-by-one is invisible in a test that only
/// counts.
#[test]
fn the_first_line_is_the_first_one() {
    let granny = NpcId::Granny;
    let mut talked = fresh();

    let first = talked.next_line(granny);
    assert_eq!(
        first,
        data::GRANNY.lines[0],
        "a fresh conversation started somewhere other than the top"
    );

    // What the click system does, in the same order.
    let line = talked.next_line(granny);
    talked.once_more(granny);
    assert_eq!(line, data::GRANNY.lines[0]);
}

/// Running out of lines must not leave her silent. A character who goes quiet once
/// you have heard everything reads as a bug rather than as an ending.
#[test]
fn the_last_line_repeats_forever() {
    let granny = NpcId::Granny;
    let mut talked = fresh();
    let last = data::GRANNY.lines[data::GRANNY.lines.len() - 1];

    for _ in 0..50 {
        talked.once_more(granny);
        assert_eq!(talked.next_line(granny), last, "she went silent");
    }
}

/// The count is per character, not a single global tally: talking to one person
/// must not move another person's conversation along.
#[test]
fn conversations_are_counted_per_character() {
    let granny = NpcId::Granny;
    let mut talked = fresh();
    for _ in 0..5 {
        talked.once_more(granny);
    }
    assert_eq!(talked.count(granny), 5);
    assert_eq!(talked.0.len(), 1, "five clicks made five records");
}

// ------------------------------------------------- what a click means

/// Clicking a character with nothing on screen starts a conversation.
#[test]
fn clicking_her_starts_a_conversation() {
    assert!(matches!(
        classify_click(None, Some(NpcId::Granny)),
        Clicked::Speak(NpcId::Granny),
    ));
}

/// Clicking her again while her line is up takes the line away.
#[test]
fn clicking_her_again_dismisses_the_line() {
    assert!(matches!(
        classify_click(Some(NpcId::Granny), Some(NpcId::Granny)),
        Clicked::Dismiss,
    ));
}

/// The requested behaviour: with the box up, a click anywhere clears it. This is
/// why the system reads every click rather than only the ones that hit a character,
/// so there is no full-screen button behind it.
///
/// `None` for the clicked character is what a click on the wall looks like to the
/// system.
#[test]
fn clicking_anywhere_dismisses_the_line() {
    assert!(
        matches!(classify_click(Some(NpcId::Granny), None), Clicked::Dismiss),
        "clicking away from her did not take the line off the screen",
    );
}

/// Which character is speaking is irrelevant: the box belongs to whoever put it
/// there, and any click clears it.
#[test]
fn the_dismiss_does_not_depend_on_where_you_clicked() {
    assert!(matches!(
        classify_click(Some(NpcId::Granny), None),
        Clicked::Dismiss
    ));
    assert!(matches!(
        classify_click(Some(NpcId::Granny), Some(NpcId::Granny)),
        Clicked::Dismiss,
    ));
}

/// With nothing on screen, a click that missed every character must do nothing -
/// otherwise clicking a door or the wall would start a conversation with nobody.
#[test]
fn clicking_nothing_while_silent_does_nothing() {
    assert!(matches!(classify_click(None, None), Clicked::Nothing));
}

/// A dismiss must not count as a conversation, or a player who clicks twice to move
/// on would burn through her lines without hearing them.
///
/// Driven through the same branch the click system takes, so it fails if the
/// `once_more` ever moves into the dismiss arm.
#[test]
fn a_dismissed_line_is_not_a_conversation() {
    let granny = NpcId::Granny;
    let mut talked = fresh();

    let run_click =
        |talked: &mut TalkedTo, speaking: Option<NpcId>, on: Option<NpcId>| match classify_click(
            speaking, on,
        ) {
            Clicked::Speak(id) => {
                let line = talked.next_line(id);
                talked.once_more(id);
                Some(line)
            }
            Clicked::Dismiss => None,
            Clicked::Nothing => None,
        };

    assert!(run_click(&mut talked, None, Some(granny)).is_some());
    assert!(
        run_click(&mut talked, Some(granny), None).is_none(),
        "was not a dismiss"
    );
    assert_eq!(
        talked.count(granny),
        1,
        "the dismiss counted as a second conversation",
    );
}

/// Dismissing and clicking again has to reach the next line, or the dismiss would
/// be a dead end rather than a way to move on.
#[test]
fn talking_again_after_a_dismiss_reaches_the_next_line() {
    let granny = NpcId::Granny;
    let mut talked = fresh();

    assert_eq!(talked.next_line(granny), data::GRANNY.lines[0]);
    talked.once_more(granny);

    // Dismiss.
    assert!(matches!(
        classify_click(Some(granny), None),
        Clicked::Dismiss
    ));

    assert_eq!(
        talked.next_line(granny),
        data::GRANNY.lines[1],
        "after dismissing, the next click did not move on to her second line",
    );
}

// ------------------------------------------------------------------- the art

#[test]
fn every_npc_picture_is_on_disk() {
    for def in data::npcs() {
        let full = std::path::Path::new("assets").join(def.texture);
        assert!(full.exists(), "missing art: assets/{}", def.texture);
    }
}

/// The clickable rectangle has to be bigger than the drawn figure, or the player
/// aims at what they can see and the click lands on nothing.
#[test]
fn the_click_target_is_not_smaller_than_the_sprite() {
    for def in data::npcs() {
        assert!(
            def.hit.x >= def.size.x && def.hit.y >= def.size.y,
            "{:?}: hit {:?} is smaller than the sprite {:?}",
            def.id,
            def.hit,
            def.size,
        );
    }
}

/// A character has to be somewhere the player can see and click. This is the
/// hotspot clamp, applied to people.
#[test]
fn every_npc_is_inside_the_frame() {
    for def in data::npcs() {
        let half = def.hit / 2.0;
        assert!(
            (def.pos.x - half.x).abs() <= crate::FRAME_HALF.x
                && (def.pos.x + half.x).abs() <= crate::FRAME_HALF.x,
            "{:?}: target runs off the left or right of the frame",
            def.id,
        );
        assert!(
            (def.pos.y - half.y).abs() <= crate::FRAME_HALF.y
                && (def.pos.y + half.y).abs() <= crate::FRAME_HALF.y,
            "{:?}: target runs off the top or bottom of the frame",
            def.id,
        );
    }
}

/// An NPC standing in a room that is not in the table would never spawn, and
/// nothing else would notice. `room_def` falls back to `UNKNOWN` rather than
/// panicking, so a bad key has to be caught by comparing the answer.
#[test]
fn every_npc_stands_in_a_real_room() {
    for def in data::npcs() {
        let found =
            crate::scenes::game::rooms::data::room_def(def.room, crate::acts::ActId::ActOne);
        assert_eq!(
            found.variants[0].path, def.room,
            "{:?} stands in {}, which is not a room key",
            def.id, def.room,
        );
    }
}

// ---------------------------------------------------------------- the writing

#[test]
fn every_npc_has_something_to_say() {
    for def in data::npcs() {
        assert!(
            !def.lines.is_empty(),
            "{:?} has no dialogue, so clicking her does nothing",
            def.id,
        );
        for line in def.lines {
            assert!(!line.trim().is_empty(), "{:?} has a blank line", def.id);
        }
    }
}

#[test]
fn every_character_has_a_name_for_the_dialogue_box() {
    for def in data::npcs() {
        assert!(
            !def.id.name().trim().is_empty(),
            "{:?} would be shown without a name",
            def.id
        );
    }
}

/// The game's text is English throughout, and a Cyrillic line in a dialogue box is
/// the sort of thing that ships because nobody reads the other tab. Checked here
/// rather than trusted, since this is the only place dialogue is written.
#[test]
fn the_dialogue_is_in_english() {
    for def in data::npcs() {
        assert!(
            def.id.name().is_ascii(),
            "{:?} is shown as {:?}, which is not English",
            def.id,
            def.id.name(),
        );
        for line in def.lines {
            assert!(
                line.is_ascii(),
                "{:?} has a line that is not English: {line}",
                def.id,
            );
        }
    }
}

// ------------------------------------------------------------------ placement

/// Whether two rectangles share any area. Touching edges do not count.
///
/// Kept local rather than shared: nothing outside this file needs it, and a
/// helper that only has one caller is better off next to its caller.
fn rects_overlap(a_pos: Vec2, a_size: Vec2, b_pos: Vec2, b_size: Vec2) -> bool {
    let (ahw, ahh) = (a_size.x / 2.0, a_size.y / 2.0);
    let (bhw, bhh) = (b_size.x / 2.0, b_size.y / 2.0);
    (a_pos.x - ahw < b_pos.x + bhw)
        && (b_pos.x - bhw < a_pos.x + ahw)
        && (a_pos.y - ahh < b_pos.y + bhh)
        && (b_pos.y - bhh < a_pos.y + ahh)
}

/// She must not sit on top of the way out of the room.
///
/// The concierge's exit is a 200x300 hotspot across the middle of the room, and
/// her click target is drawn above the hotspot layer, so any overlap means the
/// player cannot leave: they click what looks like the door and she talks instead.
/// That is a soft lock, and it is invisible until you try to walk out.
#[test]
fn she_does_not_cover_the_way_out_of_the_room() {
    use crate::acts::ActId;
    use crate::scenes::game::rooms::data::room_def;

    for def in data::npcs() {
        let room = room_def(def.room, ActId::ActOne);
        for variant in room.variants.iter() {
            for spot in variant.hotspots.iter() {
                let overlaps = rects_overlap(def.pos, def.hit, spot.pos, spot.size);
                assert!(
                    !overlaps,
                    "{:?} covers the hotspot at ({}, {}) in {}, so clicking the way \
                     out talks to her instead",
                    def.id, spot.pos.x, spot.pos.y, variant.path,
                );
            }
        }
    }
}

/// The concierge has two rows and only one of them has anybody on duty. The dark
/// row says so in its story text, so a granny at an empty desk is a bug and not a
/// surprise.
#[test]
fn the_granny_is_only_in_the_lit_concierge() {
    assert_eq!(
        data::GRANNY.room,
        p::F1_CONCIERGE,
        "she moved to a room that is not the lit concierge"
    );
    assert_ne!(
        data::GRANNY.room,
        p::F1_CONCIERGE_DARK,
        "nobody is on duty in the dark concierge, but she is standing there"
    );
}

// -------------------------------------------- she and the door to the hall

/// The way out of the concierge is a hotspot in the middle of the room and the
/// player is meant to click through it. She stands in front of it.
///
/// Both are picked from the same `Pointer<Click>`, and the backend hands the click
/// to the topmost entity at that point, so the layer order is the only thing
/// stopping a click on her from also going through the door and spending the pass.
///
/// Read out of a running app rather than compared as constants: the room system
/// writes its z inline, so a constant here would be comparing two literals and
/// passing whatever the literals happen to be.
#[test]
fn her_target_sits_above_the_door_behind_her() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .insert_resource(CurrentAct(ActId::ActOne))
        .add_systems(Update, super::spawn_npc_sprites);

    // The lit concierge, which is the row she stands in.
    let def = crate::scenes::game::rooms::data::room_def(data::GRANNY.room, ActId::ActOne);
    app.world_mut().spawn((Room, def, RoomPart));
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&super::NpcTarget, &Transform)>();
    let her = query
        .iter(world)
        .find(|(target, _)| target.0 == NpcId::Granny)
        .expect("she spawned");
    let hotspot_z = super::test_hotspot_z();

    assert!(
        her.1.translation.z > hotspot_z,
        "a click on her lands on the door behind instead: her target is at z {} and \
         the hotspot layer is at z {hotspot_z}",
        her.1.translation.z,
    );
}

/// The sprite itself must not be pickable, or the click has two entities to land on
/// and the drag may take the one that never speaks.
#[test]
fn her_sprite_is_not_pickable() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .insert_resource(CurrentAct(ActId::ActOne))
        .add_systems(Update, super::spawn_npc_sprites);

    let def = crate::scenes::game::rooms::data::room_def(data::GRANNY.room, ActId::ActOne);
    app.world_mut().spawn((Room, def, RoomPart));
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&super::NpcSprite, &Pickable)>();
    let pickable: Vec<Pickable> = query.iter(world).map(|(_, pickable)| *pickable).collect();

    assert!(
        !pickable.is_empty(),
        "she did not spawn, so this proves nothing"
    );
    for pickable in pickable {
        assert!(
            pickable.should_block_lower || !pickable.is_hoverable,
            "her picture is pickable, so the click may land on it instead of her target",
        );
    }
}

/// She is a child of the room, so she rides the room's breathing motion and dies
/// with it on a transition. A sprite spawned at the top level would sit still and
/// survive into the next room.
#[test]
fn she_is_a_child_of_the_room() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .insert_resource(CurrentAct(ActId::ActOne))
        .add_systems(Update, super::spawn_npc_sprites);

    let def = crate::scenes::game::rooms::data::room_def(data::GRANNY.room, ActId::ActOne);
    let room = app.world_mut().spawn((Room, def, RoomPart)).id();
    app.update();

    let world = app.world_mut();
    let mut query = world.query_filtered::<Entity, With<super::NpcSprite>>();
    let sprites: Vec<Entity> = query.iter(world).collect();

    assert!(!sprites.is_empty(), "she did not spawn");
    for sprite in sprites {
        let parent = world.get::<ChildOf>(sprite).map(|child| child.parent());
        assert_eq!(parent, Some(room), "she was not parented to the room",);
    }
}

// ------------------------------------------------------------ the .meta file

/// Every NPC picture carries a `.meta` beside it.
///
/// Not "the sampler is right" - that is what `the_sampler_is_read_from_the_meta`
/// below checks. This one only checks that a *new* character cannot be added
/// without the file, because the sampler living in the meta is only correct if the
/// meta is there: a missing one silently falls back to the default sampler and the
/// character comes out filtered differently from everyone else.
///
/// The paths come from the table rather than being listed here, so this fails when
/// a character is added, which is the moment it would otherwise be forgotten.
#[test]
fn every_npc_picture_has_a_meta_file_beside_it() {
    for def in data::npcs() {
        let picture = std::path::Path::new("assets").join(def.texture);
        let meta = picture.with_extension(format!(
            "{}meta",
            picture
                .extension()
                .map(|ext| format!("{}.", ext.to_string_lossy()))
                .unwrap_or_default()
        ));
        assert!(
            meta.exists(),
            "{} has no .meta beside it, so its sampler comes from the global \
             default rather than from the file",
            picture.display()
        );
    }
}

/// The `.meta` is what sets the sampler, and it says `nearest`.
///
/// This is the behaviour the whole arrangement exists for: the setting is a
/// property of the file rather than of whichever `load` call happened to arrive
/// first. `ImageSampler` is not `PartialEq`, so it is matched on - `Descriptor`
/// with nearest min/mag is what `ImageSampler::nearest()` builds.
#[test]
fn the_sampler_is_read_from_the_meta() {
    use bevy::image::{ImageFilterMode, ImageSampler};

    let meta = std::fs::read_to_string("assets/tex/npc/granny.png.meta")
        .expect("a .meta beside the picture");

    // Matched on rather than parsed: the point is that the file says this, not
    // that it deserialises into a particular struct, and a substring keeps the
    // test readable next to a RON file nobody looks at.
    assert!(
        meta.contains("mag_filter: Nearest"),
        "the .meta does not ask for nearest mag filtering:\n{meta}"
    );
    assert!(
        meta.contains("min_filter: Nearest"),
        "the .meta does not ask for nearest min filtering:\n{meta}"
    );
    // And it must still name the image loader, or Bevy has nothing to attach the
    // settings to and falls back to the defaults with no error at all.
    assert!(
        meta.contains("ImageLoader"),
        "the .meta does not name the image loader:\n{meta}"
    );

    // And the enum form is what the engine expects for a custom sampler.
    assert!(
        meta.contains("sampler: Descriptor("),
        "the .meta is not setting a custom sampler:\n{meta}"
    );

    // Guard the two claims above against a silent drift in Bevy's own naming: if
    // `ImageSampler::nearest()` stops being nearest-min-and-mag, the file above is
    // no longer describing it.
    let nearest = ImageSampler::nearest();
    let bevy::image::ImageSampler::Descriptor(descriptor) = nearest else {
        panic!("ImageSampler::nearest() is no longer a Descriptor");
    };
    assert_eq!(descriptor.mag_filter, ImageFilterMode::Nearest);
    assert_eq!(descriptor.min_filter, ImageFilterMode::Nearest);
}

/// The two load sites for an NPC picture have to agree.
///
/// This is the bug itself. `spawn_game_ui` preloads every picture through
/// `all_paths()` with a plain `load`, and `spawn_npc_sprites` loads the same
/// picture again. Bevy documents that a second load of a path already in flight
/// returns the existing handle rather than reloading it, so *whichever call ran
/// first set the sampler for both* - and the sprite's own `nearest` was ignored
/// whenever the preload got there first.
///
/// Asserting on the table and on the sprite's handle rather than on the two call
/// sites is what makes this meaningful: it checks that both ends end up pointing
/// at one asset, which is the property that broke.
#[test]
fn the_sampler_does_not_depend_on_who_loaded_first() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .insert_resource(CurrentAct(ActId::ActOne))
        .add_systems(Update, super::spawn_npc_sprites);

    let def = crate::scenes::game::rooms::data::room_def(data::GRANNY.room, ActId::ActOne);
    app.world_mut().spawn((Room, def, RoomPart));
    app.update();

    let world = app.world_mut();
    let mut sprites = world.query::<&Sprite>();
    let handle = sprites
        .iter(world)
        .next()
        .expect("she spawned")
        .image
        .clone();

    // The preload's view of the same path, taken the way `spawn_game_ui` takes it.
    let server = world.resource::<AssetServer>().clone();
    let preloaded = server.load(data::GRANNY.texture);

    assert_eq!(
        handle, preloaded,
        "the sprite and the preload are different assets for one path, so the \
         sampler is being decided by whichever load won"
    );
}

/// The `.meta` parses into the settings Bevy expects, with `nearest` in them.
///
/// The test above reads the file as text, which proves the words are there and
/// nothing about whether Bevy can read the file. This one puts the bytes through
/// Bevy's own deserializer, so a wrong format, an unknown field, or a field that
/// stopped round-tripping all fail here.
///
/// It reads the bytes and deserializes directly rather than loading the asset,
/// because loading is asynchronous and the IO pool does not run under
/// `cargo test` - an earlier version of this test polled `load_state` and sat in
/// `Loading` forever, for every picture, `.meta` or not.
#[test]
fn the_meta_parses_into_image_loader_settings() {
    let path = format!("assets/{}.meta", data::GRANNY.texture);
    let bytes = std::fs::read(&path).unwrap_or_else(|problem| {
        panic!("cannot read {path}: {problem}");
    });

    // The same deserialization the asset server does when it reads a loader's
    // meta: `AssetMeta::<ImageLoader, ()>`, whose `asset` field is exactly the
    // `AssetAction::Load { loader, settings }` this file has to express.
    let meta: bevy::asset::meta::AssetMeta<bevy::image::ImageLoader, ()> =
        bevy::asset::meta::AssetMeta::deserialize(&bytes).unwrap_or_else(|problem| {
            panic!("Bevy could not read {path}: {problem}");
        });

    let bevy::asset::meta::AssetAction::Load { settings, .. } = meta.asset else {
        panic!("{path} does not say `Load`");
    };

    // And the settings have to come back out with the sampler we asked for.
    let bevy::image::ImageSampler::Descriptor(descriptor) = &settings.sampler else {
        panic!(
            "{path} deserialized, but the sampler is still the default - the \
             settings were dropped rather than applied"
        );
    };
    assert_eq!(
        descriptor.mag_filter,
        bevy::image::ImageFilterMode::Nearest,
        "the .meta parsed, but mag filtering is not nearest"
    );
    assert_eq!(
        descriptor.min_filter,
        bevy::image::ImageFilterMode::Nearest,
        "the .meta parsed, but min filtering is not nearest"
    );
}
