use hecs::World;
use raylib::prelude::*;

const PIXEL_SIZE : f32 = 4.0;
const SCREEN_WIDTH : f32 = 800.0;
const SCREEN_HEIGHT : f32 = 600.0;
const WORLD_UNIT_SIZE : f32 = 16.0;

const PLAYER_MOVE_SPEED : f32 = 3.0;

// --- Components ---
struct Sprite {
    texture: Texture2D,
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

struct PlayerControl {
    move_speed: f32,
}
// --- Components ---

fn draw_sprite(d: &mut RaylibDrawHandle, sprite: &Sprite, transform: &Transform) {
    d.draw_texture_pro(
        &sprite.texture,
        sprite.rect,
        Rectangle {
            width : sprite.rect.width * PIXEL_SIZE * transform.scale.x,
            height : sprite.rect.height * PIXEL_SIZE * transform.scale.y,
            x : transform.position.x * PIXEL_SIZE * WORLD_UNIT_SIZE,
            y : SCREEN_HEIGHT - transform.position.y * PIXEL_SIZE * WORLD_UNIT_SIZE,
        },
        Vector2 {
            x : sprite.rect.width * PIXEL_SIZE * transform.scale.x / 2.0,
            y : sprite.rect.height * PIXEL_SIZE * transform.scale.y / 2.0,
        },
        transform.rotation,
        Color::WHITE,
    );
}

// --- Systems ---
fn static_sprite_drawing_system(world: &mut World, d: &mut RaylibDrawHandle) {
    for (static_sprite, transform) in
        world.query_mut::<(&mut StaticSprite, &mut Transform)>()
    {
        draw_sprite(d, &static_sprite.sprite, &transform);
    }
}

fn player_control_system(world: &mut World, d: &mut RaylibDrawHandle) {
    let dt = d.get_frame_time();
    for (player_control, transform) in
        world.query_mut::<(&mut PlayerControl, &mut Transform)>()
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
// --- Systems ---

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32)
        .title("iwould")
        .build();

    let mut world = World::new();

    world.spawn((
        StaticSprite {
            sprite: Sprite {
                texture: rl
                    .load_texture(&thread, "assets/player/player.png")
                    .unwrap(),
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
        d.clear_background(Color::BLACK);

        player_control_system(&mut world, &mut d);
        static_sprite_drawing_system(&mut world, &mut d);
    }
}
