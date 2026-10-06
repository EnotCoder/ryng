use std::collections::HashMap;

use bevy::prelude::*;

use crate::{ScaledNode, UiScale};
use crate::acts::{Inventory, Item};
use crate::state::GameState;

pub const INVENTORY_SLOT_COUNT: usize = 4;

/// The slot artwork is `SLOT_ART_SIZE` square with a 1px border drawn inside it,
/// so a slot is drawn smaller than its picture or the frame shows up as a border
/// around the contents.
const SLOT_ART_SIZE: f32 = 128.0;
const SLOT_BORDER: f32 = 1.5;
pub const SLOT_SIZE: f32 = SLOT_ART_SIZE / SLOT_BORDER;

/// The icon art is 100px square inside the 128px slot, so the icon is drawn at the
/// same scale as the slot border and the two line up.
const ICON_ART_SIZE: f32 = 100.0;
pub const ICON_SIZE: f32 = SLOT_SIZE * ICON_ART_SIZE / SLOT_ART_SIZE;

/// Distance from the screen edge, and between slots.
const INVENTORY_MARGIN: f32 = 12.0;
const INVENTORY_GAP: f32 = 8.0;

#[derive(Component)]
pub struct InventorySlot {
    pub index: usize,
}

#[derive(Component)]
pub struct SlotIcon {
    pub index: usize,
    pub item: Option<Item>,
}

#[derive(Resource, Default)]
pub struct ActiveInvSlot(pub usize);

#[derive(Resource)]
pub struct InventoryTextures {
    pub active_slot: Handle<Image>,
    pub disabled_slot: Handle<Image>,
    /// One icon per `Item`, keyed by the item itself so adding a variant cannot
    /// leave a slot without an icon.
    pub icons: HashMap<Item, Handle<Image>>,
}

impl InventoryTextures {
    fn icon(&self, item: &Item) -> Handle<Image> {
        self.icons.get(item).cloned().unwrap_or_default()
    }
}

pub fn spawn_inventory_ui(
    mut commands: Commands,
    ui_scale: Res<UiScale>,
    textures: Res<InventoryTextures>,
    inventory: Res<Inventory>,
    active: Res<ActiveInvSlot>,
) {
    // `UiScale` itself, not the bare f32, so the `px` helper can be used.
    let s = *ui_scale;
    commands
        .spawn((
            ScaledNode {
                bottom: Some(INVENTORY_MARGIN),
                left: Some(INVENTORY_MARGIN),
                gap: Some(INVENTORY_GAP),
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                left: s.px(INVENTORY_MARGIN),
                bottom: s.px(INVENTORY_MARGIN),
                flex_direction: FlexDirection::Row,
                column_gap: s.px(INVENTORY_GAP),
                ..default()
            },
            Pickable::IGNORE,
            DespawnOnExit(GameState::Game),
        ))
        .with_children(|parent| {
            for index in 0..INVENTORY_SLOT_COUNT {
                let item = inventory.0.get(index).copied().flatten();
                let slot_bg = if index == active.0 {
                    textures.active_slot.clone()
                } else {
                    textures.disabled_slot.clone()
                };
                parent
                    .spawn((
                        InventorySlot { index },
                        ImageNode::new(slot_bg),
                        ScaledNode::sized(SLOT_SIZE, SLOT_SIZE),
                        Node {
                            width: s.px(SLOT_SIZE),
                            height: s.px(SLOT_SIZE),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        Interaction::default(),
                        Button,
                    ))
                    .with_children(|icon| {
                        icon.spawn((
                            SlotIcon { index, item },
                            ImageNode::new(
                                item.as_ref()
                                    .map(|it| textures.icon(it))
                                    .unwrap_or_default(),
                            ),
                            ScaledNode::sized(ICON_SIZE, ICON_SIZE),
                            Node {
                                width: s.px(ICON_SIZE),
                                height: s.px(ICON_SIZE),
                                ..default()
                            },
                            if item.is_some() {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            },
                        ));
                    });
            }
        });
}

pub fn slot_click_system(
    mut interactions: Query<(&Interaction, &InventorySlot), Changed<Interaction>>,
    mut active: ResMut<ActiveInvSlot>,
) {
    for (interaction, slot) in &mut interactions {
        if *interaction == Interaction::Pressed {
            active.0 = slot.index;
        }
    }
}

/// A slot and its background plate.
type SlotPlateQuery<'w, 's> = Query<'w, 's, (&'static InventorySlot, &'static mut ImageNode)>;

/// A slot's icon, and whether it is shown at all.
///
/// `Visibility` is here because an empty slot hides its icon rather than drawing a
/// blank one, so the two are always changed together.
type SlotIconQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut SlotIcon,
        &'static mut ImageNode,
        &'static mut Visibility,
    ),
>;

/// The slot and the icon are separate entities - the slot is a button, the icon is
/// its child - so a system that writes both needs `ParamSet` to hand out one at a
/// time. Naming the two queries keeps that legible; it is the reason this is not two
/// systems, which would race on the same frame.
pub fn update_inventory_ui(
    inventory: Res<Inventory>,
    active: Res<ActiveInvSlot>,
    textures: Res<InventoryTextures>,
    mut params: ParamSet<(SlotPlateQuery, SlotIconQuery)>,
) {
    for (slot, mut bg) in &mut params.p0() {
        let slot_active = slot.index == active.0;
        bg.image = if slot_active {
            textures.active_slot.clone()
        } else {
            textures.disabled_slot.clone()
        };
    }
    for (mut icon, mut img, mut vis) in &mut params.p1() {
        let item = inventory.0.get(icon.index).copied().flatten();
        icon.item = item;
        *vis = if item.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        img.image = item
            .as_ref()
            .map(|it| textures.icon(it))
            .unwrap_or_default();
    }
}
