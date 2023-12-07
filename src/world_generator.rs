mod multi_octave_noise;
mod isize_index_matrix;

use std::collections::HashSet;
use noise::*;
use multi_octave_noise::Multi;
use robotics_lib::world::{worldgenerator::Generator, tile::Tile, tile::{TileType, Content}, environmental_conditions::{EnvironmentalConditions, WeatherType}};
use fast_poisson::Poisson2D;
use rand::{Rng, SeedableRng};
use isize_index_matrix::*;

pub struct WorldGenerator {
    seed: u32,
    world_size: usize,
}

#[derive(Clone, PartialEq)]
enum Biomes {
    Deepwater,
    ShallowWater,
    Beach,
    Desert,
    Plain,
    Forest,
    Hill,
    Mountain,
    SnowyMountain
}

impl WorldGenerator {
    pub fn new(seed: u32, world_size: usize) -> Self {
        Self { seed, world_size }
    }

    fn generate_weather(&self) -> EnvironmentalConditions {
        EnvironmentalConditions::new(
            &[WeatherType::Sunny, WeatherType::Rainy],
            1,
            0
        ).unwrap()
    }

    fn generate_altitude(&self, octaves: u8) -> Vec<Vec<f64>> {
        let mut noise_function = Multi::new(Perlin::new(self.seed), octaves, 1.0 / 180.0);
        noise_function.set_ampl_decay(0.5);

        let noise_function = Multiply::new(Constant::new(1.5), noise_function);

        // clamp values in [-1,1] range
        // let noise_function = Clamp::new(noise_function).set_bounds(-1.0, 1.0);

        let mut elevation_map = vec![vec![0.0; self.world_size]; self.world_size];

        for x in 0..self.world_size {
            for y in 0..self.world_size {
                elevation_map[x][y] = noise_function.get([x as f64, y as f64]);
            }
        }

        return elevation_map;
    }

    fn generate_temperature_map(&self) -> Vec<Vec<f64>> {
        let mut temperature_map = vec![vec![0.0; self.world_size]; self.world_size];

        let noise_function = Multiply::new(Constant::new(1.5), Multi::new(Perlin::new(self.seed + 42), 7, 1.0 / 100.0));

        for x in 0..self.world_size {
            for y in 0..self.world_size {
                temperature_map[x][y] = noise_function.get([x as f64, y as f64]);
            }
        }

        return temperature_map;
    }

    fn generate_lava_lakes(&self, world: &mut Vec<Vec<Tile>>, biomes_map: &Vec<Vec<Biomes>>) {
        let mut lava_lakes_map = vec![vec![0.0; self.world_size]; self.world_size];

        let noise_function = Multiply::new(Constant::new(1.5), Multi::new(Perlin::new(self.seed + 7), 7, 1.0 / 30.0));

        for x in 0..self.world_size {
            for y in 0..self.world_size {
                lava_lakes_map[x][y] = noise_function.get([x as f64, y as f64]);
            }
        }

        for x in 0..self.world_size {
            for y in 0..self.world_size {
                if biomes_map[x][y] == Biomes::Desert && lava_lakes_map[x][y] < -0.6 {
                    world[x][y].tile_type = TileType::Lava;
                }
            }
        }
    }

    fn generate_trees(&self, world: &mut Vec<Vec<Tile>>, biomes_map: &Vec<Vec<Biomes>>, ) {
        //forest
        Poisson2D::new().with_dimensions([self.world_size as f64, self.world_size as f64], 2.5).generate().iter()
            .map(|point| vec![point[0] as usize, point[1] as usize])
            .collect::<Vec<Vec<usize>>>().iter()
            .for_each(|coordinates| {
                let (x, y) = (coordinates[0], coordinates[1]);
                if biomes_map[x][y] == Biomes::Plain { world[x][y].content = Content::Tree(1) };
            });

        //hill
        Poisson2D::new().with_dimensions([self.world_size as f64, self.world_size as f64], 3.5).generate().iter()
            .map(|point| vec![point[0] as usize, point[1] as usize])
            .collect::<Vec<Vec<usize>>>().iter()
            .for_each(|coordinates| {
                let (x, y) = (coordinates[0], coordinates[1]);
                if biomes_map[x][y] == Biomes::Hill { world[x][y].content = Content::Tree(1) };
            });

        //mountain
        Poisson2D::new().with_dimensions([self.world_size as f64, self.world_size as f64], 4.5).generate().iter()
            .map(|point| vec![point[0] as usize, point[1] as usize])
            .collect::<Vec<Vec<usize>>>().iter()
            .for_each(|coordinates| {
                let (x, y) = (coordinates[0], coordinates[1]);
                if biomes_map[x][y] == Biomes::Mountain { world[x][y].content = Content::Tree(1) };
            });
    }

    fn generate_rocks(&self, world: &mut Vec<Vec<Tile>>, biomes_map: &Vec<Vec<Biomes>>, ) {
        //plains
        Poisson2D::new().with_dimensions([self.world_size as f64, self.world_size as f64], 5.0).generate().iter()
            .map(|point| vec![point[0] as usize, point[1] as usize])
            .collect::<Vec<Vec<usize>>>().iter()
            .for_each(|coordinates| {
                let (x, y) = (coordinates[0], coordinates[1]);
                if biomes_map[x][y] == Biomes::Forest { world[x][y].content = Content::Rock(1) };
            });

        //hills
        Poisson2D::new().with_dimensions([self.world_size as f64, self.world_size as f64], 4.0).generate().iter()
            .map(|point| vec![point[0] as usize, point[1] as usize])
            .collect::<Vec<Vec<usize>>>().iter()
            .for_each(|coordinates| {
                let (x, y) = (coordinates[0], coordinates[1]);
                if biomes_map[x][y] == Biomes::Hill { world[x][y].content = Content::Rock(1) };
            });

        //mountains
        Poisson2D::new().with_dimensions([self.world_size as f64, self.world_size as f64], 3.0).generate().iter()
            .map(|point| vec![point[0] as usize, point[1] as usize])
            .collect::<Vec<Vec<usize>>>().iter()
            .for_each(|coordinates| {
                let (x, y) = (coordinates[0], coordinates[1]);
                if biomes_map[x][y] == Biomes::Mountain { world[x][y].content = Content::Rock(1) };
            });

        //snowymountains
        Poisson2D::new().with_dimensions([self.world_size as f64, self.world_size as f64], 2.0).generate().iter()
            .map(|point| vec![point[0] as usize, point[1] as usize])
            .collect::<Vec<Vec<usize>>>().iter()
            .for_each(|coordinates| {
                let (x, y) = (coordinates[0], coordinates[1]);
                if biomes_map[x][y] == Biomes::SnowyMountain { world[x][y].content = Content::Rock(1) };
            });
    }

    fn generate_fishes(&self, world: &mut Vec<Vec<Tile>>, biomes_map: &Vec<Vec<Biomes>>, ) {
        Poisson2D::new().with_dimensions([self.world_size as f64, self.world_size as f64], 4.0).generate().iter()
            .map(|point| vec![point[0] as usize, point[1] as usize])
            .collect::<Vec<Vec<usize>>>().iter()
            .for_each(|coordinates| {
                let (x, y) = (coordinates[0], coordinates[1]);
                if biomes_map[x][y] == Biomes::ShallowWater { world[x][y].content = Content::Fish(1) };
            });

        Poisson2D::new().with_dimensions([self.world_size as f64, self.world_size as f64], 3.0).generate().iter()
            .map(|point| vec![point[0] as usize, point[1] as usize])
            .collect::<Vec<Vec<usize>>>().iter()
            .for_each(|coordinates| {
                let (x, y) = (coordinates[0], coordinates[1]);
                if biomes_map[x][y] == Biomes::Deepwater { world[x][y].content = Content::Fish(1) };
            });
    }

    fn get_biome(temperature: f64) -> Biomes {
        return match temperature {
            t if t < -0.3 => Biomes::Forest,
            t if t > 0.2 => Biomes::Desert,
            _ => Biomes::Plain
        }
    }

    fn generate_biomes(&self, elevation_map: &Vec<Vec<f64>>) -> Vec<Vec<Tile>> {
        let mut world = vec![vec![Tile { tile_type: TileType::DeepWater, content: Content::None, elevation: 0 }; self.world_size]; self.world_size];
        let mut biomes_map: Vec<Vec<Biomes>> = vec![vec![Biomes::Deepwater; self.world_size]; self.world_size];
        let temperature_map = self.generate_temperature_map();

        for x in 0..self.world_size {
            for y in 0..self.world_size {
                biomes_map[x][y] = match elevation_map[x][y] {
                    h if h < -0.65 => Biomes::Deepwater,
                    h if h < -0.40 => Biomes::ShallowWater,
                    h if h < -0.30 => Biomes::Beach,
                    h if h < 0.35 => Self::get_biome(temperature_map[x][y]),
                    h if h < 0.55 => Biomes::Hill,
                    h if h < 0.85 => Biomes::Mountain,
                    _ => Biomes::SnowyMountain,
                };
            }
        }

        for x in 0..self.world_size {
            for y in 0..self.world_size {
                world[x][y] = match biomes_map[x][y] {
                    Biomes::Deepwater => Tile { tile_type: TileType::DeepWater, content: Content::None, elevation: 0 },
                    Biomes::ShallowWater => Tile { tile_type: TileType::ShallowWater, content: Content::None, elevation: 0 },
                    Biomes::Beach => Tile { tile_type: TileType::Sand, content: Content::None, elevation: 0 },
                    Biomes::Desert => Tile { tile_type: TileType::Sand, content: Content::None, elevation: 0 },
                    Biomes::Plain => Tile { tile_type: TileType::Grass, content: Content::None, elevation: 0 },
                    Biomes::Forest => Tile { tile_type: TileType::Grass, content: Content::None, elevation: 0 },
                    Biomes::Hill => Tile { tile_type: TileType::Hill, content: Content::None, elevation: 0 },
                    Biomes::Mountain => Tile { tile_type: TileType::Mountain, content: Content::None, elevation: 0 },
                    Biomes::SnowyMountain => Tile { tile_type: TileType::Snow, content: Content::None, elevation: 0 },
                };
            }
        }

        self.generate_lava_lakes(&mut world, &biomes_map);
        self.generate_trees(&mut world, &biomes_map);
        self.generate_rocks(&mut world, &biomes_map);
        self.generate_fishes(&mut world, &biomes_map);

        return world;
    }

    fn generate_rivers(&self, world: &mut Vec<Vec<Tile>>, elevation: &Vec<Vec<f64>>, rivers_amount: f64) {
        let number_of_rivers = (world.len() as f64 * world.len() as f64 * rivers_amount / 1000.0) as usize;

        let rng_seed = {
            let mut rng_seed = [0u8; 32];
            rng_seed[0..4].copy_from_slice(&self.seed.to_le_bytes());
            rng_seed
        };

        let mut rng = rand::rngs::StdRng::from_seed((rng_seed).into());

        let world_pos_distribution = rand::distributions::Uniform::new(0, world.len() as isize);
        let float_distribution = rand::distributions::Uniform::new(0.0, 1.0);

        for _ in 0..number_of_rivers {
            let river_tiles = loop {
                let start_coords = loop {
                    let coords = (rng.sample(world_pos_distribution), rng.sample(world_pos_distribution));

                    if *elevation.at(coords) > 0.35 && world.at(coords).tile_type != TileType::ShallowWater {
                        break coords;
                    }
                };

                let mut river_tiles_stack = vec![start_coords];

                let mut avoid_tiles = HashSet::new();
                let mut river_tiles_set = HashSet::new();
                let mut river_inertia = (0.0, 0.0);

                loop {
                    if river_tiles_stack.is_empty() {
                        println!("no valid paths from {start_coords:?}");
                        break;
                    }
                    let coords = *river_tiles_stack.last().unwrap();
                    if !(1..world.len() as isize - 1).contains(&coords.0) || !(1..world.len() as isize - 1).contains(&coords.1) {
                        if rng.sample(float_distribution) < 0.7 {
                            println!("ran off map at {coords:?}");
                            break;
                        }
                    }

                    if [TileType::ShallowWater, TileType::DeepWater].contains(&world.at(coords).tile_type) {
                        println!("reached water at {coords:?}");
                        break;
                    }
                    if !(0..world.len() as isize).contains(&coords.0) || !(0..world.len() as isize).contains(&coords.1) {
                        if rng.sample(float_distribution) < 0.65 {
                            if rng.sample(float_distribution) < 0.75 {
                                println!("off the edge of the world at {coords:?}");
                            } else {
                                river_tiles_stack.clear();
                            }
                            break;
                        }
                    }

                    let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                    let candidates: Vec<_> =
                        directions.iter()
                            .filter_map(|(x, y)| {
                                let c = (coords.0 + x, coords.1 + y);

                                if avoid_tiles.contains(&c) {
                                    return None;
                                }
                                if !(0..world.len() as isize).contains(&c.0) || !(0..world.len() as isize).contains(&c.1) {
                                    return None;
                                }
                                //discard tiles which are adjacent to a river tile (other than the last)
                                for d in directions {
                                    let c_near = (c.0 + d.0, c.1 + d.1);
                                    if c_near == coords { continue; }

                                    if river_tiles_set.contains(&c_near) {
                                        return None;
                                    }
                                }
                                return Some(c);
                            }).collect();

                    if candidates.len() == 0 {
                        avoid_tiles.insert(coords);
                        let tile = river_tiles_stack.pop().unwrap();
                        river_tiles_set.remove(&tile);
                        continue;
                    }
                    let mut min_index = 0;

                    for i in 1..candidates.len() {
                        let c = candidates[i];
                        let m = candidates[min_index];

                        let inertia_factor = 0.004;

                        let direction_c = ((c.0 - coords.0) as f64, (c.1 - coords.1) as f64);
                        let inertia_c = (direction_c.0 * river_inertia.0 + direction_c.1 * river_inertia.1) * inertia_factor;

                        let direction_m = ((m.0 - coords.0) as f64, (m.1 - coords.1) as f64);
                        let inertia_m = (direction_m.0 * river_inertia.0 + direction_m.1 * river_inertia.1) * inertia_factor;

                        if elevation.at(c) - inertia_c < elevation.at(m) - inertia_m { min_index = i; }
                    }

                    let direction = (candidates[min_index].0 - coords.0, candidates[min_index].1 - coords.1);

                    river_inertia.0 = (river_inertia.0 * 1.0 + direction.0 as f64).clamp(-4.0, 4.0);
                    river_inertia.1 = (river_inertia.1 * 1.0 + direction.1 as f64).clamp(-4.0, 4.0);

                    avoid_tiles.insert(candidates[min_index]);
                    river_tiles_stack.push(candidates[min_index]);
                    river_tiles_set.insert(candidates[min_index]);
                }

                if river_tiles_stack.is_empty() {
                    continue;
                } else {
                    break river_tiles_stack;
                }
            };

            for coords in river_tiles {
                world.at_mut(coords).tile_type = TileType::ShallowWater;
            }
        }
    }
    fn generate_spawnpoint(&self) -> (usize, usize) {
        (0, 0)
    }
}

impl Generator for WorldGenerator {
    fn gen(&mut self) -> (Vec<Vec<Tile>>, (usize, usize), EnvironmentalConditions, f32) {    
        let altitude_map = self.generate_altitude(5);
        let mut world = self.generate_biomes(&altitude_map);
        self.generate_rivers(&mut world, &self.generate_altitude(7), 0.03);

        let weather = self.generate_weather();
        let spawnpoint = self.generate_spawnpoint();
        let score = 100.0;

        (world, spawnpoint, weather, score)
    }
}
