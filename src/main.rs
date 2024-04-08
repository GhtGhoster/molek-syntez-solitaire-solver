use std::{collections::HashSet, fmt::Display, hash::Hash, thread::sleep, time::{Duration, Instant}};
use enigo::{Enigo, MouseControllable};
use rand::{thread_rng, Rng};
use screenshots::{image::{io::Reader, GenericImageView, ImageBuffer}, Screen};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[allow(dead_code)]
const BOX_WIDTH: u32 = 22;
#[allow(dead_code)]
const BOX_HEIGHT: u32 = 18;
const OFFSET_H: i32 = 488;
const OFFSET_V: i32 = 298;
const SPACE_H: i32 = 652 - OFFSET_H;
const SPACE_V: i32 = 330 - OFFSET_V;

const STEP_LIMIT: usize = 2000;
const PAST_LIMIT: usize = 20000;
const ACCEPTABLE_SOLUTION_LEN: usize = 400;

#[derive(Clone, Copy, PartialEq, Debug, EnumIter)]
enum Card {
    Tits = 8,
    King = 7,
    Diva = 6,
    Viva = 5,
    Ten = 4,
    Nine = 3,
    Eight = 2,
    Seven = 1,
    Six = 0,
}

impl Card {
    fn from_char(value: char) -> Option<Self> {
        match value {
            't' | 'T' => Some(Card::Tits),
            'k' | 'K' => Some(Card::King),
            'd' | 'D' => Some(Card::Diva),
            'v' | 'V' => Some(Card::Viva),
            '0' | '1' => Some(Card::Ten),
            '9' => Some(Card::Nine),
            '8' => Some(Card::Eight),
            '7' => Some(Card::Seven),
            '6' => Some(Card::Six),
            _ => None,
        }
    }

    fn to_char(&self) -> char {
        let character = match self {
            Card::Tits => 'T',
            Card::King => 'K',
            Card::Diva => 'D',
            Card::Viva => 'V',
            Card::Ten => '0',
            Card::Nine => '9',
            Card::Eight => '8',
            Card::Seven => '7',
            Card::Six => '6',
        };
        character
    }
    
    fn from_image(image: ImageBuffer<screenshots::image::Rgba<u8>, Vec<u8>>) -> Option<Card> {
        'card_loop: for card_type in Card::iter() {
            let card_image = Reader::open(format!("assets/{}.png", card_type.to_char())).unwrap().decode().unwrap();
            for x in 0..BOX_WIDTH {
                for y in 0..BOX_HEIGHT {
                    if &card_image.get_pixel(x, y) != image.get_pixel(x, y) {
                        continue 'card_loop;
                    }
                }
            }
            return Some(card_type);
        }
        None
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[derive(Default, Clone)]
struct Stack {
    cards: Vec<Card>,
    collapsed: bool,
    cheated: bool,
}

impl Stack {
    fn highest_orderly_count(&self) -> usize {
        let len = self.cards.len();
        if len < 2 {
            return len;
        }
        let mut ret = 1;
        while self.cards[len-ret] as usize + 1 == self.cards[len-ret-1] as usize {
            ret += 1;
            if ret == self.cards.len() {
                return ret;
            }
        }
        ret
    }

    fn is_orderly(&self, card: Card) -> bool {
        if self.cards.len() == 0 {
            return true;
        }
        let last_stack_card_index = self.cards.len()-1;
        let last_stack_card_num = self.cards[last_stack_card_index] as usize;
        let card_num = card as usize;
        card_num + 1 == last_stack_card_num
    }
}

#[derive(PartialEq)]
enum MoveValidity {
    ValidNormal,
    ValidCheat,
    Invalid,
}

// 9 different types of cards, 4 sets of cards, 6 columns, 6 rows at the start, 36 cards total
#[derive(Default)]
struct Matrix {
    stacks: [Stack; 6],
    available_moves: Vec<(Move, Matrix)>,
    past_moves: Vec<Move>,
}

impl PartialEq for Matrix {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for Matrix {}

impl Hash for Matrix {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

impl Matrix {
    #[allow(dead_code)]
    fn from_screen() -> Option<Matrix> {
        let mut matrix: Matrix = Default::default();

        // grab the screen. This is specifically set up for my use-case
        // aka 3 monitors at FHD, game running on middle monitor with whatever order I've set up
        let screens = Screen::all().unwrap();
        let screen = screens[0];

        for x in 0..6 {
            for y in 0..6 {
                let image: ImageBuffer<screenshots::image::Rgba<u8>, Vec<u8>> = screen.capture_area(
                    OFFSET_H + (x * SPACE_H),
                    OFFSET_V + (y * SPACE_V),
                    BOX_WIDTH,
                    BOX_HEIGHT,
                ).unwrap();
                if let Some(card) = Card::from_image(image) {
                    matrix.stacks[x as usize].cards.push(card);
                } else {
                    return None
                }
            }
        }

        Some(matrix)
    }

    #[allow(dead_code)]
    fn random() -> Matrix {
        let mut matrix: Matrix = Default::default();
        let mut matrix_index = 0;
        let mut rng = thread_rng();

        for _ in 0..4 {
            let mut chars = "67890VDKT".to_string();
            while chars.len() != 0 {
                let random_index = rng.gen_range(0..chars.len());
                let character = chars.remove(random_index);
                matrix.stacks[matrix_index].cards.push(Card::from_char(character).unwrap());
                matrix_index += 1;
                matrix_index %= 6;
            }
        }

        matrix
    }

    fn check_validity(&self, mov: Move) -> MoveValidity {
        let Move{from, to, count} = mov;
        if
            self.stacks[to].collapsed ||
            self.stacks[from].collapsed ||
            self.stacks[to].cheated ||
            to == from
        {
            return MoveValidity::Invalid;
        }
        let first_card_index_from = self.stacks[from].cards.len() - count;
        if count == 1 {
            if self.stacks[to].is_orderly(self.stacks[from].cards[first_card_index_from]) {
                return MoveValidity::ValidNormal;
            }
            if self.stacks[from].cheated {
                return MoveValidity::Invalid;
            }
            return MoveValidity::ValidCheat;
        }
        if self.stacks[from].highest_orderly_count() > count {
            return MoveValidity::Invalid;
        }
        if self.stacks[to].cards.is_empty() {
            return MoveValidity::ValidNormal;
        }
        let last_card_index_to = self.stacks[to].cards.len() - 1;
        if self.stacks[to].cards[last_card_index_to] as usize == self.stacks[from].cards[first_card_index_from] as usize + 1 {
            return MoveValidity::ValidNormal;
        }
        return MoveValidity::Invalid;
    }

    fn move_stack(&mut self, mov: Move) -> bool {
        let Move{from, to, count} = mov;

        // validity check
        let validity: MoveValidity = self.check_validity(mov);
        if validity == MoveValidity::Invalid {
            return false;
        }
        if validity == MoveValidity::ValidCheat {
            self.stacks[to].cheated = true;
        }

        // actual movement
        let mut moving_cards: Vec<Card> =  vec![];

        self.stacks[from].cheated = false;
        for _ in 0..count {
            moving_cards.push(self.stacks[from].cards.pop().unwrap());
        }
        for _ in 0..count {
            self.stacks[to].cards.push(moving_cards.pop().unwrap());
        }

        // collapsed check
        if
            self.stacks[to].highest_orderly_count() == self.stacks[to].cards.len() &&
            self.stacks[to].cards.len() == 9
        {
            self.stacks[to].collapsed = true;
        }

        return true;
    }

    fn is_win(&self) -> bool {
        let mut collapsed_count = 0;
        for i in 0..6 {
            if self.stacks[i].collapsed {
                collapsed_count += 1;
            }
        }
        collapsed_count == 4
    }

    fn valid_moves(&self) -> Vec<Move> {
        let mut ret = vec![];

        for from in 0..6 {
            let highest_orderly_count = self.stacks[from].highest_orderly_count();
            for count in 1..=highest_orderly_count {
                for to in 0..6 {
                    let mov: Move = Move {from, to , count};
                    if self.check_validity(mov) != MoveValidity::Invalid {
                        ret.push(mov);
                    }
                }
            }
        }

        ret
    }

    fn copy(&self) -> Matrix {
        let mut matrix: Matrix = Default::default();

        // copy
        for i in 0..6 {
            for j in 0..self.stacks[i].cards.len() {
                matrix.stacks[i].cards.push(self.stacks[i].cards[j]);
            }
            matrix.stacks[i].cheated = self.stacks[i].cheated;
            matrix.stacks[i].collapsed = self.stacks[i].collapsed;
        }
        for i in 0..self.past_moves.len() {
            matrix.past_moves.push(self.past_moves[i]);
        }

        matrix
    }

    fn copy_after_move(&self, mov: Move) -> Matrix {
        let mut matrix: Matrix = self.copy();
        matrix.move_stack(mov);
        matrix.past_moves.push(mov);
        matrix
    }

    fn to_string(&self) -> String {
        let mut ret = String::new();

        for stack in &self.stacks {
            ret.push('S');
            for card in &stack.cards {
                ret.push(card.to_char());
            }
            if stack.cheated {
                ret.push('_');
            }
        }

        ret
    }

    fn save_moves(&mut self, allow_cheats: bool) {
        self.available_moves = self
            .valid_moves()
            .iter()
            .map(|mov| (*mov, self.copy_after_move(*mov)))
            .filter(|(mov, _)| allow_cheats || self.check_validity(*mov) != MoveValidity::ValidCheat)
            .collect();
        // prioritize low move count
        self.available_moves.sort_by(|(_, a), (_, b)| a.valid_moves().len().cmp(&b.valid_moves().len()));
    }

    fn prune(&mut self, past_matrices: &HashSet<Matrix>) {
        for i in (0..self.available_moves.len()).rev() {
            if past_matrices.contains(&self.available_moves[i].1) {
                self.available_moves.remove(i);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Move {
    from: usize,
    to: usize,
    count: usize,
}

fn main() {
    // TODO:
    // - improve found solutions via past matrices, cutting out middle parts (the end-game is atrocious due to most moves being preferred)
    // - add CLI

    let start_time = Instant::now();

    let mut matrix = Matrix::random();
    let start_matrix = matrix.copy();

    let mut winners: Vec<Matrix> = vec![];
    let mut past_matrices: HashSet<Matrix> = HashSet::new();
    while past_matrices.len() < PAST_LIMIT {
        if let Some(winner) = find_win(&mut matrix, &mut past_matrices, true) {
            println!("Found winner: {}", winner.past_moves.len());
            winners.push(winner);
        }
    }
    optimize_solutions(start_matrix, &winners, &past_matrices);

    loop_wins(0, true);

    println!("Finished after {} seconds", start_time.elapsed().as_secs_f32());
}

fn optimize_solutions(start_matrix: Matrix, winner_matrices: &Vec<Matrix>, past_matrices: &HashSet<Matrix>) {
    for winner_matrix in winner_matrices {
        // reconstruct the matrices on the way to a winning matrix
        let mut winner_matrix_history: Vec<Matrix> = vec![];
        let mut last_matrix = start_matrix.copy();
        for mov in &winner_matrix.past_moves {
            last_matrix = last_matrix.copy_after_move(*mov);
            winner_matrix_history.push(last_matrix.copy());
        }
    }

    // depending on how much time this shit takes:
    // I can also check each matrix for undiscovered moves x depth down
    // the more stacks that are collapsed the deeper I think I can afford to go
    // prune() also saves all matrices 
}

fn loop_wins(target_wins: usize, allow_cheats: bool) {
    let mut enigo = Enigo::new();
    let mut iter_count = 0;
    while iter_count < target_wins {
        // focus window
        enigo.mouse_move_to(
            1920 + OFFSET_H - SPACE_H,
            OFFSET_V - SPACE_V,
        );
        sleep(Duration::from_millis(100));
        enigo.mouse_down(enigo::MouseButton::Left);
        sleep(Duration::from_millis(50));
        enigo.mouse_up(enigo::MouseButton::Left);
        sleep(Duration::from_millis(100));

        // click new game
        enigo.mouse_move_to(
            1920 + (1920/2),
            1080 - 40,
        );
        sleep(Duration::from_millis(100));
        enigo.mouse_down(enigo::MouseButton::Left);
        sleep(Duration::from_millis(50));
        enigo.mouse_up(enigo::MouseButton::Left);
        
        // wait for game to be set up
        let mut matrix_option = Matrix::from_screen();
        while matrix_option.is_none() {
            sleep(Duration::from_millis(1500));
            matrix_option = Matrix::from_screen();
        }
        let mut matrix = matrix_option.unwrap();
        let start_matrix = matrix.copy();
        
        // find solutions
        let mut winners: Vec<Matrix> = vec![];
        let mut past_matrices: HashSet<Matrix> = HashSet::new();
        while past_matrices.len() < PAST_LIMIT {
            if let Some(winner) = find_win(&mut matrix, &mut past_matrices, allow_cheats) {
                winners.push(winner);
                if !allow_cheats {
                    break;
                }
            }
        }

        // TODO:
        // optimize solution(s)?
        optimize_solutions(start_matrix, &winners, &past_matrices);

        // execute best solution
        if !winners.is_empty() {
            winners.sort_by(|a, b| a.past_moves.len().cmp(&b.past_moves.len()));
            if allow_cheats && winners[0].past_moves.len() > ACCEPTABLE_SOLUTION_LEN {
                continue;
            }
            // execute_moves(&mut matrix, &winners[0].past_moves);
            println!("Shortest win found: {}", winners[0].past_moves.len());
            iter_count += 1;
        }
    }
}

#[allow(dead_code)]
fn execute_moves(matrix: &mut Matrix, moves: &Vec<Move>) {
    let mut enigo = Enigo::new();

    // focus window but don't pick a card if window already focused
    enigo.mouse_move_to(
        1920 + OFFSET_H - SPACE_H,
        OFFSET_V - SPACE_V,
    );
    sleep(Duration::from_millis(100));
    enigo.mouse_down(enigo::MouseButton::Left);
    sleep(Duration::from_millis(50));
    enigo.mouse_up(enigo::MouseButton::Left);
    sleep(Duration::from_millis(100));

    for mov in moves {
        let y_from = matrix.stacks[mov.from].cards.len() - mov.count;
        enigo.mouse_move_to(
            1920 + OFFSET_H + (mov.from as i32 * SPACE_H),
            OFFSET_V + (y_from.max(0) as i32 * SPACE_V),
        );
        enigo.mouse_down(enigo::MouseButton::Left);
        sleep(Duration::from_millis(50));
        enigo.mouse_up(enigo::MouseButton::Left);

        sleep(Duration::from_millis(50));

        let y_to = (matrix.stacks[mov.to].cards.len() as i32 - 1).max(0) as usize;
        enigo.mouse_move_to(
            1920 + OFFSET_H + (mov.to as i32 * SPACE_H),
            OFFSET_V + (y_to as i32 * SPACE_V),
        );
        enigo.mouse_down(enigo::MouseButton::Left);
        sleep(Duration::from_millis(50));
        enigo.mouse_up(enigo::MouseButton::Left);

        sleep(Duration::from_millis(100));

        matrix.move_stack(*mov);
    }
}

#[allow(dead_code)]
fn find_win(matrix: &mut Matrix, past_matrices: &mut HashSet<Matrix>, allow_cheats: bool) -> Option<Matrix> {
    past_matrices.insert(matrix.copy());
    if allow_cheats && past_matrices.len() > PAST_LIMIT {
        return None;
    }

    matrix.save_moves(allow_cheats);
    matrix.prune(&past_matrices);

    if matrix.available_moves.is_empty() {
        if matrix.is_win() {
            return Some(matrix.copy());
        } else {
            return None;
        }
    } else {
        if allow_cheats && matrix.past_moves.len() > STEP_LIMIT {
            return None;
        }
        for i in 0..matrix.available_moves.len() {
            let result = find_win(&mut matrix.available_moves[i].1, past_matrices, allow_cheats);
            if result.is_none() {
                continue;
            } else {
                return result;
            }
        }
    }
    None
}
