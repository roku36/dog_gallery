use bevy::{color::palettes::css, prelude::*};
use std::f32::consts::PI;
use std::time::Duration;
use bevy_panorbit_camera::*;
use bevy::{
    animation::animate_targets,
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
};
use bevy_mod_raycast::prelude::*;
// use bevy::render::settings::WgpuSettings;

const SKY_COLOR: Color = Color::srgb(0.4, 0.7, 0.9);
const GROUND_COLOR: Color = Color::srgb(0.2, 0.5, 0.3);
const SUN_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: SUN_COLOR,
            brightness: 1000.,
        })
        .insert_resource(ClearColor(SKY_COLOR))
        .insert_resource(RaycastPluginState::<()>::default().with_debug_cursor())
        .add_plugins((
            DefaultPlugins
                .set(bevy_mod_raycast::low_latency_window_plugin())
                .set(WindowPlugin {
                    primary_window: Window {
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }
                        .into(),
                    ..default()
                }),
            PanOrbitCameraPlugin,
            DeferredRaycastingPlugin::<()>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, setup_scene_once_loaded.before(animate_targets))
        .add_systems(Update, button_interaction_system)
        .add_systems(Update, get_cursor_position)
        .add_systems(Update, make_scene_pickable)
        .run();
}

#[derive(Resource)]
struct Animations {
    animations: Vec<AnimationNodeIndex>,
    #[allow(dead_code)]
    graph: Handle<AnimationGraph>,
}

#[derive(Component)]
struct Dog;

#[derive(Component)]
struct Gizmo;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Build the animation graph
    let mut graph = AnimationGraph::new();
    let animations = graph
        .add_clips(
            [
                GltfAssetLabel::Animation(1).from_asset("models/dog.glb"),
                GltfAssetLabel::Animation(2).from_asset("models/dog.glb"),
                GltfAssetLabel::Animation(0).from_asset("models/dog.glb"),
            ]
                .into_iter()
                .map(|path| asset_server.load(path)),
            1.0,
            graph.root,
        )
        .collect();

    // Insert a resource with the current scene information
    let graph = graphs.add(graph);
    commands.insert_resource(Animations {
        animations,
        graph: graph.clone(),
    });

    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(100.0, 100.0, 150.0)
                .looking_at(Vec3::new(0.0, 20.0, 0.0), Vec3::Y),
            ..default()
        },
        PanOrbitCamera {
            focus: Vec3::new(0.0, 20.0, 0.0),
            zoom_upper_limit: Some(320.0),
            zoom_lower_limit: Some(100.0),
            pan_sensitivity: 0.0,
            ..default()
        },
        RaycastSource::<()>::new_cursor(),
    ));

    // Plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(500000.0, 500000.0)),
            material: materials.add(GROUND_COLOR),
            ..default()
        },
    ));

    // Light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 400.0,
            ..default()
        }
            .into(),
        ..default()
    });

    // Gizmo
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere { radius: 1.5 }),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 0.0),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -100.0, 0.0), // 初期位置はカメラに見えない場所
            ..default()
        },
        Gizmo, // Gizmoのタグを付ける
    ));

    // Dog model
    commands.spawn((
        SceneBundle {
            scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/dog.glb")),
            // scene: asset_server.load("models/dog.glb#Scene0"),
            ..default()
        },
        Dog,
        // RaycastMesh::<()>::default(), // Make this mesh ray cast-able;
    ));

    
    // Sphere
    // commands.spawn((
    //     PbrBundle {
    //         mesh: meshes.add(Sphere { radius: 50.0 }),
    //         material: materials.add(Color::from(css::GRAY)),
    //         transform: Transform::from_xyz(50.0, 50.0, -80.0),
    //         ..default()
    //     },
    //     RaycastMesh::<()>::default(), // Make this mesh ray cast-able;
    // ));

    // UI Buttons for animation control
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                margin: UiRect {
                    bottom: Val::Px(-50.0),
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            spawn_animation_button(parent, &asset_server, "images/sit0.png", "images/sit1.png", 0);
            spawn_animation_button(parent, &asset_server, "images/walk0.png", "images/walk1.png", 1);
            spawn_animation_button(parent, &asset_server, "images/run0.png", "images/run1.png", 2);
        });
}

fn spawn_animation_button(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    normal_image_path: &str,
    pressed_image_path: &str,
    animation_index: usize,
) {
    let normal_image = asset_server.load(normal_image_path.to_string());
    let pressed_image = asset_server.load(pressed_image_path.to_string());

    parent.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(100.0),
                height: Val::Px(100.0),
                margin: UiRect::all(Val::Px(15.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            image: UiImage::new(normal_image.clone()), // initial image is unpressed
            ..default()
        },
        AnimationButton {
            animation_index,
            normal_image,
            pressed_image,
        },
    ));
}

#[derive(Component)]
struct AnimationButton {
    animation_index: usize,
    normal_image: Handle<Image>,
    pressed_image: Handle<Image>,
}

fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    mut interaction_query: Query<(&Interaction, &AnimationButton), (Changed<Interaction>, With<Button>)>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }

    // Process button presses to change animations
    for (interaction, animation_button) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            for (_, mut player) in &mut players {
                let mut transitions = AnimationTransitions::new();
                transitions
                    .play(&mut player, animations.animations[animation_button.animation_index], Duration::from_millis(250))
                    .repeat();
            }
        }
    }
}


fn button_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiImage, &AnimationButton),
        With<Button>,
    >,
    animations: Res<Animations>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    mut selected_animation: Local<Option<usize>>,
) {
    // find selected button
    let mut new_selection = None;
    for (interaction, _image, animation_button) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            if let Some(selected_index) = *selected_animation {
                if selected_index == animation_button.animation_index {
                    return;
                }
            }
            new_selection = Some(animation_button.animation_index);
            break;
        }
    }

    if let Some(new_animation_index) = new_selection {
        // play new animation
        *selected_animation = Some(new_animation_index);

        for (mut player, mut transitions) in &mut animation_players {
            transitions
                .play(
                    &mut player,
                    animations.animations[new_animation_index],
                    Duration::from_millis(250),
                )
                .repeat();
        }

        // reset button state
        for (_, mut image, animation_button) in &mut interaction_query {
            if animation_button.animation_index == new_animation_index {
                *image = animation_button.pressed_image.clone().into(); // change selected button
            } else {
                *image = animation_button.normal_image.clone().into(); // change unselected button
            }
        }
    }
}

fn get_cursor_position(
    query: Query<&RaycastSource<()>>,
    mut gizmo_query: Query<&mut Transform, With<Gizmo>>,
) {
    // info!("draw_cursor");
    for ray_source in &query {
        // if let Some((entity, intersection)) = ray_source.intersections().first() {
        //     info!("Hit entity: {:?}, position: {:?}", entity, intersection.position());
        // }
        if let Some((_entity, intersection)) = ray_source.intersections().first() {
            // Gizmoの位置を更新する
            if let Ok(mut gizmo_transform) = gizmo_query.get_single_mut() {
                gizmo_transform.translation = intersection.position();
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn make_scene_pickable(
    mut commands: Commands,
    mesh_query: Query<Entity, (With<Handle<Mesh>>, Without<RaycastMesh<()>>, Without<Gizmo>)>,
) {
    for entity in &mesh_query {
        commands
            .entity(entity)
            .insert(RaycastMesh::<()>::default()); // Make this mesh ray cast-able
    }
}
