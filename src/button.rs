use bevy::prelude::*;
use std::collections::HashSet;

pub const NORMAL_BUTTON: Color = Color::srgb(0.2, 0.6, 0.9);
pub const HOVERED_BUTTON: Color = Color::srgb(0.4, 0.8, 0.6);
pub const PRESSED_BUTTON: Color = Color::srgb(0.9, 0.3, 0.3);

pub const BUTTON_SIZE: Vec2 = Vec2::new(150.0, 50.0);
pub const BUTTON_GAP: f32 = 50.0;
pub const FONT_SIZE: FontSize = FontSize::Px(20.0);

#[derive(Component)]
pub struct ClickHandler(Box<dyn Fn() + Send + Sync>);

pub fn draw_button(
    parent: &mut ChildSpawnerCommands<'_>, text: &str,
    on_click: impl Fn() + Send + Sync + 'static,
){
    parent.spawn((
        Button,
        ClickHandler(Box::new(on_click)),
        Node {
            width: Val::Px(BUTTON_SIZE.x),
            height: Val::Px(BUTTON_SIZE.y),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Interaction::default(),
        BackgroundColor(NORMAL_BUTTON),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new(text),
            TextFont {
                font_size: FONT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

pub fn button_system(
    mut query: Query<
        (Entity, &Interaction, &ClickHandler, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut was_pressed: Local<HashSet<Entity>>,
) {
    for (entity, interaction, handler, mut color) in &mut query {
        match *interaction {
            Interaction::Pressed => {
                was_pressed.insert(entity);
                color.0 = PRESSED_BUTTON;
            }
            Interaction::Hovered => {
                let released = was_pressed.remove(&entity);
                color.0 = HOVERED_BUTTON;
                if released {
                    (handler.0)();
                }
            }
            Interaction::None => {
                let released = was_pressed.remove(&entity);
                color.0 = NORMAL_BUTTON;
                if released {
                    (handler.0)();
                }
            }
        }
    }
}

pub fn on_click_settings(){
    println!("Settings");
}

pub fn on_click_play(){
    println!("Play");
}

pub fn on_click_quit(){
    println!("Quit");
}
