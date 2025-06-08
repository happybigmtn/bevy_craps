use bevy::color::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(ThrowPower::default())
        .add_systems(Startup, setup_system)
        .add_systems(Update, (camera_control_system, throw_system))
        .run();
}

#[derive(Component)]
struct PlayerCamera {
    yaw: f32,
    pitch: f32,
}

#[derive(Component)]
struct PowerMeterFill;

#[derive(Component)]
struct Dice;

#[derive(Component)]
struct DiceId(u8);

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient: ResMut<AmbientLight>,
) {
    commands.spawn((
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            fov: 35.0_f32.to_radians(),
            near: 0.1,
            far: 100.0,
            aspect_ratio: 16.0 / 9.0,
        }),
        Transform::from_xyz(-10.0, 6.5, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
        PlayerCamera {
            yaw: std::f32::consts::FRAC_PI_4,
            pitch: -0.2,
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

    ambient.color = Color::WHITE;
    ambient.brightness = 0.7;

    let table_size_x = 8.0;
    let table_size_z = 4.0;
    let table_mesh = meshes.add(Plane3d::default().mesh().size(table_size_x, table_size_z));
    let table_material = materials.add(StandardMaterial {
        base_color: Srgba::hex("#0B0B0B").unwrap().into(),
        metallic: 0.1,
        perceptual_roughness: 0.0,
        ..default()
    });

    commands
        .spawn((
            Mesh3d(table_mesh),
            MeshMaterial3d(table_material),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ))
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(
            table_size_x / 2.0,
            0.05,
            table_size_z / 2.0,
        ))
        .insert(Restitution::coefficient(0.8));

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

    let black_brushed = materials.add(StandardMaterial {
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

    // Power Meter
    let meter_width = 200.0;
    let meter_height = 20.0;
    commands
        .spawn((
            Node {
                width: Val::Px(meter_width),
                height: Val::Px(meter_height),
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                bottom: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(0.0),
                    height: Val::Px(meter_height),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.0, 0.8, 0.0)),
                PowerMeterFill,
            ));
        });
}

fn camera_control_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut cam_q: Query<(&mut PlayerCamera, &mut Transform)>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut window_q: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !mouse_buttons.pressed(MouseButton::Right) {
        return;
    }

    let mut delta = Vec2::ZERO;
    for ev in mouse_motion_events.read() {
        delta += ev.delta;
    }
    if delta == Vec2::ZERO {
        return;
    }

    let Ok((mut cam, mut transform)) = cam_q.single_mut() else {
        return;
    };
    const SENS: f32 = 0.004;
    cam.yaw -= delta.x * SENS;
    cam.pitch = (cam.pitch - delta.y * SENS).clamp(-1.54, 1.54);

    transform.rotation = Quat::from_rotation_y(cam.yaw) * Quat::from_rotation_x(cam.pitch);

    if let Ok(mut window) = window_q.single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

#[derive(Resource)]
struct ThrowPower {
    current: f32,
    max: f32,
    charging: bool,
}

impl Default for ThrowPower {
    fn default() -> Self {
        Self {
            current: 0.0,
            max: 15.0,
            charging: false,
        }
    }
}

fn throw_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut power_res: ResMut<ThrowPower>,
    time: Res<Time>,
    cam_q: Query<&Transform, With<PlayerCamera>>,
    mut fill_query: Query<&mut Node, With<PowerMeterFill>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    if keys.just_pressed(KeyCode::Space) {
        power_res.current = 0.0;
        power_res.charging = true;
    }
    if keys.pressed(KeyCode::Space) && power_res.charging {
        power_res.current += 30.0 * time.delta_secs();
        if power_res.current > power_res.max {
            power_res.current = power_res.max;
        }
        if let Ok(mut fill_node) = fill_query.single_mut() {
            let percent = power_res.current / power_res.max;
            fill_node.width = Val::Px(percent * 200.0);
        }
    }
    if keys.just_released(KeyCode::Space) && power_res.charging {
        power_res.charging = false;
        let &cam_transform = cam_q.single().unwrap();
        let cam_forward = cam_transform.forward();

        let forward_flat = Vec3::new(cam_forward.x, 0.0, cam_forward.z).normalize();
        let mut throw_origin = cam_transform.translation + forward_flat * 1.0;
        throw_origin.y = 1.2; // 20 cm cube => centre 1.2 m keeps bottom clear
        let right_vec = forward_flat.cross(Vec3::Y).normalize();

        // --- NEW: clamp inside table box (8×4) so it never spawns past a wall ---
        let half_x = 4.0; // table_size_x / 2.0
        let half_z = 2.0; // table_size_z / 2.0
        let margin = 0.3; // stay 30 cm from any wall
        throw_origin.x = throw_origin.x.clamp(-half_x + margin, half_x - margin);
        throw_origin.z = throw_origin.z.clamp(-half_z + margin, half_z - margin);

        let horizontal_power = power_res.current.min(12.0);
        let impulse_main = forward_flat * horizontal_power;
        let impulse_spin = Vec3::new(0.0, 0.3, 0.0);

        commands
            .spawn((
                RigidBody::Dynamic,
                Collider::cuboid(0.2, 0.2, 0.2),
                Restitution::coefficient(0.25),
                Friction::coefficient(0.25),
                Damping {
                    linear_damping: 1.0,
                    angular_damping: 2.0,
                },
                Ccd::enabled(),
                Mesh3d(meshes.add(Cuboid::default())),
                Transform::from_translation(throw_origin + right_vec * 0.25),
                Dice,
                DiceId(1),
                Name::new("Dice1"),
                // Velocity::linear(forward_flat * power_res.current),
            ))
            .insert(ExternalImpulse {
                impulse: impulse_main + impulse_spin,
                torque_impulse: Vec3::new(2.0, 7.0, 0.0),
            });

        commands
            .spawn((
                RigidBody::Dynamic,
                Collider::cuboid(0.2, 0.2, 0.2),
                Restitution::coefficient(0.25),
                Friction::coefficient(0.8),
                Damping {
                    linear_damping: 1.0,
                    angular_damping: 2.0,
                },
                Ccd::enabled(),
                Mesh3d(meshes.add(Cuboid::default())),
                Transform::from_translation(throw_origin - right_vec * 0.25),
                Dice,
                DiceId(2),
                Name::new("Dice2"),
                // Velocity::linear(forward_flat * power_res.current - right_vec * 1.5),
            ))
            .insert(ExternalImpulse {
                impulse: impulse_main - right_vec * 1.2 + impulse_spin,
                torque_impulse: Vec3::new(-2.0, 6.0, 0.0),
            });

        power_res.current = 0.0;
        if let Ok(mut fill_node) = fill_query.single_mut() {
            fill_node.width = Val::Px(0.0);
        }
    }
}
