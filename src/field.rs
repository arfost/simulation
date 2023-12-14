use crate::bot::{Action, Bot};
use crate::conf::get_conf;
use rand::Rng;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::collections::HashMap;
use std::fmt::format;

const BASE_SIZE: i32 = 10;

#[derive(Clone, Debug)]

pub enum Color {
    None,
    Green,
    Yellow,
    Red,
}

#[derive(Debug)]
pub struct OtherBot {
    pub id: usize,
    pub energy: isize,
}

pub struct KillZone {
    pub time_appear: usize,
    pub shape: (usize, usize, usize, usize),
}

pub struct BotPosition {
    pub bot: Bot,
    pub x: usize,
    pub y: usize,
}

pub struct Field {
    cells: HashMap<String, BotPosition>,
    colors: HashMap<String, Color>,
    age: usize,
    killzones: Vec<KillZone>,
}

impl Field {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            killzones: vec![KillZone {
                shape: (0, 0, 0, 0),
                time_appear: 200,
            }],
            age: 0,
            colors: HashMap::new(),
            cells: (0..20)
                .map(|x| {
                    let x = rand::thread_rng().gen_range(0..width);
                    let y = rand::thread_rng().gen_range(0..height);
                    (
                        format!("{}:{}", x, y),
                        BotPosition {
                            bot: Bot::new(),
                            x: x,
                            y: y,
                        },
                    )
                })
                .collect(),
        }
    }

    fn get_cell(&self, x: isize, y: isize) -> (i8, Option<OtherBot>) {
        let c = get_conf();
        if x >= 0 && y >= 0 && x < c.field_width as isize - 1 && y < c.field_height as isize - 1 {
            if self.cells.contains_key(&format!("{}{}", x, y)) {
                let bot_pos = self.cells.get(&format!("{}{}", x, y)).unwrap();
                (
                    1,
                    Some(OtherBot {
                        energy: bot_pos.bot.energy,
                        id: bot_pos.bot.id as usize,
                    }),
                )
            } else {
                (0, None)
            }
        } else {
            (-1, None)
        }
    }

    fn get_new_coordinates(x: usize, y: usize, angle: usize) -> (usize, usize) {
        match angle {
            0 => (x - 1, y - 1),
            1 => (x, y - 1),
            2 => (x + 1, y - 1),
            3 => (x - 1, y),
            4 => (x + 1, y),
            5 => (x - 1, y + 1),
            6 => (x, y + 1),
            7 => (x + 1, y + 1),
            _ => (x, y),
        }
    }

    pub fn step(&mut self) {
        self.colors.clear();

        self.age += 1;

        let mut new_pos = Vec::<BotPosition>::new();
        for (_key, bot_pos) in self.cells.iter() {
            let x = bot_pos.x.clone() as isize;
            let y = bot_pos.y.clone() as isize;

            let (angle, action) = bot_pos.bot.step(vec![
                self.get_cell(x - 1, y - 1),
                self.get_cell(x, y - 1),
                self.get_cell(x + 1, y - 1),
                self.get_cell(x - 1, y),
                self.get_cell(x + 1, y),
                self.get_cell(x - 1, y + 1),
                self.get_cell(x, y + 1),
                self.get_cell(x + 1, y + 1),
            ]);


            let (new_x, new_y) = Self::get_new_coordinates(bot_pos.x, bot_pos.y, angle as usize);
            match action {
                Action::Move => {

                  let mut np = BotPosition{
                    bot:bot_pos.bot.clone(),
                    x: new_x,
                    y: new_y
                  };

                  np.bot.energy += -2;
                  np.bot.action_color = Color::Yellow;
                  new_pos.push(np);

                }
                Action::Reproduction => {

                  let mut np = BotPosition{
                    bot:bot_pos.bot.clone(),
                    x:x.clone() as usize,
                    y:y.clone() as usize,
                  };

                  np.bot.action_color = Color::Green;
                  
                  np.bot.energy += -3;
                  new_pos.push(np);

                  let new_bot = bot_pos.bot.mutate();

                  new_pos.push(BotPosition{
                    bot:new_bot,
                    x:new_x,
                    y:new_y
                  });

                }
                Action::Attack => {
                  let attacked = self.cells.get(&format!("{}{}", new_x, new_y)).unwrap();

                  let mut np = BotPosition{
                    bot:bot_pos.bot.clone(),
                    x:x.clone() as usize,
                    y:y.clone() as usize,
                  };

                  np.bot.action_color = Color::Red;
                  np.bot.energy += 3;
                  new_pos.push(np);

                }
                Action::Heal => {
                
                  let mut np = BotPosition{
                    bot:bot_pos.bot.clone(),
                    x:x.clone() as usize,
                    y:y.clone() as usize,
                  };

                  np.bot.action_color = Color::None;
                  np.bot.energy += 3;
                  new_pos.push(np);
                }
            }
        }

        self.cells.clear();
        for np in new_pos {
          if np.bot.energy>=0 {
            self.cells.insert(format!("{}{}", np.x, np.y), np);
          }
        }
    }


    pub fn draw(&self, canvas: &mut WindowCanvas) {
        for (key, bot_pos) in self.cells.iter() {
            let rect = Rect::new(
                bot_pos.x as i32 * BASE_SIZE,
                bot_pos.y as i32 * BASE_SIZE,
                BASE_SIZE as u32,
                BASE_SIZE as u32,
            );
            canvas.set_draw_color(sdl2::pixels::Color::RGB(
                bot_pos.bot.color.0,
                bot_pos.bot.color.1,
                bot_pos.bot.color.2,
            ));
            canvas.fill_rect(rect).unwrap();

            match bot_pos.bot.action_color 
            {
                Color::None => {
                    canvas.set_draw_color(sdl2::pixels::Color::WHITE);
                }
                Color::Green => {
                    canvas.set_draw_color(sdl2::pixels::Color::GREEN);
                }
                Color::Yellow => {
                    canvas.set_draw_color(sdl2::pixels::Color::YELLOW);
                }
                Color::Red => {
                    canvas.set_draw_color(sdl2::pixels::Color::RED);
                }
            };
            canvas.draw_rect(rect).unwrap();
        }
    }
}
