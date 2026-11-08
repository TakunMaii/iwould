use hecs::World;
use raylib::prelude::*;

const PIXEL_SIZE : f32 = 3.0;
const SCREEN_WIDTH : f32 = 800.0;
const SCREEN_HEIGHT : f32 = 600.0;
const WORLD_UNIT_SIZE : f32 = 16.0;

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

fn static_sprite_drawing_system(world: &mut World, d: &mut RaylibDrawHandle) {
    for (static_sprite, transform) in
        world.query_mut::<(&mut StaticSprite, &mut Transform)>()
    {
        draw_sprite(d, &static_sprite.sprite, &transform);
    }
}

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
    ));

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        static_sprite_drawing_system(&mut world, &mut d);
    }
}
