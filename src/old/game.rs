extern crate rand;
use std::f64::consts::PI;
use std::ops::{Index, IndexMut, Add, Mul};
use std::vec::Vec;

/// Player position is addressed in the same way the indicies in field blocks
/// are addressed, but in float, and (0, 0) position is in the top-left corner.
/// That means if we round down the coordinates, we get the cell where the
/// player stands.
#[derive(Debug, Copy, Clone)]
pub struct Player {
    pub x: f64,
    pub y: f64,
    pub angle: f64,
    pub vel_x: f64,
    pub vel_y: f64,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub blocks: Vec<Vec<u8>>,
    pub player: Player,
}

const ANGLE_FOR_200: f64 = PI * 80.0 / 180.0;
const ANGLE_STEP: f64 = ANGLE_FOR_200 / 200.0;

const TOO_FAR_WALL: f64 = 6.0;

struct Grid<'a, Arr, Ind> {
    arr: &'a mut Arr,
    width: Ind,
}

impl<'a, Arr, Ind> Index<(Ind, Ind)> for Grid<'a, Arr, Ind>
    where Ind: Add<Output=Ind> + Mul<Output=Ind> + Copy,
          Arr: Index<Ind>,
{
    type Output = Arr::Output;

    fn index(&self, (x, y): (Ind, Ind)) -> &Self::Output {
        &self.arr[ x + self.width * y ]
    }
}

impl<'a, Arr, Ind> IndexMut<(Ind, Ind)> for Grid<'a, Arr, Ind>
    where Ind: Add<Output=Ind> + Mul<Output=Ind> + Copy,
          Arr: IndexMut<Ind>,
{
    fn index_mut(&mut self, (x, y): (Ind, Ind)) -> &mut Self::Output {
        &mut self.arr[ x + self.width * y ]
    }
}

pub fn draw_scene(field: &Field, (width, height): (usize, usize), buffer: &mut Vec<char>) -> () {
    buffer.resize((width * height) as usize, 'a');
    let mut grid = Grid{ arr: buffer, width: width as usize };

    let field_width = field.blocks.len();
    let field_height = field.blocks[0].len();

    let view_angle = ANGLE_FOR_200 * width as f64 / 200.0;
    let mut current_angle = field.player.angle + (view_angle / 2.0);

    for col_x in 0..width {
        // trace one column
        let mut ray_x = field.player.x;
        let mut ray_y = field.player.y;
        let delta = 0.02;
        let delta_x = current_angle.cos() * delta;
        let delta_y = current_angle.sin() * delta;
        let mut distance = 0.0;

        // find distance to wall in this ray
        loop {
            let x = ray_x.floor() as i64;
            let y = ray_y.floor() as i64;
            if x < 0 || y < 0 || x >= field_width as i64 || y >= field_height as i64 {
                distance = TOO_FAR_WALL;
                break;
            }
            if field.blocks[x as usize][y as usize] == b'#' {
                // found block
                break;
            }

            distance += delta;
            ray_x += delta_x;
            ray_y += delta_y;
            if distance >= TOO_FAR_WALL {
                break;
            }
        }

//        let scale = TOO_FAR_WALL - distance;
//        let wall_size = height as f64 * scale / TOO_FAR_WALL;
        let wall_size = height as f64 / (distance + 1.0);
        let wall_bottom = (height as f64 + wall_size) / 2.0; // visual bottom
        let wall_top = (height as f64 - wall_size) / 2.0;
        let texture = if distance <= TOO_FAR_WALL / 3.0 {
            '▓'
        } else if distance <= TOO_FAR_WALL * 2.0 / 3.0 {
            '▒'
        } else {
            '░'
        };

        // draw ceiling
        for col_y in 0 .. wall_top as usize {
            grid[(col_x, col_y)] = ' ';
        }
        // draw wall
        for col_y in wall_top as usize .. wall_bottom as usize {
            grid[(col_x, col_y)] = texture;
        }
        // draw floor
        for col_y in wall_bottom as usize .. height {
            grid[(col_x, col_y)] = '.';
        }

        current_angle -= ANGLE_STEP;
    }

    grid[(0, 0)] = 'a';
}

