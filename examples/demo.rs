use bevy::prelude::*;
use bevy::{math::vec2, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_rapier2d::prelude::*;
use std::convert;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Marching Pixels Demo".to_string(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
            EguiPlugin,
            RapierPhysicsPlugin::<bevy_rapier2d::plugin::NoUserData>::default(),
            bevy_rapier2d::render::RapierDebugRenderPlugin::default(),
        ))
        .init_resource::<ShapeMask>()
        .init_resource::<CursorPosition>()
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                (shape_mask_update, cursor_position_show).chain(),
                shape_collider_update,
                cursor_position_update,
            ),
        )
        .run();
}

const SCALE: f32 = 50.0;

const TRANSFORM: Transform = Transform::from_translation(
    vec2(
        -0.5 * SCALE * ShapeMask::WIDTH as f32,
        0.5 * SCALE * ShapeMask::HEIGHT as f32,
    )
    .extend(0.0),
)
.with_scale(vec2(SCALE, -SCALE).extend(1.0));

#[derive(Component)]
struct ShapeTag;

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((ShapeTag, TransformBundle::from_transform(TRANSFORM)));
}

#[derive(Debug, Resource, Deref, DerefMut, Default)]
struct ShapeMask([[bool; Self::WIDTH]; Self::HEIGHT]);

impl ShapeMask {
    const WIDTH: usize = 16;
    const HEIGHT: usize = 16;

    fn collider(&self) -> Option<Collider> {
        if !self.0.into_iter().flatten().any(convert::identity) {
            return None;
        }
        let args = marching_pixels::Args {
            width: Self::WIDTH,
            height: Self::HEIGHT,
            pixels: self.0.into_iter().flatten(),
        };
        let mut algorithm = marching_pixels::Algorithm::with_capacity(Self::WIDTH, Self::HEIGHT);
        let (vertices, indices) = algorithm.search(args);
        vertices
            .clone()
            .enumerate()
            .for_each(|(index, vertex)| info!(?vertex, ?index));
        indices.clone().for_each(|[from, to]| info!(?from, ?to));
        let vertices: Vec<Vec2> = vertices.map(|[x, y]| vec2(x as _, y as _)).collect();

        let indices: Vec<[u32; 2]> = indices.map(|[x, y]| [x as _, y as _]).collect();
        Some(Collider::convex_decomposition_with_params(
            &vertices,
            &indices,
            &VHACDParameters {
                fill_mode: FillMode::FloodFill {
                    detect_cavities: true,
                    detect_self_intersections: true,
                },
                ..Default::default()
            },
        ))
    }
}

fn shape_mask_update(
    mut contexts: EguiContexts,
    mut mask: ResMut<ShapeMask>,
    keys: Res<Input<KeyCode>>,
) {
    egui::Window::new("Shape Mask")
        .auto_sized()
        .show(contexts.ctx_mut(), |ui| {
            let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
            let delete = keys.pressed(KeyCode::Back) || keys.pressed(KeyCode::Delete);
            for row_index in 0..ShapeMask::HEIGHT {
                ui.horizontal(|ui| {
                    for column_index in 0..ShapeMask::WIDTH {
                        let current = mask[row_index][column_index];
                        let response = ui.radio(current, "");
                        if response.clicked()
                            || (shift && !current || delete && current) && response.hovered()
                        {
                            mask[row_index][column_index] = !current;
                        }
                    }
                });
            }
        });
}

fn shape_collider_update(
    mut commands: Commands,
    mask: Res<ShapeMask>,
    q: Query<Entity, With<ShapeTag>>,
) {
    if mask.is_changed() {
        match mask.collider() {
            Some(collider) => {
                for entity in &q {
                    commands.entity(entity).insert(collider.clone());
                }
            }
            None => {
                for entity in &q {
                    commands.entity(entity).remove::<Collider>();
                }
            }
        }
    }
}

#[derive(Debug, Resource, Deref, DerefMut, Default)]
struct CursorPosition(Option<[u16; 2]>);

fn cursor_position_update(
    q: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut pos: ResMut<CursorPosition>,
) {
    for window in &q {
        **pos = window
            .cursor_position()
            .map(|v| (v - 0.5 * vec2(window.width(), window.height())) / SCALE)
            .map(|v| v + 0.5 * vec2(ShapeMask::WIDTH as _, ShapeMask::HEIGHT as _))
            .map(|v| [v.x.round() as i32, v.y.round() as i32])
            .filter(|v| v[0] >= 0 && v[1] >= 0)
            .map(|[x, y]| [x as _, y as _]);
    }
}

fn cursor_position_show(mut contexts: EguiContexts, pos: Res<CursorPosition>) {
    egui::Window::new("Cursor Position")
        .auto_sized()
        .show(contexts.ctx_mut(), |ui| match **pos {
            Some([x, y]) => {
                ui.label(format!("x: {x}"));
                ui.label(format!("y: {y}"));
            }
            None => {
                ui.label("Out of bounds");
            }
        });
}
