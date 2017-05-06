extern crate rand;
use rand::Rng;
use std::collections::HashMap;
use std::io;


const INITIAL_ROBOTS :i32 = 10;
const FIELD_WIDTH : i32 = 30;
const FIELD_HEIGHT : i32 = 12;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]
struct Position {
    x : i32,
    y : i32,
}

#[derive(Debug)]
#[derive(PartialEq)]
enum EntityKind {
    Player,
    Robot,
    Garbage
}

#[derive(Debug)]
struct Entity {
    kind : EntityKind,
    position : Position,
}

struct GameField
{
    /// Entries on the game field. 0th entry is always the playe
    entities : Vec<Entity>,

    /// Maps (x,y) position to index in the entities array
    pos_to_idx : HashMap<(i32, i32), usize>,
}

impl Entity{
    fn new_at(kind : EntityKind, x : i32, y : i32) -> Entity
    {
        Entity {
            kind : kind,
            position : Position::new(x,y),
        }

    }

    /// Marks entity as destroyed garbage
    fn destroy(&mut self) {
        self.kind = EntityKind::Garbage;
    }
}
impl Position
{
    /// Creates new position with given x, y coordinates
    fn new(x : i32, y : i32) -> Position {
        Position {
            x,y
        }
    }
            
    fn clone_add(&self, dx : i32, dy : i32) -> Position {
        Position::new(self.x + dx, self.y + dy) 
    }
            
}

impl GameField{
    /// Makes new game field with given number of robots and player in the middle
    fn new(robots : i32) -> GameField {
        let mut new_field = GameField {
            entities : vec![],
            pos_to_idx : HashMap::new(),
        };

        new_field.spawn_player();

        for _ in 0..robots { 
            new_field.spawn_robot() 
        }

        new_field
    }

    /// Return anindex of entity at given coordinates  
    /// or None if there is no entity at that position
    fn index_at(&self, x : i32, y:i32) -> Option<usize>
    {
        self.pos_to_idx.get(&(x,y)).map(|i| *i)
    }

    /// Return anindex of entity at given coordinates  
    /// or None if there is no entity at that position
    fn index_at_pos(&self, pos : &Position) -> Option<usize> 
    {
        self.index_at(pos.x, pos.y)
    }

    /// Return true iff game field has any kind of entity at given coords
    fn has_entity_at(&self, x : i32, y : i32) -> bool 
    {
        self.index_at(x, y).is_some()
    }

    /// Append entity to list of enties
    /// And place its position into position-to-entity hash map so we can find it later
    fn append_entity(&mut self, e : Entity) {
        let n = self.entities.len();
        self.pos_to_idx.insert((e.position.x,e.position.y), n);
        self.entities.push(e);
    }

    /// Spawn the player in the middle of game fiield
    /// Should be called first, as game assumes that entities[0] is the player
    fn spawn_player(&mut self) {
        let player = Entity::new_at(EntityKind::Player, FIELD_WIDTH / 2, FIELD_HEIGHT / 2);
        self.append_entity(player);
    }

    /// Spawn robot at random unoccupied position
    fn spawn_robot(&mut self) {
        loop {
            let x = rand::thread_rng().gen_range(0, FIELD_WIDTH);
            let y = rand::thread_rng().gen_range(0, FIELD_HEIGHT);
            if self.has_entity_at(x, y) {
                continue;
            }

            let robot = Entity::new_at(EntityKind::Robot, x, y);
            self.append_entity(robot);
            break;
        }
    }

    /// Print game field to the console's stdout
    #[allow(dead_code)] 
    fn debug_print(&self) {
        for y in 0..FIELD_HEIGHT {
            let mut row = String::new();
            for x in 0 .. FIELD_WIDTH {
                let ch = match self.index_at(x,y) {
                    Some(idx) => match (&self.entities[idx]).kind {
                        EntityKind::Player => '@',
                        EntityKind::Robot => '$',
                        EntityKind::Garbage => '#',
                    },
                    None => '.'
                };
                row.push(ch);
            }
            println!("{}", row);
        }
        println!("");
    }

    /// Return a refence to player
    fn player(&self) -> &Entity
    {
        return &self.entities[0];
    }


    /// Move robots toward the current position of the player
    fn move_robots(&mut self)
    {
        /// Clamp delta to [-1;1]
        fn clamp_delta(i : i32) -> i32 {
            if i > 0 { return 1; }
            else if i < 0 { return -1; }
            return 0;
        };

        let px = self.player().position.x;
        let py = self.player().position.y;

        // First, calculate where to we are moving
        for entity in &mut self.entities {
            if entity.kind == EntityKind::Garbage || entity.kind == EntityKind::Player {
                continue;
            }
            let robot = entity;
            if robot.kind != EntityKind::Robot {
                continue;
            }

            let dx = clamp_delta(px - robot.position.x);
            let dy = clamp_delta(py - robot.position.y);
            robot.position = robot.position.clone_add(dx, dy);
        }


        /// Rebuild map
        self.pos_to_idx.clear();
        for i in 0..(self.entities.len()) {
            self.pos_to_idx.insert((self.entities[i].position.x, self.entities[i].position.y), i);
        }

        // Transform collided entities
        for i in 0..(self.entities.len()) {
            self.check_collision(i)
        }
    }

    // check if i and  are collided,
    // and mark them as junk if they do
    fn check_collision(&mut self, i:usize) {
        if let Some(other) = self.index_at_pos(&self.entities[i].position){
            if other != i {
                (&mut self.entities[i]).destroy();
                (&mut self.entities[other]).destroy();
            }
        }
    }

    /// Move player
    fn move_player(&mut self, dx : i32, dy : i32) {
        let (px, py) = (self.entities[0].position.x,  self.entities[0].position.y);
        
        let dx = 
            if dx < 0 && px == 0 {0} 
            else if dx > 0 && px == FIELD_WIDTH - 1 {0}
            else {dx};
        
        let dy = 
            if dy < 0 && py == 0 {0} 
            else if dy > 0 && py == FIELD_HEIGHT - 1 {0}
            else {dy};

        if dx == 0 && dy == 0 {
            return
        }

        self.entities[0].position = self.entities[0].position.clone_add(dx, dy);
        self.check_collision(0)
    }

    /// Return true iff player is dead
    fn player_is_dead(&self) -> bool {
        self.entities[0].kind == EntityKind::Garbage
    }

    /// Teleport the player to random location on the map.
    fn teleport_player(&mut self) {
        let x = rand::thread_rng().gen_range(0, FIELD_WIDTH);
        let y = rand::thread_rng().gen_range(0, FIELD_HEIGHT);
        self.pos_to_idx.remove(&(self.entities[0].position.x, self.entities[0].position.y));
        (&mut self.entities[0]).position = Position::new(x, y);
        self.check_collision(0);
        self.pos_to_idx.insert((self.entities[0].position.x, self.entities[0].position.y),0);
    }
}








/// Main function
fn main() {
    
    let mut game_field = GameField::new(INITIAL_ROBOTS);

    loop 
    {
        game_field.debug_print();
        if game_field.player_is_dead() {
            println!("You have lost.");
            break;
        }

        let mut buf =  String::new();
        io::stdin().read_line(&mut buf)
            .expect("Cannot read command from stdin");
        let buf = buf.trim();
        if buf == "." {
            game_field.move_robots()
        } else if buf == "q" {
            break;
        } else if buf == "w" {
            game_field.move_player(0, -1);
            game_field.move_robots();
        } else if buf == "a" {
            game_field.move_player(-1, 0);
            game_field.move_robots();
        } else if buf == "s" {
            game_field.move_player(0, 1);
            game_field.move_robots();
        } else if buf == "d" {
            game_field.move_player(1, 0);
            game_field.move_robots();
        } else if buf == "t" {
            game_field.teleport_player();
        }
    }
}
