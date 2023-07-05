//! Input reading and key mapping.

use bevy::{ecs::system::SystemParam, input::mouse::MouseMotion, prelude::*};

pub struct BindingsPlugin;

impl Plugin for BindingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Bindings>();
    }
}

/// What inputs are bound to what actions?
#[derive(Debug, Clone, Resource)]
pub struct Bindings {
    pub look: Axis2,

    pub walk: Axis2,
    pub up_down: Axis,

    pub toggle_cursor: Button,
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            look: Axis2::Mouse {
                sensitivity: Vec2::splat(0.004),
            },

            walk: Axis2::Composite {
                vertical: Axis::Composite {
                    pos: Button::Key(KeyCode::W),
                    neg: Button::Key(KeyCode::S),
                },
                horizontal: Axis::Composite {
                    pos: Button::Key(KeyCode::A),
                    neg: Button::Key(KeyCode::D),
                },
            },
            up_down: Axis::Composite {
                pos: Button::Key(KeyCode::Space),
                neg: Button::Key(KeyCode::LControl),
            },

            toggle_cursor: Button::Key(KeyCode::Escape),
        }
    }
}

/// An input that can be bound to an action.
pub trait Binding {
    /// Inputs used to read state of this
    type Inputs<'w, 's>;

    type Output;

    fn value(&self, inputs: &mut Self::Inputs<'_, '_>) -> Self::Output;
}

/// A button keybind.
#[derive(Debug, Clone)]
pub enum Button {
    Key(KeyCode),
}

#[derive(Debug, SystemParam)]
pub struct ButtonInputs<'w> {
    keys: Res<'w, Input<KeyCode>>,
}

impl Binding for Button {
    type Inputs<'w, 's> = ButtonInputs<'w>;

    type Output = bool;

    /// Returns true if the button is currently pressed or was just pressed.
    fn value(&self, inputs: &mut Self::Inputs<'_, '_>) -> Self::Output {
        match self {
            &Button::Key(code) => inputs.keys.pressed(code) || inputs.keys.just_pressed(code),
        }
    }
}

impl Button {
    pub fn just_pressed(
        &self,
        inputs: &mut <Self as Binding>::Inputs<'_, '_>,
    ) -> <Self as Binding>::Output {
        match self {
            Button::Key(code) => inputs.keys.just_pressed(*code),
        }
    }
}

/// A 1-dimensional axis keybind.
#[derive(Debug, Clone)]
pub enum Axis {
    Composite { pos: Button, neg: Button },
}

#[derive(Debug, SystemParam)]
pub struct AxisInputs<'w, 's> {
    button: <Button as Binding>::Inputs<'w, 's>,
}

impl<'w, 's> AsMut<ButtonInputs<'w>> for AxisInputs<'w, 's> {
    fn as_mut(&mut self) -> &mut ButtonInputs<'w> {
        &mut self.button
    }
}

impl Binding for Axis {
    type Inputs<'w, 's> = AxisInputs<'w, 's>;

    type Output = f32;

    /// How much to move whatever is bound to this input.
    ///
    /// Note that while this function usually returns values in the range [-1.0,
    /// 1.0], that is not guaranteed.
    fn value(&self, inputs: &mut Self::Inputs<'_, '_>) -> Self::Output {
        match self {
            Axis::Composite { pos, neg } => {
                pos.value(inputs.as_mut()) as u8 as f32 - neg.value(inputs.as_mut()) as u8 as f32
            }
        }
    }
}

impl Axis {
    pub fn value_clamped(
        &self,
        inputs: &mut <Self as Binding>::Inputs<'_, '_>,
    ) -> <Self as Binding>::Output {
        self.value(inputs).clamp(-1.0, 1.0)
    }
}

/// A 2-dimensional axis keybind.
#[derive(Debug, Clone)]
pub enum Axis2 {
    Mouse { sensitivity: Vec2 },
    Composite { horizontal: Axis, vertical: Axis },
}

#[derive(Debug, SystemParam)]
pub struct Axis2Inputs<'w, 's> {
    mouse: EventReader<'w, 's, MouseMotion>,
    axis: <Axis as Binding>::Inputs<'w, 's>,
}

impl<'w, 's> AsMut<AxisInputs<'w, 's>> for Axis2Inputs<'w, 's> {
    fn as_mut(&mut self) -> &mut AxisInputs<'w, 's> {
        &mut self.axis
    }
}

impl Binding for Axis2 {
    type Inputs<'w, 's> = Axis2Inputs<'w, 's>;

    type Output = Vec2;

    /// How much to move whatever is bound to this input.
    ///
    /// Note that while this function usually returns vectors with magnitudes in
    /// the range [0.0, 1.0], it may return vectors with magnitude greater than
    /// 1.0.  If that will cause problems, use [`Self::value_clamped`].
    fn value(&self, inputs: &mut Self::Inputs<'_, '_>) -> Self::Output {
        match self {
            &Self::Mouse { sensitivity } => {
                inputs
                    .mouse
                    .into_iter()
                    .map(|event| event.delta)
                    .sum::<Vec2>()
                    * sensitivity
            }
            Self::Composite {
                horizontal,
                vertical,
            } => Vec2::new(
                horizontal.value(inputs.as_mut()),
                vertical.value(inputs.as_mut()),
            ),
        }
    }
}

impl Axis2 {
    pub fn value_clamped(
        &self,
        inputs: &mut <Self as Binding>::Inputs<'_, '_>,
    ) -> <Self as Binding>::Output {
        let value = self.value(inputs);
        if value.length_squared() <= 1.0 {
            value
        } else {
            value.normalize()
        }
    }
}
