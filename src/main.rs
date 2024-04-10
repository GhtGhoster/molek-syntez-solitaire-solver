use std::{collections::HashSet, fmt::Display, hash::Hash, thread::sleep, time::{Duration, Instant}};
use enigo::{Enigo, MouseControllable};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use screenshots::{image::{io::Reader, GenericImageView, ImageBuffer}, Screen};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const BOX_WIDTH: u32 = 22;
const BOX_HEIGHT: u32 = 18;
const OFFSET_H: i32 = 488;
const OFFSET_V: i32 = 298;
const SPACE_H: i32 = 652 - OFFSET_H;
const SPACE_V: i32 = 330 - OFFSET_V;

// strike a balance between fast, non-breaking, not missing a solve too often
const STEP_LIMIT: usize = 2000;
const PAST_LIMIT: usize = 20000;
const ACCEPTABLE_SOLUTION_LEN: usize = 100;

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

#[derive(Clone, Copy, PartialEq)]
enum Heuristic {
    #[allow(unused)]
    MoveCount,
    HighestOrder,
    None,
}

impl Heuristic {
    // lower score is preferred
    fn calculate(&self, matrix: &Matrix) -> usize {
        match self {
            Heuristic::MoveCount => matrix.valid_moves().len(),
            Heuristic::HighestOrder => {
                let mut ret = 40;
                for stack in &matrix.stacks {
                    if stack.collapsed {
                        ret -= 10
                    } else {
                        ret -= stack.highest_orderly_count()
                    }
                }
                ret
            },
            Heuristic::None => 0,
        }
    }
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

    fn random(rng: &mut impl Rng) -> Matrix { // rng: &mut Rng
        let mut matrix: Matrix = Default::default();
        let mut matrix_index = 0;
        // let mut rng = SmallRng::seed_from_u64(1337);

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

    fn save_moves(&mut self, allow_cheats: bool, heuristic: Heuristic) {
        self.available_moves = self
            .valid_moves()
            .iter()
            .map(|mov| (*mov, self.copy_after_move(*mov)))
            .filter(|(mov, _)| allow_cheats || self.check_validity(*mov) != MoveValidity::ValidCheat)
            .collect();
        
        if heuristic != Heuristic::None {
            self.available_moves.sort_by(
                |(_, a), (_, b)| {
                    let ah = heuristic.calculate(a);
                    let bh = heuristic.calculate(b);
                    ah.cmp(&bh)
                }
            );
        }
    }

    fn prune(&mut self, past_matrices: &HashSet<Matrix>) {
        for i in (0..self.available_moves.len()).rev() {
            if past_matrices.contains(&self.available_moves[i].1) {
                self.available_moves.remove(i);
            }
        }
    }

    fn past_matrices(&self, start_matrix: Matrix) -> Vec<Matrix> {
        let mut ret: Vec<Matrix> = vec![];
        let mut last_matrix = start_matrix.copy();
        for mov in &self.past_moves {
            last_matrix = last_matrix.copy_after_move(*mov);
            ret.push(last_matrix.copy());
        }
        ret
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
    // - enable brute_force to also find "wins" by finding a winnable state
    //      - if winnable_state.moves_left + current_matrix.past_moves < max_len: return it
    //      - need a structure to hold winnable states and their moves_left
    //      - brute_force (and by extension optimize_solution) would need to return Vec<Move> instead
    // - sort stacks alphabetically when generating matrix string, hash, and comparison
    //      - this should prevent equivalent gamestates with different order of stacks which doesn't matter
    // - update acceptable solution length
    //      - figure out average solution length
    //      - add the time it takes to reroll divided by time per move
    //      - keep as new acceptable solution length
    // - add CLI

    let start_time = Instant::now();

    // let mut matrix = Matrix::random();
    // let start_matrix = matrix.copy();

    // let mut winners: Vec<Matrix> = vec![];
    // let mut past_matrices: HashSet<Matrix> = HashSet::new();
    // while past_matrices.len() < PAST_LIMIT {
    //     if let Some(winner) = find_win(&mut matrix, &mut past_matrices, true) {
    //         winners.push(winner);
    //     }
    // }
    // if !winners.is_empty() {
    //     winners.sort_by(|a, b| a.past_moves.len().cmp(&b.past_moves.len()));
    //     println!("Best before opti: {}", winners[0].past_moves.len());
    //     optimize_solutions(start_matrix, &winners);
    // }

    loop_wins(5, true, false);

    println!("Finished after {:.02} seconds", start_time.elapsed().as_secs_f32());
}

// This expects sorted winner matrices
#[allow(dead_code)]
fn optimize_solutions(start_matrix: Matrix, winner_matrices: &Vec<Matrix>) -> Matrix {
    let winner_matrices_past_matrices: Vec<Vec<Matrix>> = winner_matrices
        .iter()
        .map(|winner_matrix| {
            winner_matrix.past_matrices(start_matrix.copy())
        })
        .collect();
    let mut best_matrix = winner_matrices[0].copy();

    let starting_cutoff = 2; // I could probably start way earlier, there's a lot of goofing near the end of almost every solve

    for winner_matrix_past_matrices in winner_matrices_past_matrices {
        for i in (0..winner_matrix_past_matrices.len()-starting_cutoff).rev() {
            // TODO: prevent this from firing if it'd take too long (if the bruteforce depth is too big)
            let mut past_matrix = winner_matrix_past_matrices[i].copy();
            let brute_force_depth = best_matrix.past_moves.len() as isize - past_matrix.past_moves.len() as isize;
            if brute_force_depth > 6 {break;} // brute_force could take too long, stick to 6 or 7 (max 7 or 8)
            if brute_force_depth < 0 {continue;} // brute_force would immediately end, save some performance


            // run bruteforce
            let mut discovered_matrices: HashSet<Matrix> = HashSet::new();
            let better_matrix_option = brute_force(&mut past_matrix, best_matrix.past_moves.len(), &mut discovered_matrices);
            if let Some(better_matrix) = better_matrix_option {
                if better_matrix.past_moves.len() < best_matrix.past_moves.len() {
                    best_matrix = better_matrix;
                }
            }
        }
    }

    best_matrix
}

fn brute_force(matrix: &mut Matrix, max_len: usize, discovered_matrices: &mut HashSet<Matrix>) -> Option<Matrix> {
    discovered_matrices.insert(matrix.copy());
    if matrix.past_moves.len() >= max_len {
        return None;
    }
    matrix.save_moves(true, Heuristic::None); // we don't need to find optimizations for non-cheated runs, only one is good enough anyway
    if matrix.available_moves.is_empty() {
        if matrix.is_win() {
            return Some(matrix.copy());
        } else {
            return None
        }
    }
    let mut best_matrix_option: Option<Matrix> = None;
    for (_, next_matrix) in &matrix.available_moves {
        if let Some(winner) = brute_force(&mut next_matrix.copy(), max_len, discovered_matrices) {
            if let Some(best_matrix) = &best_matrix_option {
                if best_matrix.past_moves.len() > winner.past_moves.len() {
                    best_matrix_option = Some(winner);
                }
            } else {
                best_matrix_option = Some(winner);
            }
        }
    }
    best_matrix_option
}

fn loop_wins(target_wins: usize, allow_cheats: bool, dry_run: bool) {
    let mut rng = SmallRng::seed_from_u64(1337);
    let mut enigo = Enigo::new();
    let mut iter_count = 0;
    let mut matrix_option = if dry_run {
        Matrix::from_screen()
    } else {
        None
    };
    while iter_count < target_wins {
        let mut matrix = if dry_run {
            Matrix::random(&mut rng)
        } else {
            if let Some(matrix) = &matrix_option {
                matrix.copy()
            } else {
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
                matrix_option.unwrap()
            }
        };
        
        // find solutions
        let mut winners = find_multiple_wins(matrix.copy(), allow_cheats, Heuristic::HighestOrder);
        println!("Best solution found: {} moves", winners[0].past_moves.len());
        
        // check for optimizations and different solutions or retry
        if winners.is_empty() || (allow_cheats && winners[0].past_moves.len() > ACCEPTABLE_SOLUTION_LEN) {
            if winners.is_empty() {
                // no winners, try finding wins with a different heuristic
                winners = find_multiple_wins(matrix.copy(), allow_cheats, Heuristic::MoveCount);
                if winners.is_empty() || (allow_cheats && winners[0].past_moves.len() > ACCEPTABLE_SOLUTION_LEN) {
                    if winners.is_empty() {
                        println!("No solution found, reattempting...");
                    } else {
                        println!("Secondary solution over {} moves, reattempting...", ACCEPTABLE_SOLUTION_LEN);
                    }
                    matrix_option = None;
                    continue;
                } else {
                    println!("Found a secondary solution!");
                }
            } else {
                // solution too long, try to optimize it
                // made redundant by heuristic improvements, keeping for testing purposes
                let winner = optimize_solutions(matrix.copy(), &winners);
                winners = vec![winner];
                if allow_cheats && winners[0].past_moves.len() > ACCEPTABLE_SOLUTION_LEN {
                    println!("Couldn't optimize primary solution, reattempting...");
                    matrix_option = None;
                    continue;
                } else {
                    println!("Optimized primary solution!");
                }
            }
        }

        // execute best solution
        println!("Executing solution: {} moves", winners[0].past_moves.len());
        if !dry_run {
            execute_moves(&mut matrix, &winners[0].past_moves);
        }
        iter_count += 1;
    }
}

fn find_multiple_wins(matrix: Matrix, allow_cheats: bool, heuristic: Heuristic) -> Vec<Matrix> {
    let mut winners: Vec<Matrix> = vec![];
    let mut past_matrices: HashSet<Matrix> = HashSet::new();

    while past_matrices.len() < PAST_LIMIT {
        if let Some(winner) = find_win(&mut matrix.copy(), &mut past_matrices, allow_cheats, heuristic) {
            // println!("Found solution: {} moves", winner.past_moves.len());
            winners.push(winner);
            if !allow_cheats {
                break;
            }
        }
    }

    winners.sort_by(|a, b| a.past_moves.len().cmp(&b.past_moves.len()));
    winners
}

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

fn find_win(matrix: &mut Matrix, past_matrices: &mut HashSet<Matrix>, allow_cheats: bool, heuristic: Heuristic) -> Option<Matrix> {
    past_matrices.insert(matrix.copy());
    if allow_cheats && past_matrices.len() > PAST_LIMIT {
        return None;
    }

    matrix.save_moves(allow_cheats, heuristic);
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
            let result = find_win(&mut matrix.available_moves[i].1, past_matrices, allow_cheats, heuristic);
            if result.is_none() {
                continue;
            } else {
                return result;
            }
        }
    }
    None
}
