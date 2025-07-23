// Think of 'use' statements like bringing tools from your garage into your workshop.
// Instead of walking back to get each tool, you bring them all at once.
use bevy::color::prelude::*; // Color tools - for painting our 3D objects
use bevy::input::ButtonInput; // Keyboard/mouse detection - like sensors that tell us when buttons are pressed
use bevy::input::mouse::MouseMotion; // Mouse movement tracking - measures how far the mouse moved
use bevy::prelude::*; // The main Bevy toolkit - cameras, meshes, transforms, etc.
use bevy::window::{CursorGrabMode, PrimaryWindow}; // Window control - for hiding/locking the mouse cursor
use bevy_rapier3d::prelude::*; // Physics engine - makes things fall, bounce, and collide realistically

// The main function is like the conductor of an orchestra - it organizes all the parts
// but doesn't play any instruments itself.
fn main() {
    App::new() // Create a new Bevy application - like opening a new blank 3D canvas
        .add_plugins(DefaultPlugins) // Add Bevy's standard features: rendering, input, audio, etc.
        // Like installing a game engine's basic components
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default()) // Add physics simulation
        // The ::<NoUserData> is a "type parameter" - we're saying "we don't need
        // to attach custom data to physics objects"
        .add_plugins(RapierDebugRenderPlugin::default()) // Shows physics collision boxes as wireframes
        // Helpful for debugging - like X-ray vision
        .insert_resource(ThrowPower::default()) // Add a shared "power meter" that all systems can access
        // Resources are like global variables but safer
        .add_systems(Startup, setup_system) // Run setup_system once when the app starts
        // Like setting up the game board before playing
        .add_systems(Update, (camera_control_system, throw_system)) // Run these every frame
        // The parentheses group multiple systems to run in parallel
        // Like having multiple workers doing different jobs simultaneously
        .run(); // Start the game loop - this keeps running until you close the window
}

// #[derive(Component)] is like putting a special sticker on our struct that says
// "this can be attached to entities in the game world"
// Without this sticker, Bevy wouldn't know this struct is meant to be a component
#[derive(Component)]
struct PlayerCamera {
    yaw: f32, // Horizontal rotation (left/right) - like turning your head side to side
    // f32 means "32-bit floating point number" - decimals like 3.14
    pitch: f32, // Vertical rotation (up/down) - like nodding your head
                // We use radians, where 2π radians = 360 degrees
}

// A component with no data - just a "tag" to mark entities
// Like putting a name tag on something without writing anything on it
#[derive(Component)]
struct PowerMeterFill; // Marks which UI element shows the power level

#[derive(Component)]
struct Dice; // Tags an entity as being a die - helps us find all dice later

// This component stores data - the number in parentheses
// It's called a "tuple struct" - like a struct with unnamed fields
#[derive(Component)]
struct DiceId(u8); // u8 = unsigned 8-bit integer (0-255)
// Identifies which die is which (die #1, die #2, etc.)

// This function sets up our game world - like arranging furniture in a room
// The parameters are "resources" we can use to create things:
fn setup_system(
    mut commands: Commands, // The "commands" let us spawn entities (things in the world)
    // 'mut' means we can modify it (mutable)
    mut meshes: ResMut<Assets<Mesh>>, // Storage for 3D shapes (cube, plane, etc.)
    // ResMut = Resource Mutable - we can add new meshes
    mut materials: ResMut<Assets<StandardMaterial>>, // Storage for surface properties (color, shine)
    mut ambient: ResMut<AmbientLight>,               // Controls the general lighting in the scene
) {
    // Spawn a camera - this is our "eyes" in the 3D world
    commands.spawn((
        // spawn() creates a new entity, the double parentheses group components
        Camera3d::default(), // A standard 3D camera
        Projection::from(PerspectiveProjection {
            fov: 35.0_f32.to_radians(), // Field of view - 35 degrees (narrow = zoom in)
            // The _f32 suffix ensures it's a 32-bit float
            near: 0.1,  // Closest distance we can see (anything closer is invisible)
            far: 100.0, // Farthest distance we can see (anything further is invisible)
            aspect_ratio: 16.0 / 9.0, // Width/height ratio - matches most monitors
        }),
        Transform::from_xyz(-10.0, 6.5, -5.0) // Position: 10 units left, 6.5 up, 5 back
            .looking_at(Vec3::ZERO, Vec3::Y), // Point camera at origin (0,0,0)
        // Vec3::Y means "up" is the Y direction
        PlayerCamera {
            yaw: std::f32::consts::FRAC_PI_4, // Start rotated 45 degrees (π/4 radians)
            pitch: -0.2,                      // Slightly tilted down
        },
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 30000.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.92, 0.85),
            ..default()
        },
        Transform {
            translation: Vec3::new(-1.0, 5.0, -4.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
    ));

    // Set up ambient lighting - like turning on soft overhead lights
    ambient.color = Color::WHITE; // White light (no color tint)
    ambient.brightness = 0.7; // 70% brightness - not too harsh

    // Define the craps table dimensions
    let table_size_x = 8.0; // Table width (left-right)
    let table_size_z = 4.0; // Table depth (front-back)

    // Create the table surface mesh (3D shape)
    let table_mesh = meshes.add(
        // add() stores the mesh and returns a handle to it
        Plane3d::default() // A flat plane facing upward
            .mesh() // Convert to mesh data
            .size(table_size_x, table_size_z), // Set the size
    );

    // Create the table material (how it looks)
    let table_material = materials.add(StandardMaterial {
        base_color: Srgba::hex("#0B0B0B").unwrap().into(), // Very dark gray (almost black)
        // hex() converts color code, unwrap() handles errors, into() converts type
        metallic: 0.1,             // Slightly metallic (10%)
        perceptual_roughness: 0.0, // Very smooth (0 = mirror-like)
        ..default() // Use defaults for other properties (the .. is "struct update syntax")
    });

    // Create the table entity with visual and physics components
    commands
        .spawn((
            // First spawn with visual components
            Mesh3d(table_mesh),             // The 3D shape to render
            MeshMaterial3d(table_material), // How to render it (color, shine, etc.)
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)), // Position at origin
        ))
        // Chain .insert() calls to add physics components
        .insert(RigidBody::Fixed) // Fixed = doesn't move (unlike Dynamic which falls)
        .insert(Collider::cuboid(
            // Invisible box for physics collisions
            table_size_x / 2.0, // Half-width (cuboid uses half-extents)
            0.05,               // Very thin (5cm thick)
            table_size_z / 2.0, // Half-depth
        ))
        .insert(Restitution::coefficient(0.1)) // Bounciness: 0.1 = 10% energy retained
        .insert(Friction::coefficient(0.8)); // Friction: 0.8 = pretty grippy

    let wall_thickness = 0.2;
    let wall_height = 1.0;
    let half_x = table_size_x / 2.0;
    let half_z = table_size_z / 2.0;
    let long_wall = meshes.add(Cuboid::new(
        wall_thickness,
        wall_height,
        table_size_z + wall_thickness * 2.0,
    ));
    let short_wall = meshes.add(Cuboid::new(
        table_size_x + wall_thickness * 2.0,
        wall_height,
        wall_thickness,
    ));

    let _black_brushed = materials.add(StandardMaterial {
        base_color: Srgba::hex("#141414").unwrap().into(),
        metallic: 1.0,
        perceptual_roughness: 0.25,
        ..default()
    });
    let papaya_orange = materials.add(StandardMaterial {
        base_color: Srgba::hex("#FF5300").unwrap().into(),
        metallic: 1.0,
        perceptual_roughness: 0.35,
        ..default()
    });

    // long sides
    commands
        .spawn((
            Mesh3d(long_wall.clone()),
            MeshMaterial3d(papaya_orange.clone()),
            Transform::from_xyz(-half_x - wall_thickness / 2.0, wall_height / 2.0, 0.0),
        ))
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(
            wall_thickness / 2.0,
            wall_height / 2.0,
            (table_size_z + wall_thickness * 2.0) / 2.0,
        ))
        .insert(Restitution::coefficient(0.08));

    commands
        .spawn((
            Mesh3d(long_wall.clone()),
            MeshMaterial3d(papaya_orange.clone()),
            Transform::from_xyz(half_x + wall_thickness / 2.0, wall_height / 2.0, 0.0),
        ))
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(
            wall_thickness / 2.0,
            wall_height / 2.0,
            (table_size_z + wall_thickness * 2.0) / 2.0,
        ))
        .insert(Restitution::coefficient(0.08));

    // short sides
    commands
        .spawn((
            Mesh3d(short_wall.clone()),
            MeshMaterial3d(papaya_orange.clone()),
            Transform::from_xyz(0.0, wall_height / 2.0, half_z + wall_thickness / 2.0),
        ))
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(
            (table_size_x + wall_thickness * 2.0) / 2.0,
            wall_height / 2.0,
            wall_thickness / 2.0,
        ))
        .insert(Restitution::coefficient(0.08));

    commands
        .spawn((
            Mesh3d(short_wall.clone()),
            MeshMaterial3d(papaya_orange.clone()),
            Transform::from_xyz(0.0, wall_height / 2.0, -half_z - wall_thickness / 2.0),
        ))
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(
            (table_size_x + wall_thickness * 2.0) / 2.0,
            wall_height / 2.0,
            wall_thickness / 2.0,
        ))
        .insert(Restitution::coefficient(0.08));

    // Power Meter UI - shows how hard you're throwing
    let meter_width = 200.0; // Width in pixels
    let meter_height = 20.0; // Height in pixels

    commands
        .spawn((
            // Create the meter background (dark gray bar)
            Node {
                // Node is Bevy's UI building block
                width: Val::Px(meter_width), // Val::Px = value in pixels
                height: Val::Px(meter_height),
                position_type: PositionType::Absolute, // Position relative to screen edges
                left: Val::Px(20.0),                   // 20 pixels from left edge
                bottom: Val::Px(20.0),                 // 20 pixels from bottom edge
                ..default()                            // Other properties use defaults
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)), // Dark gray background
        ))
        .with_children(|parent| {
            // Add child nodes inside this one
            parent.spawn((
                // The green fill bar that grows
                Node {
                    width: Val::Px(0.0),           // Starts at 0 width (empty)
                    height: Val::Px(meter_height), // Same height as parent
                    ..default()
                },
                BackgroundColor(Color::srgb(0.0, 0.8, 0.0)), // Bright green
                PowerMeterFill, // Tag so we can find and update it later
            ));
        });
}

// System to control camera rotation with mouse (like a first-person game)
// Systems are functions that run every frame to update the game
fn camera_control_system(
    mut mouse_motion_events: EventReader<MouseMotion>, // Stream of mouse movement events
    // EventReader lets us process events that happened this frame
    mut cam_q: Query<(&mut PlayerCamera, &mut Transform)>, // Find entities with both components
    // Query is like a database search - "find all things with X and Y"
    mouse_buttons: Res<ButtonInput<MouseButton>>, // Current state of mouse buttons
    // Res = Resource (shared data)
    mut window_q: Query<&mut Window, With<PrimaryWindow>>, // Find the main window
                                                           // With<T> = "must also have component T"
) {
    // Only rotate camera when right mouse button is held
    if !mouse_buttons.pressed(MouseButton::Right) {
        return; // Exit early - like a guard at a door
    }

    // Accumulate all mouse movements this frame
    let mut delta = Vec2::ZERO; // Vec2 = 2D vector (x, y)
    for ev in mouse_motion_events.read() {
        // Loop through all movement events
        delta += ev.delta; // Add up all the movements
    }
    if delta == Vec2::ZERO {
        // No movement? Nothing to do
        return;
    }

    // Get the camera entity (should only be one)
    let Ok((mut cam, mut transform)) = cam_q.single_mut() else {
        return; // If no camera found or multiple cameras, exit
    };
    // This is a "let-else" pattern - like try-catch but cleaner
    // Apply mouse movement to camera rotation
    const SENS: f32 = 0.004; // Sensitivity - how fast camera rotates
    cam.yaw -= delta.x * SENS; // Horizontal rotation (negative because mouse right = look right)
    cam.pitch = (cam.pitch - delta.y * SENS) // Vertical rotation
        .clamp(-1.54, 1.54); // Limit to ~88 degrees up/down to prevent flipping

    transform.rotation = Quat::from_rotation_y(cam.yaw) * Quat::from_rotation_x(cam.pitch);

    if let Ok(mut window) = window_q.single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

// #[derive(Resource)] marks this as shareable data across systems
// Resources are like global variables that systems can access
#[derive(Resource)]
struct ThrowPower {
    current: f32,   // Current power level (0 to max)
    max: f32,       // Maximum power allowed
    charging: bool, // Is spacebar currently held down?
}

// impl Default tells Rust how to create a ThrowPower with default values
// This is used when we call ThrowPower::default()
impl Default for ThrowPower {
    fn default() -> Self {
        // Self = ThrowPower (shorthand when inside impl)
        Self {
            current: 0.0,    // Start with no power
            max: 15.0,       // Maximum power units
            charging: false, // Not charging initially
        }
    }
}

// System that handles throwing dice when spacebar is pressed
fn throw_system(
    keys: Res<ButtonInput<KeyCode>>, // Keyboard state - which keys are pressed
    mut commands: Commands,          // For spawning new dice
    mut power_res: ResMut<ThrowPower>, // Our power meter data (ResMut = can modify)
    time: Res<Time>,                 // Game time - for frame-independent movement
    cam_q: Query<&Transform, With<PlayerCamera>>, // Find camera position/rotation
    mut fill_query: Query<&mut Node, With<PowerMeterFill>>, // Find power meter UI
    mut meshes: ResMut<Assets<Mesh>>, // For creating dice meshes
    _materials: ResMut<Assets<StandardMaterial>>, // For dice appearance
    _asset_server: Res<AssetServer>, // Not used here, but available for loading files
) {
    // Start charging when space is first pressed
    if keys.just_pressed(KeyCode::Space) {
        // just_pressed = this exact frame
        power_res.current = 0.0; // Reset power to zero
        power_res.charging = true; // Start charging up
    }
    // While holding space, increase power
    if keys.pressed(KeyCode::Space) && power_res.charging {
        // Increase power based on time (frame-independent)
        power_res.current += 30.0 * time.delta_secs(); // 30 units per second
        // delta_secs() = seconds since last frame

        // Cap at maximum power
        if power_res.current > power_res.max {
            power_res.current = power_res.max;
        }

        // Update the visual power meter
        if let Ok(mut fill_node) = fill_query.single_mut() {
            let percent = power_res.current / power_res.max; // 0.0 to 1.0
            fill_node.width = Val::Px(percent * 200.0); // Scale to meter width
        }
    }
    if keys.just_released(KeyCode::Space) && power_res.charging {
        power_res.charging = false;
        let &cam_transform = cam_q.single().unwrap();
        let cam_forward = cam_transform.forward();

        // Calculate throw direction from camera
        let forward_flat = Vec3::new(cam_forward.x, 0.0, cam_forward.z) // Remove Y component
            .normalize(); // normalize() makes length = 1 (unit vector)

        // Spawn dice 1 unit in front of camera
        let mut throw_origin = cam_transform.translation + forward_flat * 1.0;
        throw_origin.y = 0.5; // Fixed height above table

        // Calculate right vector for separating dice
        let right_vec = forward_flat.cross(Vec3::Y).normalize();
        // cross product gives perpendicular vector

        // Keep dice spawn point inside table bounds
        let half_x = 4.0; // Half of table width (8.0 / 2)
        let half_z = 2.0; // Half of table depth (4.0 / 2)
        let margin = 0.3; // Safety margin from walls (30cm)

        // clamp() limits value between min and max
        throw_origin.x = throw_origin.x.clamp(-half_x + margin, half_x - margin);
        throw_origin.z = throw_origin.z.clamp(-half_z + margin, half_z - margin);

        // Convert power meter to physics impulse
        let horizontal_power = power_res.current * 0.8; // Reasonable power scaling
        let impulse_main = forward_flat * horizontal_power; // Direction * magnitude

        // Spawn first die
        commands
            .spawn((
                // Group of components that make up a die
                RigidBody::Dynamic, // Dynamic = affected by gravity and forces
                Collider::cuboid(0.2, 0.2, 0.2), // Physics collision box (half-extents)
                Restitution::coefficient(0.15), // Bounciness (15% energy retained)
                Friction::coefficient(0.7), // How much it grips surfaces
                Damping {
                    // Slows down over time (air resistance)
                    linear_damping: 2.0,  // Slows movement
                    angular_damping: 3.0, // Slows rotation
                },
                Ccd::enabled(), // Continuous Collision Detection - prevents tunneling
                ColliderMassProperties::Density(2.0), // Higher density = heavier dice
                Mesh3d(meshes.add(Cuboid::new(0.4, 0.4, 0.4))), // Visual size (full extents)
                Transform::from_translation(throw_origin + right_vec * 0.25), // Position
                Dice,           // Tag as dice
                DiceId(1),      // First die
                Name::new("Dice1"), // Debug name
            ))
            .insert(ExternalImpulse {
                // Apply throwing force
                impulse: impulse_main,                     // Linear push
                torque_impulse: Vec3::new(0.1, 0.2, 0.05), // Reduced spin
            });

        commands
            .spawn((
                RigidBody::Dynamic,
                Collider::cuboid(0.2, 0.2, 0.2),
                Restitution::coefficient(0.15),
                Friction::coefficient(0.7),
                Damping {
                    linear_damping: 2.0,
                    angular_damping: 3.0,
                },
                Ccd::enabled(),
                ColliderMassProperties::Density(2.0), // Higher density = heavier dice
                Mesh3d(meshes.add(Cuboid::new(0.4, 0.4, 0.4))),
                Transform::from_translation(throw_origin - right_vec * 0.25),
                Dice,
                DiceId(2),
                Name::new("Dice2"),
                // Velocity::linear(forward_flat * power_res.current - right_vec * 1.5),
            ))
            .insert(ExternalImpulse {
                impulse: impulse_main - right_vec * 0.5, // Reasonable separation
                torque_impulse: Vec3::new(-0.1, 0.2, -0.05), // Reduced spin
            });

        // Reset power meter
        power_res.current = 0.0;
        if let Ok(mut fill_node) = fill_query.single_mut() {
            fill_node.width = Val::Px(0.0); // Empty the green bar
        }
    }
}
