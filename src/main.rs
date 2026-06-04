use bevy::prelude::*;
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

#[derive(Debug, Clone)]
struct LSystem {
    axiom: String,
    rules: HashMap<char, String>,
}

impl LSystem {
    fn new(axiom: impl Into<String>) -> Self {
        Self {
            axiom: axiom.into(),
            rules: HashMap::new(),
        }
    }

    fn add_rule(mut self, symbol: char, replacement: impl Into<String>) -> Self {
        self.rules.insert(symbol, replacement.into());
        self
    }

    fn generate(&self, iterations: usize) -> String {
        let mut current = self.axiom.clone();

        for _ in 0..iterations {
            let mut next = String::new();

            for c in current.chars() {
                if let Some(rule) = self.rules.get(&c) {
                    next.push_str(rule);
                } else {
                    next.push(c);
                }
            }

            current = next;
        }

        current
    }
}

#[derive(Clone, Copy)]
struct TurtleState {
    position: Vec3,
    rotation: Quat,
    depth: u32,
}

#[derive(Clone)]
struct Segment {
    start: Vec3,
    end: Vec3,
    depth: u32,
}

fn interpret(commands: &str, step: f32, angle_deg: f32) -> Vec<Segment> {
    let mut state = TurtleState {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        depth: 0,
    };

    let mut stack = Vec::<TurtleState>::new();
    let mut segments = Vec::<Segment>::new();

    let angle = angle_deg.to_radians();

    for c in commands.chars() {
        match c {
            'F' => {
                let dir = state.rotation * Vec3::Y;
                let next = state.position + dir * step;

                segments.push(Segment {
                    start: state.position,
                    end: next,
                    depth: state.depth,
                });

                state.position = next;
            }

            '+' => {
                state.rotation *= Quat::from_rotation_z(angle);
            }

            '-' => {
                state.rotation *= Quat::from_rotation_z(-angle);
            }

            '&' => {
                state.rotation *= Quat::from_rotation_x(angle);
            }

            '^' => {
                state.rotation *= Quat::from_rotation_x(-angle);
            }

            '\\' => {
                state.rotation *= Quat::from_rotation_y(angle);
            }

            '/' => {
                state.rotation *= Quat::from_rotation_y(-angle);
            }

            '[' => {
                stack.push(state);
                state.depth += 1;
            }

            ']' => {
                if let Some(saved) = stack.pop() {
                    state = saved;
                }
            }

            _ => {}
        }
    }

    segments
}

fn spawn_branch(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    start: Vec3,
    end: Vec3,
    radius: f32,
) {
    let direction = end - start;
    let length = direction.length();

    if length <= 0.0001 {
        return;
    }

    let midpoint = (start + end) * 0.5;

    let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());

    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(radius, length))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.35, 0.22, 0.12),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform {
            translation: midpoint,
            rotation,
            ..default()
        },
    ));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(GlobalAmbientLight {
        brightness: 500.0,
        ..default()
    });

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-15.0, 10.0, 15.0).looking_at(Vec3::new(0.0, 6.0, 0.0), Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 80000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, -0.8, 0.0)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.35, 0.2),
            perceptual_roughness: 1.0,
            ..default()
        })),
    ));

    let lsystem = LSystem::new("F").add_rule('F', "FF-[-F+F+F]+[+F-F-F]");

    let generated = lsystem.generate(5);

    let segments = interpret(&generated, 0.5, 22.5);

    let max_depth = segments.iter().map(|s| s.depth).max().unwrap_or(1);

    for segment in &segments {
        let t = segment.depth as f32 / max_depth as f32;

        let radius = 0.15 * (1.0 - t).powf(1.8) + 0.01;

        spawn_branch(
            &mut commands,
            &mut meshes,
            &mut materials,
            segment.start,
            segment.end,
            radius,
        );

        if segment.depth > max_depth.saturating_sub(2) {
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.08))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.15, 0.55, 0.18),
                    ..default()
                })),
                Transform::from_translation(segment.end),
            ));
        }
    }
}
