use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use nova_gameplay::sections::prelude::*;

pub(crate) fn register_sections(mut commands: Commands, game_assets: Res<super::GameAssets>) {
    // TODO: This should be loaded from a JSON file, but for now it is fine.

    commands.insert_resource(GameSections(vec![
        SectionConfig {
            base: BaseSectionConfig {
                name: "Basic Hull Section".to_string(),
                description: "A basic hull section for spaceships.".to_string(),
                mass: 1.0,
            },
            kind: SectionKind::Hull(HullSectionConfig { render_mesh: None }),
        },
        SectionConfig {
            base: BaseSectionConfig {
                name: "Reinforced Hull Section".to_string(),
                description: "A reinforced hull section for spaceships.".to_string(),
                mass: 1.0,
            },
            kind: SectionKind::Hull(HullSectionConfig {
                render_mesh: Some(game_assets.hull_01.clone()),
            }),
        },
        SectionConfig {
            base: BaseSectionConfig {
                name: "Basic Thruster Section".to_string(),
                description: "A basic thruster section for spaceships.".to_string(),
                mass: 1.0,
            },
            kind: SectionKind::Thruster(ThrusterSectionConfig {
                magnitude: 10.0,
                render_mesh: None,
            }),
        },
        SectionConfig {
            base: BaseSectionConfig {
                name: "Basic Controller Section".to_string(),
                description: "A basic controller section for spaceships.".to_string(),
                mass: 1.0,
            },
            kind: SectionKind::Controller(ControllerSectionConfig {
                frequency: 2.0,
                damping_ratio: 2.0,
                max_torque: 1.0,
                render_mesh: None,
            }),
        },
        SectionConfig {
            base: BaseSectionConfig {
                name: "Basic Turret Section".to_string(),
                description: "A basic turret section for spaceships.".to_string(),
                mass: 1.0,
            },
            kind: SectionKind::Turret(TurretSectionConfig {
                yaw_speed: std::f32::consts::PI,   // 180 degrees per second
                pitch_speed: std::f32::consts::PI, // 180 degrees per second
                min_pitch: Some(-std::f32::consts::FRAC_PI_6),
                max_pitch: Some(std::f32::consts::FRAC_PI_2),
                render_mesh_base: None,
                base_offset: Vec3::new(0.0, -0.5, 0.0),
                render_mesh_yaw: None,
                yaw_offset: Vec3::new(0.0, 0.1, 0.0),
                render_mesh_pitch: None,
                pitch_offset: Vec3::new(0.0, 0.2, 0.0),
                render_mesh_barrel: None,
                barrel_offset: Vec3::new(0.1, 0.2, 0.0),
                muzzle_offset: Vec3::new(0.0, 0.0, -0.5),
                fire_rate: 100.0,
                projectile: BulletProjectileConfig {
                    muzzle_speed: 100.0,
                    lifetime: 5.0,
                    mass: 0.1,
                    render_mesh: None,
                },
                muzzle_effect: None,
            }),
        },
    ]));
}
