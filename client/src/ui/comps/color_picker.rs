use std::f32::consts::PI;

use bevy::{color::palettes::css::{BLACK, RED, WHITE}, ecs::world, math::VectorSpace, prelude::*, render::render_resource::AsBindGroup, ui::RelativeCursorPosition};
use lightyear::prelude::AppMessageExt;
use serde::{Deserialize, Serialize};

use crate::observe::observe;

const SHADER_ASSET_PATH: &str = "shaders/color_picker_material.wgsl";

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct ColorPickerMaterial {
    #[uniform(0)]
    pub color: Vec4
}

impl UiMaterial for ColorPickerMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

#[derive(Component)]
pub struct ColorPicker;

#[derive(Component)]
pub struct HuePicker(Color);

#[derive(EntityEvent)]
#[entity_event(propagate, auto_propagate)]
pub struct ColorPicked {
    pub entity: Entity,
    pub color: LinearRgba,
}

pub struct ColorPickerPlugin;

impl Plugin for ColorPickerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<ColorPickerMaterial>::default());
    }
}



fn pick_hue() -> impl Fn(On<Pointer<Click>>, Query<(&RelativeCursorPosition, &HuePicker)>, Commands) {
    move |trigger, background_gradient_query, mut commands| {
        let Ok((rel_pos, hue_picker)) = background_gradient_query.get(trigger.entity) else {
            return;
        };


        let Some(normal_mouse_pos) = rel_pos.normalized else {
            return;
        };

        let horizontal_color = LinearRgba::WHITE.lerp(hue_picker.0.to_linear(), normal_mouse_pos.x + 0.5);
        let true_color = horizontal_color.lerp(LinearRgba::BLACK, normal_mouse_pos.y + 0.5);

        info!("Triggering Color Picked");
        commands.trigger(ColorPicked {
            entity: trigger.entity,
            color: true_color
        });

    }
}





pub fn single_color_gradient(
    id: u32,
    color: LinearRgba,
    box_size: Val,
    ui_materials: &mut ResMut<Assets<ColorPickerMaterial>>,
) -> impl Bundle
{
    let picker_material_handle = ui_materials.add(ColorPickerMaterial {
        color: color.to_f32_array().into()
    });
    (
        Node {
            width: Val::Auto,
            height: Val::Auto,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            border: UiRect::all(px(2.0)),
            
            ..Default::default()
        },
        BorderColor::all(BLACK),
        children![
            (
                Node {
                    width: box_size,
                    height: box_size,
                    ..Default::default()
                },
                Interaction::None,
                RelativeCursorPosition::default(),
                observe(pick_hue()),
                MaterialNode(picker_material_handle.clone()),
                HuePicker(LinearRgba::RED.into())
            ),
            (
                Node {
                    width: box_size,
                    height: box_size/6.25,
                    ..Default::default()
                },
                Interaction::None,
                RelativeCursorPosition::default(),
                BackgroundGradient(vec![LinearGradient {
                    color_space: InterpolationColorSpace::LinearRgba,
                    angle: PI / 2.0,
                    stops: vec![
                        ColorStop {
                            color: LinearRgba::RED.into(),
                            ..Default::default()
                        },
                        ColorStop {
                            color: LinearRgba::GREEN.into(),
                            ..Default::default()
                        },
                        ColorStop {
                            color: LinearRgba::BLUE.into(),
                            ..Default::default()
                        },
                        ColorStop {
                            color: LinearRgba::RED.into(),
                            ..Default::default()
                        }
                    ]
                }.into()]),
                ColorPicker,
                observe(
                move |
                    trigger: On<Pointer<Click>>, 
                    pos: Query<&RelativeCursorPosition, With<ColorPicker>>, 
                    mut hue: Single<&mut HuePicker>,
                    mut ui_materials: ResMut<Assets<ColorPickerMaterial>>
                | {
                    let handle_copy = picker_material_handle.clone();
                    let Ok(mouse_pos) = pos.get(trigger.entity) else { return;};
                    let Some(mut norm_pos) = mouse_pos.normalized else { return;};
                    norm_pos += 0.5;
                    let clicked_color = match norm_pos.x {
                        x if x <= 0.333 => LinearRgba::RED.lerp(LinearRgba::GREEN, x / 0.333),
                        x if x <= 0.666 => LinearRgba::GREEN.lerp(LinearRgba::BLUE, (x - 0.333) / 0.333),
                        x if x <= 1.0 => LinearRgba::BLUE.lerp(LinearRgba::RED, (x - 0.666) / 0.333),
                        _ => panic!("")
                    };

                    hue.0 = clicked_color.into();

                    let Some(mat) = ui_materials.get_mut(&handle_copy) else { info!("Could not find material"); return; };
                    mat.color = clicked_color.to_f32_array().into();

                })
            )
        ]

    )
}
