use std::collections::HashMap;

use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use leafwing_input_manager::prelude::*;
use ourcraft::{Pos, Blocs, ChunkPos, CHUNK_S1, MAX_HEIGHT, Chunk};
use crate::render3d::Meshable;
use crate::texture_array::{TextureMap, TextureArrayPlugin};
use crate::{player::Dir, load_cols::{ColLoadOrders, ColUnloadEvent}};
const CAMERA_PAN_RATE: f32 = 0.1;

pub fn setup(mut commands: Commands, mut windows: Query<&mut Window>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 150., 10.)
            .looking_at(Vec3 {x: 0., y: 150., z: 0.}, Vec3::Y),
        ..Default::default()
    })
    .insert(InputManagerBundle::<CameraMovement> {
        input_map: InputMap::default()
            // This will capture the total continuous value, for direct use.
            // Note that you can also use discrete gesture-like motion, via the `MouseMotionDirection` enum.
            .insert(DualAxis::mouse_motion(), CameraMovement::Pan)
            .build(),
        ..default()
    });
    let mut window = windows.single_mut();
    window.cursor.grab_mode = CursorGrabMode::Locked;
}

fn pan_camera(mut query: Query<(&mut Transform, &ActionState<CameraMovement>), With<Camera3d>>, time: Res<Time>) {
    let (mut camera_transform, action_state) = query.single_mut();
    let camera_pan_vector = action_state.axis_pair(CameraMovement::Pan).unwrap();
    let c = time.delta_seconds() * CAMERA_PAN_RATE;
    camera_transform.rotate_y(-c * camera_pan_vector.x());
    camera_transform.rotate_local_x(-c * camera_pan_vector.y());
}

pub fn translate_cam(
    mut cam_query: Query<&mut Transform, With<Camera>>,
    player_query: Query<&Pos, (With<ActionState<Dir>>, Changed<Pos>)>
) {
    if let Ok(mut cam_pos) = cam_query.get_single_mut() {
        if let Ok(player_pos) = player_query.get_single() {
            cam_pos.translation.x = player_pos.x;
            cam_pos.translation.y = player_pos.y;
            cam_pos.translation.z = player_pos.z;
        }
    }
}

pub fn on_col_load(
    mut commands: Commands,
    mut ev_load: ResMut<ColLoadOrders>,
    mut blocs: ResMut<Blocs>,
    mut col_ents: ResMut<ChunkEntities>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    texture_map: Res<TextureMap>
) {
    if let Some(col) = ev_load.0.pop_back() {
        for (cy, chunk) in blocs.cols.get(&col).unwrap().chunks.iter().enumerate().rev() {
            if chunk.is_some() {
                let chunk_pos = ChunkPos {x: col.x, y: cy as i32, z: col.z, realm: col.realm};
                let ent = commands.spawn(PbrBundle {
                    mesh: meshes.add(blocs.fast_mesh(chunk_pos, &texture_map)),
                    material: materials.add(Color::rgb(0.7, 0.3, 0.7).into()),
                    transform: Transform::from_translation(
                        Vec3::new(col.x as f32, cy as f32, col.z as f32) * CHUNK_S1 as f32,
                    ),
                    ..Default::default()
                }).id();
                col_ents.0.insert(chunk_pos, ent);
            }
        }
        // activate change tracking for the col so that we can update the mesh when data changes
        blocs.track(col);
        println!("Loaded ({:?})", col);
    }
}

pub fn on_col_unload(
    mut commands: Commands,
    mut ev_unload: EventReader<ColUnloadEvent>,
    mut chunk_ents: ResMut<ChunkEntities>,
) {
    for col_ev in ev_unload.iter() {
        for i in 0..(MAX_HEIGHT / CHUNK_S1) {
            if let Some(ent) = chunk_ents.0.remove(&ChunkPos {
                x: col_ev.0.x,
                y: i as i32,
                z: col_ev.0.z,
                realm: col_ev.0.realm
            }) {
                commands.entity(ent).despawn();
            }
        }
    }
}

pub fn process_bloc_changes(
    mut blocs: ResMut<Blocs>, 
    mesh_query: Query<&Handle<Mesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_ents: Res<ChunkEntities>
) {

}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Reflect)]
enum CameraMovement {
    Pan,
}


#[derive(Resource)]
pub struct ChunkEntities(pub HashMap::<ChunkPos, Entity>);

impl ChunkEntities {
    pub fn new() -> Self {
        ChunkEntities(HashMap::new())
    }
}

pub struct Draw3d;

impl Plugin for Draw3d {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(InputManagerPlugin::<CameraMovement>::default())
            .add_plugins(TextureArrayPlugin)
            .insert_resource(ChunkEntities::new())
            .add_systems(Startup, setup)
            .add_systems(Update, translate_cam)
            .add_systems(Update, pan_camera)
            .add_systems(Update, on_col_load)
            .add_systems(Update, on_col_unload);
    }
}
