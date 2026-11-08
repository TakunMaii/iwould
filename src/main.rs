use std::collections::HashMap;

use hecs::{CommandBuffer, Entity, World};
use raylib::prelude::*;

const PIXEL_SIZE: f32 = 4.0;
const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;
const WORLD_UNIT_SIZE: f32 = 16.0;

const PLAYER_MOVE_SPEED: f32 = 3.0;
const PLAYER_ATTACK_TIME: f32 = 0.3;

struct Resources {
    textures: HashMap<String, Texture2D>,
}

// --- Components ---
struct Sprite {
    texture_name: String,
    rect: Rectangle,
}

struct StaticSprite {
    sprite: Sprite,
}

struct Transform {
    // world position in world units
    position: Vector2,
    rotation: f32,
    scale: Vector2,
}

struct DieInTime {
    rest_time: f32,
}

struct PlayerControl {
    move_speed: f32,
}

struct PlayerAttacking {
    rest_time: f32,
}

// --- Components ---

fn draw_sprite(
    d: &mut RaylibDrawHandle,
    sprite: &Sprite,
    transform: &Transform,
    resources: &Resources,
) {
    d.draw_texture_pro(
        resources.textures.get(&sprite.texture_name).unwrap(),
        sprite.rect,
        Rectangle {
            width: sprite.rect.width * PIXEL_SIZE * transform.scale.x,
            height: sprite.rect.height * PIXEL_SIZE * transform.scale.y,
            x: transform.position.x * PIXEL_SIZE * WORLD_UNIT_SIZE,
            y: SCREEN_HEIGHT - transform.position.y * PIXEL_SIZE * WORLD_UNIT_SIZE,
        },
        Vector2 {
            x: sprite.rect.width * PIXEL_SIZE * transform.scale.x / 2.0,
            y: sprite.rect.height * PIXEL_SIZE * transform.scale.y / 2.0,
        },
        transform.rotation,
        Color::WHITE,
    );
}

// --- Systems ---
fn static_sprite_drawing_system(
    world: &mut World,
    d: &mut RaylibDrawHandle,
    resources: &Resources,
) {
    for (static_sprite, transform) in world.query_mut::<(&mut StaticSprite, &mut Transform)>() {
        draw_sprite(d, &static_sprite.sprite, &transform, resources);
    }
}

fn player_movement_system(world: &mut World, d: &mut RaylibDrawHandle, resources: &Resources) {
    let dt = d.get_frame_time();
    for (player_control, transform) in world
        .query_mut::<(&mut PlayerControl, &mut Transform)>()
        .without::<&PlayerAttacking>()
    {
        let mut move_vec = Vector2::new(0.0, 0.0);

        // keyboard input
        if d.is_key_down(KeyboardKey::KEY_W) {
            move_vec.y += 1.0;
        }
        if d.is_key_down(KeyboardKey::KEY_S) {
            move_vec.y -= 1.0;
        }
        if d.is_key_down(KeyboardKey::KEY_A) {
            move_vec.x -= 1.0;
        }
        if d.is_key_down(KeyboardKey::KEY_D) {
            move_vec.x += 1.0;
        }

        // joystick input
        let joystick_move_vec = Vector2::new(
            d.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_X),
            d.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_Y),
        );
        move_vec += joystick_move_vec;

        move_vec = if move_vec.length() > 0.0 {
            move_vec.normalize()
        } else {
            move_vec
        };

        transform.position += move_vec * player_control.move_speed * dt;
    }
}

fn player_generate_sword(
    cmd: &mut CommandBuffer,
    player_entity: Entity,
    player_transform: &Transform,
    position_bias: Vector2,
    rotation: f32,
    d: &mut RaylibDrawHandle,
    thread: &raylib::prelude::RaylibThread,
    resources: &Resources,
) {
    cmd.spawn((
        StaticSprite {
            sprite: Sprite {
                texture_name: "sword".to_string(),
                rect: Rectangle::new(0.0, 0.0, 16.0, 16.0),
            },
        },
        Transform {
            position: Vector2::new(
                player_transform.position.x + position_bias.x,
                player_transform.position.y + position_bias.y,
            ),
            rotation: rotation,
            scale: Vector2::new(1.0, 1.0),
        },
        DieInTime {
            rest_time: PLAYER_ATTACK_TIME,
        },
    ));

    cmd.insert_one(
        player_entity,
        PlayerAttacking {
            rest_time: PLAYER_ATTACK_TIME,
        },
    );
}

fn player_attack_system(
    world: &mut World,
    d: &mut RaylibDrawHandle,
    thread: &raylib::prelude::RaylibThread,
    resources: &Resources,
) {
    let mut cmd = CommandBuffer::new();
    for (entity, _, transform) in world
        .query_mut::<(Entity, &mut PlayerControl, &mut Transform)>()
        .without::<&PlayerAttacking>()
    {
        // generate a sword entity when the player presses the attack button
        // which are arrow keys, UP, DOWN, LEFT, RIGHT

        if d.is_key_pressed(KeyboardKey::KEY_UP) {
            player_generate_sword(
                &mut cmd,
                entity,
                transform,
                Vector2 { x: 0., y: 1. },
                -90.0,
                d,
                thread,
                resources,
            );
        } else if d.is_key_pressed(KeyboardKey::KEY_DOWN) {
            player_generate_sword(
                &mut cmd,
                entity,
                transform,
                Vector2 { x: 0., y: -1. },
                90.0,
                d,
                thread,
                resources,
            );
        } else if d.is_key_pressed(KeyboardKey::KEY_LEFT) {
            player_generate_sword(
                &mut cmd,
                entity,
                transform,
                Vector2 { x: -1., y: 0. },
                180.0,
                d,
                thread,
                resources,
            );
        } else if d.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            player_generate_sword(
                &mut cmd,
                entity,
                transform,
                Vector2 { x: 1., y: 0. },
                0.0,
                d,
                thread,
                resources,
            );
        }
    }

    cmd.run_on(world);
}

fn player_attack_cooldown_system(world: &mut World, d: &mut RaylibDrawHandle) {
    let dt = d.get_frame_time();
    let mut cmd = CommandBuffer::new();

    for (entity, attacking) in world.query_mut::<(Entity, &mut PlayerAttacking)>() {
        attacking.rest_time -= dt;
        if attacking.rest_time <= 0.0 {
            cmd.remove_one::<PlayerAttacking>(entity);
        }
    }
    cmd.run_on(world);
}

fn die_in_time_system(world: &mut World, d: &mut RaylibDrawHandle) {
    let dt = d.get_frame_time();
    let mut cmd = CommandBuffer::new();

    for (entity, die_in_time) in world.query_mut::<(Entity, &mut DieInTime)>() {
        die_in_time.rest_time -= dt;
        if die_in_time.rest_time <= 0.0 {
            cmd.despawn(entity);
        }
    }
    cmd.run_on(world);
}

// --- Systems ---

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32)
        .title("iwould")
        .build();

    let mut resources = Resources {
        textures: HashMap::new(),
    };
    // load textures
    resources.textures.insert(
        "player".to_string(),
        rl.load_texture(&thread, "assets/player/player.png")
            .unwrap(),
    );
    resources.textures.insert(
        "sword".to_string(),
        rl.load_texture(&thread, "assets/player/sword.png").unwrap(),
    );

    let mut world = World::new();

    world.spawn((
        StaticSprite {
            sprite: Sprite {
                texture_name: "player".to_string(),
                rect: Rectangle::new(0.0, 0.0, 16.0, 16.0),
            },
        },
        Transform {
            position: Vector2::new(1.0, 1.0),
            rotation: 0.0,
            scale: Vector2::new(1.0, 1.0),
        },
        PlayerControl {
            move_speed: PLAYER_MOVE_SPEED,
        },
    ));

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        die_in_time_system(&mut world, &mut d);

        player_movement_system(&mut world, &mut d, &resources);
        player_attack_system(&mut world, &mut d, &thread, &resources);
        player_attack_cooldown_system(&mut world, &mut d);

        d.clear_background(Color::BLACK);
        static_sprite_drawing_system(&mut world, &mut d, &resources);
    }
}
