use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

const HEX_RADIUS: f32 = 40.0;
const BOARD_SIZE: i32 = 3;

#[derive(Component)]
struct HexTile {
    q: i32,
    r: i32,
}

#[derive(Component)]
struct Player;

#[derive(Resource)]
struct PlayerPosition {
    q: i32,
    r: i32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Catan Hex Board".into(),
                resolution: (1200.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(PlayerPosition { q: 0, r: 0 })
        .add_systems(Startup, (setup_camera, setup_hex_board, spawn_player))
        .add_systems(Update, handle_input)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_hex_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let hex_mesh = create_perfect_hexagon();
    let mesh_handle = meshes.add(hex_mesh);
    
    for q in -BOARD_SIZE..=BOARD_SIZE {
        let r1 = (-BOARD_SIZE).max(-q - BOARD_SIZE);
        let r2 = BOARD_SIZE.min(-q + BOARD_SIZE);
        
        for r in r1..=r2 {
            let (x, y) = hex_to_world(q, r);
            
            let colors = [
                Color::srgb(0.4, 0.8, 0.4), // Forest green
                Color::srgb(0.9, 0.7, 0.3), // Wheat yellow
                Color::srgb(0.7, 0.4, 0.2), // Brown hills
                Color::srgb(0.6, 0.6, 0.6), // Stone gray
                Color::srgb(0.3, 0.5, 0.8), // Water blue
                Color::srgb(0.8, 0.3, 0.3), // Red clay
            ];
            
            let color_index = ((q * 7 + r * 11).abs() as usize) % colors.len();
            let color = colors[color_index];
            
            commands.spawn((
                Mesh2d(mesh_handle.clone()),
                MeshMaterial2d(materials.add(ColorMaterial::from(color))),
                Transform::from_translation(Vec3::new(x, y, 0.0)),
                HexTile { q, r },
            ));
        }
    }
}

fn create_perfect_hexagon() -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    // Center point
    vertices.push([0.0, 0.0, 0.0]);
    
    // Six vertices of the hexagon (flat-top orientation)
    for i in 0..6 {
        let angle = (i as f32) * std::f32::consts::PI / 3.0;
        let x = HEX_RADIUS * angle.cos();
        let y = HEX_RADIUS * angle.sin();
        vertices.push([x, y, 0.0]);
    }
    
    // Create triangular faces from center to each edge
    for i in 0..6 {
        let current = i + 1;
        let next = if i == 5 { 1 } else { i + 2 };
        indices.extend_from_slice(&[0, current as u32, next as u32]);
    }
    
    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_indices(Indices::U32(indices))
}

fn hex_to_world(q: i32, r: i32) -> (f32, f32) {
    // For flat-top hexagons, correct tessellation formulas:
    let x = HEX_RADIUS * (3.0 / 2.0 * q as f32);
    let y = HEX_RADIUS * (3.0_f32.sqrt() / 2.0 * q as f32 + 3.0_f32.sqrt() * r as f32);
    (x, y)
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let (x, y) = hex_to_world(0, 0);
    
    commands.spawn((
        Sprite {
            image: asset_server.load("character_sprite.png"),
            custom_size: Some(Vec2::new(30.0, 30.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(x, y, 1.0)),
        Player,
    ));
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_pos: ResMut<PlayerPosition>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let mut new_q = player_pos.q;
    let mut new_r = player_pos.r;
    
    if keyboard_input.just_pressed(KeyCode::KeyW) {
        new_r += 1;
    } else if keyboard_input.just_pressed(KeyCode::KeyS) {
        new_r -= 1;
    } else if keyboard_input.just_pressed(KeyCode::KeyA) {
        new_q -= 1;
    } else if keyboard_input.just_pressed(KeyCode::KeyD) {
        new_q += 1;
    } else if keyboard_input.just_pressed(KeyCode::KeyQ) {
        new_q -= 1;
        new_r += 1;
    } else if keyboard_input.just_pressed(KeyCode::KeyE) {
        new_q += 1;
        new_r -= 1;
    }
    
    if is_valid_hex(new_q, new_r) && (new_q != player_pos.q || new_r != player_pos.r) {
        player_pos.q = new_q;
        player_pos.r = new_r;
        
        let (x, y) = hex_to_world(new_q, new_r);
        
        if let Ok(mut transform) = player_query.get_single_mut() {
            transform.translation = Vec3::new(x, y, 1.0);
        }
    }
}

fn is_valid_hex(q: i32, r: i32) -> bool {
    let s = -q - r;
    q.abs() <= BOARD_SIZE && r.abs() <= BOARD_SIZE && s.abs() <= BOARD_SIZE
}