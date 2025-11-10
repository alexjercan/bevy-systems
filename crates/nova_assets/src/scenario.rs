use bevy::{platform::collections::HashMap, prelude::*};
use nova_gameplay::prelude::*;
use rand::prelude::*;

pub(crate) fn register_scenario(mut commands: Commands, game_assets: Res<super::GameAssets>) {
    commands.insert_resource(GameScenarios(HashMap::from([
        ("asteroid_field".to_string(), asteroid_field(&game_assets)),
        ("asteroid_next".to_string(), asteroid_next(&game_assets)),
    ])));
}

pub fn asteroid_field(game_assets: &super::GameAssets) -> ScenarioConfig {
    let mut rng = rand::rng();

    let mut objects = Vec::new();
    for i in 0..20 {
        let pos = Vec3::new(
            rng.random_range(-100.0..100.0),
            rng.random_range(-20.0..20.0),
            rng.random_range(-100.0..100.0),
        );
        let radius = rng.random_range(1.0..3.0);
        let texture = game_assets.asteroid_texture.clone();

        objects.push(ScenarioObjectConfig {
            base: BaseScenarioObjectConfig {
                id: format!("asteroid_{}", i),
                name: format!("Asteroid {}", i),
                position: pos,
                rotation: Quat::IDENTITY,
                health: 100.0,
            },
            kind: ScenarioObjectKind::Asteroid(AsteroidConfig { radius, texture }),
        });
    }

    let spaceship = SpaceshipConfig {
        controller: SpaceshipController::Player(PlayerControllerConfig {}),
        sections: vec![
            SpaceshipSectionConfig {
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                config: SectionConfig {
                    base: BaseSectionConfig {
                        name: "Basic Controller Section".to_string(),
                        description: "A basic controller section for spaceships.".to_string(),
                        mass: 1.0,
                    },
                    kind: SectionKind::Controller(ControllerSectionConfig {
                        frequency: 4.0,
                        damping_ratio: 4.0,
                        max_torque: 100.0,
                        render_mesh: None,
                    }),
                },
            },
            SpaceshipSectionConfig {
                position: Vec3::new(0.0, 0.0, 1.0),
                rotation: Quat::IDENTITY,
                config: SectionConfig {
                    base: BaseSectionConfig {
                        name: "Basic Hull Section".to_string(),
                        description: "A basic hull section for spaceships.".to_string(),
                        mass: 1.0,
                    },
                    kind: SectionKind::Hull(HullSectionConfig { render_mesh: None }),
                },
            },
            SpaceshipSectionConfig {
                position: Vec3::new(0.0, 0.0, -1.0),
                rotation: Quat::IDENTITY,
                config: SectionConfig {
                    base: BaseSectionConfig {
                        name: "Basic Hull Section".to_string(),
                        description: "A basic hull section for spaceships.".to_string(),
                        mass: 1.0,
                    },
                    kind: SectionKind::Hull(HullSectionConfig { render_mesh: None }),
                },
            },
            SpaceshipSectionConfig {
                position: Vec3::new(0.0, 0.0, 2.0),
                rotation: Quat::IDENTITY,
                config: SectionConfig {
                    base: BaseSectionConfig {
                        name: "Basic Thruster Section".to_string(),
                        description: "A basic thruster section for spaceships.".to_string(),
                        mass: 1.0,
                    },
                    kind: SectionKind::Thruster(ThrusterSectionConfig {
                        magnitude: 1.0,
                        render_mesh: None,
                    }),
                },
            },
            SpaceshipSectionConfig {
                position: Vec3::new(0.0, 0.0, -2.0),
                rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                config: SectionConfig {
                    base: BaseSectionConfig {
                        name: "Basic Turret Section".to_string(),
                        description: "A basic turret section for spaceships.".to_string(),
                        mass: 1.0,
                    },
                    kind: SectionKind::Turret(TurretSectionConfig {
                        yaw_speed: std::f32::consts::PI,
                        pitch_speed: std::f32::consts::PI,
                        min_pitch: Some(-std::f32::consts::FRAC_PI_6),
                        max_pitch: Some(std::f32::consts::FRAC_PI_2),
                        render_mesh_base: None,
                        base_offset: Vec3::new(0.0, -0.5, 0.0),
                        render_mesh_yaw: Some(game_assets.turret_yaw_01.clone()),
                        yaw_offset: Vec3::new(0.0, 0.1, 0.0),
                        render_mesh_pitch: Some(game_assets.turret_pitch_01.clone()),
                        pitch_offset: Vec3::new(0.0, 0.332706, 0.303954),
                        render_mesh_barrel: Some(game_assets.turret_barrel_01.clone()),
                        barrel_offset: Vec3::new(0.0, 0.128437, -0.110729),
                        muzzle_offset: Vec3::new(0.0, 0.0, -1.2),
                        fire_rate: 100.0,
                        muzzle_speed: 100.0,
                        projectile_lifetime: 5.0,
                        projectile_mass: 0.1,
                        projectile_render_mesh: None,
                        muzzle_effect: None,
                    }),
                },
            },
        ],
    };
    objects.push(ScenarioObjectConfig {
        base: BaseScenarioObjectConfig {
            id: "player_spaceship".to_string(),
            name: "Player Spaceship".to_string(),
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            health: 500.0,
        },
        kind: ScenarioObjectKind::Spaceship(spaceship),
    });

    let spaceship = SpaceshipConfig {
        controller: SpaceshipController::None,
        sections: vec![
            SpaceshipSectionConfig {
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                config: SectionConfig {
                    base: BaseSectionConfig {
                        name: "Basic Controller Section".to_string(),
                        description: "A basic controller section for spaceships.".to_string(),
                        mass: 1.0,
                    },
                    kind: SectionKind::Controller(ControllerSectionConfig {
                        frequency: 4.0,
                        damping_ratio: 4.0,
                        max_torque: 100.0,
                        render_mesh: None,
                    }),
                },
            },
            SpaceshipSectionConfig {
                position: Vec3::new(0.0, 0.0, 1.0),
                rotation: Quat::IDENTITY,
                config: SectionConfig {
                    base: BaseSectionConfig {
                        name: "Basic Hull Section".to_string(),
                        description: "A basic hull section for spaceships.".to_string(),
                        mass: 1.0,
                    },
                    kind: SectionKind::Hull(HullSectionConfig { render_mesh: None }),
                },
            },
            SpaceshipSectionConfig {
                position: Vec3::new(0.0, 0.0, -1.0),
                rotation: Quat::IDENTITY,
                config: SectionConfig {
                    base: BaseSectionConfig {
                        name: "Basic Hull Section".to_string(),
                        description: "A basic hull section for spaceships.".to_string(),
                        mass: 1.0,
                    },
                    kind: SectionKind::Hull(HullSectionConfig { render_mesh: None }),
                },
            },
            SpaceshipSectionConfig {
                position: Vec3::new(0.0, 0.0, 2.0),
                rotation: Quat::IDENTITY,
                config: SectionConfig {
                    base: BaseSectionConfig {
                        name: "Basic Thruster Section".to_string(),
                        description: "A basic thruster section for spaceships.".to_string(),
                        mass: 1.0,
                    },
                    kind: SectionKind::Thruster(ThrusterSectionConfig {
                        magnitude: 1.0,
                        render_mesh: None,
                    }),
                },
            },
            SpaceshipSectionConfig {
                position: Vec3::new(0.0, 0.0, -2.0),
                rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                config: SectionConfig {
                    base: BaseSectionConfig {
                        name: "Basic Turret Section".to_string(),
                        description: "A basic turret section for spaceships.".to_string(),
                        mass: 1.0,
                    },
                    kind: SectionKind::Turret(TurretSectionConfig {
                        yaw_speed: std::f32::consts::PI,
                        pitch_speed: std::f32::consts::PI,
                        min_pitch: Some(-std::f32::consts::FRAC_PI_6),
                        max_pitch: Some(std::f32::consts::FRAC_PI_2),
                        render_mesh_base: None,
                        base_offset: Vec3::new(0.0, -0.5, 0.0),
                        render_mesh_yaw: Some(game_assets.turret_yaw_01.clone()),
                        yaw_offset: Vec3::new(0.0, 0.1, 0.0),
                        render_mesh_pitch: Some(game_assets.turret_pitch_01.clone()),
                        pitch_offset: Vec3::new(0.0, 0.332706, 0.303954),
                        render_mesh_barrel: Some(game_assets.turret_barrel_01.clone()),
                        barrel_offset: Vec3::new(0.0, 0.128437, -0.110729),
                        muzzle_offset: Vec3::new(0.0, 0.0, -1.2),
                        fire_rate: 100.0,
                        muzzle_speed: 100.0,
                        projectile_lifetime: 5.0,
                        projectile_mass: 0.1,
                        projectile_render_mesh: None,
                        muzzle_effect: None,
                    }),
                },
            },
        ],
    };
    objects.push(ScenarioObjectConfig {
        base: BaseScenarioObjectConfig {
            id: "other_spaceship".to_string(),
            name: "Other Spaceship".to_string(),
            position: Vec3::new(10.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            health: 100.0,
        },
        kind: ScenarioObjectKind::Spaceship(spaceship),
    });

    let events = vec![
        ScenarioEventConfig {
            name: EventConfig::OnStart,
            filters: vec![],
            actions: objects
                .into_iter()
                .map(|o| EventActionConfig::SpawnScenarioObject(o))
                .collect::<_>(),
        },
        ScenarioEventConfig {
            name: EventConfig::OnStart,
            filters: vec![],
            actions: vec![EventActionConfig::Objective(ObjectiveActionConfig::new(
                "destroy_asteroids",
                "Objective: Destroy 5 asteroids!",
            ))],
        },
        ScenarioEventConfig {
            name: EventConfig::OnStart,
            filters: vec![],
            actions: vec![EventActionConfig::VariableSet(VariableSetActionConfig {
                key: "asteroids_destroyed".to_string(),
                expression: VariableExpressionNode::new_term(VariableTermNode::new_factor(
                    VariableFactorNode::new_literal(VariableLiteral::Number(0.0)),
                )),
            })],
        },
        ScenarioEventConfig {
            name: EventConfig::OnStart,
            filters: vec![],
            actions: vec![EventActionConfig::VariableSet(VariableSetActionConfig {
                key: "objective_destroy_asteroids".to_string(),
                expression: VariableExpressionNode::new_term(VariableTermNode::new_factor(
                    VariableFactorNode::new_literal(VariableLiteral::Boolean(false)),
                )),
            })],
        },
        ScenarioEventConfig {
            name: EventConfig::OnDestroyed,
            filters: vec![EventFilterConfig::Entity(EntityFilterConfig {
                id: Some("player_spaceship".to_string()),
                type_name: None,
            })],
            actions: vec![
                EventActionConfig::DebugMessage(DebugMessageActionConfig {
                    message: "The player's spaceship was destroyed!".to_string(),
                }),
                EventActionConfig::NextScenario(NextScenarioActionConfig {
                    scenario_id: "asteroid_field".to_string(),
                    linger: true,
                }),
            ],
        },
        ScenarioEventConfig {
            name: EventConfig::OnDestroyed,
            filters: vec![EventFilterConfig::Entity(EntityFilterConfig {
                id: None,
                type_name: Some("asteroid".to_string()),
            })],
            actions: vec![EventActionConfig::VariableSet(VariableSetActionConfig {
                key: "asteroids_destroyed".to_string(),
                expression: VariableExpressionNode::new_add(
                    VariableTermNode::new_factor(VariableFactorNode::new_name(
                        "asteroids_destroyed".to_string(),
                    )),
                    VariableExpressionNode::new_term(VariableTermNode::new_factor(
                        VariableFactorNode::new_literal(VariableLiteral::Number(1.0)),
                    )),
                ),
            })],
        },
        ScenarioEventConfig {
            name: EventConfig::OnDestroyed,
            filters: vec![
                EventFilterConfig::Entity(EntityFilterConfig {
                    id: None,
                    type_name: Some("asteroid".to_string()),
                }),
                EventFilterConfig::Expression(ExpressionFilterConfig(
                    VariableConditionNode::new_greater_than(
                        VariableExpressionNode::new_term(VariableTermNode::new_factor(
                            VariableFactorNode::new_name("asteroids_destroyed".to_string()),
                        )),
                        VariableExpressionNode::new_term(VariableTermNode::new_factor(
                            VariableFactorNode::new_literal(VariableLiteral::Number(4.0)),
                        )),
                    ),
                )),
                EventFilterConfig::Expression(ExpressionFilterConfig(
                    VariableConditionNode::new_equals(
                        VariableExpressionNode::new_term(VariableTermNode::new_factor(
                            VariableFactorNode::new_name("objective_destroy_asteroids".to_string()),
                        )),
                        VariableExpressionNode::new_term(VariableTermNode::new_factor(
                            VariableFactorNode::new_literal(VariableLiteral::Boolean(false)),
                        )),
                    ),
                )),
            ],
            actions: vec![
                EventActionConfig::DebugMessage(DebugMessageActionConfig {
                    message: "Objective Complete: Destroyed 5 asteroids!".to_string(),
                }),
                EventActionConfig::VariableSet(VariableSetActionConfig {
                    key: "objective_destroy_asteroids".to_string(),
                    expression: VariableExpressionNode::new_term(VariableTermNode::new_factor(
                        VariableFactorNode::new_literal(VariableLiteral::Boolean(true)),
                    )),
                }),
                EventActionConfig::ObjectiveComplete(ObjectiveCompleteActionConfig {
                    id: "destroy_asteroids".to_string(),
                }),
                EventActionConfig::NextScenario(NextScenarioActionConfig {
                    scenario_id: "asteroid_next".to_string(),
                    linger: true,
                }),
            ],
        },
        ScenarioEventConfig {
            name: EventConfig::OnDestroyed,
            filters: vec![EventFilterConfig::Entity(EntityFilterConfig {
                id: None,
                type_name: Some("asteroid".to_string()),
            })],
            actions: vec![EventActionConfig::DebugMessage(DebugMessageActionConfig {
                message: "An asteroid was destroyed!".to_string(),
            })],
        },
    ];

    ScenarioConfig {
        id: "asteroid_field".to_string(),
        name: "Asteroid Field".to_string(),
        description: "A dense asteroid field.".to_string(),
        cubemap: game_assets.cubemap.clone(),
        events,
    }
}

pub fn asteroid_next(game_assets: &super::GameAssets) -> ScenarioConfig {
    ScenarioConfig {
        id: "asteroid_next".to_string(),
        name: "Asteroid Field - Next".to_string(),
        description: "The next scenario after the asteroid field.".to_string(),
        cubemap: game_assets.cubemap.clone(),
        events: vec![],
    }
}
