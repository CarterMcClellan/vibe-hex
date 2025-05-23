use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

const HEX_RADIUS: f32 = 40.0;
const CHUNK_SIZE: i32 = 7;
const VIEW_DISTANCE: i32 = 2;

#[derive(Component)]
struct HexTile {
    q: i32,
    r: i32,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct PlayerMovement {
    target_position: Vec3,
    start_position: Vec3,
    move_timer: f32,
    move_duration: f32,
    is_moving: bool,
}

#[derive(Component)]
struct Chunk {
    chunk_q: i32,
    chunk_r: i32,
}

#[derive(Component)]
struct ChunkDisplay;

#[derive(Resource)]
struct PlayerPosition {
    q: i32,
    r: i32,
}

#[derive(Resource, Default)]
struct LoadedChunks {
    chunks: std::collections::HashSet<(i32, i32)>,
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
        .insert_resource(LoadedChunks::default())
        .add_systems(Startup, (setup_camera, spawn_player, load_initial_chunks, setup_ui))
        .add_systems(Update, (handle_input, animate_player_movement, update_camera, manage_chunks, update_chunk_display))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn load_initial_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {
    let hex_mesh = create_perfect_hexagon();
    let smaller_hex_mesh = create_smaller_hexagon();
    let mesh_handle = meshes.add(hex_mesh);
    let smaller_mesh_handle = meshes.add(smaller_hex_mesh);
    
    let grass_texture = asset_server.load("grass_texture.png");
    let grass_material = materials.add(ColorMaterial::from(grass_texture));
    let border_material = materials.add(ColorMaterial::from(Color::BLACK));
    
    // Load chunks around player starting position
    for chunk_q in -VIEW_DISTANCE..=VIEW_DISTANCE {
        for chunk_r in -VIEW_DISTANCE..=VIEW_DISTANCE {
            if chunk_q.abs() <= VIEW_DISTANCE && chunk_r.abs() <= VIEW_DISTANCE && (chunk_q + chunk_r).abs() <= VIEW_DISTANCE {
                load_chunk(
                    &mut commands,
                    chunk_q,
                    chunk_r,
                    &mesh_handle,
                    &smaller_mesh_handle,
                    &grass_material,
                    &border_material,
                );
                loaded_chunks.chunks.insert((chunk_q, chunk_r));
            }
        }
    }
}

fn load_chunk(
    commands: &mut Commands,
    chunk_q: i32,
    chunk_r: i32,
    mesh_handle: &Handle<Mesh>,
    smaller_mesh_handle: &Handle<Mesh>,
    grass_material: &Handle<ColorMaterial>,
    border_material: &Handle<ColorMaterial>,
) {
    let chunk_offset_q = chunk_q * CHUNK_SIZE;
    let chunk_offset_r = chunk_r * CHUNK_SIZE;
    
    for local_q in 0..CHUNK_SIZE {
        for local_r in 0..CHUNK_SIZE {
            let q = chunk_offset_q + local_q;
            let r = chunk_offset_r + local_r;
            let (x, y) = hex_to_world(q, r);
            
            // Spawn black background hex (full size)
            commands.spawn((
                Mesh2d(mesh_handle.clone()),
                MeshMaterial2d(border_material.clone()),
                Transform::from_translation(Vec3::new(x, y, -0.1)),
                Chunk { chunk_q, chunk_r },
            ));
            
            // Spawn smaller grass-textured hex on top
            commands.spawn((
                Mesh2d(smaller_mesh_handle.clone()),
                MeshMaterial2d(grass_material.clone()),
                Transform::from_translation(Vec3::new(x, y, 0.0)),
                HexTile { q, r },
                Chunk { chunk_q, chunk_r },
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

fn create_smaller_hexagon() -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    let border_width = 2.0;
    let smaller_radius = HEX_RADIUS - border_width;
    
    // Center point
    vertices.push([0.0, 0.0, 0.0]);
    
    // Six vertices of the smaller hexagon (flat-top orientation)
    for i in 0..6 {
        let angle = (i as f32) * std::f32::consts::PI / 3.0;
        let x = smaller_radius * angle.cos();
        let y = smaller_radius * angle.sin();
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
    let position = Vec3::new(x, y, 1.0);
    
    commands.spawn((
        Sprite {
            image: asset_server.load("character_sprite.png"),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
        Transform::from_translation(position),
        Player,
        PlayerMovement {
            target_position: position,
            start_position: position,
            move_timer: 0.0,
            move_duration: 0.3,
            is_moving: false,
        },
    ));
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_pos: ResMut<PlayerPosition>,
    mut player_query: Query<(&mut PlayerMovement, &mut Sprite), With<Player>>,
) {
    if let Ok((mut movement, mut sprite)) = player_query.get_single_mut() {
        // Don't handle input if already moving
        if movement.is_moving {
            return;
        }
        
        let mut new_q = player_pos.q;
        let mut new_r = player_pos.r;
        let mut moved = false;
        
        if keyboard_input.just_pressed(KeyCode::KeyW) {
            new_r += 1;
            moved = true;
        } else if keyboard_input.just_pressed(KeyCode::KeyS) {
            new_r -= 1;
            moved = true;
        } else if keyboard_input.just_pressed(KeyCode::KeyA) {
            new_q -= 1;
            moved = true;
            // Face left (default sprite direction)
            sprite.flip_x = false;
        } else if keyboard_input.just_pressed(KeyCode::KeyD) {
            new_q += 1;
            moved = true;
            // Face right (flip sprite)
            sprite.flip_x = true;
        } else if keyboard_input.just_pressed(KeyCode::KeyQ) {
            new_q -= 1;
            new_r += 1;
            moved = true;
            // Face left for diagonal left movement
            sprite.flip_x = false;
        } else if keyboard_input.just_pressed(KeyCode::KeyE) {
            new_q += 1;
            new_r -= 1;
            moved = true;
            // Face right for diagonal right movement
            sprite.flip_x = true;
        }
        
        if moved && is_valid_hex(new_q, new_r) && (new_q != player_pos.q || new_r != player_pos.r) {
            player_pos.q = new_q;
            player_pos.r = new_r;
            
            let (x, y) = hex_to_world(new_q, new_r);
            let target_position = Vec3::new(x, y, 1.0);
            
            // Start movement animation
            movement.start_position = movement.target_position;
            movement.target_position = target_position;
            movement.move_timer = 0.0;
            movement.is_moving = true;
        }
    }
}

fn is_valid_hex(_q: i32, _r: i32) -> bool {
    // Remove board size restriction for infinite world
    true
}

fn animate_player_movement(
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &mut PlayerMovement), With<Player>>,
) {
    if let Ok((mut transform, mut movement)) = player_query.get_single_mut() {
        if movement.is_moving {
            movement.move_timer += time.delta_secs();
            
            if movement.move_timer >= movement.move_duration {
                // Movement complete
                transform.translation = movement.target_position;
                movement.is_moving = false;
                movement.move_timer = 0.0;
            } else {
                // Interpolate position with smooth easing
                let t = movement.move_timer / movement.move_duration;
                // Use smoothstep for nice easing
                let smooth_t = t * t * (3.0 - 2.0 * t);
                
                transform.translation = movement.start_position.lerp(movement.target_position, smooth_t);
            }
        }
    }
}

fn update_camera(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            camera_transform.translation = player_transform.translation;
        }
    }
}

fn manage_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    player_pos: Res<PlayerPosition>,
    mut loaded_chunks: ResMut<LoadedChunks>,
    chunk_query: Query<(Entity, &Chunk)>,
) {
    let player_chunk_q = if player_pos.q >= 0 { player_pos.q / CHUNK_SIZE } else { (player_pos.q - CHUNK_SIZE + 1) / CHUNK_SIZE };
    let player_chunk_r = if player_pos.r >= 0 { player_pos.r / CHUNK_SIZE } else { (player_pos.r - CHUNK_SIZE + 1) / CHUNK_SIZE };
    
    // Determine which chunks should be loaded
    let mut required_chunks = std::collections::HashSet::new();
    for chunk_q in (player_chunk_q - VIEW_DISTANCE)..=(player_chunk_q + VIEW_DISTANCE) {
        for chunk_r in (player_chunk_r - VIEW_DISTANCE)..=(player_chunk_r + VIEW_DISTANCE) {
            if (chunk_q - player_chunk_q).abs() <= VIEW_DISTANCE && 
               (chunk_r - player_chunk_r).abs() <= VIEW_DISTANCE &&
               ((chunk_q - player_chunk_q) + (chunk_r - player_chunk_r)).abs() <= VIEW_DISTANCE {
                required_chunks.insert((chunk_q, chunk_r));
            }
        }
    }
    
    // Unload chunks that are too far away
    let chunks_to_unload: Vec<(i32, i32)> = loaded_chunks.chunks
        .iter()
        .filter(|&&chunk| !required_chunks.contains(&chunk))
        .copied()
        .collect();
    
    for (chunk_q, chunk_r) in chunks_to_unload {
        // Remove all entities belonging to this chunk
        for (entity, chunk) in chunk_query.iter() {
            if chunk.chunk_q == chunk_q && chunk.chunk_r == chunk_r {
                commands.entity(entity).despawn();
            }
        }
        loaded_chunks.chunks.remove(&(chunk_q, chunk_r));
    }
    
    // Load new chunks
    let hex_mesh = create_perfect_hexagon();
    let smaller_hex_mesh = create_smaller_hexagon();
    let mesh_handle = meshes.add(hex_mesh);
    let smaller_mesh_handle = meshes.add(smaller_hex_mesh);
    
    let grass_texture = asset_server.load("grass_texture.png");
    let grass_material = materials.add(ColorMaterial::from(grass_texture));
    let border_material = materials.add(ColorMaterial::from(Color::BLACK));
    
    for (chunk_q, chunk_r) in required_chunks {
        if !loaded_chunks.chunks.contains(&(chunk_q, chunk_r)) {
            load_chunk(
                &mut commands,
                chunk_q,
                chunk_r,
                &mesh_handle,
                &smaller_mesh_handle,
                &grass_material,
                &border_material,
            );
            loaded_chunks.chunks.insert((chunk_q, chunk_r));
        }
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        Text::new("Chunk: (0, 0)"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        ChunkDisplay,
    ));
}

fn update_chunk_display(
    player_pos: Res<PlayerPosition>,
    mut chunk_display_query: Query<&mut Text, With<ChunkDisplay>>,
) {
    if player_pos.is_changed() {
        let player_chunk_q = if player_pos.q >= 0 { player_pos.q / CHUNK_SIZE } else { (player_pos.q - CHUNK_SIZE + 1) / CHUNK_SIZE };
        let player_chunk_r = if player_pos.r >= 0 { player_pos.r / CHUNK_SIZE } else { (player_pos.r - CHUNK_SIZE + 1) / CHUNK_SIZE };
        
        if let Ok(mut text) = chunk_display_query.get_single_mut() {
            **text = format!("Chunk: ({}, {})", player_chunk_q, player_chunk_r);
        }
    }
}