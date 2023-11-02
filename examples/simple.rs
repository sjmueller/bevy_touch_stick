use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_touch_stick::{
    prelude::*, TintColor, VirtualJoystickEvent, VirtualJoystickEventType,
    VirtualJoystickInteractionArea,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(VirtualJoystickPlugin::<String>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update_joystick)
        .run();
}

#[derive(Component)]
struct Player {
    max_speed: f32,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0., 0., 5.0),
        ..default()
    });

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0., 0., 0.),
                ..default()
            },
            texture: asset_server.load("knob.png"),
            sprite: Sprite {
                color: Color::PURPLE,
                custom_size: Some(Vec2::new(50., 50.)),
                ..default()
            },
            ..default()
        },
        Player { max_speed: 50. },
    ));

    // Spawn a stick at horizontal center
    commands.spawn((
        VirtualJoystickInteractionArea,
        VirtualJoystickBundle::new(VirtualJoystickNode {
            border_image: asset_server.load("outline.png"),
            knob_image: asset_server.load("knob.png"),
            knob_size: Vec2::new(80., 80.),
            dead_zone: 0.,
            id: "UniqueJoystick".to_string(),
            behaviour: VirtualJoystickType::Floating,
        })
        .set_color(TintColor(Color::WHITE.with_a(0.2)))
        .set_style(Style {
            width: Val::Px(150.),
            height: Val::Px(150.),
            position_type: PositionType::Absolute,
            left: Val::Percent(50.),
            bottom: Val::Percent(15.),
            ..default()
        }),
        // Make it easy to see the area in which the stick can be interacted with
        BackgroundColor(Color::ORANGE_RED.with_a(0.3)),
    ));
}

fn update_joystick(
    mut stick_events: EventReader<VirtualJoystickEvent<String>>,
    mut stick_colors: Query<(&mut TintColor, &VirtualJoystickNode<String>)>,
    mut players: Query<(&mut Transform, &Player)>,
    time: Res<Time>,
) {
    let (mut player, player_data) = players.single_mut();

    for stick in stick_events.iter() {
        match stick.get_type() {
            VirtualJoystickEventType::Press | VirtualJoystickEventType::Drag => {
                let (mut color, node) = stick_colors.single_mut();
                if node.id == stick.id() {
                    *color = TintColor(Color::WHITE);
                }
            }
            VirtualJoystickEventType::Up => {
                let (mut color, node) = stick_colors.single_mut();
                if node.id == stick.id() {
                    *color = TintColor(Color::WHITE.with_a(0.2));
                }
            }
        }

        let move_delta = stick.value() * player_data.max_speed * time.delta_seconds();
        player.translation += move_delta.extend(0.);
    }
}
