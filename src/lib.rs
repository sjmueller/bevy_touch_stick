//! Virtual touch screen analog joysticks for the Bevy game engine.
//!
//! see the examples for more detailed usage
//!
//! ## Example
//!```no_run
// # use bevy::prelude::*;
// use bevy_touch_stick::prelude::*
//! commands.spawn((
//!     BackgroundColor(Color::BLUE),
//!     TouchStickUiBundle {
//!         stick: TouchStick {
//!             id: Stick::Right,
//!             stick_type: TouchStickType::Dynamic,
//!             ..default()
//!         },
//!         style: Style {
//!             width: Val::Px(150.),
//!             height: Val::Px(150.),
//!             position_type: PositionType::Absolute,
//!             right: Val::Px(35.),
//!             bottom: Val::Percent(15.),
//!             ..default()
//!         },
//!         ..default()
//!     }
//! ));
//!```
//!
use bevy::{prelude::*, reflect::TypePath, ui::UiSystem};
use std::{hash::Hash, marker::PhantomData};

mod behavior;
#[cfg(feature = "gamepad_mapping")]
mod gamepad;
mod input;
mod ui;

/// commonly used exports from this crate
pub mod prelude {
    #[cfg(feature = "gamepad_mapping")]
    pub use crate::TouchStickGamepadMapping;
    pub use crate::{
        TouchStick, TouchStickPlugin, TouchStickType, TouchStickUi, TouchStickUiBundle,
    };
}

#[cfg(feature = "gamepad_mapping")]
use crate::gamepad::GamepadMappingPlugin;
#[cfg(feature = "gamepad_mapping")]
pub use crate::gamepad::TouchStickGamepadMapping;

pub use crate::{
    behavior::TouchStickType,
    ui::{
        TouchStickInteractionArea, TouchStickUi, TouchStickUiBundle, TouchStickUiKnob,
        TouchStickUiOutline,
    },
};
use crate::{
    input::{
        send_drag_events_from_mouse, send_drag_events_from_touch, update_sticks_from_drag_events,
        DragEvent,
    },
    ui::TouchStickUiPlugin,
};

/// pure data, independent of bevy_ui
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component, Default)]
pub struct TouchStick<S: StickIdType> {
    /// Type used for identifying this [`TouchStick`]
    pub id: S,
    /// what drag event sequence is currently affecting this `TouchStick`
    pub drag_id: Option<u64>,
    /// values smaller than this will not send `TouchStickEvent`
    pub dead_zone: f32,
    /// last drag positon of touchstick. only applies too `TouchStickType::Dynamic`
    ///
    /// `Vec2::ZERO` if node is released
    pub base_position: Vec2,
    /// The screen position where the drag was started
    pub drag_start: Vec2,
    /// The screen position where the drag is currently at
    pub drag_position: Vec2,
    /// Value with maximum magnitude 1
    pub value: Vec2,
    /// In input space (y-down)
    pub interactable_zone: Rect,
    /// Defines the positioning behavior of the `TouchStick`
    pub stick_type: TouchStickType,
}

impl<S: StickIdType> Default for TouchStick<S> {
    fn default() -> Self {
        Self {
            id: default(),
            drag_id: None,
            dead_zone: 0.,
            base_position: default(),
            drag_start: default(),
            drag_position: default(),
            value: default(),
            interactable_zone: Rect {
                min: Vec2::MIN,
                max: Vec2::MAX,
            },
            stick_type: default(),
        }
    }
}

impl<S: StickIdType> From<S> for TouchStick<S> {
    fn from(id: S) -> Self {
        Self::new(id)
    }
}

impl<S: StickIdType> TouchStick<S> {
    /// creates a new `TouchStick` with the given id.
    pub fn new(id: S) -> Self {
        Self { id, ..default() }
    }
}

/// plugin holding `TouchStick` functionality
pub struct TouchStickPlugin<S> {
    _marker: PhantomData<S>,
}

impl<S> Default for TouchStickPlugin<S> {
    fn default() -> Self {
        Self { _marker: default() }
    }
}

impl<S: StickIdType> Plugin for TouchStickPlugin<S> {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<TouchStickInteractionArea>()
            .register_type::<TouchStickUi<S>>()
            .register_type::<TouchStick<S>>()
            .register_type::<TouchStickType>()
            .register_type::<TouchStickEventType>()
            .add_event::<TouchStickEvent<S>>()
            .add_event::<DragEvent>()
            .add_plugins(TouchStickUiPlugin::<S>::default())
            .add_systems(
                PreUpdate,
                (
                    // todo: resolve ambiguity
                    send_drag_events_from_touch.before(update_sticks_from_drag_events::<S>),
                    send_drag_events_from_mouse.before(update_sticks_from_drag_events::<S>),
                ),
            )
            .add_systems(PreUpdate, update_sticks_from_drag_events::<S>)
            .add_systems(
                PostUpdate,
                map_input_zones_from_ui_nodes::<S>.before(UiSystem::Layout),
            );

        #[cfg(feature = "gamepad_mapping")]
        app.add_plugins(GamepadMappingPlugin::<S>::default());
    }
}

/// type definition for TouchStick identifier
pub trait StickIdType:
    Hash + Sync + Send + Clone + Default + Reflect + FromReflect + TypePath + 'static
{
}

impl<S: Hash + Sync + Send + Clone + Default + Reflect + FromReflect + TypePath + 'static>
    StickIdType for S
{
}

fn map_input_zones_from_ui_nodes<S: StickIdType>(
    interaction_areas: Query<(&Transform, &Node), With<TouchStickInteractionArea>>,
    mut sticks: Query<&mut TouchStick<S>>,
) {
    for (transform, node) in &interaction_areas {
        // todo: match stick ids!

        let pos = transform.translation.truncate();
        let size = node.size();
        let interaction_area = Rect::from_center_size(pos, size);

        for mut stick in sticks.iter_mut() {
            stick.interactable_zone = interaction_area;
        }
    }
}

/// what action the TouchStick is experiencing
#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[reflect]
pub enum TouchStickEventType {
    /// `TouchStick` was activated
    Press,
    /// `TouchStick` was moved
    Drag,
    /// `TouchStick` was deactivated
    Release,
}

/// event sent whenever the touchstick is interacted.
#[derive(Event)]
pub struct TouchStickEvent<S: StickIdType> {
    id: S,
    event: TouchStickEventType,
    value: Vec2,
}

impl<S: StickIdType> TouchStickEvent<S> {
    /// Returns the id for the stick that sent the event
    pub fn id(&self) -> S {
        self.id.clone()
    }

    /// Value of the joystick, maximum length 1
    pub fn value(&self) -> Vec2 {
        self.value
    }

    /// Return the Type of Joystick Event
    pub fn get_type(&self) -> TouchStickEventType {
        self.event
    }
}
