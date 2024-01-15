
use raylib::core::math::{Rectangle, Vector2};
use raylib::prelude::*;
use std::vec::Vec;

struct Position {
  x: f32,
  y: f32
}

struct Physics {
  bounding_box: Vector2,
  velocity: Vector2,
  gravity: Vector2,

  max_fallspeed: f32,
  jump_velocity: f32,

  max_runspeed: f32,
  run_acceleration: f32,
  ground_friction: f32,

  max_airspeed: f32,
  air_acceleration: f32,
  air_friction: f32,

  grounded: bool
}

struct Sprite {
  image: Image,
  texture: Option<Texture2D>,
  size: Vector2,
}

enum Component {
  PositionComponent(Box<Position>),
  PhysicsComponent(Box<Physics>),
  SpriteComponent(Box<Sprite>)
}

use Component::*;

trait PickMut<'a> {
  type Item;

  fn pick_out(self: &'a mut Self, at: usize)
    -> Option<(&'a mut Self::Item, &mut [Self::Item])>;
}

/*
impl<'a, T: 'a> PickMut<'a> for Vec<T> {
  type Item = T;

  fn pick_out(self: &'a mut Vec<T>, at: usize)
    -> Option<(&mut T, &mut [T])>
  {
    std::mem::swap(&mut self[0], &mut self[at]);

    if let Some((item, after)) = self.split_first_mut() {
      Some((item, after))
    } else {
      None
    }
  }
}
*/

impl<'a, T: 'a> PickMut<'a> for [T] {
  type Item = T;

  fn pick_out(self: &'a mut [T], at: usize)
    -> Option<(&'a mut T, &'a mut [T])>
  {
    self.swap(0, at);

    self.split_first_mut()
  }
}

macro_rules! find_component_mut {
  ($pat:ident, $things:expr) => {
    {
      let mut result = None;
      let things = $things;
      for i in 0..things.len() {
        match things[i] {
          $pat(_) => {
            let (boxed, rest) = things.pick_out(i).unwrap();
            let $pat(ref mut x) = *boxed else { panic!(); };
            result = Some((x, rest));
            break;
          }
          _ => ()
        }
      }
      result
    }
  }
}

macro_rules! find_components_mut_ {
  ({}, $new:expr, {$($results:expr)*}) => {
    Some(($($results),*, $new))
  };

  ({$pat: ident $($pats:ident)*}, $ent:expr, {$($results:expr),*}) => {
    {
      if let Some((comp, newexpr)) = find_component_mut!($pat, $ent) {
        find_components_mut_!({$($pats)*}, newexpr, {$($results),* comp})
      } else {
        None
      }
    }
  }
}

macro_rules! find_components_mut {
  ({$($pats:ident)*}, $things:expr) => {
    find_components_mut_!({$($pats)*}, $things, {})
  }
}

macro_rules! find_component {
  ($pat:ident, $things:expr) => {
    {
      let &things = $things;
      let mut result = None;
      for i in 0..things.len() {
        match things[i] {
          $pat(ref x) => {
            result = Some(x);
            break;
          }
          _ => ()
        }
      }
      result
    }
  }
}

macro_rules! find_components_ {
  ({}, $new:expr, {$($results:expr)*}) => {
    Some(($($results),*, $new))
  };

  ({$pat: ident $($pats:ident)*}, $ent:expr, {$($results:expr),*}) => {
    {
      if let Some(comp) = find_component!($pat, $ent) {
        find_components_!({$($pats)*}, $ent, {$($results),* comp})
      } else {
        None
      }
    }
  }
}

macro_rules! find_components {
  ({$($pats:ident)*}, $ent:expr) => {
    {
      let e = $ent;
      find_components_!({$($pats)*}, e, {})
    }
  }
}

type Entity = Vec<Component>;

fn physics(idx: usize, state: &mut Gamestate, timedelta: f32) {
  if let Some((ent, _rest)) = state.actors.pick_out(idx) {
    let (phys, pos, _) = find_components_mut!({PhysicsComponent PositionComponent}, ent).unwrap();

    let newvel = Vector2 {
      x: phys.velocity.x + phys.gravity.x * timedelta,
      y: phys.velocity.y + phys.gravity.y * timedelta,
    };

    let newpos = Box::new(Position {
      x: pos.x + newvel.x * timedelta,
      y: pos.y + newvel.y * timedelta,
    });

    *pos = newpos;
    phys.velocity = newvel;
  }
}

fn draw(ent: &Entity, _game: &Gamestate, drawer: &mut RaylibTextureMode<RaylibDrawHandle>) {

  if let Some((sprite, pos, _)) = find_components!({SpriteComponent PositionComponent}, &ent)
  {
    drawer.draw_texture(
      sprite.texture.as_ref().unwrap(),
      (pos.x - sprite.size.x / 2f32) as i32,
      (pos.y - sprite.size.y) as i32,
      Color::WHITE,
    );

    drawer.draw_circle(
      pos.x as i32,
      pos.y as i32,
      4.0,
      Color::RED,
    );
  }
}

struct Platform {
  bbox: Rectangle,
}

trait Stage {
  fn draw(&self, drawer: &mut RaylibTextureMode<RaylibDrawHandle>);
  fn bounding_box(&self) -> Rectangle;
}

impl Stage for Platform {
  fn bounding_box(&self) -> Rectangle {
    return self.bbox;
  }

  fn draw(&self, drawer: &mut RaylibTextureMode<RaylibDrawHandle>) {
    drawer.draw_rectangle_rec(self.bbox, Color::RED);
  }
}

struct Gamestate {
  actors: Vec<Box<Entity>>,
  stages: Vec<Box<dyn Stage>>,
}

fn main() {
  let (mut rl, thread) = raylib::init()
    .size(640, 480)
    .vsync()
    .title("Hello, World")
    .build();

  let img = core::texture::Image::load_image("test.png").unwrap();

  let tex = rl.load_texture_from_image(&thread, &img).unwrap();

  let mut c = vec![
    PositionComponent(Box::new(Position { x: 100., y: 200. })),
    PhysicsComponent(Box::new(Physics {
      bounding_box: Vector2 { x: 16., y: 16. },
      gravity: Vector2 { x: 0., y: 200. },
      max_fallspeed: 400.,
      velocity: Vector2 {
        x: 0.,
        y: -200.
      },
      jump_velocity: 200.,
      max_runspeed: 400.,
      run_acceleration: 200.,
      ground_friction: 200.,
      max_airspeed: 100.,
      air_acceleration: 50.,
      air_friction: 100.,
      grounded: false
    })),
    SpriteComponent(Box::new(Sprite {
      image: img,
      texture: Some(tex),
      size: Vector2 {
        x: 0.,
        y: 0.
      },
    })),
  ];

  {
    let (sprite, _) = find_component_mut!(SpriteComponent, &mut c).unwrap();

    sprite.size.x = sprite.texture.as_ref().unwrap().width() as f32;
    sprite.size.y = sprite.texture.as_ref().unwrap().height() as f32;
  }

  let mut w = Gamestate {
    actors: vec![Box::new(c)],
    stages: vec![Box::new(Platform {
      bbox: Rectangle {
        x: 50.,
        y: 200.,
        height: 30.,
        width: 300.,
      },
    })],
  };

  const FRAME: f32 = 1.0 / 60.0;

  let mut starttime = rl.get_time() as f32;
  let mut accumtime: f32 = 0.0;

  let mut tex = rl.load_render_texture(&thread, 640 / 2, 480 / 2).unwrap();

  while !rl.window_should_close() {
    let curtime = rl.get_time() as f32;
    accumtime += curtime - starttime;

    while accumtime > FRAME {
      for i in 0..w.actors.len() {
        physics(i, &mut w, FRAME);
      }

      accumtime -= FRAME;
    }

    let mut d = rl.begin_drawing(&thread);
    d.clear_background(Color::WHITE);

    {
      let mut mode = d.begin_texture_mode(&thread, &mut tex);

      mode.clear_background(Color::WHITE);
      mode.draw_text(
        &format!("Hello, world! {:.2}", 1.0 / (curtime - starttime))[..],
        12,
        12,
        20,
        Color::BLACK,
      );

      for a in &w.actors {
        draw(&**a, &w, &mut mode);
      }

      for s in &w.stages {
        s.draw(&mut mode);
      }
    }

    d.draw_texture_pro(
      &tex,
      Rectangle {
        width: 640. / 4.,
        height: -480. / 2.,
        x: 0.0,
        y: 0.0,
      },
      Rectangle {
        width: 640.0 / 2.,
        height: 480.0,
        x: 640. / 2.,
        y: 0.0,
      },
      Vector2 { x: 0.0, y: 0.0 },
      0.0,
      Color::WHITE,
    );

    d.draw_texture_pro(
      &tex,
      Rectangle {
        width: 640. / 4.,
        height: -480. / 2.,
        x: 0.0,
        y: 0.0,
      },
      Rectangle {
        width: 640.0 / 2.,
        height: 480.0,
        x: 0.0,
        y: 0.0,
      },
      Vector2 { x: 0.0, y: 0.0 },
      0.0,
      Color::WHITE,
    );

    starttime = curtime;
  }
}
