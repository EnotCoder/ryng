use bevy::prelude::*;

use crate::UiScale;
use crate::acts::{Inventory, Item};
use crate::state::GameState;

pub const INVENTORY_SLOT_COUNT: usize = 4;
pub const SLOT_SIZE: f32 = 128.0 / 1.5;
pub const ICON_SIZE: f32 = SLOT_SIZE * 100.0 / 128.0;

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
    pub icon_pass: Handle<Image>,
    pub icon_main_key: Handle<Image>,
}

fn icon_handle(textures: &InventoryTextures, item: &Item) -> Handle<Image> {
    match item {
        Item::Pass => textures.icon_pass.clone(),
        Item::MainKey => textures.icon_main_key.clone(),
    }
}

pub fn spawn_inventory_ui(
    mut commands: Commands,
    ui_scale: Res<UiScale>,
    textures: Res<InventoryTextures>,
    inventory: Res<Inventory>,
    active: Res<ActiveInvSlot>,
) {
    let s = ui_scale.0;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(12.0 * s),
                bottom: Val::Px(12.0 * s),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0 * s),
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
                        Node {
                            width: Val::Px(SLOT_SIZE * s),
                            height: Val::Px(SLOT_SIZE * s),
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
                                    .map(|it| icon_handle(&textures, it))
                                    .unwrap_or_default(),
                            ),
                            Node {
                                width: Val::Px(ICON_SIZE * s),
                                height: Val::Px(ICON_SIZE * s),
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

pub fn update_inventory_ui(
    inventory: Res<Inventory>,
    active: Res<ActiveInvSlot>,
    textures: Res<InventoryTextures>,
    mut params: ParamSet<(
        Query<(&InventorySlot, &mut ImageNode)>,
        Query<(&mut SlotIcon, &mut ImageNode, &mut Visibility)>,
    )>,
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
            .map(|it| icon_handle(&textures, it))
            .unwrap_or_default();
    }
}
